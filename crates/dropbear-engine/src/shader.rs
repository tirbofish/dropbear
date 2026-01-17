//! Deals with shaders, including WESL shaders

use std::ops::Deref;
use crate::graphics::SharedGraphicsContext;
use std::sync::Arc;
use wgpu::ShaderModule;

/// A nice little struct that stored basic information about a WGPU shader.
pub struct Shader {
    pub label: String,
    pub module: ShaderModule,
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

        log::debug!("Created new shader under the label: {:?}", label);

        Self {
            label: match label {
                Some(label) => label.into(),
                None => "shader".into(),
            },
            module,
        }
    }
}
