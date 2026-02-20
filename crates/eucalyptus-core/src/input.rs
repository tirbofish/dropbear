//! Input management and input state.

pub mod gamepad;

use crate::scripting::jni::utils::ToJObject;
use crate::scripting::native::DropbearNativeError;
use crate::scripting::result::DropbearNativeResult;
use dropbear_engine::gilrs::{Button, GamepadId};
use glam::Vec2;
use jni::JNIEnv;
use jni::objects::JObject;
use jni::sys::jlong;
use std::sync::Arc;
use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};
use winit::window::Window;
use winit::{event::MouseButton, keyboard::KeyCode};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Gamepad {
    id: i32,
    left_stick_pos: Vec2,
    right_stick_pos: Vec2,
}

/// Shows the information about the input at that current time.
#[derive(Clone, Debug)]
pub struct InputState {
    pub window: Option<Arc<Window>>,

    #[allow(dead_code)]
    pub last_key_press_times: HashMap<KeyCode, Instant>,
    #[allow(dead_code)]
    pub double_press_threshold: Duration,

    pub mouse_pos: (f64, f64),
    pub mouse_button: HashSet<MouseButton>,
    pub pressed_keys: HashSet<KeyCode>,
    pub mouse_delta: Option<(f64, f64)>,
    pub is_cursor_locked: bool,
    pub is_cursor_hidden: bool,

    /// This is not used, the mouse delta and/or the mouse position is used instead
    pub last_mouse_pos: Option<(f64, f64)>,

    pub connected_gamepads: HashSet<GamepadId>,
    pub pressed_buttons: HashMap<GamepadId, HashSet<Button>>,
    pub left_stick_position: HashMap<GamepadId, (f32, f32)>,
    pub right_stick_position: HashMap<GamepadId, (f32, f32)>,

    pub cached_gamepads: Vec<Gamepad>,
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

impl InputState {
    pub fn new() -> Self {
        Self {
            window: None,
            mouse_pos: Default::default(),
            mouse_button: Default::default(),
            pressed_keys: HashSet::new(),
            last_key_press_times: HashMap::new(),
            double_press_threshold: Duration::from_millis(300),
            mouse_delta: None,
            is_cursor_locked: false,
            is_cursor_hidden: false,
            last_mouse_pos: Default::default(),
            connected_gamepads: Default::default(),
            pressed_buttons: Default::default(),
            left_stick_position: Default::default(),
            right_stick_position: Default::default(),
            cached_gamepads: vec![],
        }
    }

    pub fn lock_cursor(&mut self, toggle: bool) {
        self.is_cursor_locked = toggle;
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    #[allow(clippy::unnecessary_map_or)]
    pub fn is_button_pressed(&self, gamepad_id: GamepadId, button: Button) -> bool {
        self.pressed_buttons
            .get(&gamepad_id)
            .map_or(false, |buttons| buttons.contains(&button))
    }

    pub fn get_left_stick(&self, gamepad_id: GamepadId) -> (f32, f32) {
        self.left_stick_position
            .get(&gamepad_id)
            .copied()
            .unwrap_or((0.0, 0.0))
    }

    pub fn get_right_stick(&self, gamepad_id: GamepadId) -> (f32, f32) {
        self.right_stick_position
            .get(&gamepad_id)
            .copied()
            .unwrap_or((0.0, 0.0))
    }

    pub fn is_gamepad_connected(&self, gamepad_id: GamepadId) -> bool {
        self.connected_gamepads.contains(&gamepad_id)
    }
}

pub mod shared {
    use super::*;
    use crate::command::{CommandBuffer, WindowCommand};
    use crate::types::NVector2;
    use crossbeam_channel::Sender;

    pub fn map_ordinal_to_mouse_button(ordinal: i32) -> Option<MouseButton> {
        match ordinal {
            0 => Some(MouseButton::Left),
            1 => Some(MouseButton::Right),
            2 => Some(MouseButton::Middle),
            3 => Some(MouseButton::Back),
            4 => Some(MouseButton::Forward),
            ordinal if ordinal >= 0 => Some(MouseButton::Other(ordinal as u16)),
            _ => None,
        }
    }

    pub fn is_key_pressed(input: &InputState, key_ordinal: i32) -> bool {
        if let Some(key) = crate::utils::keycode_from_ordinal(key_ordinal) {
            input.is_key_pressed(key)
        } else {
            false
        }
    }

    pub fn is_mouse_button_pressed(input: &InputState, btn_ordinal: i32) -> bool {
        if let Some(btn) = map_ordinal_to_mouse_button(btn_ordinal) {
            input.mouse_button.contains(&btn)
        } else {
            false
        }
    }

    pub fn get_mouse_position(input: &InputState) -> NVector2 {
        NVector2 {
            x: input.mouse_pos.0,
            y: input.mouse_pos.1,
        }
    }

    pub fn get_mouse_delta(input: &InputState) -> NVector2 {
        input
            .mouse_delta
            .map(NVector2::from)
            .unwrap_or(NVector2 { x: 0.0, y: 0.0 })
    }

    pub fn get_last_mouse_pos(input: &InputState) -> NVector2 {
        input
            .last_mouse_pos
            .map(NVector2::from)
            .unwrap_or(NVector2 { x: 0.0, y: 0.0 })
    }

    pub fn set_cursor_locked(
        input: &mut InputState,
        sender: &Sender<CommandBuffer>,
        locked: bool,
    ) -> DropbearNativeResult<()> {
        input.is_cursor_locked = locked;

        sender
            .send(CommandBuffer::WindowCommand(WindowCommand::WindowGrab(
                locked,
            )))
            .map_err(|_| DropbearNativeError::SendError)?;

        Ok(())
    }

    pub fn set_cursor_hidden(
        input: &mut InputState,
        sender: &Sender<CommandBuffer>,
        hidden: bool,
    ) -> DropbearNativeResult<()> {
        input.is_cursor_hidden = hidden;
        sender
            .send(CommandBuffer::WindowCommand(WindowCommand::HideCursor(
                hidden,
            )))
            .map_err(|_| DropbearNativeError::SendError)?;
        Ok(())
    }

    pub fn get_connected_gamepads(input: &InputState) -> Vec<u64> {
        input
            .connected_gamepads
            .iter()
            .map(|id| Into::<usize>::into(*id) as u64)
            .collect()
    }
}

#[repr(C)]
struct ConnectedGamepadIds {
    ids: Vec<u64>,
}

impl ToJObject for ConnectedGamepadIds {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let output = env
            .new_long_array(self.ids.len() as i32)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        let long_ids: Vec<jlong> = self.ids.iter().map(|&id| id as jlong).collect();
        env.set_long_array_region(&output, 0, &long_ids)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;
        Ok(JObject::from(output))
    }
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.input.InputStateNative",
        func = "printInputState"
    ),
    c
)]
fn print_input_state(
    #[dropbear_macro::define(crate::ptr::InputStatePtr)] input: &InputState,
) -> DropbearNativeResult<()> {
    println!("Input State: {:?}", input);
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.input.InputStateNative", func = "isKeyPressed"),
    c
)]
fn is_key_pressed(
    #[dropbear_macro::define(crate::ptr::InputStatePtr)] input: &InputState,
    key_code: i32,
) -> DropbearNativeResult<bool> {
    Ok(shared::is_key_pressed(input, key_code))
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.input.InputStateNative",
        func = "getMousePosition"
    ),
    c
)]
fn get_mouse_position(
    #[dropbear_macro::define(crate::ptr::InputStatePtr)] input: &InputState,
) -> DropbearNativeResult<crate::types::NVector2> {
    Ok(shared::get_mouse_position(input))
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.input.InputStateNative",
        func = "isMouseButtonPressed"
    ),
    c
)]
fn is_mouse_button_pressed(
    #[dropbear_macro::define(crate::ptr::InputStatePtr)] input: &InputState,
    button_ordinal: i32,
) -> DropbearNativeResult<bool> {
    Ok(shared::is_mouse_button_pressed(input, button_ordinal))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.input.InputStateNative", func = "getMouseDelta"),
    c
)]
fn get_mouse_delta(
    #[dropbear_macro::define(crate::ptr::InputStatePtr)] input: &InputState,
) -> DropbearNativeResult<crate::types::NVector2> {
    Ok(shared::get_mouse_delta(input))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.input.InputStateNative", func = "isCursorLocked"),
    c
)]
fn is_cursor_locked(
    #[dropbear_macro::define(crate::ptr::InputStatePtr)] input: &InputState,
) -> DropbearNativeResult<bool> {
    Ok(input.is_cursor_locked)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.input.InputStateNative",
        func = "setCursorLocked"
    ),
    c
)]
fn set_cursor_locked(
    #[dropbear_macro::define(crate::ptr::CommandBufferPtr)]
    command_buffer: &crate::ptr::CommandBufferUnwrapped,
    #[dropbear_macro::define(crate::ptr::InputStatePtr)] input: &mut InputState,
    locked: bool,
) -> DropbearNativeResult<()> {
    shared::set_cursor_locked(input, command_buffer, locked)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.input.InputStateNative",
        func = "getLastMousePos"
    ),
    c
)]
fn get_last_mouse_pos(
    #[dropbear_macro::define(crate::ptr::InputStatePtr)] input: &InputState,
) -> DropbearNativeResult<crate::types::NVector2> {
    Ok(shared::get_last_mouse_pos(input))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.input.InputStateNative", func = "isCursorHidden"),
    c
)]
fn is_cursor_hidden(
    #[dropbear_macro::define(crate::ptr::InputStatePtr)] input: &InputState,
) -> DropbearNativeResult<bool> {
    Ok(input.is_cursor_hidden)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.input.InputStateNative",
        func = "setCursorHidden"
    ),
    c
)]
fn set_cursor_hidden(
    #[dropbear_macro::define(crate::ptr::CommandBufferPtr)]
    command_buffer: &crate::ptr::CommandBufferUnwrapped,
    #[dropbear_macro::define(crate::ptr::InputStatePtr)] input: &mut InputState,
    hidden: bool,
) -> DropbearNativeResult<()> {
    shared::set_cursor_hidden(input, command_buffer, hidden)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.input.InputStateNative",
        func = "getConnectedGamepads"
    ),
    c
)]
fn get_connected_gamepads(
    #[dropbear_macro::define(crate::ptr::InputStatePtr)] input: &InputState,
) -> DropbearNativeResult<ConnectedGamepadIds> {
    Ok(ConnectedGamepadIds {
        ids: shared::get_connected_gamepads(input),
    })
}
