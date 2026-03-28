use crate::graphics::{InstanceRaw, SharedGraphicsContext};
use crate::model;
use crate::model::Vertex;
use crate::pipelines::DropbearShaderPipeline;
use crate::shader::Shader;
use crate::texture::Texture;
use std::sync::Arc;
use wgpu::{CompareFunction, DepthBiasState, StencilState};

/// As defined in `shaders/shader.wgsl`
pub struct MainRenderPipeline {
    shader: Shader,
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: wgpu::RenderPipeline,

    pub per_frame: Option<wgpu::BindGroup>,
    pub per_material: Option<wgpu::BindGroup>,
    pub animation: Option<wgpu::BindGroup>,
    pub environment: Option<wgpu::BindGroup>,
}

impl DropbearShaderPipeline for MainRenderPipeline {
    fn new(graphics: Arc<SharedGraphicsContext>) -> Self {
        let shader = Shader::new(
            graphics.clone(),
            include_str!("../shaders/shader.wgsl"),
            Some("viewport shaders"),
        );

        let bind_group_layouts = vec![
            Some(&graphics.layouts.per_frame_layout),
            Some(&graphics.layouts.material_bind_layout),
            Some(&graphics.layouts.animation_layout),
            Some(&graphics.layouts.environment_layout),
        ];

        let pipeline_layout =
            graphics
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("main render pipeline layout"),
                    bind_group_layouts: bind_group_layouts.as_slice(),
                    immediate_size: 0,
                });

        let hdr_format = graphics.hdr.read().format();
        let sample_count: u32 = (*graphics.antialiasing.read()).into();
        let pipeline = graphics
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("main render pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader.module,
                    entry_point: Some("vs_main"),
                    buffers: &[model::ModelVertex::desc(), InstanceRaw::desc()],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader.module,
                    entry_point: Some("s_fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: hdr_format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: Texture::DEPTH_FORMAT,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(CompareFunction::Greater),
                    stencil: StencilState::default(),
                    bias: DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: sample_count,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                cache: None,
                multiview_mask: None,
            });

        log::debug!("Created main render pipeline");

        Self {
            shader,
            pipeline_layout,
            pipeline,
            per_frame: None,
            per_material: None,
            animation: None,
            environment: None,
        }
    }

    fn pipeline_layout(&self) -> &wgpu::PipelineLayout {
        &self.pipeline_layout
    }

    fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    fn shader(&self) -> &Shader {
        &self.shader
    }
}

impl MainRenderPipeline {
    pub fn per_frame_bind_group(
        &mut self,
        graphics: Arc<SharedGraphicsContext>,
        globals_buffer: &wgpu::Buffer,
        camera_buffer: &wgpu::Buffer,
        light_array_buffer: &wgpu::Buffer,
    ) -> &wgpu::BindGroup {
        if self.per_frame.is_none() {
            let bind_group = graphics
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("per frame bind group"),
                    layout: &graphics.layouts.per_frame_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: globals_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: camera_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: light_array_buffer.as_entire_binding(),
                        },
                    ],
                });

            self.per_frame = Some(bind_group);
        }

        self.per_frame.as_ref().unwrap() // safe as its guaranteed to always have some content
    }

    pub fn per_material_bind_group(
        &mut self,
        graphics: Arc<SharedGraphicsContext>,
        material_uniform_buffer: &wgpu::Buffer,
        diffuse_texture: &Texture,
        normal_texture: &Texture,
        emissive_texture: &Texture,
        metallic_texture: &Texture,
        occlusion_texture: &Texture,
    ) -> &wgpu::BindGroup {
        if self.per_material.is_none() {
            let bind_group = graphics
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("per material bind group"),
                    layout: &graphics.layouts.material_bind_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: material_uniform_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: wgpu::BindingResource::TextureView(&emissive_texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: wgpu::BindingResource::Sampler(&emissive_texture.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 7,
                            resource: wgpu::BindingResource::TextureView(&metallic_texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 8,
                            resource: wgpu::BindingResource::Sampler(&metallic_texture.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 9,
                            resource: wgpu::BindingResource::TextureView(&occlusion_texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 10,
                            resource: wgpu::BindingResource::Sampler(&occlusion_texture.sampler),
                        },
                    ],
                });

            self.per_material = Some(bind_group);
        }

        self.per_material.as_ref().unwrap()
    }

    pub fn animation_bind_group(
        &self,
        graphics: Arc<SharedGraphicsContext>,
        skinning_buffer: &wgpu::Buffer,
        morph_deltas_buffer: &wgpu::Buffer,
        morph_weights_buffer: &wgpu::Buffer,
        morph_info_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("animation bind group"),
                layout: &graphics.layouts.animation_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: skinning_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: morph_deltas_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: morph_weights_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: morph_info_buffer.as_entire_binding(),
                    },
                ],
            })
    }

    pub fn environment_bind_group(
        &mut self,
        graphics: Arc<SharedGraphicsContext>,
        environment_view: &wgpu::TextureView,
        environment_sampler: &wgpu::Sampler,
    ) -> &wgpu::BindGroup {
        if self.environment.is_none() {
            let bind_group = graphics
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("environment bind group"),
                    layout: &graphics.layouts.environment_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(environment_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(environment_sampler),
                        },
                    ],
                });

            self.environment = Some(bind_group);
        }

        self.environment.as_ref().unwrap()
    }
}
