fn main() {
    let manifest_dir =
        std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("Could not locate workspace root from CARGO_MANIFEST_DIR");

    let src_dir = manifest_dir.join("crate").join("src");
    let output_header = manifest_dir.join("include").join("surface_nets_plugin.h");
    let type_search_root = workspace_root.join("crates");

    goanna_gen::generate_c_header_for(&src_dir, &output_header, &type_search_root)
        .expect("Failed to generate surface_nets_plugin.h");

    println!("cargo:rerun-if-changed=crate/src/exports.rs");
    println!("cargo:rerun-if-changed=crate/src/component.rs");
}