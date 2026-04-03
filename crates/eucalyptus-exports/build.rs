fn main() {
    // fuck you windows :(
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-arg=/FORCE:MULTIPLE");
        println!("cargo:rustc-link-arg=/NODEFAULTLIB:libcmt.lib");
    }

    goanna_gen::generate_c_header().unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src");
}
