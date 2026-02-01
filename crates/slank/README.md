# slank

A rust version of the slang compiler, ready to be used by wgpu, vulkan or directx (or any graphics library really, i personally
use wgpu).

# Usage

Add `slank` to your `[build-dependencies]`.

In your `build.rs`:
```rust
use slank::{ShaderStage, SlangShaderBuilder, SlangTarget};
use std::path::Path;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("shader.spv");

    SlangShaderBuilder::new("shader_label")
        .add_source_path("src/shader.slang").unwrap()
        .entry_with_stage("vs_main", ShaderStage::Vertex)
        .entry_with_stage("fs_main", ShaderStage::Fragment)
        .build(SlangTarget::SpirV).unwrap()
        .output(&dest_path).unwrap();

    println!("cargo:rerun-if-changed=src/shader.slang");
}
```

Then in your main code:

```rust,ignore
let shader_bytes = slank::include_slang!("shader_label");

// optionally
slank::CompiledSlangShader::from_bytes("shader_label", shader_bytes);
```

# Features
There are two main features currently available:

- `download-slang` - Slank downloads the latest slangc compiler from the GitHub releases and stores it 
                     in the user cache. The SLANG_DIR will be set to the directory of the slangc compiler.
                     Note: Using this in CI will require you to add a `GITHUB_TOKEN` env var to avoid the rate limits. Locally, you are fine. 
- `use-wgpu` - Enables wgpu as a dependency and unlocks utility traits for wgpu. 

# Contribution

This is part of the larger `dropbear-engine` project, however that shouldn't stop you from contributing to this
repository. It is still missing a lot of different stuff, such as modules, more arguments and better Vulkan/DirectX
library utility support. 

Your help is appreciated. 