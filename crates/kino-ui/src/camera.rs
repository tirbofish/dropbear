use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2};
use crate::math::Rect;

pub struct Camera2D {
    position: Vec2,
    zoom: f32,
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

impl Camera2D {
    /// Returns the orthographic view-projection matrix for the current camera state
    pub(crate) fn view_proj(&self, screen_size: Vec2) -> Mat4 {
        let width = screen_size.x / self.zoom;
        let height = screen_size.y / self.zoom;

        let left = self.position.x;
        let right = self.position.x + width;
        let top = self.position.y;
        let bottom = self.position.y + height;

        Mat4::orthographic_lh(left, right, bottom, top, -1.0, 1.0)
    }

    /// Set the camera's position (top-left corner of view)
    pub fn target(&mut self, position: Vec2) {
        self.position = position;
    }

    /// Center the camera on a position
    pub fn center(&mut self, position: Vec2, screen_size: Vec2) {
        self.position = position - screen_size / (2.0 * self.zoom);
    }

    /// Set zoom level, clamped between 0.1 & 10.0 to avoid insanity
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.1, 10.0);
    }

    /// Returns the viewport rectangle in world coordinates, factoring in zoom
    /// Useful for culling or visibility checks
    pub fn viewport(&self, screen_size: Vec2) -> Rect {
        let size = screen_size / self.zoom;
        Rect::new(self.position, size)
    }
    /// Converts a point from world space to screen space (pixels)
    pub fn world_to_screen(&self, world: Vec2) -> Vec2 {
        (world - self.position) * self.zoom
    }

    /// Converts a point from screen space back to world space
    pub fn screen_to_world(&self, screen: Vec2) -> Vec2 {
        screen / self.zoom + self.position
    }
}

pub struct CameraRendering {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}