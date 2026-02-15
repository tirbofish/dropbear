//! Shader code relating to displaying physics collider rendering.

use std::mem::size_of;
use std::sync::Arc;

use dropbear_engine::{entity::Transform, texture::Texture};
use dropbear_engine::graphics::SharedGraphicsContext;
use dropbear_engine::shader::Shader;
use glam::Mat4;
use dropbear_engine::pipelines::DropbearShaderPipeline;
use dropbear_engine::wgpu::{BlendState, BufferAddress, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState, DepthStencilState, FragmentState, FrontFace, MultisampleState, PipelineLayout, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, StencilState, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode};
use crate::physics::collider::{ColliderShape, WireframeGeometry};

pub struct ColliderWireframePipeline {
    pub shader: Shader,
    pub pipeline_layout: PipelineLayout,
    pub pipeline: RenderPipeline,
}

impl DropbearShaderPipeline for ColliderWireframePipeline {
    fn new(graphics: Arc<SharedGraphicsContext>) -> Self {
        let shader = Shader::new(
            graphics.clone(),
            include_str!("shaders/collider.wgsl"),
            Some("collider wireframe shaders"),
        );

        let pipeline_layout = graphics.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("collider wireframe pipeline layout descriptor"),
            bind_group_layouts: &[
                &graphics.layouts.camera_bind_group_layout, // @group(0)
            ],
            push_constant_ranges: &[],
        });

        let hdr_format = graphics.hdr.read().format();
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
                    format: hdr_format,
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
                format: Texture::DEPTH_FORMAT,
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
            shader,
            pipeline_layout,
            pipeline,
        }
    }

    fn shader(&self) -> &Shader {
        &self.shader
    }

    fn pipeline_layout(&self) -> &PipelineLayout {
        &self.pipeline_layout
    }

    fn pipeline(&self) -> &RenderPipeline {
        &self.pipeline
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

pub fn create_wireframe_geometry(
    graphics: Arc<SharedGraphicsContext>,
    shape: &ColliderShape,
) -> WireframeGeometry {
    match shape {
        ColliderShape::Box { half_extents } => {
            WireframeGeometry::box_wireframe(graphics, half_extents.to_float_array())
        }
        ColliderShape::Sphere { radius } => {
            WireframeGeometry::sphere_wireframe(graphics, *radius, 16, 16)
        }
        ColliderShape::Capsule { half_height, radius } => {
            WireframeGeometry::capsule_wireframe(graphics, *half_height, *radius, 16)
        }
        ColliderShape::Cylinder { half_height, radius } => {
            WireframeGeometry::cylinder_wireframe(graphics, *half_height, *radius, 16)
        }
        ColliderShape::Cone { half_height, radius } => {
            WireframeGeometry::cone_wireframe(graphics, *half_height, *radius, 16)
        }
    }
}