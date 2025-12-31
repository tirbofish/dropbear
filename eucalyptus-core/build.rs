fn main() -> anyhow::Result<()> {

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
