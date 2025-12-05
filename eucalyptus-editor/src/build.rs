use anyhow::{bail, Context};
use app_dirs2::{app_root, AppDataType};
use crossbeam_channel::Sender;
use eucalyptus_core::config::ProjectConfig;
use eucalyptus_core::runtime::RuntimeProjectConfig;
use eucalyptus_core::scene::SceneConfig;
use eucalyptus_core::APP_INFO;
use semver::Version;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use tokio::{fs as tokio_fs, process::Command, task};

/// Builds a eucalyptus project into a single bundle.
///
/// Returns the path of the build directory
pub fn build(project_config: PathBuf) -> anyhow::Result<PathBuf> {
    log::info!("Started project building");
    // create a build directory
    let project_root = project_config
        .parent()
        .ok_or(anyhow::anyhow!("Unable to locate parent folder of config"))?
        .to_path_buf();
    let build_dir = project_root.join("build/output");

    if build_dir.exists() {
        fs::remove_dir_all(&build_dir)?;
    }
    fs::create_dir_all(&build_dir)?;
    log::debug!("Readied build directory");

    // load the project config manually to avoid overwriting global state
    let ron_str = fs::read_to_string(&project_config)?;
    let mut config: ProjectConfig = ron::de::from_str(&ron_str)?;
    config.project_path = project_root.clone();
    log::debug!("Loaded project config");

    // load scenes
    let mut scenes = Vec::new();
    let scene_folder = project_root.join("scenes");
    if scene_folder.exists() {
        for entry in fs::read_dir(scene_folder)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("eucs") {
                match SceneConfig::read_from(&path) {
                    Ok(scene) => {
                        scenes.push(scene);
                    }
                    Err(e) => {
                        log::warn!("Failed to load scene {:?} during build: {}", path, e);
                    }
                }
            }
        }
    }

    // convert to runtime project config
    let runtime_config = RuntimeProjectConfig {
        project_name: config.project_name.clone(),
        runtime_settings: config.runtime_settings.clone(),
        scenes,
        authors: config.authors.clone(),
        editor_version: Version::parse(env!("CARGO_PKG_VERSION"))?,
        project_version: Version::parse(
            config
                .project_version
                .clone()
                .unwrap_or(String::from("0.1.0"))
                .as_str(),
        )?,
        initial_scene: config.runtime_settings.initial_scene.ok_or(anyhow::anyhow!("Project was expected to be an initial scene"))?,
    };
    log::debug!("Converted to runtime project config");

    // export to .eupak
    let eupak_path = build_dir.join("data.eupak");
    let config_bytes = bincode::encode_to_vec(&runtime_config, bincode::config::standard())?;
    fs::write(&eupak_path, config_bytes)?;
    log::debug!("Exported scene config to {:?}", eupak_path);

    // copy resources
    let resources_src = project_root.join("resources");
    let resources_dst = build_dir.join("resources");
    if resources_src.exists() {
        copy_dir_recursive(&resources_src, &resources_dst)?;
        log::debug!("Copied resources to {:?}", resources_dst);
    }

    log::info!("Done!");

    Ok(build_dir)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// Reads the contents of a data.eupak file into a pretty print format.
///
/// Returns the contents of the project config.
pub fn read(eupak: PathBuf) -> anyhow::Result<RuntimeProjectConfig> {
    let bytes = std::fs::read(&eupak)?;
    let (content, _): (RuntimeProjectConfig, usize) =
        bincode::decode_from_slice(&bytes, bincode::config::standard())?;
    println!("{} contents: {:#?}", eupak.display(), content);
    Ok(content)
}

#[derive(Debug, Clone)]
pub enum PackageStatus {
    Info(String),
    Progress { step: &'static str, detail: String },
    Error(String),
}

fn emit_status(tx: &Option<Sender<PackageStatus>>, status: PackageStatus) {
    if let Some(sender) = tx {
        let _ = sender.send(status);
    }
}

pub async fn package(
    project_config: PathBuf,
    status_tx: Option<Sender<PackageStatus>>,
) -> anyhow::Result<PathBuf> {
    if !project_config.exists() {
        bail!("Project config not found: {}", project_config.display());
    }

    emit_status(
        &status_tx,
        PackageStatus::Info(format!("Packaging project at {}", project_config.display())),
    );

    let project_root = project_config
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Unable to locate parent folder of config"))?
        .to_path_buf();

    let project_contents = tokio_fs::read_to_string(&project_config).await?;
    let project_data: ProjectConfig = ron::de::from_str(&project_contents)?;
    let project_name = sanitize_filename(&project_data.project_name);

    let templates_dir = app_root(AppDataType::UserData, &APP_INFO)?.join("templates");
    tokio_fs::create_dir_all(&templates_dir).await?;

    emit_status(
        &status_tx,
        PackageStatus::Progress {
            step: "Locating runtime",
            detail: format!("Searching for runtime executable in {}", templates_dir.display()),
        },
    );

    let runtime_source = {
        let dir = templates_dir.clone();
        task::spawn_blocking(move || locate_runtime_binary(&dir)).await??
    };

    let package_dir = project_root.join("build/package");
    if package_dir.exists() {
        tokio_fs::remove_dir_all(&package_dir).await?;
    }
    tokio_fs::create_dir_all(&package_dir).await?;

    let runtime_filename = runtime_source
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Runtime template missing filename"))?;
    let runtime_dest = package_dir.join(runtime_filename);
    tokio_fs::copy(&runtime_source, &runtime_dest).await?;
    emit_status(
        &status_tx,
        PackageStatus::Info(format!(
            "Copied runtime template to {}",
            runtime_dest.display()
        )),
    );

    emit_status(
        &status_tx,
        PackageStatus::Progress {
            step: "Build project",
            detail: "Building runtime configuration".to_string(),
        },
    );

    let build_dir = {
        let config_path = project_config.clone();
        task::spawn_blocking(move || build(config_path)).await??
    };

    let data_src = build_dir.join("data.eupak");
    if !data_src.exists() {
        bail!("Expected {} to exist", data_src.display());
    }
    tokio_fs::copy(&data_src, package_dir.join("data.eupak")).await?;

    let resources_src = build_dir.join("resources");
    let resources_dst = package_dir.join("resources");
    if resources_src.exists() && !resources_dst.exists() {
        let src = resources_src.clone();
        let dst = resources_dst.clone();
        task::spawn_blocking(move || copy_dir_recursive(&src, &dst)).await??;
    }

    emit_status(
        &status_tx,
        PackageStatus::Progress {
            step: "magna-carta",
            detail: "Generating script bindings via magna-carta".to_string(),
        },
    );

    let magna_output_dir = project_root.join("build/magna-carta/nativeLibMain");
    tokio_fs::create_dir_all(&magna_output_dir).await?;

    let magna_status = Command::new("magna-carta")
        .arg("--input")
        .arg(project_root.join("src"))
        .arg("--target")
        .arg("native")
        .arg("--output")
        .arg(&magna_output_dir)
        .current_dir(&project_root)
        .status()
        .await
        .context("Failed to execute magna-carta")?;

    if !magna_status.success() {
        bail!("magna-carta failed with status {magna_status:?}");
    }

    emit_status(
        &status_tx,
        PackageStatus::Progress {
            step: "Gradle",
            detail: "Running Gradle build".to_string(),
        },
    );
    run_gradle_build(&project_root).await?;
    log::info!("Gradle build completed successfully");

    emit_status(
        &status_tx,
        PackageStatus::Progress {
            step: "NativeLibrary",
            detail: "Copying native library artifact".to_string(),
        },
    );
    log::info!("Copying native library artifact");

    let release_dir = project_root.join("build/bin/nativeLib/releaseShared");
    let extension = native_library_extension();
    let library_source = {
        let dir = release_dir.clone();
        let ext = extension.to_string();
        task::spawn_blocking(move || pick_latest_library(&dir, &ext)).await??
    };

    let library_dest = package_dir.join(format!("{}.{}", project_name, extension));
    tokio_fs::copy(&library_source, &library_dest).await?;

    emit_status(
        &status_tx,
        PackageStatus::Progress {
            step: "RuntimeLibs",
            detail: "Copying runtime dependencies".to_string(),
        },
    );

    let libs_dir = project_root.join("libs");
    tokio_fs::create_dir_all(&libs_dir).await?;

    let discovered_libs = task::spawn_blocking(discover_runtime_libraries).await??;

    #[cfg(windows)]
    let (shared_objects, import_libs) = {
        let DiscoveredLibraries {
            shared_objects,
            import_libs,
        } = discovered_libs;
        (shared_objects, import_libs)
    };

    #[cfg(not(windows))]
    let shared_objects = {
        let DiscoveredLibraries { shared_objects, .. } = discovered_libs;
        shared_objects
    };

    for lib in shared_objects {
        let file_name = lib
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Runtime dependency missing filename: {}", lib.display()))?
            .to_owned();
        let libs_target = libs_dir.join(&file_name);
        if !format!("{}", file_name.display()).contains("eucalyptus_core") { continue; }
        tokio_fs::copy(&lib, &libs_target).await?;
        let package_target = package_dir.join(&file_name);
        tokio_fs::copy(&lib, &package_target).await?;
        log::info!(
            "Copied runtime dependency {} to {} and {}",
            lib.display(),
            libs_target.display(),
            package_target.display()
        );
    }

    #[cfg(windows)]
    {
        for import_lib in import_libs {
            let file_name = import_lib
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("Import library missing filename: {}", import_lib.display()))?
                .to_owned();
            let libs_target = libs_dir.join(&file_name);
            tokio_fs::copy(&import_lib, &libs_target).await?;
            log::info!(
                "Copied Windows import library {} to {}",
                import_lib.display(),
                libs_target.display()
            );
        }
    }

    emit_status(
        &status_tx,
        PackageStatus::Info(format!(
            "Packaged build available at {}",
            package_dir.display()
        )),
    );

    log::info!("Packaged build available at {}", package_dir.display());

    Ok(package_dir)
}

fn locate_runtime_binary(templates_dir: &Path) -> anyhow::Result<PathBuf> {
    for name in runtime_name_candidates() {
        let candidate = templates_dir.join(name);
        if is_runtime_binary(&candidate) {
            return Ok(candidate);
        }
    }

    for entry in fs::read_dir(templates_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !is_runtime_binary(&path) {
            continue;
        }
        if entry
            .file_name()
            .to_string_lossy()
            .to_ascii_lowercase()
            .contains("redback-runtime")
        {
            return Ok(path);
        }
    }

    let current_exe = std::env::current_exe()?;
    let current_dir = current_exe.parent().ok_or(anyhow::anyhow!("Unable to locate parent folder of current executable"))?;
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !is_runtime_binary(&path) {
            continue;
        }
        if entry
            .file_name()
            .to_string_lossy()
            .to_ascii_lowercase()
            .contains("redback-runtime")
        {
            return Ok(path);
        }
    }

    bail!(
        "Unable to locate redback runtime executable in {}",
        templates_dir.display()
    )
}

fn runtime_name_candidates() -> &'static [&'static str] {
    #[cfg(windows)]
    {
        &["redback-runtime.exe", "redback-runtime"]
    }
    #[cfg(target_os = "macos")]
    {
        &["redback-runtime.app", "redback-runtime"]
    }
    #[cfg(all(not(windows), not(target_os = "macos")))]
    {
        &["redback-runtime"]
    }
}

fn is_runtime_binary(path: &Path) -> bool {
    let Ok(metadata) = path.metadata() else {
        return false;
    };

    #[cfg(windows)]
    {
        if !metadata.is_file() {
            return false;
        }
        return path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("exe"))
            .unwrap_or(false);
    }

    #[cfg(target_os = "macos")]
    {
        if metadata.is_dir() {
            return path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("app"))
                .unwrap_or(false);
        }
        metadata.is_file()
    }

    #[cfg(all(not(windows), not(target_os = "macos")))]
    {
        metadata.is_file()
    }
}

fn sanitize_filename(name: &str) -> String {
    let mut result = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
            result.push(ch);
        } else if ch.is_whitespace() {
            result.push('_');
        }
    }
    if result.is_empty() {
        "project".to_string()
    } else {
        result
    }
}

fn native_library_extension() -> &'static str {
    if cfg!(windows) {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    }
}

fn pick_latest_library(dir: &Path, extension: &str) -> anyhow::Result<PathBuf> {
    if !dir.exists() {
        bail!("Native library directory does not exist: {}", dir.display());
    }

    let mut matches = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let path = entry.path();
        let ext_matches = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case(extension))
            .unwrap_or(false);
        if !ext_matches {
            continue;
        }
        let modified = entry
            .metadata()
            .ok()
            .and_then(|meta| meta.modified().ok())
            .unwrap_or(UNIX_EPOCH);
        matches.push((modified, path));
    }

    if matches.is_empty() {
        bail!(
            "Unable to locate any *.{extension} libraries in {}",
            dir.display()
        );
    }

    matches.sort_by_key(|(modified, _)| *modified);
    let (_, path) = matches
        .pop()
        .expect("matches is not empty so pop must succeed");
    Ok(path)
}

struct DiscoveredLibraries {
    shared_objects: Vec<PathBuf>,
    #[cfg_attr(not(windows), allow(dead_code))]
    import_libs: Vec<PathBuf>,
}

fn discover_runtime_libraries() -> anyhow::Result<DiscoveredLibraries> {
    let current_exe = std::env::current_exe()?;
    let current_dir = current_exe
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Unable to locate parent folder of current executable"))?
        .to_path_buf();

    let mut libs = DiscoveredLibraries {
        shared_objects: Vec::new(),
        import_libs: Vec::new(),
    };

    for entry in fs::read_dir(&current_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }

        let path = entry.path();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase())
            .unwrap_or_default();

        if matches!(extension.as_str(), "dll" | "dylib" | "so") {
            libs.shared_objects.push(path);
            continue;
        }

        if extension.as_str() == "lib" {
            if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
                if file_name.to_ascii_lowercase().ends_with(".dll.lib") {
                    libs.import_libs.push(path);
                }
            }
        }
    }

    if libs.shared_objects.is_empty() {
        bail!(
            "Unable to locate any runtime libraries next to {}",
            current_dir.display()
        );
    }

    Ok(libs)
}

async fn run_gradle_build(project_root: &Path) -> anyhow::Result<()> {
    #[cfg(windows)]
    {
        let script = project_root.join("gradlew.bat");
        if !script.exists() {
            bail!("Gradle wrapper not found at {}", script.display());
        }
        let status = Command::new("cmd")
            .arg("/C")
            .arg(script)
            .arg("build")
            .current_dir(project_root)
            .status()
            .await
            .context("Failed to run gradlew.bat")?;
        if !status.success() {
            bail!("Gradle build failed (status {status:?})");
        }
    }

    #[cfg(not(windows))]
    {
        let script = project_root.join("gradlew");
        if !script.exists() {
            bail!("Gradle wrapper not found at {}", script.display());
        }
        let status = Command::new(script)
            .arg("build")
            .current_dir(project_root)
            .status()
            .await
            .context("Failed to run ./gradlew")?;
        if !status.success() {
            bail!("Gradle build failed (status {status:?})");
        }
    }

    Ok(())
}
