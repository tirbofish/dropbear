//! Shader code relating to displaying physics collider rendering, you know, QOL stuff...

use std::sync::Arc;
use dropbear_engine::entity::Transform;
use dropbear_engine::graphics::SharedGraphicsContext;
use dropbear_engine::shader::Shader;
use dropbear_engine::wgpu::*;

pub struct ColliderWireframePipeline {
    pub pipeline: RenderPipeline,
    pub bind_group_layout: BindGroupLayout,
}

impl ColliderWireframePipeline {
    pub fn new(
        graphics: Arc<SharedGraphicsContext>,
        camera_bind_group_layout: &BindGroupLayout,
    ) -> Self {
        let bind_group_layout = graphics.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("collider_bind_group_layout"),
        });

        let shader = Shader::new(
            graphics.clone(),
            dropbear_engine::shader::shader_wesl::COLLIDER_SHADER,
            Some("collider wireframe shader"),
        );

        let pipeline_layout = graphics.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("collider wireframe pipeline layout descriptor"),
            bind_group_layouts: &[
                camera_bind_group_layout, // @group(0)
                &bind_group_layout,       // @group(1)
            ],
            push_constant_ranges: &[],
        });

        let pipeline = graphics.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Collider Wireframe Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader.module,
                entry_point: Some("vs_main"),
                buffers: &[
                    VertexBufferLayout {
                        array_stride: size_of::<[f32; 3]>() as BufferAddress,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &[
                            VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: VertexFormat::Float32x3,
                            },
                        ],
                    },
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader.module,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Rgba8Unorm,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: CompareFunction::Always,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
        }
    }
}

#[derive(Debug, Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct ColliderUniform {
    model_matrix: [[f32; 4]; 4],
    color: [f32; 4],
}

impl ColliderUniform {
    pub fn new(transform: &Transform, color: [f32; 4]) -> Self {
        Self {
            model_matrix: transform.matrix().as_mat4().to_cols_array_2d(),
            color,
        }
    }
}