use std::env;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let root_dir = PathBuf::from(&crate_dir).parent().unwrap().to_path_buf();

    let expanded = std::process::Command::new("cargo")
        .args(&["expand", "--lib", "-p", "eucalyptus-core"])
        .current_dir(&root_dir)
        .output()?;

    let temp_file = "target/generated/expanded.rs";
    let temp_file = root_dir.join(temp_file);
    std::fs::create_dir_all(temp_file.parent().unwrap())?;
    std::fs::write(&root_dir.join(&temp_file), expanded.stderr)?;

    let output_file = root_dir
        .join("headers")
        .join("dropbear.h");

    let config = cbindgen::Config::from_file("../cbindgen.toml")
        .expect("Could not find cbindgen.toml");

    cbindgen::Builder::new()
        .with_src(temp_file)
        .with_config(config)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(output_file);

    println!("cargo:rerun-if-changed=cbindgen.toml");

    // fuck you windows :(
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-arg=/FORCE:MULTIPLE");
        println!("cargo:rustc-link-arg=/NODEFAULTLIB:libcmt.lib");
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");
    Ok(())
}
