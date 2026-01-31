use std::sync::Arc;
use glam::Vec2;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::window::Window;

pub struct KinoWinitWindowing {
    // mouse
    pub mouse_position: Vec2,
    pub mouse_button: MouseButton,
    pub mouse_press_state: ElementState,

    // keyboard

    // windowing
    _window: Arc<Window>,
    /// The top-left most pixel
    pub viewport_offset: Vec2,
}

impl KinoWinitWindowing {
    /// Creates a new instance of [KinoWinitWindowing] with a specified viewport texture offset.
    pub fn new(window: Arc<Window>) -> Self {
        Self {
            mouse_position: Default::default(),
            mouse_button: MouseButton::Left,
            mouse_press_state: ElementState::Released,
            _window: window,
            viewport_offset: Default::default(),
        }
    }

    pub(crate) fn handle_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let screen_pos = Vec2::new(position.x as f32, position.y as f32);
                self.mouse_position = screen_pos - self.viewport_offset;
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.mouse_button = *button;
                self.mouse_press_state = *state;
            }
            WindowEvent::Resized(_) => {}
            WindowEvent::KeyboardInput { .. } => {}
            WindowEvent::MouseWheel { .. } => {}
            WindowEvent::ScaleFactorChanged { .. } => {}
            _ => {}
        }
    }
}