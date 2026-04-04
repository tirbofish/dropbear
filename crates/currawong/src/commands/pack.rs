use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use eucalyptus_core::bundle::{
    Arch, BundleAssetEntry, BundleManifest, BuildProfile, NativeLib, Platform,
};
use indicatif::{ProgressBar, ProgressStyle};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

#[derive(Args, Debug)]
pub struct PackArgs {
    /// Path to the manifest.eucc file. If omitted, you will be prompted interactively.
    pub manifest: Option<PathBuf>,

    /// Path to the plugin .jar file. Placed at the bundle root. Optional.
    #[arg(long)]
    pub jar: Option<PathBuf>,

    /// Path to a native shared library (.so / .dll / .dylib). Can be repeated for multiple platforms.
    #[arg(long)]
    pub lib: Vec<PathBuf>,

    /// Path to the assets directory to include. Optional.
    #[arg(long)]
    pub assets: Option<PathBuf>,

    /// Output path for the bundle.
    /// Defaults to `<name>-<version>-<profile>.eucplugin` in the current directory.
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

fn build_manifest_interactively() -> Result<BundleManifest> {
    let theme = ColorfulTheme::default();

    let name: String = Input::with_theme(&theme)
        .with_prompt("Plugin name (lowercase-hyphenated)")
        .validate_with(|s: &String| {
            if s.is_empty() {
                Err("name is required".to_string())
            } else {
                Ok(())
            }
        })
        .interact_text()?;

    let version: semver::Version = {
        let s: String = Input::with_theme(&theme)
            .with_prompt("Version")
            .default("0.1.0".into())
            .validate_with(|s: &String| {
                semver::Version::parse(s)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            })
            .interact_text()?;
        semver::Version::parse(&s)?
    };

    let description: Option<String> = {
        let s: String = Input::with_theme(&theme)
            .with_prompt("Description (Enter to skip)")
            .allow_empty(true)
            .interact_text()?;
        if s.is_empty() { None } else { Some(s) }
    };

    let authors: Vec<String> = {
        let s: String = Input::with_theme(&theme)
            .with_prompt("Authors (comma-separated, Enter to skip)")
            .allow_empty(true)
            .interact_text()?;
        s.split(',')
            .map(|a| a.trim().to_string())
            .filter(|a| !a.is_empty())
            .collect()
    };

    let license: Option<String> = {
        let s: String = Input::with_theme(&theme)
            .with_prompt("License (e.g. MIT, Enter to skip)")
            .allow_empty(true)
            .interact_text()?;
        if s.is_empty() { None } else { Some(s) }
    };

    let profile = {
        let idx = Select::with_theme(&theme)
            .with_prompt("Build profile")
            .items(&["Release", "Debug"])
            .default(0)
            .interact()?;
        if idx == 0 { BuildProfile::Release } else { BuildProfile::Debug }
    };

    let engine_api_version: semver::VersionReq = {
        let s: String = Input::with_theme(&theme)
            .with_prompt("Engine API version requirement (semver req)")
            .default("*".into())
            .validate_with(|s: &String| {
                semver::VersionReq::parse(s)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            })
            .interact_text()?;
        semver::VersionReq::parse(&s)?
    };

    Ok(BundleManifest {
        name,
        version,
        description,
        authors,
        license,
        profile,
        engine_api_version,
        native_libs: vec![],
        jar: None,
        assets: vec![],
        dependencies: vec![],
        content_hash: None,
    })
}

pub fn run(args: &PackArgs) -> Result<()> {
    let mut manifest: BundleManifest = match &args.manifest {
        Some(path) => {
            let src = std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read '{}'", path.display()))?;
            ron::from_str(&src).context("Failed to parse manifest.eucc")?
        }
        None => build_manifest_interactively()?,
    };

    let mut entries: Vec<(PathBuf, String)> = Vec::new();

    if let Some(jar_path) = &args.jar {
        let jar_filename = jar_path
            .file_name()
            .context("--jar path has no filename")?
            .to_string_lossy()
            .into_owned();
        entries.push((jar_path.clone(), jar_filename));
    }

    // native libs go to libs/{platform}/{arch}/{filename}
    for lib_path in &args.lib {
        let lib_filename = lib_path
            .file_name()
            .context("--lib path has no filename")?
            .to_string_lossy()
            .into_owned();
        let platform = detect_platform(&lib_filename);
        let arch = detect_arch(&lib_filename);
        let lib_dest = format!("libs/{platform}/{arch}/{lib_filename}");
        manifest.native_libs.push(NativeLib {
            path: lib_dest.clone(),
            platform,
            arch,
        });
        entries.push((lib_path.clone(), lib_dest));
    }

    // assets dir: assets/<>
    if let Some(assets_dir) = &args.assets {
        let asset_entries: Vec<_> = WalkDir::new(assets_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| {
                let rel = e
                    .path()
                    .strip_prefix(assets_dir)
                    .ok()?
                    .to_string_lossy()
                    .replace('\\', "/");
                let dest = format!("assets/{rel}");
                Some((e.into_path(), dest))
            })
            .collect();

        for (_src, dest) in &asset_entries {
            manifest.assets.push(BundleAssetEntry {
                path: dest.clone(),
                kind: eucalyptus_core::bundle::AssetKind::Data,
                compressed: manifest.profile == BuildProfile::Release,
            });
        }
        entries.extend(asset_entries);
    }

    entries.sort_by(|a, b| a.1.cmp(&b.1));

    let mut hasher = Sha256::new();
    for (path, _) in &entries {
        let bytes = std::fs::read(path)
            .with_context(|| format!("Failed to read '{}'", path.display()))?;
        hasher.update(&bytes);
    }
    manifest.content_hash = Some(
        hasher.finalize().iter().fold(String::new(), |mut s, b| {
            use std::fmt::Write;
            write!(s, "{b:02x}").unwrap();
            s
        }),
    );

    let output_path = args
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from(manifest.output_filename()));

    let file = std::fs::File::create(&output_path)
        .with_context(|| format!("Failed to create '{}'", output_path.display()))?;
    let mut zip = zip::ZipWriter::new(file);

    let file_opts = if manifest.profile == BuildProfile::Release {
        zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .compression_level(Some(9))
    } else {
        zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
    };
    let manifest_opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);

    let manifest_ron =
        ron::ser::to_string_pretty(&manifest, ron::ser::PrettyConfig::default())
            .context("Failed to serialize manifest")?;
    zip.start_file("manifest.eucc", manifest_opts)?;
    zip.write_all(manifest_ron.as_bytes())?;

    let pb = ProgressBar::new(entries.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} Packing [{bar:40.cyan/blue}] {pos}/{len}  {msg}",
        )
        .unwrap()
        .progress_chars("=>-"),
    );

    for (path, dest) in &entries {
        pb.set_message(dest.clone());
        let bytes = std::fs::read(path)
            .with_context(|| format!("Failed to read '{}'", path.display()))?;
        zip.start_file(dest, file_opts)?;
        zip.write_all(&bytes)?;
        pb.inc(1);
    }

    pb.finish_and_clear();
    zip.finish()?;

    println!(
        "Packed '{}' ({} file(s)) → {}",
        manifest.name,
        entries.len(),
        output_path.display()
    );
    Ok(())
}

fn detect_platform(filename: &str) -> Platform {
    if filename.ends_with(".dll") {
        Platform::Windows
    } else if filename.ends_with(".dylib") {
        Platform::MacOs
    } else if filename.ends_with(".so") || filename.contains(".so.") {
        Platform::Linux
    } else {
        Platform::All
    }
}

fn detect_arch(filename: &str) -> Arch {
    let lower = filename.to_lowercase();
    if lower.contains("aarch64") || lower.contains("arm64") {
        Arch::Arm64
    } else {
        Arch::X64
    }
}
