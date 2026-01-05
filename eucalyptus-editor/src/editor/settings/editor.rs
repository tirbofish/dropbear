//! The scene for a window that opens up settings related to the eucalyptus-editor. 

use egui::{CentralPanel};
use gilrs::{Button, GamepadId};
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::{WindowId};
use dropbear_engine::graphics::RenderContext;
use dropbear_engine::input::{Controller, Keyboard, Mouse};
use dropbear_engine::scene::{Scene, SceneCommand};
use eucalyptus_core::input::InputState;

pub struct EditorSettingsWindow {
    scene_command: SceneCommand,
    input_state: InputState,
    window: Option<WindowId>,
}

impl EditorSettingsWindow {
    pub fn new() -> Self {
        Self {
            scene_command: SceneCommand::None,
            input_state: Default::default(),
            window: None,
        }
    }
}

impl Scene for EditorSettingsWindow {
    fn load(&mut self, graphics: &mut RenderContext) {
        self.window = Some(graphics.shared.window.id());
    }

    fn physics_update(&mut self, _dt: f32, _graphics: &mut RenderContext) {}

    fn update(&mut self, _dt: f32, graphics: &mut RenderContext) {
        CentralPanel::default().show(&graphics.shared.get_egui_context(), |ui| {
            ui.label("Hello Debug Window!");
        });

        self.window = Some(graphics.shared.window.id());
    }

    fn render(&mut self, _graphics: &mut RenderContext) {
    }

    fn exit(&mut self, _event_loop: &ActiveEventLoop) {}

    fn run_command(&mut self) -> SceneCommand {
        std::mem::replace(&mut self.scene_command, SceneCommand::None)
    }
}

impl Keyboard for EditorSettingsWindow {
    fn key_down(&mut self, key: KeyCode, _event_loop: &ActiveEventLoop) {
        match key {
            KeyCode::Escape => {
                if let Some(id) = self.window {
                    self.scene_command = SceneCommand::CloseWindow(id);
                }
            }
            _ => {
                self.input_state.pressed_keys.insert(key);
            }
        }
        self.input_state.pressed_keys.insert(key);
    }

    fn key_up(&mut self, key: KeyCode, _event_loop: &ActiveEventLoop) {
        self.input_state.pressed_keys.remove(&key);
    }
}

impl Mouse for EditorSettingsWindow {
    fn mouse_move(&mut self, position: PhysicalPosition<f64>, delta: Option<(f64, f64)>) {
        self.input_state.last_mouse_pos = Some(<(f64, f64)>::from(position));
        self.input_state.mouse_delta = delta;
        self.input_state.mouse_pos = (position.x, position.y);
    }

    fn mouse_down(&mut self, button: MouseButton) {
        self.input_state.mouse_button.insert(button);
    }

    fn mouse_up(&mut self, button: MouseButton) {
        self.input_state.mouse_button.remove(&button);
    }
}

impl Controller for EditorSettingsWindow {
    fn button_down(&mut self, button: Button, id: GamepadId) {
        self.input_state
            .pressed_buttons
            .entry(id)
            .or_default()
            .insert(button);
    }

    fn button_up(&mut self, button: Button, id: GamepadId) {
        if let Some(buttons) = self.input_state.pressed_buttons.get_mut(&id) {
            buttons.remove(&button);
        }
    }

    fn left_stick_changed(&mut self, x: f32, y: f32, id: GamepadId) {
        self.input_state.left_stick_position.insert(id, (x, y));
    }

    fn right_stick_changed(&mut self, x: f32, y: f32, id: GamepadId) {
        self.input_state.right_stick_position.insert(id, (x, y));
    }

    fn on_connect(&mut self, id: GamepadId) {
        self.input_state.connected_gamepads.insert(id);
    }

    fn on_disconnect(&mut self, id: GamepadId) {
        self.input_state.connected_gamepads.remove(&id);
        self.input_state.pressed_buttons.remove(&id);
        self.input_state.left_stick_position.remove(&id);
        self.input_state.right_stick_position.remove(&id);
    }
}