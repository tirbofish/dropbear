use std::path::Path;
use slank::{SlangShaderBuilder, SlangTarget};

fn main() -> anyhow::Result<()> {
    // to copy paste:
    // let shader = Shader::from_slang(graphics.clone(), &slank::compiled::CompiledSlangShader::from_bytes("light cube", include_slang!("light_cube")));

    SlangShaderBuilder::new("light_cube")
        .add_source_path("src/shaders/light.slang")?
        .compile_to_out_dir(SlangTarget::SpirV)?;

    SlangShaderBuilder::new("blit_shader")
        .add_source_path("src/shaders/blit.slang")?
        .compile_to_out_dir(SlangTarget::SpirV)?;

    println!("cargo:rerun-if-changed=src/shaders");
    
    validate_wgsl()?;

    Ok(())
}

fn validate_wgsl() -> anyhow::Result<()> {
    let shader_dir = Path::new("src/shaders");

    for entry in std::fs::read_dir(shader_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("wgsl") {
            continue;
        }

        println!("cargo:rerun-if-changed={}", path.display());

        let src = std::fs::read_to_string(&path)?;

        let mut frontend = naga::front::wgsl::Frontend::new();
        let module = frontend.parse(&src).unwrap_or_else(|e| {
            panic!("WGSL parse error in {}:\n{}", path.display(), e.emit_to_string(&src));
        });

        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );
        validator.validate(&module).unwrap_or_else(|e| {
            panic!("WGSL validation error in {}:\n{:?}", path.display(), e);
        });
    }

    Ok(())
}
