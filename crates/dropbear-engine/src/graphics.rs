use crate::{BindGroupLayouts, texture};
use crate::{
    State,
    egui_renderer::EguiRenderer,
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
    pub supports_storage: bool,
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
            supports_storage: state.supports_storage,
        }
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

/// Maps to
/// ```wgsl
/// struct InstanceInput {
///     @location(5) model_matrix_0: vec4<f32>,
///     @location(6) model_matrix_1: vec4<f32>,
///     @location(7) model_matrix_2: vec4<f32>,
///     @location(8) model_matrix_3: vec4<f32>,
///
///     @location(9) normal_matrix_0: vec3<f32>,
///     @location(10) normal_matrix_1: vec3<f32>,
///     @location(11) normal_matrix_2: vec3<f32>,
/// };
/// ```
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
                // model_matrix_0
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // model_matrix_1
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // model_matrix_2
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // model_matrix_3
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },

                // normal_matrix_0
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },

                // normal_matrix_1
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },

                // normal_matrix_2
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
