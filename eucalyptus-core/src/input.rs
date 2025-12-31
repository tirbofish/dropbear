//! Input management and input state.

pub mod gamepad;

use dropbear_engine::gilrs::{Button, GamepadId};
use std::sync::Arc;
use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};
use glam::Vec2;
use winit::window::Window;
use winit::{event::MouseButton, keyboard::KeyCode};

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
    use crossbeam_channel::Sender;
    use crate::command::{CommandBuffer, WindowCommand};
    use crate::scripting::native::DropbearNativeError;
    use crate::scripting::result::DropbearNativeResult;
    use crate::types::Vector2;
    use super::*;

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

    pub fn get_mouse_position(input: &InputState) -> Vector2 {
        Vector2 {
            x: input.mouse_pos.0,
            y: input.mouse_pos.1,
        }
    }

    pub fn get_mouse_delta(input: &InputState) -> Vector2 {
        input.mouse_delta.map(Vector2::from).unwrap_or(Vector2 { x: 0.0, y: 0.0 })
    }

    pub fn get_last_mouse_pos(input: &InputState) -> Vector2 {
        input.last_mouse_pos.map(Vector2::from).unwrap_or(Vector2 { x: 0.0, y: 0.0 })
    }

    pub fn set_cursor_locked(
        input: &mut InputState,
        sender: &Sender<CommandBuffer>,
        locked: bool
    ) -> DropbearNativeResult<()> {
        input.is_cursor_locked = locked;

        sender.send(CommandBuffer::WindowCommand(WindowCommand::WindowGrab(locked)))
            .map_err(|_| DropbearNativeError::SendError)?;

        Ok(())
    }

    pub fn set_cursor_hidden(
        input: &mut InputState,
        sender: &Sender<CommandBuffer>,
        hidden: bool
    ) -> DropbearNativeResult<()> {
        input.is_cursor_hidden = hidden;
        sender.send(CommandBuffer::WindowCommand(WindowCommand::HideCursor(hidden)))
            .map_err(|_| DropbearNativeError::SendError)?;
        Ok(())
    }

    pub fn get_connected_gamepads(input: &InputState) -> Vec<u64> {
        input.connected_gamepads.iter()
            .map(|id| Into::<usize>::into(*id) as u64)
            .collect()
    }
}

pub mod jni {
    #![allow(non_snake_case)]
    use jni::JNIEnv;
    use jni::objects::{JClass, JObject};
    use jni::sys::{jboolean, jint, jlong, jlongArray, jobject};
    use crate::input::InputState;
    use crate::command::CommandBuffer;
    use crate::scripting::jni::utils::ToJObject;

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_input_InputStateNative_printInputState(
        _env: JNIEnv,
        _class: JClass,
        input_ptr: jlong,
    ) {
        let input = crate::convert_ptr!(input_ptr => InputState);
        println!("Input State: {:?}", input);
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_input_InputStateNative_isKeyPressed(
        _env: JNIEnv,
        _class: JClass,
        input_ptr: jlong,
        key_code: jint,
    ) -> jboolean {
        let input = crate::convert_ptr!(input_ptr => InputState);
        if super::shared::is_key_pressed(&input, key_code) { 1 } else { 0 }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_input_InputStateNative_getMousePosition(
        mut env: JNIEnv,
        _class: JClass,
        input_ptr: jlong,
    ) -> jobject {
        let input = crate::convert_ptr!(input_ptr => InputState);
        let vec = super::shared::get_mouse_position(&input);
        match vec.to_jobject(&mut env) {
            Ok(obj) => obj.into_raw(),
            Err(_) => std::ptr::null_mut(),
        }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_input_InputStateNative_isMouseButtonPressed(
        mut env: JNIEnv,
        _class: JClass,
        input_ptr: jlong,
        mouse_button_obj: JObject,
    ) -> jboolean {
        let input = crate::convert_ptr!(input_ptr => InputState);

        let ordinal_res = env.call_method(&mouse_button_obj, "ordinal", "()I", &[]);

        let ordinal = match ordinal_res.and_then(|v| v.i()) {
            Ok(i) => i,
            Err(e) => {
                eprintln!("Failed to get MouseButton ordinal: {:?}", e);
                return 0;
            }
        };

        if super::shared::is_mouse_button_pressed(&input, ordinal) { 1 } else { 0 }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_input_InputStateNative_getMouseDelta(
        mut env: JNIEnv,
        _class: JClass,
        input_ptr: jlong,
    ) -> jobject {
        let input = crate::convert_ptr!(input_ptr => InputState);
        let vec = super::shared::get_mouse_delta(&input);
        match vec.to_jobject(&mut env) {
            Ok(obj) => obj.into_raw(),
            Err(_) => std::ptr::null_mut(),
        }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_input_InputStateNative_isCursorLocked(
        _env: JNIEnv,
        _class: JClass,
        input_ptr: jlong,
    ) -> jboolean {
        let input = crate::convert_ptr!(input_ptr => InputState);
        if input.is_cursor_locked { 1 } else { 0 }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_input_InputStateNative_setCursorLocked(
        mut env: JNIEnv,
        _class: JClass,
        cmd_ptr: jlong,
        input_ptr: jlong,
        locked: jboolean,
    ) {
        let input = crate::convert_ptr!(mut input_ptr => InputState);
        let sender = crate::convert_ptr!(cmd_ptr => crossbeam_channel::Sender<CommandBuffer>);

        if let Err(e) = super::shared::set_cursor_locked(input, sender, locked != 0) {
            let _ = env.throw_new("java/lang/RuntimeException", format!("{:?}", e));
        }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_input_InputStateNative_getLastMousePos(
        mut env: JNIEnv,
        _class: JClass,
        input_ptr: jlong,
    ) -> jobject {
        let input = crate::convert_ptr!(input_ptr => InputState);
        let vec = super::shared::get_last_mouse_pos(&input);
        match vec.to_jobject(&mut env) {
            Ok(obj) => obj.into_raw(),
            Err(_) => std::ptr::null_mut(),
        }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_input_InputStateNative_isCursorHidden(
        _env: JNIEnv,
        _class: JClass,
        input_ptr: jlong,
    ) -> jboolean {
        let input = crate::convert_ptr!(input_ptr => InputState);
        if input.is_cursor_hidden { 1 } else { 0 }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_input_InputStateNative_setCursorHidden(
        mut env: JNIEnv,
        _class: JClass,
        cmd_ptr: jlong,
        input_ptr: jlong,
        hidden: jboolean,
    ) {
        let input = crate::convert_ptr!(mut input_ptr => InputState);
        let sender = crate::convert_ptr!(cmd_ptr => crossbeam_channel::Sender<CommandBuffer>);

        if let Err(e) = super::shared::set_cursor_hidden(input, sender, hidden != 0) {
            let _ = env.throw_new("java/lang/RuntimeException", format!("{:?}", e));
        }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_input_InputStateNative_getConnectedGamepads(
        env: JNIEnv,
        _class: JClass,
        input_ptr: jlong,
    ) -> jlongArray {
        let input = crate::convert_ptr!(input_ptr => InputState);
        let ids = super::shared::get_connected_gamepads(&input);

        let long_ids: Vec<i64> = ids.iter().map(|&id| id as i64).collect();

        let output = match env.new_long_array(long_ids.len() as i32) {
            Ok(arr) => arr,
            Err(_) => return std::ptr::null_mut(),
        };

        if env.set_long_array_region(&output, 0, &long_ids).is_ok() {
            output.into_raw()
        } else {
            std::ptr::null_mut()
        }
    }
}

#[dropbear_macro::impl_c_api]
pub mod native {
    use crate::ptr::{InputStatePtr, CommandBufferPtr};
    use crate::input::{InputState};
    use crate::command::CommandBuffer;
    use crate::convert_ptr;
    use crate::scripting::native::DropbearNativeError;
    use crate::scripting::result::DropbearNativeResult;
    use crate::types::Vector2;

    pub fn dropbear_print_input_state(input_ptr: InputStatePtr) -> DropbearNativeResult<()> {
        let input = convert_ptr!(input_ptr => InputState);
        println!("Input State: {:?}", input);
        DropbearNativeResult::Ok(())
    }

    pub fn dropbear_is_key_pressed(input_ptr: InputStatePtr, key_ordinal: i32) -> DropbearNativeResult<bool> {
        let input = convert_ptr!(input_ptr => InputState);
        DropbearNativeResult::Ok(super::shared::is_key_pressed(input, key_ordinal))
    }

    pub fn dropbear_get_mouse_position(input_ptr: InputStatePtr) -> DropbearNativeResult<Vector2> {
        let input = convert_ptr!(input_ptr => InputState);
        DropbearNativeResult::Ok(super::shared::get_mouse_position(input))
    }

    pub fn dropbear_is_mouse_button_pressed(input_ptr: InputStatePtr, btn_ordinal: i32) -> DropbearNativeResult<bool> {
        let input = convert_ptr!(input_ptr => InputState);
        DropbearNativeResult::Ok(super::shared::is_mouse_button_pressed(input, btn_ordinal))
    }

    pub fn dropbear_get_mouse_delta(input_ptr: InputStatePtr) -> DropbearNativeResult<Vector2> {
        let input = convert_ptr!(input_ptr => InputState);
        DropbearNativeResult::Ok(super::shared::get_mouse_delta(input))
    }

    pub fn dropbear_is_cursor_locked(input_ptr: InputStatePtr) -> DropbearNativeResult<bool> {
        let input = convert_ptr!(input_ptr => InputState);
        DropbearNativeResult::Ok(input.is_cursor_locked)
    }

    pub fn dropbear_set_cursor_locked(
        cmd_ptr: CommandBufferPtr,
        input_ptr: InputStatePtr,
        locked: bool
    ) -> DropbearNativeResult<()> {
        let input = convert_ptr!(mut input_ptr => InputState);
        let sender = convert_ptr!(cmd_ptr => crossbeam_channel::Sender<CommandBuffer>);
        super::shared::set_cursor_locked(input, sender, locked)
    }

    pub fn dropbear_get_last_mouse_pos(input_ptr: InputStatePtr) -> DropbearNativeResult<Vector2> {
        let input = convert_ptr!(input_ptr => InputState);
        DropbearNativeResult::Ok(super::shared::get_last_mouse_pos(input))
    }

    pub fn dropbear_is_cursor_hidden(input_ptr: InputStatePtr) -> DropbearNativeResult<bool> {
        let input = convert_ptr!(input_ptr => InputState);
        DropbearNativeResult::Ok(input.is_cursor_hidden)
    }

    pub fn dropbear_set_cursor_hidden(
        cmd_ptr: CommandBufferPtr,
        input_ptr: InputStatePtr,
        hidden: bool
    ) -> DropbearNativeResult<()> {
        let input = convert_ptr!(mut input_ptr, InputStatePtr => InputState);
        let sender = convert_ptr!(cmd_ptr => crossbeam_channel::Sender<CommandBuffer>);
        super::shared::set_cursor_hidden(input, sender, hidden)
    }

    pub fn dropbear_get_connected_gamepads(
        input_ptr: InputStatePtr,
        out_count: *mut usize,
    ) -> DropbearNativeResult<*mut u64> {
        let input = convert_ptr!(input_ptr => InputState);

        let mut ids = super::shared::get_connected_gamepads(input);

        ids.shrink_to_fit();

        if out_count.is_null() {
            return DropbearNativeResult::Err(DropbearNativeError::NullPointer);
        }

        unsafe { *out_count = ids.len(); }

        let ptr = ids.as_mut_ptr();
        std::mem::forget(ids);

        DropbearNativeResult::Ok(ptr)
    }
}