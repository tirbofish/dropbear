use crate::scene::RuntimeScene;
use dropbear_engine::gilrs::{Button, GamepadId};
use dropbear_engine::input::{Controller, Keyboard, Mouse};
use dropbear_engine::winit::dpi::PhysicalPosition;
use dropbear_engine::winit::event::MouseButton;
use dropbear_engine::winit::event_loop::ActiveEventLoop;
use dropbear_engine::winit::keyboard::KeyCode;

impl Keyboard for RuntimeScene {
    fn key_down(&mut self, key: KeyCode, _event_loop: &ActiveEventLoop) {
        self.input_state.pressed_keys.insert(key);
    }

    fn key_up(&mut self, key: KeyCode, _event_loop: &ActiveEventLoop) {
        self.input_state.pressed_keys.remove(&key);
    }
}

impl Mouse for RuntimeScene {
    fn mouse_move(&mut self, position: PhysicalPosition<f64>, delta: Option<(f64, f64)>) {
        let delta = if delta.is_none() {
            if let Some(last_pos) = self.input_state.last_mouse_pos {
                Some((last_pos.0 - position.x, last_pos.1 - position.y))
            } else {
                None
            }
        } else {
            delta
        };

        self.input_state.mouse_delta = delta;
        self.input_state.mouse_pos = (position.x, position.y);
        self.input_state.last_mouse_pos = Some(<(f64, f64)>::from(position));
    }

    fn mouse_down(&mut self, button: MouseButton) {
        self.input_state.mouse_button.insert(button);
    }

    fn mouse_up(&mut self, button: MouseButton) {
        self.input_state.mouse_button.remove(&button);
    }
}

impl Controller for RuntimeScene {
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
