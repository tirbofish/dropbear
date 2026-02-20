use slank::{SlangShaderBuilder, SlangTarget};

fn main() {
    // to copy paste:
    // let shader = Shader::from_slang(graphics.clone(), &slank::compiled::CompiledSlangShader::from_bytes("light cube", include_slang!("light_cube")));

    SlangShaderBuilder::new("light_cube")
        .add_source_path("src/shaders/light.slang")
        .unwrap()
        .compile_to_out_dir(SlangTarget::SpirV)
        .unwrap();

    SlangShaderBuilder::new("blit_shader")
        .add_source_path("src/shaders/blit.slang")
        .unwrap()
        .compile_to_out_dir(SlangTarget::SpirV)
        .unwrap();

    println!("cargo:rerun-if-changed=src/shaders");
}
