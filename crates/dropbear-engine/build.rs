use slank::{SlangShaderBuilder, SlangTarget};

fn main() -> anyhow::Result<()> {

    println!("cargo:rerun-if-changed=src/shaders");

    compile_slang_shaders()?;

    compile_wesl_shaders()?;

    Ok(())
}

fn compile_slang_shaders() -> anyhow::Result<()> {
    // to copy paste:
    // let shader = Shader::from_slang(graphics.clone(), &slank::compiled::CompiledSlangShader::from_bytes("light cube", include_slang!("light_cube")));

    SlangShaderBuilder::new("light_cube")
        .add_source_path("src/shaders/light.slang")?
        .compile_to_out_dir(SlangTarget::SpirV)?;

    SlangShaderBuilder::new("blit_shader")
        .add_source_path("src/shaders/blit.slang")?
        .compile_to_out_dir(SlangTarget::SpirV)?;

    Ok(())
}

fn compile_wesl_shaders() -> anyhow::Result<()> {
    wesl::PkgBuilder::new("dropbear_shaders")
        .scan_root("src/shaders")
        .expect("failed to scan WESL files")
        .validate()
        .inspect_err(|e| {
            eprintln!("{e}");
            panic!("{e}");
        })?
        .build_artifact()
        .expect("failed to build artifact");

    Ok(())
}