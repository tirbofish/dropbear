use slank::{SlangShaderBuilder, SlangTarget};

fn main() {
    // to copy paste:         
    // let shader = Shader::from_slang(graphics.clone(), &slank::compiled::CompiledSlangShader::from_bytes("light cube", include_slang!("light_cube")));

    SlangShaderBuilder::new("light_cube")
        .add_source_path("src/pipelines/shaders/light.slang").unwrap()
        .compile_to_out_dir(SlangTarget::SpirV).unwrap();

    SlangShaderBuilder::new("blit_shader")
        .add_source_path("src/pipelines/shaders/blit.slang").unwrap()
        .compile_to_out_dir(SlangTarget::SpirV).unwrap();

    // unable to do mipmap.slang because it required texture_storage_2d, one write and one read. 
    // just wasn't possible with slang :(

    
}