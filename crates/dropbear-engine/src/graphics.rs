use std::ops::{Deref, DerefMut};
use crate::{BindGroupLayouts, texture};
use crate::{
    State,
    egui_renderer::EguiRenderer,
};
use dropbear_future_queue::FutureQueue;
use egui::{Context, TextureId};
use glam::{DMat4, DQuat, DVec3, Mat3};
use parking_lot::{Mutex, RwLock};
use std::sync::Arc;
use wgpu::*;
use winit::window::Window;

use crate::mipmap::MipMapper;

pub const NO_TEXTURE: &[u8] = include_bytes!("../../../resources/textures/no-texture.png");

pub struct SharedGraphicsContext {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub surface: Arc<Surface<'static>>,
    pub surface_format: TextureFormat,
    pub surface_config: Arc<RwLock<SurfaceConfiguration>>,
    pub instance: Arc<wgpu::Instance>,
    pub layouts: Arc<BindGroupLayouts>,
    pub window: Arc<Window>,
    pub viewport_texture: texture::Texture,
    pub depth_texture: texture::Texture,
    pub egui_renderer: Arc<Mutex<EguiRenderer>>,
    pub texture_id: Arc<TextureId>,
    pub future_queue: Arc<FutureQueue>,
    pub supports_storage: bool,
    pub mipmapper: Arc<MipMapper>,
    // pub yakui_renderer: Arc<Mutex<yakui_wgpu::YakuiWgpu>>,
    // pub yakui_texture: yakui::TextureId,
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
            mipmapper: state.mipmapper.clone(),
            // yakui_renderer: state.yakui_renderer.clone(),
            // yakui_texture: state.yakui_texture.clone(),
            surface_config: state.config.clone(),
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


/// A wrapper to the [wgpu::CommandEncoder]
pub struct CommandEncoder {
    queue: Arc<Queue>,
    inner: wgpu::CommandEncoder,
}

impl Deref for CommandEncoder {
    type Target = wgpu::CommandEncoder;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for CommandEncoder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl CommandEncoder {
    /// Creates a new instance of a command encoder. 
    pub fn new(graphics: Arc<SharedGraphicsContext>, label: Option<&str>) -> Self {
        Self {
            queue: graphics.queue.clone(),
            inner: graphics.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label }),
        }
    }

    /// Submits the command encoder for execution.
    ///
    /// Panics if an unwinding error is caught, or just returns the error as normal.
    pub fn submit(self) -> anyhow::Result<()> {
        let command_buffer = self.inner.finish();

        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.queue.submit(std::iter::once(command_buffer));
        })) {
            Ok(_) => {Ok(())}
            Err(_) => {
                log::error!("Failed to submit command buffer, device may be lost");
                return Err(anyhow::anyhow!("Command buffer submission failed"));
            }
        }
    }
}