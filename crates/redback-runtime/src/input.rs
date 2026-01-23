use gilrs::{Button, GamepadId};
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use dropbear_engine::input::{Controller, Keyboard, Mouse};
use crate::PlayMode;
use eucalyptus_core::ui::UI_CONTEXT;

impl Keyboard for PlayMode {
    fn key_down(&mut self, key: KeyCode, _event_loop: &ActiveEventLoop) {
        self.input_state.pressed_keys.insert(key);
    }

    fn key_up(&mut self, key: KeyCode, _event_loop: &ActiveEventLoop) {
        self.input_state.pressed_keys.remove(&key);
    }
}

impl Mouse for PlayMode {
    fn mouse_move(&mut self, position: PhysicalPosition<f64>, delta: Option<(f64, f64)>) {
        let delta = if delta.is_none() {
            if let Some(last_pos) = self.input_state.last_mouse_pos {
                Some((position.x - last_pos.0, position.y - last_pos.1))
            } else {
                None
            }
        } else {
            delta
        };

        self.input_state.mouse_delta = delta;
        self.input_state.mouse_pos = (position.x, position.y);
        self.input_state.last_mouse_pos = Some(<(f64, f64)>::from(position));

        UI_CONTEXT.with(|ctx| {
            let yak = ctx.borrow();
            let mut yakui = yak.yakui_state.lock();
            let relative_x = (position.x as f32) - self.viewport_offset.0;
            let relative_y = (position.y as f32) - self.viewport_offset.1;
            
            yakui.handle_event(yakui::event::Event::CursorMoved(Some(yakui::geometry::Vec2::new(relative_x, relative_y))));
        });
    }

    fn mouse_down(&mut self, button: MouseButton) {
        self.input_state.mouse_button.insert(button);
        
        UI_CONTEXT.with(|ctx| {
            let yak = ctx.borrow();
            let mut yakui = yak.yakui_state.lock();
            let btn = match button {
                MouseButton::Left => Some(yakui::input::MouseButton::One),
                MouseButton::Right => Some(yakui::input::MouseButton::Two),
                MouseButton::Middle => Some(yakui::input::MouseButton::Three),
                _ => None,
            };
            if let Some(b) = btn {
                yakui.handle_event(yakui::event::Event::MouseButtonChanged {
                    button: b,
                    down: true,
                });
            }
        });
    }

    fn mouse_up(&mut self, button: MouseButton) {
        self.input_state.mouse_button.remove(&button);
        
        UI_CONTEXT.with(|ctx| {
            let yak = ctx.borrow();
            let mut yakui = yak.yakui_state.lock();
            let btn = match button {
                MouseButton::Left => Some(yakui::input::MouseButton::One),
                MouseButton::Right => Some(yakui::input::MouseButton::Two),
                MouseButton::Middle => Some(yakui::input::MouseButton::Three),
                _ => None,
            };
            if let Some(b) = btn {
                yakui.handle_event(yakui::event::Event::MouseButtonChanged {
                    button: b,
                    down: false,
                });
            }
        });
    }
}

impl Controller for PlayMode {
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