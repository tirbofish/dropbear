use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::Args;
use eucalyptus_core::bundle::BundleManifest;
use sha2::{Digest, Sha256};

#[derive(Args, Debug)]
pub struct UnpackArgs {
    /// Path to the .eucplugin bundle to unpack.
    pub bundle: PathBuf,

    /// Output directory.
    /// Defaults to the bundle filename without its extension.
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

/// Resolves `base/entry_name` while preventing path traversal outside `base`.
///
/// Entries like `../../etc/passwd` are rejected with an error.
fn safe_extract_path(base: &Path, entry_name: &str) -> Result<PathBuf> {
    let mut result = base.to_path_buf();
    for component in Path::new(entry_name).components() {
        match component {
            Component::Normal(part) => result.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                bail!("Unsafe path in bundle entry: '{}'", entry_name);
            }
        }
    }
    Ok(result)
}

// to be honest, you could just extract the file instead since its just a zip file. 
pub fn run(args: &UnpackArgs) -> Result<()> {
    let file = std::fs::File::open(&args.bundle)
        .with_context(|| format!("Failed to open '{}'", args.bundle.display()))?;
    let mut archive = zip::ZipArchive::new(file).context("Not a valid .eucplugin bundle")?;

    let manifest: BundleManifest = {
        let mut entry = archive
            .by_name("manifest.eucc")
            .context("Bundle is missing manifest.eucc")?;
        let mut contents = String::new();
        entry
            .read_to_string(&mut contents)
            .context("Failed to read manifest.eucc")?;
        ron::from_str(&contents).context("Failed to parse manifest.eucc")?
    };

    if let Some(expected_hash) = &manifest.content_hash {
        let mut names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .filter(|n| n != "manifest.eucc")
            .collect();
        names.sort();

        let mut hasher = Sha256::new();
        for name in &names {
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
            bail!("Content hash mismatch — bundle may be corrupted or tampered");
        }
    }

    let out_dir = args.output.clone().unwrap_or_else(|| {
        PathBuf::from(
            args.bundle
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .as_ref(),
        )
    });
    std::fs::create_dir_all(&out_dir)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let entry_name = entry.name().to_string();
        let outpath = safe_extract_path(&out_dir, &entry_name)?;

        if entry_name.ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = std::fs::File::create(&outpath)
                .with_context(|| format!("Failed to create '{}'", outpath.display()))?;
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes)?;
            outfile.write_all(&bytes)?;
        }
    }

    println!(
        "Unpacked '{}' v{} → {}",
        manifest.name,
        manifest.version,
        out_dir.display()
    );
    Ok(())
}
