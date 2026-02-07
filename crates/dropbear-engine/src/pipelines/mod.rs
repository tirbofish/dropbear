use std::sync::Arc;
use crate::graphics::SharedGraphicsContext;
use crate::shader::Shader;

pub mod shader;
pub mod light_cube;
pub mod globals;
pub mod hdr;

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

pub fn create_render_pipeline(
    label: Option<&str>,
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    topology: wgpu::PrimitiveTopology,
    shader: wgpu::ShaderModuleDescriptor,
) -> wgpu::RenderPipeline {
    create_render_pipeline_ex(
        label,
        device,
        layout,
        color_format,
        depth_format,
        vertex_layouts,
        topology,
        shader,
        true, // depth_write_enabled
        wgpu::CompareFunction::LessEqual,
    )
}

pub fn create_render_pipeline_ex(
    label: Option<&str>,
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    topology: wgpu::PrimitiveTopology,
    shader: wgpu::ShaderModuleDescriptor,
    depth_write_enabled: bool,
    depth_compare: wgpu::CompareFunction,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label,
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: vertex_layouts,
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology, // NEW!
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled,
            depth_compare,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        cache: None,
        multiview: None,
    })
}
