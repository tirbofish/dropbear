fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace_root = std::path::PathBuf::from(&manifest_dir)
        .parent() // crates/
        .and_then(|p| p.parent()) // workspace root
        .expect("Could not determine workspace root from CARGO_MANIFEST_DIR")
        .to_path_buf();

    let jar_path = workspace_root.join("build/libs/dropbear-1.0-SNAPSHOT-all.jar");

    if !jar_path.exists() {
        build_gradle(&workspace_root);
    }

    println!("cargo:rerun-if-changed={}", jar_path.display());
}

fn build_gradle(workspace_root: &std::path::Path) {
    let gradlew = if cfg!(windows) {
        workspace_root.join("gradlew.bat")
    } else {
        workspace_root.join("gradlew")
    };

    println!("cargo:warning=Running Gradle shadowJar...");

    let status = std::process::Command::new(&gradlew)
        .arg("shadowJar")
        .current_dir(workspace_root)
        .status()
        .expect("Failed to spawn Gradle");

    if !status.success() {
        panic!("Gradle shadowJar failed");
    }

    println!(
        "cargo:rerun-if-changed={}",
        workspace_root.join("build.gradle.kts").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        workspace_root.join("scripting").display()
    );
}