use std::sync::Arc;
use wgpu::{CompareFunction, DepthBiasState, StencilState};
use crate::graphics::{InstanceRaw, SharedGraphicsContext};
use crate::model;
use crate::model::Vertex;
use crate::pipelines::DropbearShaderPipeline;
use crate::shader::Shader;
use crate::texture::Texture;

/// As defined in `shaders/shader.wgsl`
pub struct MainRenderPipeline {
    shader: Shader,
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: wgpu::RenderPipeline,
}

impl DropbearShaderPipeline for MainRenderPipeline {
    fn new(graphics: Arc<SharedGraphicsContext>) -> Self {
        let shader = Shader::new(
            graphics.clone(),
            include_str!("../shaders/shader.wgsl"),
            Some("viewport shaders"),
        );

        let bind_group_layouts = vec![
            &graphics.layouts.scene_globals_bind_group_layout, // @group(0)
            &graphics.layouts.material_bind_layout, // @group(1)
            &graphics.layouts.scene_light_skin_bind_group_layout, // @group(2)
        ];

        let pipeline_layout =
            graphics.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("main render pipeline layout"),
                    bind_group_layouts: bind_group_layouts.as_slice(),
                    push_constant_ranges: &[],
                });

        let hdr_format = graphics.hdr.read().format();
        let pipeline =
            graphics.device
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
                        depth_write_enabled: true,
                        depth_compare: CompareFunction::Greater,
                        stencil: StencilState::default(),
                        bias: DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                    cache: None,
                });

        log::debug!("Created main render pipeline");

        Self {
            shader,
            pipeline_layout,
            pipeline,
        }
    }

    fn shader(&self) -> &Shader {
        &self.shader
    }

    fn pipeline_layout(&self) -> &wgpu::PipelineLayout {
        &self.pipeline_layout
    }

    fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}
