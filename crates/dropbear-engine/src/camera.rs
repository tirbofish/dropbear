//! Camera and components related to cameras.

use std::sync::Arc;

use glam::{DMat4, DQuat, DVec3, Mat4};
use serde::{Deserialize, Serialize};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BufferBindingType, ShaderStages,
};

use crate::{buffer::UniformBuffer, graphics::SharedGraphicsContext};

/// Matrix that converts OpenGL (from [`glam`]) to [`wgpu`] values
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 0.5, 0.0],
    [0.0, 0.0, 0.5, 1.0],
];

/// Shared tuning data for camera movement and projection.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CameraSettings {
    pub speed: f64,
    pub sensitivity: f64,
    pub fov_y: f64,
}

impl CameraSettings {
    pub const fn new(speed: f64, sensitivity: f64, fov_y: f64) -> Self {
        Self {
            speed,
            sensitivity,
            fov_y,
        }
    }
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self::new(1.0, 0.002, 75.0)
    }
}

/// The basic values of a Camera.
#[derive(Debug, Clone)]
pub struct Camera {
    /// The name of the camera
    pub label: String,

    /// Eye of camera / Position
    pub eye: DVec3,
    /// Target of camera / Looking at
    pub target: DVec3,
    /// Up
    pub up: DVec3,
    /// Aspect ratio
    pub aspect: f64,
    /// Near buffer?
    pub znear: f64,
    /// Far buffer?
    pub zfar: f64,
    /// Yaw (rotation)
    pub yaw: f64,
    /// Pitch (rotation)
    pub pitch: f64,

    /// Tuning values that control movement and projection
    pub settings: CameraSettings,

    /// Uniform/interface for Rust and the GPU
    pub uniform: CameraUniform,

    buffer: UniformBuffer<CameraUniform>,
    pub bind_group: BindGroup,

    /// View matrix
    pub view_mat: DMat4,
    /// Projection Matrix
    pub proj_mat: DMat4,
}

/// A simple builder/struct that allows you to build a [`Camera`]
pub struct CameraBuilder {
    pub eye: DVec3,
    pub target: DVec3,
    pub up: DVec3,
    pub aspect: f64,
    pub znear: f64,
    pub zfar: f64,
    pub settings: CameraSettings,
}

impl Camera {
    pub const CAMERA_BIND_GROUP_LAYOUT: wgpu::BindGroupLayoutDescriptor<'_> =
        BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX.union(ShaderStages::FRAGMENT),
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        };

    /// Creates a new camera
    pub fn new(
        graphics: Arc<SharedGraphicsContext>,
        builder: CameraBuilder,
        label: Option<&str>,
    ) -> Self {
        let mut uniform = CameraUniform::new();

        let dir = (builder.target - builder.eye).normalize();
        let pitch = dir.y.clamp(-1.0, 1.0).asin();
        let yaw = dir.z.atan2(dir.x);

        let view = DMat4::look_at_lh(builder.eye, builder.target, builder.up);
        let proj = DMat4::perspective_infinite_reverse_lh(
            builder.settings.fov_y.to_radians(),
            builder.aspect,
            builder.znear,
        );

        let view_mat = view;
        let proj_mat = proj;

        let mvp = DMat4::from_cols_array_2d(&OPENGL_TO_WGPU_MATRIX) * proj * view;
        uniform.view_proj = mvp.as_mat4().to_cols_array_2d();

        let buffer = UniformBuffer::new(&graphics.device, "camera uniform");

        let bind_group = graphics.device.create_bind_group(&BindGroupDescriptor {
            layout: &graphics.layouts.camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.buffer().as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let camera = Self {
            eye: builder.eye,
            target: builder.target,
            up: builder.up,
            aspect: builder.aspect,
            znear: builder.znear,
            zfar: builder.zfar,
            uniform,
            buffer,
            bind_group,
            yaw,
            pitch,
            settings: builder.settings,
            label: if let Some(l) = label {
                l.to_string()
            } else {
                String::from("Camera")
            },
            view_mat,
            proj_mat,
        };

        log::debug!(
            "Created new camera{}",
            if let Some(l) = label {
                format!(" with the label {}", l)
            } else {
                String::new()
            }
        );
        camera
    }

    /// Creates a default camera
    pub fn predetermined(graphics: Arc<SharedGraphicsContext>, label: Option<&str>) -> Self {
        Self::new(
            graphics.clone(),
            CameraBuilder {
                eye: DVec3::new(0.0, 1.0, 2.0),
                target: DVec3::new(0.0, 0.0, 0.0),
                up: DVec3::Y,
                aspect: (graphics.window.inner_size().width / graphics.window.inner_size().height)
                    .into(),
                znear: 0.1,
                zfar: 100.0,
                settings: CameraSettings::default(),
            },
            label,
        )
    }

    pub fn rotation(&self) -> DQuat {
        let yaw = DQuat::from_axis_angle(DVec3::Y, self.yaw);
        let pitch = DQuat::from_axis_angle(DVec3::X, self.pitch);
        yaw * pitch
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        self.buffer.buffer()
    }

    pub fn forward(&self) -> DVec3 {
        (self.target - self.eye).normalize()
    }

    pub fn position(&self) -> DVec3 {
        self.eye
    }

    /// Prints out the values of the camera.
    pub fn debug_camera_state(&self) {
        let camera = self;
        log::debug!("Camera state:");
        log::debug!("  Eye: {:?}", camera.eye);
        log::debug!("  Target: {:?}", camera.target);
        log::debug!("  Up: {:?}", camera.up);
        log::debug!("  FOV Y: {}", camera.settings.fov_y);
        log::debug!("  Aspect: {}", camera.aspect);
        log::debug!("  Z Near: {}", camera.znear);
        log::debug!("  Proj Mat finite: {}", camera.proj_mat.is_finite());
        log::debug!("  View Mat finite: {}", camera.view_mat.is_finite());
    }

    fn build_vp(&mut self) -> DMat4 {
        let view = DMat4::look_at_lh(self.eye, self.target, self.up);
        let proj = DMat4::perspective_infinite_reverse_lh(
            self.settings.fov_y.to_radians(),
            self.aspect,
            self.znear,
        );

        self.view_mat = view;
        self.proj_mat = proj;

        DMat4::from_cols_array_2d(&OPENGL_TO_WGPU_MATRIX) * proj * view
    }

    pub fn update(&mut self, graphics: Arc<SharedGraphicsContext>) {
        puffin::profile_function!();
        self.update_view_proj();
        self.buffer.write(&graphics.queue, &self.uniform);
    }

    pub fn update_view_proj(&mut self) {
        puffin::profile_function!();
        let mut uniform = self.uniform;
        uniform.update(self);
        self.uniform = uniform;
    }

    pub fn move_forwards(&mut self, dt: f32) {
        let forward = (self.target - self.eye).normalize();
        self.eye += forward * self.settings.speed * dt as f64;
        self.target += forward * self.settings.speed * dt as f64;
    }

    pub fn move_back(&mut self, dt: f32) {
        let forward = (self.target - self.eye).normalize();
        self.eye -= forward * self.settings.speed * dt as f64;
        self.target -= forward * self.settings.speed * dt as f64;
    }

    pub fn move_right(&mut self, dt: f32) {
        let forward = (self.target - self.eye).normalize();
        // LH: right = up.cross(forward)
        let right = self.up.cross(forward).normalize();
        self.eye += right * self.settings.speed * dt as f64;
        self.target += right * self.settings.speed * dt as f64;
    }

    pub fn move_left(&mut self, dt: f32) {
        let forward = (self.target - self.eye).normalize();
        let right = self.up.cross(forward).normalize();
        self.eye -= right * self.settings.speed * dt as f64;
        self.target -= right * self.settings.speed * dt as f64;
    }

    pub fn move_up(&mut self, dt: f32) {
        let up = self.up.normalize();
        self.eye += up * self.settings.speed * dt as f64;
        self.target += up * self.settings.speed * dt as f64;
    }

    pub fn move_down(&mut self, dt: f32) {
        let up = self.up.normalize();
        self.eye -= up * self.settings.speed * dt as f64;
        self.target -= up * self.settings.speed * dt as f64;
    }

    pub fn track_mouse_delta(&mut self, dx: f64, dy: f64) {
        let sensitivity = self.settings.sensitivity;
        self.yaw -= dx * sensitivity;
        self.pitch -= dy * sensitivity;
        let max_pitch = std::f64::consts::FRAC_PI_2 - 0.01;
        self.pitch = self.pitch.clamp(-max_pitch, max_pitch);
        let dir = DVec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        );
        self.target = self.eye + dir;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_position: [f32; 4],
    pub view: [[f32; 4]; 4],
    pub view_proj: [[f32; 4]; 4],
    pub inv_proj: [[f32; 4]; 4],
    pub inv_view: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view: Mat4::IDENTITY.to_cols_array_2d(),
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            inv_proj: Mat4::IDENTITY.to_cols_array_2d(),
            inv_view: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update(&mut self, camera: &mut Camera) {
        self.view_position = camera.eye.as_vec3().extend(1.0).to_array();

        let vp = camera.build_vp();
        let view = camera.view_mat;
        let proj = camera.proj_mat;

        let wgpu_matrix = DMat4::from_cols_array_2d(&OPENGL_TO_WGPU_MATRIX);
        self.view = view.as_mat4().to_cols_array_2d();
        self.view_proj = vp.as_mat4().to_cols_array_2d();

        self.inv_proj = (wgpu_matrix * proj).inverse().as_mat4().to_cols_array_2d();
        self.inv_view = view.inverse().as_mat4().to_cols_array_2d();
    }
}
