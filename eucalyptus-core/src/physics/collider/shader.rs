//! Shader code relating to displaying physics collider rendering.

use std::mem::size_of;
use std::sync::Arc;

use dropbear_engine::entity::Transform;
use dropbear_engine::graphics::SharedGraphicsContext;
use dropbear_engine::shader::Shader;
use dropbear_engine::wgpu::*;
use glam::Mat4;

pub struct ColliderWireframePipeline {
    pub pipeline: RenderPipeline,
}

impl ColliderWireframePipeline {
    pub fn new(
        graphics: Arc<SharedGraphicsContext>,
        camera_bind_group_layout: &BindGroupLayout,
    ) -> Self {
        let shader = Shader::new(
            graphics.clone(),
            dropbear_engine::shader::shader_wesl::COLLIDER_SHADER,
            Some("collider wireframe shader"),
        );

        let pipeline_layout = graphics.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("collider wireframe pipeline layout descriptor"),
            bind_group_layouts: &[
                camera_bind_group_layout, // @group(0)
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
                    ColliderInstanceRaw::desc(),
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader.module,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Rgba16Float,
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

        Self { pipeline }
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

    pub fn from_matrix(matrix: glam::Mat4, color: [f32; 4]) -> Self {
        Self {
            model_matrix: matrix.to_cols_array_2d(),
            color,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColliderInstanceRaw {
    pub model: [[f32; 4]; 4],
    pub color: [f32; 4],
}

impl ColliderInstanceRaw {
    pub fn from_matrix(matrix: Mat4, color: [f32; 4]) -> Self {
        Self {
            model: matrix.to_cols_array_2d(),
            color,
        }
    }

    pub fn desc() -> VertexBufferLayout<'static> {
        const ATTRIBS: [VertexAttribute; 5] = [
            VertexAttribute {
                offset: 0,
                shader_location: 1,
                format: VertexFormat::Float32x4,
            },
            VertexAttribute {
                offset: size_of::<[f32; 4]>() as BufferAddress,
                shader_location: 2,
                format: VertexFormat::Float32x4,
            },
            VertexAttribute {
                offset: size_of::<[f32; 8]>() as BufferAddress,
                shader_location: 3,
                format: VertexFormat::Float32x4,
            },
            VertexAttribute {
                offset: size_of::<[f32; 12]>() as BufferAddress,
                shader_location: 4,
                format: VertexFormat::Float32x4,
            },
            VertexAttribute {
                offset: size_of::<[f32; 16]>() as BufferAddress,
                shader_location: 5,
                format: VertexFormat::Float32x4,
            },
        ];

        VertexBufferLayout {
            array_stride: size_of::<ColliderInstanceRaw>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &ATTRIBS,
        }
    }
}