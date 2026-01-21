use std::num::NonZeroU64;
use std::sync::Arc;
use anyhow::Context;
use wgpu::{include_wgsl, BindGroupLayoutEntry, ShaderStages, VertexAttribute, VertexStepMode};
use crate::{GumContext, KinoUICommandBuffer};
use crate::math::Size;

pub struct KinoRenderer {
    pub(crate) context: GumContext,
    pub(crate) render: KinoRenderPipeline,
    pub(crate) ui: Arc<KinoUICommandBuffer>,
}

pub(crate) struct KinoRenderPipeline {
    pub(crate) globals_uniform_layout: wgpu::BindGroupLayout,
    pub(crate) pipeline: wgpu::RenderPipeline,
    pub(crate) device: Arc<wgpu::Device>,
    pub(crate) queue: Arc<wgpu::Queue>,
}

impl KinoRenderer {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        texture_format: wgpu::TextureFormat,
    ) -> anyhow::Result<Self> {
        let shader = device.create_shader_module(include_wgsl!("shaders/primitive.wgsl"));

        let globals_uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("kino globals uniform"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("kino render pipeline layout"),
            bind_group_layouts: &[
                &globals_uniform_layout
            ],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("kino render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[
                    VertexInput::desc()
                ],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: texture_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: Default::default(),
                    })
                ],
            }),
            multiview: None,
            cache: None,
        });

        Ok(Self {
            context: GumContext::new(),
            render: KinoRenderPipeline {
                globals_uniform_layout,
                pipeline,
                device,
                queue,
            },
            ui: Arc::new(KinoUICommandBuffer::new()),
        })
    }

    pub fn get_ui(&self) -> Arc<KinoUICommandBuffer> {
        self.ui.clone()
    }

    /// Uses [self.render] in the backend, creates an encoder with the device provided, and
    /// renders the content.
    pub fn render_without_encoder(
        &mut self,
        view: &wgpu::TextureView,
        size_to_render: Size,
    ) {
        let mut encoder = self.render.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("kino render encoder"),
        });

        self.render(&mut encoder, view, size_to_render);

        self.render.queue.submit(std::iter::once(encoder.finish()));
    }

    /// Renders the content onto the provided texture/view.
    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        size_to_render: Size,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("kino render pass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })
            ],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.context.screen_size = size_to_render;
        
        let mut contents = self.ui.process();
        for mut widget in contents.drain(..) {
            widget.draw(&self, &mut pass);
        }
    }
}

pub struct VertexInput {
    position: [f32; 3],
    fill_colour: [f32; 4],
}

impl VertexInput {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: 0,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                // position
                VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },

                // fill_colour
                VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: size_of::<[f32; 3]>() as u64,
                    shader_location: 1,
                }
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[derive(Default)]
pub struct Globals {
    pub proj: [[f32; 4]; 4],
    pub screen_size: [f32; 2],
    pub _padding: [f32; 2],
}

impl Globals {
    pub fn new(proj: [[f32; 4]; 4], screen_size: [f32; 2]) -> Self {
        Self {
            proj,
            screen_size,
            _padding: [0.0; 2],
        }
    }
}