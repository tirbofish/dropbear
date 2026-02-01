use std::sync::Arc;
use crate::graphics::SharedGraphicsContext;
use crate::shader::Shader;

pub mod shader;
pub mod light_cube;
pub mod globals;
mod hdr;

pub use globals::{Globals, GlobalsUniform};

/// A helper in defining a pipelines required information, as well as getters. 
/// 
/// This contains the bare minimum for any pipeline. 
pub trait DropbearShaderPipeline {
    /// Creates a new instance of a pipeline. 
    fn new(graphics: Arc<SharedGraphicsContext>) -> Self;
    /// Fetches the shader property
    fn shader(&self) -> &Shader;
    /// Fetches the pipeline layout
    fn pipeline_layout(&self) -> &wgpu::PipelineLayout;
    /// Fetches the pipeline
    fn pipeline(&self) -> &wgpu::RenderPipeline;
}