use std::fs;
use std::path::Path;
use std::env;

fn main() {
    let shader_dir = Path::new("src/shaders");

    if shader_dir.exists() {
        println!("cargo:rerun-if-changed={}", shader_dir.display());

        if let Ok(entries) = fs::read_dir(shader_dir) {
            for entry in entries.flatten() {
                println!("cargo:rerun-if-changed={}", entry.path().display());
            }
        }
    }

    wesl::PkgBuilder::new("dropbear")
        .scan_root("src/shaders")
        .expect("failed to scan for dropbear wesl shaders")
        .validate()
        .map_err(|e| eprintln!("{e}"))
        .expect("validation error")
        .build_artifact()
        .expect("failed to build artifact");

    wesl::Wesl::new("src/shaders")
        .build_artifact(&"package::light".parse().unwrap(), "dropbear_light");
    wesl::Wesl::new("src/shaders")
        .build_artifact(&"package::shader".parse().unwrap(), "dropbear_shader");
    wesl::Wesl::new("src/shaders")
        .build_artifact(&"package::shadow".parse().unwrap(), "dropbear_shadow");
    wesl::Wesl::new("src/shaders")
        .build_artifact(&"package::outline".parse().unwrap(), "dropbear_outline");
    wesl::Wesl::new("src/shaders")
        .build_artifact(&"package::collider".parse().unwrap(), "dropbear_collider");
    wesl::Wesl::new("src/shaders")
        .build_artifact(&"package::mipmap".parse().unwrap(), "dropbear_mipmap");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dropbear_path = Path::new(&out_dir).join("dropbear.rs");

    if let Ok(content) = fs::read_to_string(&dropbear_path) {
        let fixed_content = format!("#[allow(dead_code)]\n\n{}", content);
        fs::write(&dropbear_path, fixed_content)
            .expect("failed to write fixed dropbear.rs");
    }
}