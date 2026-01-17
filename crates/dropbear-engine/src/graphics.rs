use crate::shader::Shader;
use crate::texture::Texture;
use crate::{BindGroupLayouts, texture};
use crate::{
    State,
    egui_renderer::EguiRenderer,
    model::{self, Vertex},
};
use dropbear_future_queue::FutureQueue;
use egui::{Context, TextureId};
use glam::{DMat4, DQuat, DVec3, Mat3};
use parking_lot::Mutex;
use std::sync::Arc;
use wgpu::*;
use winit::window::Window;

pub const NO_TEXTURE: &[u8] = include_bytes!("../../../resources/textures/no-texture.png");

pub struct FrameGraphicsContext<'a> {
    pub view: TextureView,
    pub encoder: &'a mut CommandEncoder,
}

pub struct SharedGraphicsContext {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub surface: Arc<Surface<'static>>,
    pub surface_format: TextureFormat,
    pub instance: Arc<wgpu::Instance>,
    pub layouts: Arc<BindGroupLayouts>,
    pub window: Arc<Window>,
    pub viewport_texture: texture::Texture,
    pub depth_texture: texture::Texture,
    pub egui_renderer: Arc<Mutex<EguiRenderer>>,
    pub texture_id: Arc<TextureId>,
    pub future_queue: Arc<FutureQueue>,
}

impl SharedGraphicsContext {
    pub const MODEL_UNIFORM_BIND_GROUP_LAYOUT: wgpu::BindGroupLayoutDescriptor<'_> = 
        wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("model_uniform_bind_group_layout"),
            };

    pub fn get_egui_context(&self) -> Context {
        self.egui_renderer.lock().context().clone()
    }
}

impl SharedGraphicsContext {
    pub(crate) fn from_state(
        state: &State,
    ) -> Self {
        SharedGraphicsContext {
            future_queue: state.future_queue.clone(),
            device: state.device.clone(),
            queue: state.queue.clone(),
            instance: state.instance.clone(),
            layouts: state.layouts.clone(),
            window: state.window.clone(),
            viewport_texture: state.viewport_texture.clone(),
            depth_texture: state.depth_texture.clone(),
            egui_renderer: state.egui_renderer.clone(),
            texture_id: state.texture_id.clone(),
            surface: state.surface.clone(),
            surface_format: state.surface_format,
        }
    }

    pub fn create_render_pipline(
        &self,
        shader: &Shader,
        bind_group_layouts: Vec<&BindGroupLayout>,
        label: Option<&str>,
    ) -> RenderPipeline {
        let render_pipeline_layout =
            self.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(label.unwrap_or("Render Pipeline Descriptor")),
                    bind_group_layouts: bind_group_layouts.as_slice(),
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            self.device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(label.unwrap_or("Render Pipeline")),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader.module,
                        entry_point: Some("vs_main"),
                        buffers: &[model::ModelVertex::desc(), InstanceRaw::desc()],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader.module,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Rgba16Float,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
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
        log::debug!("Created new render pipeline");
        render_pipeline
    }
}

#[derive(Default, Clone)]
pub struct Instance {
    pub position: DVec3,
    pub rotation: DQuat,
    pub scale: DVec3,
}

impl Instance {
    pub fn new(position: DVec3, rotation: DQuat, scale: DVec3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    pub fn to_raw(&self) -> InstanceRaw {
        let model_matrix =
            DMat4::from_scale_rotation_translation(self.scale, self.rotation, self.position);
        InstanceRaw {
            model: model_matrix.as_mat4().to_cols_array_2d(),
            normal: Mat3::from_quat(self.rotation.as_quat()).to_cols_array_2d(),
        }
    }

    pub fn from_matrix(mat: DMat4) -> Self {
        let (scale, rotation, position) = mat.to_scale_rotation_translation();
        Instance {
            position,
            rotation,
            scale,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
    normal: [[f32; 3]; 3],
}

impl InstanceRaw {
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<InstanceRaw>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // model
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // normal
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
