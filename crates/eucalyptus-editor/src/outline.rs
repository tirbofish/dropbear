//! Outline shader
//!
//! General idea:
//! - 1. get that one entity as a sort of map
//! - 2. run an edge detection, and apply an orange outline for the edge (perhaps a specific width)

use std::sync::Arc;
use dropbear_engine::graphics::{CommandEncoder, SharedGraphicsContext};
use dropbear_engine::pipelines::builder::RenderPipelineBuilder;
use dropbear_engine::pipelines::DropbearShaderPipeline;
use dropbear_engine::texture::{Texture, TextureBuilder};

pub struct OutlineShader {
    depth_stencil: Texture,
    mask_bind_group: wgpu::BindGroup,
    pub mask_pipeline: wgpu::RenderPipeline,
}

impl OutlineShader {
    pub fn new(graphics: Arc<SharedGraphicsContext>) -> anyhow::Result<Self> {
        let size = graphics.window.inner_size();

        let depth_stencil_format = wgpu::TextureFormat::Depth24PlusStencil8;
        let depth_stencil = TextureBuilder::new(&graphics.device)
            .label("depth stencil")
            .size(size.width, size.height)
            .format(depth_stencil_format)
            .usage(wgpu::TextureUsages::RENDER_ATTACHMENT)
            .build();

        let source = wesl::Wesl::new("src/shaders")
            .add_package(&dropbear_engine::shader::code::PACKAGE)
            .compile(&"dropbear_shaders::mask".parse()?)
            .inspect_err(|e| {
                panic!("{e}");
            })?
            .to_string();

        let mask_shader = wgpu::ShaderModuleDescriptor {
            label: Some("mask shader"),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        };

        let mask_bind_group_layout = graphics.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("mask bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        let mask_bind_group = graphics.device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("mask_bind_group"),
                layout: &mask_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Sampler(&mask_texture.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&mask_texture.view),
                    },
                ],
            });

        let mask_pipeline_layout = graphics.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mask pipeline layout"),
            bind_group_layouts: &[
                Some(&mask_bind_group_layout)
            ],
            immediate_size: 0,
        });

        let mask_pipeline = RenderPipelineBuilder::new()
            .vertex_shader(mask_shader.clone())
            .fragment_shader(mask_shader.clone())
            .fragment_entry_point("fs_mask")
            .cull_mode(Some(wgpu::Face::Back))
            .depth_stencil(wgpu::DepthStencilState {
                format: depth_stencil_format,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: wgpu::StencilState {
                    write_mask: 0xFF,
                    read_mask: 0xFF,
                    front: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Always,
                        pass_op: wgpu::StencilOperation::Replace,
                        ..Default::default()
                    },
                    back: wgpu::StencilFaceState::IGNORE,
                },
                bias: wgpu::DepthBiasState::default(),
            })
            .layout(&mask_pipeline_layout)
            .build(&graphics.device)?;

        Self {
            depth_stencil,
            mask_bind_group: (),
            mask_pipeline,
        }
    }

    pub fn draw(&self, graphics: Arc<SharedGraphicsContext>, encoder: &mut CommandEncoder) {
        {
            let mut draw_mask_stencil = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("draw mask stencil"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_stencil.view, // draw onto depth_stencil
                    depth_ops: None,
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: wgpu::StoreOp::Store,
                    }),
                }),
                timestamp_writes: None,
                multiview_mask: None,
                occlusion_query_set: None,
            });

            draw_mask_stencil.set_stencil_reference(0xFF);
            draw_mask_stencil.set_pipeline(&self.mask_pipeline);
            draw_mask_stencil.set_bind_group(0, &self.mask_bind_group, &[]);
            draw_mask_stencil.draw(0..3, 0..1);
        }
    }
}