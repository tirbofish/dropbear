//! Deals with shaders, primarily around the [Shader] struct. 

use std::ops::Deref;
use crate::graphics::SharedGraphicsContext;
use std::sync::Arc;
use wgpu::ShaderModule;

/// A nice little struct that stored basic information about a WGPU shaders.
pub struct Shader {
    /// The label of the shader. 
    /// 
    /// If it is not set in [Shader::new], the default is "shader". 
    pub label: String,

    /// The compiled content of the WGSL shader.
    /// 
    /// When [Shader] is dereferenced (such as that in `&shader`), it will automatically reference 
    /// this module.  
    pub module: ShaderModule,

    /// The content of the shader as a readable string content, in the case you need to look
    /// at the original source. 
    pub content: String,
}

impl Deref for Shader {
    type Target = ShaderModule;

    fn deref(&self) -> &Self::Target {
        &self.module
    }
}

impl Shader {
    /// Creates a new [`ShaderModule`] from its file contents.
    pub fn new(
        graphics: Arc<SharedGraphicsContext>,
        shader_file_contents: &str,
        label: Option<&str>,
    ) -> Self {
        let module = graphics
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label,
                source: wgpu::ShaderSource::Wgsl(shader_file_contents.into()),
            });

        log::debug!("Created new shaders under the label: {:?}", label);

        Self {
            label: match label {
                Some(label) => label.into(),
                None => "shader".into(),
            },
            module,
            content: shader_file_contents.to_string(),
        }
    }
}
