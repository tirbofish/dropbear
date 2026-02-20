use crate::KinoState;
use glam::Vec2;
use std::sync::Arc;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::window::Window;

pub struct KinoWinitWindowing {
    // mouse
    pub mouse_position: Vec2,
    pub mouse_button: MouseButton,
    pub mouse_press_state: ElementState,

    // windowing
    window: Arc<Window>,
    /// The top-left most pixel
    pub viewport_offset: Vec2,
    /// Scale from screen-space to viewport texture space
    pub viewport_scale: Vec2,

    pub scale_factor: f32,
    pub auto_scale: bool,
}

impl KinoWinitWindowing {
    /// Creates a new instance of [KinoWinitWindowing] with a specified viewport texture offset.
    pub fn new(window: Arc<Window>, scale_factor: Option<f32>) -> Self {
        let auto_scale = scale_factor.is_none();
        let scale_factor = scale_factor.unwrap_or(window.scale_factor() as f32);
        Self {
            mouse_position: Default::default(),
            mouse_button: MouseButton::Left,
            mouse_press_state: ElementState::Released,
            window,
            viewport_offset: Default::default(),
            viewport_scale: Vec2::ONE,
            scale_factor,
            auto_scale,
        }
    }

    /// Get the physical size of the window in pixels
    pub fn physical_size(&self) -> (u32, u32) {
        let size = self.window.inner_size();
        (size.width, size.height)
    }

    /// Get the logical size of the window (physical size / scale_factor)
    pub fn logical_size(&self) -> Vec2 {
        let (w, h) = self.physical_size();
        Vec2::new(w as f32, h as f32) / self.scale_factor
    }

    pub(crate) fn handle_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let screen_pos = Vec2::new(position.x as f32, position.y as f32);
                let local = screen_pos - self.viewport_offset;
                self.mouse_position = local * self.viewport_scale;
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.mouse_button = *button;
                self.mouse_press_state = *state;
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if self.auto_scale {
                    return;
                }
                self.scale_factor = *scale_factor as f32;
            }
            WindowEvent::Resized(_) => {}
            WindowEvent::KeyboardInput { .. } => {}
            WindowEvent::MouseWheel { .. } => {}
            _ => {}
        }
    }
}

impl KinoState {
    /// Get the current DPI scale factor
    pub fn scale_factor(&self) -> f32 {
        self.windowing.scale_factor
    }

    /// Scale a logical size to physical pixels
    pub fn to_physical(&self, logical: f32) -> f32 {
        logical * self.windowing.scale_factor
    }

    /// Scale a logical Vec2 to physical pixels
    pub fn to_physical_vec(&self, logical: Vec2) -> Vec2 {
        logical * self.windowing.scale_factor
    }

    /// Scale physical pixels to logical size
    pub fn to_logical(&self, physical: f32) -> f32 {
        physical / self.windowing.scale_factor
    }

    /// Scale physical Vec2 to logical size
    pub fn to_logical_vec(&self, physical: Vec2) -> Vec2 {
        physical / self.windowing.scale_factor
    }

    pub fn set_scale_factor(&mut self, factor: Option<f32>) {
        if let Some(factor) = factor {
            self.windowing.scale_factor = factor;
            self.windowing.auto_scale = false;
        } else {
            self.windowing.auto_scale = true;
        }
    }
}
