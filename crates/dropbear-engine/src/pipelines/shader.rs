use crate::graphics::{InstanceRaw, SharedGraphicsContext};
use crate::model;
use crate::model::Vertex;
use crate::pipelines::HotPipeline;
use crate::texture::Texture;
use std::sync::Arc;
use wesl::ModulePath;
use wgpu::{CompareFunction, DepthBiasState, StencilState};

/// As defined in `shaders/shader.wesl`
pub struct MainRenderPipeline {
    pipeline: HotPipeline,

    pub per_frame: Option<wgpu::BindGroup>,
    pub per_material: Option<wgpu::BindGroup>,
    pub animation: Option<wgpu::BindGroup>,
    pub environment: Option<wgpu::BindGroup>,
}

impl MainRenderPipeline {
    pub fn new(graphics: Arc<SharedGraphicsContext>) -> Self {
        let pipeline_layout = Arc::new(
            graphics
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("main render pipeline layout"),
                    bind_group_layouts: &[
                        Some(&graphics.layouts.per_frame_layout),
                        Some(&graphics.layouts.material_bind_layout),
                        Some(&graphics.layouts.animation_layout),
                        Some(&graphics.layouts.environment_layout),
                    ],
                    immediate_size: 0,
                }),
        );

        let hdr_format = graphics.hdr.read().format();
        let sample_count: u32 = (*graphics.antialiasing.read()).into();
        let device = graphics.device.clone();

        let shader_dir = std::path::PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/shaders"
        ));

        let pipeline = HotPipeline::new(
            device,
            shader_dir.clone(),
            move |device| {
                let source = wesl::Wesl::new(&shader_dir)
                    .compile(&ModulePath::from_path("/shader.wesl"))
                    .map_err(|e| anyhow::anyhow!("{e}"))?
                    .to_string();

                // fixes early read
                for ep in ["vs_main", "s_fs_main"] {
                    if !source.contains(&format!("fn {ep}(")) {
                        return Err(anyhow::anyhow!(
                            "compiled shader is missing entry point '{ep}' \
                            (file may have been read mid-write)"
                        ));
                    }
                }

                log::debug!("Compiled WGSL: {} bytes", source.len());

                wgpu::naga::front::wgsl::parse_str(&source)
                    .map_err(|e| anyhow::anyhow!("WGSL parse error: {}", e.emit_to_string(&source)))?;

                let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("viewport shaders"),
                    source: wgpu::ShaderSource::Wgsl(source.into()),
                });

                let render_pipeline =
                    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("main render pipeline"),
                        layout: Some(&pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &shader,
                            entry_point: Some("vs_main"),
                            buffers: &[model::ModelVertex::desc(), InstanceRaw::desc()],
                            compilation_options: wgpu::PipelineCompilationOptions::default(),
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader,
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
                Ok(render_pipeline)
            },
        )
        .expect("failed to build initial main render pipeline");

        Self {
            pipeline,
            per_frame: None,
            per_material: None,
            animation: None,
            environment: None,
        }
    }

    pub fn pipeline(&self) -> arc_swap::Guard<Arc<wgpu::RenderPipeline>> {
        self.pipeline.get()
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
