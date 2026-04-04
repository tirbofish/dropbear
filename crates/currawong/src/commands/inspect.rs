use std::io::Read;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;
use eucalyptus_core::bundle::BundleManifest;

#[derive(Args, Debug)]
pub struct InspectArgs {
    /// Path to the .eucplugin bundle to inspect.
    pub bundle: PathBuf,
}

pub fn run(args: &InspectArgs) -> Result<()> {
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

    println!("{:#?}", manifest);
    Ok(())
}
