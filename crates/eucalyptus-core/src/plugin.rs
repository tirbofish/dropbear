use std::any::TypeId;
use std::collections::HashMap;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use libloading as lib;
use sha2::{Digest, Sha256};

use crate::APP_INFO;
use crate::bundle::{Arch, BundleManifest, Platform};
use crate::component::{InspectableComponent, LanguageTypeId};

/// FFI signature for the plugin entry point exported as `eucalyptus_plugin_init`.
pub type PluginInitFn = unsafe extern "C" fn(*mut PluginRegistry);

/// The loaded artifacts for a `.eucplugin` bundle.
///
/// A plugin can ship as JVM-only, native-only, or a hybrid of both.
pub enum LoadedPlugin {
    /// Pure JVM plugin — only a JAR was bundled.
    Jvm {
        /// Path to the staged JAR in the app-data dependencies folder.
        jar_path: PathBuf,
        manifest: BundleManifest,
    },
    /// Pure native plugin — only a platform shared library was bundled.
    NativeLib {
        /// Retaining ownership keeps the DSO mapped in process memory.
        /// Dropping it unloads the library.
        library: lib::Library,
        manifest: BundleManifest,
    },
    /// Hybrid plugin — both a JAR and a native shared library were bundled.
    JvmAndNativeLib {
        jar_path: PathBuf,
        library: lib::Library,
        manifest: BundleManifest,
    },
}

impl LoadedPlugin {
    pub fn manifest(&self) -> &BundleManifest {
        match self {
            Self::Jvm { manifest, .. } => manifest,
            Self::NativeLib { manifest, .. } => manifest,
            Self::JvmAndNativeLib { manifest, .. } => manifest,
        }
    }
}

pub struct PluginRegistry {
    tokens: HashMap<PluginRegistrationToken, LanguageTypeId>,
    /// Maps plugin manifest to plugin type id.
    ty: HashMap<PluginManifest, LanguageTypeId>,
    /// Maps plugin token to the list of component type ids registered under that plugin.
    components: HashMap<uuid::Uuid, Vec<LanguageTypeId>>,
    /// Maps bundle name → loaded plugin artifacts.
    plugins: HashMap<String, LoadedPlugin>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
            ty: HashMap::new(),
            components: HashMap::new(),
            plugins: HashMap::new(),
        }
    }

    /// Loads a single `.eucplugin` bundle.
    pub fn load_plugin(&mut self, bundle_path: PathBuf) -> Result<()> {
        log::debug!("Loading plugin: {}", bundle_path.display());

        // open the archive
        let file = std::fs::File::open(&bundle_path)
            .with_context(|| format!("Failed to open plugin bundle '{}'", bundle_path.display()))?;
        let mut archive =
            zip::ZipArchive::new(file).context("Plugin bundle is not a valid ZIP archive")?;

        // parse the manifest
        let manifest: BundleManifest = {
            let mut entry = archive
                .by_name("manifest.eucc")
                .context("Plugin bundle is missing manifest.eucc")?;
            let mut contents = String::new();
            entry
                .read_to_string(&mut contents)
                .context("Failed to read manifest.eucc")?;
            ron::from_str(&contents).context("Failed to parse manifest.eucc")?
        };

        if self.plugins.contains_key(&manifest.name) {
            log::warn!(
                "Plugin '{}' v{} is already loaded, skipping.",
                manifest.name,
                manifest.version
            );
            return Ok(());
        }

        // integrity check
        if let Some(expected_hash) = &manifest.content_hash {
            let mut entry_names: Vec<String> = (0..archive.len())
                .map(|i| archive.by_index(i).unwrap().name().to_string())
                .filter(|n| n != "manifest.eucc")
                .collect();
            entry_names.sort();

            let mut hasher = Sha256::new();
            for name in &entry_names {
                let mut entry = archive.by_name(name)?;
                let mut bytes = Vec::new();
                entry.read_to_end(&mut bytes)?;
                hasher.update(&bytes);
            }
            let actual: String = hasher.finalize().iter().fold(String::new(), |mut s, b| {
                use std::fmt::Write;
                write!(s, "{b:02x}").unwrap();
                s
            });
            if &actual != expected_hash {
                bail!(
                    "Content hash mismatch in plugin '{}'. Bundle might be corrupted or tampered.",
                    manifest.name
                );
            }
        }

        // extract plugins
        let root = app_dirs2::app_root(app_dirs2::AppDataType::UserData, &APP_INFO)
            .context("Failed to locate app data directory")?;
        let plugin_dir = root
            .join("plugins")
            .join(&manifest.name)
            .join(manifest.version.to_string());
        std::fs::create_dir_all(&plugin_dir).with_context(|| {
            format!("Failed to create plugin directory '{}'", plugin_dir.display())
        })?;

        let platform = current_platform();
        let arch = current_arch();
        let native_lib_path: Option<String> = manifest
            .native_libs
            .iter()
            .find(|lib| {
                (lib.platform == platform || lib.platform == Platform::All) && lib.arch == arch
            })
            .map(|lib| lib.path.clone());

        let bundle_name = manifest.name.clone();
        let bundle_version = manifest.version.to_string();
        let jar_zip_path: Option<String> = manifest.jar.clone();

        // create a loaded plugin
        let loaded: LoadedPlugin = match (jar_zip_path, native_lib_path) {
            // jvm only
            (Some(jar_rel), None) => {
                let jar_dest = extract_entry(&mut archive, &jar_rel, &plugin_dir)?;
                let dep_jar = stage_jar_for_jvm(&root, &jar_dest, &bundle_name, &bundle_version)?;
                log::info!(
                    "Plugin '{}' v{}: JVM JAR staged at '{}'. Effective on next JVM initialisation.",
                    bundle_name,
                    bundle_version,
                    dep_jar.display(),
                );
                LoadedPlugin::Jvm { jar_path: dep_jar, manifest }
            }

            // native only
            (None, Some(native_path)) => {
                let lib_dest = extract_entry(&mut archive, &native_path, &plugin_dir)?;
                let library = unsafe { lib::Library::new(&lib_dest) }.with_context(|| {
                    format!(
                        "Failed to load native library '{}' for plugin '{}'",
                        lib_dest.display(),
                        bundle_name
                    )
                })?;
                call_plugin_init_if_present(&library, self)?;
                log::info!(
                    "Plugin '{}' v{}: native library loaded.",
                    bundle_name,
                    bundle_version,
                );
                LoadedPlugin::NativeLib { library, manifest }
            }

            // jvm + native
            (Some(jar_rel), Some(native_path)) => {
                let jar_dest = extract_entry(&mut archive, &jar_rel, &plugin_dir)?;
                let dep_jar = stage_jar_for_jvm(&root, &jar_dest, &bundle_name, &bundle_version)?;
                let lib_dest = extract_entry(&mut archive, &native_path, &plugin_dir)?;
                let library = unsafe { lib::Library::new(&lib_dest) }.with_context(|| {
                    format!(
                        "Failed to load native library '{}' for plugin '{}'",
                        lib_dest.display(),
                        bundle_name
                    )
                })?;
                call_plugin_init_if_present(&library, self)?;
                log::info!(
                    "Plugin '{}' v{}: JVM JAR staged and native library loaded.",
                    bundle_name,
                    bundle_version,
                );
                LoadedPlugin::JvmAndNativeLib { jar_path: dep_jar, library, manifest }
            }

            (None, None) => {
                bail!(
                    "Plugin '{}' v{} contains neither a JAR nor a native library compatible \
                     with this platform ({:?} / {:?}).",
                    bundle_name,
                    bundle_version,
                    platform,
                    arch,
                );
            }
        };

        let name = loaded.manifest().name.clone();
        self.plugins.insert(name, loaded);
        log::info!("Plugin '{}' v{} loaded successfully.", bundle_name, bundle_version);
        Ok(())
    }

    /// Loads all `.eucplugin` bundles found in `plugin_folder_dir`.
    pub fn load_plugins(&mut self, plugin_folder_dir: PathBuf) -> Result<()> {
        let dir = plugin_folder_dir.read_dir()?;
        let mut count = 0;
        for entry in dir {
            let entry = entry?;
            if entry.path().extension() == Some("eucplugin".as_ref()) {
                log::info!("Found plugin: {:?}", entry.file_name());
                self.load_plugin(entry.path())?;
                count+=1;
            }
        }

        if count == 0 {
            log::info!("No plugins found");
        }
        Ok(())
    }

    pub fn unload_plugin(&mut self, plugin_id: &str) {
        let Some(loaded) = self.plugins.remove(plugin_id) else {
            log::warn!(
                "Attempted to unload plugin '{}', but it is not loaded.",
                plugin_id
            );
            return;
        };

        match loaded {
            LoadedPlugin::Jvm { jar_path, manifest } => {
                remove_staged_jar(&jar_path, &manifest.name);
                log::info!(
                    "Unloaded JVM plugin '{}' v{}. Classes remain in the current JVM session until process restart.",
                    manifest.name,
                    manifest.version,
                );
            }
            LoadedPlugin::NativeLib { library, manifest } => {
                drop(library);
                log::info!(
                    "Unloaded native plugin '{}' v{}.",
                    manifest.name,
                    manifest.version,
                );
            }
            LoadedPlugin::JvmAndNativeLib { jar_path, library, manifest } => {
                remove_staged_jar(&jar_path, &manifest.name);
                drop(library);
                log::info!(
                    "Unloaded hybrid plugin '{}' v{}. Native library unloaded; JVM classes remain until process restart.",
                    manifest.name,
                    manifest.version,
                );
            }
        }
    }

    pub fn register_plugin<T>(&mut self) -> PluginRegistrationToken
    where
        T: ExternalPlugin + Send + Sync + 'static,
    {
        let type_id = LanguageTypeId::Rust(TypeId::of::<T>());
        let token = PluginRegistrationToken(uuid::Uuid::new_v4());
        self.ty.insert(T::plugin_manifest(), type_id.clone());
        self.tokens.insert(token.clone(), type_id);
        token
    }

    pub fn register_component<T>(&mut self, token: PluginRegistrationToken)
    where
        T: crate::component::Component + InspectableComponent + Send + Sync + 'static,
        T::SerializedForm: Send + Sync + 'static,
        T::RequiredComponentTypes: Send + Sync + 'static,
    {
        let component_type_id = LanguageTypeId::Rust(TypeId::of::<T>());
        self.components
            .entry(token.0)
            .or_default()
            .push(component_type_id);
    }

    // todo
    pub fn register_dock<T>(&mut self, _token: PluginRegistrationToken) {}

    /// Iterates over loaded plugins as `(bundle_name, &LoadedPlugin)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &LoadedPlugin)> {
        self.plugins.iter().map(|(name, plugin)| (name.as_str(), plugin))
    }
}

/// Returns `base / entry_name` while rejecting path-traversal components.
fn safe_extract_path(base: &Path, entry_name: &str) -> Result<PathBuf> {
    let mut result = base.to_path_buf();
    for component in Path::new(entry_name).components() {
        match component {
            Component::Normal(part) => result.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                bail!(
                    "Unsafe path in bundle entry '{}' — path traversal rejected.",
                    entry_name
                );
            }
        }
    }
    Ok(result)
}

/// Extracts a single ZIP entry to `base / entry_name`, creating parent directories as needed.
fn extract_entry(
    archive: &mut zip::ZipArchive<std::fs::File>,
    entry_name: &str,
    base: &Path,
) -> Result<PathBuf> {
    let dest = safe_extract_path(base, entry_name)?;
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut bytes = Vec::new();
    archive
        .by_name(entry_name)
        .with_context(|| format!("Entry '{}' not found in bundle", entry_name))?
        .read_to_end(&mut bytes)?;
    std::fs::write(&dest, &bytes)
        .with_context(|| format!("Failed to write '{}'", dest.display()))?;
    Ok(dest)
}

/// Copies `jar_dest` into `<root>/dependencies/plugin-<name>-<version>.jar` so that
/// [`crate::scripting::jni::JavaContext`] picks it up on the next JVM initialisation.
fn stage_jar_for_jvm(root: &Path, jar_dest: &Path, name: &str, version: &str) -> Result<PathBuf> {
    let deps_dir = root.join("dependencies");
    std::fs::create_dir_all(&deps_dir)?;
    let dep_jar = deps_dir.join(format!("plugin-{}-{}.jar", name, version));
    std::fs::copy(jar_dest, &dep_jar).with_context(|| {
        format!(
            "Failed to stage JAR '{}' → '{}'",
            jar_dest.display(),
            dep_jar.display()
        )
    })?;
    Ok(dep_jar)
}

/// Removes the staged JAR from the dependencies folder, logging a warning on failure.
fn remove_staged_jar(jar_path: &Path, plugin_name: &str) {
    if jar_path.exists() {
        if let Err(e) = std::fs::remove_file(jar_path) {
            log::warn!(
                "Failed to remove staged JAR '{}' for plugin '{}': {e}",
                jar_path.display(),
                plugin_name,
            );
        }
    }
}

/// Calls `eucalyptus_plugin_init` on `library` if the symbol is exported.
///
/// The symbol is optional — pure JVM plugins will not export it, and that is expected.
fn call_plugin_init_if_present(library: &lib::Library, registry: &mut PluginRegistry) -> Result<()> {
    // Safety: `eucalyptus_plugin_init` is a known-good `extern "C"` symbol with a stable ABI
    // defined by `PluginInitFn`. Plugins compiled against eucalyptus-core are responsible for
    // upholding this contract.
    unsafe {
        if let Ok(init_fn) = library.get::<PluginInitFn>(b"eucalyptus_plugin_init\0") {
            init_fn(registry as *mut PluginRegistry);
        }
    }
    Ok(())
}

/// Returns the [`Platform`] this binary was compiled for.
fn current_platform() -> Platform {
    if cfg!(target_os = "windows") {
        Platform::Windows
    } else if cfg!(target_os = "macos") {
        Platform::MacOs
    } else {
        Platform::Linux
    }
}

/// Returns the [`Arch`] this binary was compiled for.
fn current_arch() -> Arch {
    if cfg!(target_arch = "aarch64") {
        Arch::Arm64
    } else {
        Arch::X64
    }
}


/// Used as a temporary form of registering Components and other types under one plugin.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PluginRegistrationToken(pub(crate) uuid::Uuid);

pub trait ExternalPlugin {
    fn plugin_manifest() -> PluginManifest;
}

#[derive(Hash, Eq, PartialEq)]
pub struct PluginManifest {
    pub display_name: String,
    pub description: String,
    pub version: String,
    pub authors: Vec<String>,
    pub dependencies: Vec<String>,
}

