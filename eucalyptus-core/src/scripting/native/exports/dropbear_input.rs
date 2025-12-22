use crate::ptr::{CommandBufferPtr, InputStatePtr};
use crate::scripting::native::DropbearNativeError;
use crate::scripting::native::exports::dropbear_common::{Bool, DropbearNativeReturn, Handle};
use crate::utils::keycode_from_ordinal;
use crate::command::{CommandBuffer, WindowCommand};
use crate::scripting::native::exports::dropbear_math::Vector2D;
use crate::scripting::utils::button_from_ordinal;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Gamepad {
    id: i32,
    left_stick_pos: Vector2D,
    right_stick_pos: Vector2D,
}

/// Prints the input state to the console. Does not return anything, failure does not do anything.
///
/// Can be useful for debugging.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_print_input_state(input_state_ptr: InputStatePtr) {
    if input_state_ptr.is_null() {
        eprintln!("[dropbear_print_input_state] [ERROR] Input state pointer is null");
        return;
    }

    let input_state = unsafe { &*input_state_ptr };
    println!("{:#?}", input_state);
}

/// Checks if a key is currently pressed. If pressed, returns 1, otherwise 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_is_key_pressed(
    input_state_ptr: InputStatePtr,
    keycode: i32,
    out_value: *mut Bool,
) -> DropbearNativeReturn {
    if input_state_ptr.is_null() || out_value.is_null() {
        return DropbearNativeError::NullPointer as i32;
    }

    let input = unsafe { &*input_state_ptr };

    match keycode_from_ordinal(keycode) {
        Some(k) => {
            let is_pressed = input.pressed_keys.contains(&k);
            unsafe { *out_value = if is_pressed { 1 } else { 0 } };
            DropbearNativeError::Success as i32
        }
        None => {
            eprintln!("[dropbear_is_key_pressed] [WARN] Invalid keycode");
            unsafe { *out_value = 0 };
            DropbearNativeError::Success as i32
        }
    }
}

/// Fetches the current mouse position for that frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_mouse_position(
    input_state_ptr: InputStatePtr,
    out_x: *mut f32,
    out_y: *mut f32,
) -> DropbearNativeReturn {
    if input_state_ptr.is_null() || out_x.is_null() || out_y.is_null() {
        return DropbearNativeError::NullPointer as i32;
    }

    let input = unsafe { &*input_state_ptr };

    unsafe {
        *out_x = input.mouse_pos.0 as f32;
        *out_y = input.mouse_pos.1 as f32;
    }

    DropbearNativeError::Success as i32
}

/// Checks if a mouse button is currently pressed. If pressed, returns 1, otherwise 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_is_mouse_button_pressed(
    input_state_ptr: InputStatePtr,
    button_code: i32,
    out_pressed: *mut Bool,
) -> DropbearNativeReturn {
    if input_state_ptr.is_null() || out_pressed.is_null() {
        return DropbearNativeError::NullPointer as i32;
    }

    let input = unsafe { &*input_state_ptr };

    match keycode_from_ordinal(button_code) {
        None => {
            eprintln!("[dropbear_is_mouse_button_pressed] [WARN] Invalid button code");
            unsafe { *out_pressed = 0 };
        }
        Some(key) => {
            if input.pressed_keys.contains(&key) {
                unsafe { *out_pressed = 1 };
            } else {
                unsafe { *out_pressed = 0 };
            }
        }
    }

    DropbearNativeError::Success as i32
}

/// Fetches the delta of the mouse position since the last frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_mouse_delta(
    input_state_ptr: InputStatePtr,
    out_delta_x: *mut f32,
    out_delta_y: *mut f32,
) -> DropbearNativeReturn {
    if input_state_ptr.is_null() || out_delta_x.is_null() || out_delta_y.is_null() {
        return DropbearNativeError::NullPointer as i32;
    }

    let input = unsafe { &mut *(input_state_ptr as InputStatePtr) };

    if let Some(pos) = input.mouse_delta.take() {
        unsafe {
            *out_delta_x = pos.0 as f32;
            *out_delta_y = pos.1 as f32;
        }
    } else {
        unsafe {
            *out_delta_x = 0.0;
            *out_delta_y = 0.0;
        }
    }

    DropbearNativeError::Success as i32
}

/// Checks if the cursor is currently locked. If locked, returns 1, otherwise 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_is_cursor_locked(
    input_state_ptr: InputStatePtr,
    out_locked: *mut Bool,
) -> DropbearNativeReturn {
    if input_state_ptr.is_null() || out_locked.is_null() {
        return DropbearNativeError::NullPointer as i32;
    }

    let input = unsafe { &*input_state_ptr };

    unsafe { *out_locked = if input.is_cursor_locked { 1 } else { 0 } };

    DropbearNativeError::Success as i32
}

/// Sets the mouse cursor to be locked or unlocked.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_set_cursor_locked(
    input_state_ptr: InputStatePtr,
    queue_ptr: CommandBufferPtr,
    locked: Bool,
) -> DropbearNativeReturn {
    if input_state_ptr.is_null() || queue_ptr.is_null() {
        return DropbearNativeError::NullPointer as i32;
    }

    let input = unsafe { &mut *(input_state_ptr as InputStatePtr) };

    let graphics = unsafe { &*(queue_ptr as CommandBufferPtr) };

    input.is_cursor_locked = locked != 0;

    if graphics
        .send(CommandBuffer::WindowCommand(WindowCommand::WindowGrab(
            input.is_cursor_locked,
        )))
        .is_err()
    {
        DropbearNativeError::SendError as i32
    } else {
        DropbearNativeError::Success as i32
    }
}

/// Fetches the mouse position of the previous frame.
///
/// Can be used to calculate the delta of the mouse position.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_last_mouse_pos(
    input_state_ptr: InputStatePtr,
    out_x: *mut f32,
    out_y: *mut f32,
) -> DropbearNativeReturn {
    if input_state_ptr.is_null() || out_x.is_null() || out_y.is_null() {
        return DropbearNativeError::NullPointer as i32;
    }

    let input = unsafe { &*input_state_ptr };

    if let Some(pos) = input.last_mouse_pos {
        unsafe {
            *out_x = pos.0 as f32;
            *out_y = pos.1 as f32;
        }
    } else {
        unsafe {
            *out_x = 0.0;
            *out_y = 0.0;
        }
    }

    DropbearNativeError::Success as i32
}

/// Checks if the cursor is currently hidden. If hidden, returns 1, otherwise 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_is_cursor_hidden(
    input_state_ptr: InputStatePtr,
    out_hidden: *mut Bool,
) -> DropbearNativeReturn {
    if input_state_ptr.is_null() || out_hidden.is_null() {
        return DropbearNativeError::NullPointer as i32;
    }

    let input = unsafe { &*input_state_ptr };

    unsafe { *out_hidden = if input.is_cursor_hidden { 1 } else { 0 } };

    DropbearNativeError::Success as i32
}

/// Sets the cursor to either hidden (invisible) or visible
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_set_cursor_hidden(
    input_state_ptr: InputStatePtr,
    queue_ptr: CommandBufferPtr,
    hidden: Bool,
) -> DropbearNativeReturn {
    if input_state_ptr.is_null() || queue_ptr.is_null() {
        return DropbearNativeError::NullPointer as i32;
    }

    let input = unsafe { &mut *(input_state_ptr as InputStatePtr) };
    let graphics = unsafe { &*(queue_ptr as CommandBufferPtr) };
    input.is_cursor_hidden = hidden != 0;

    if graphics
        .send(CommandBuffer::WindowCommand(WindowCommand::HideCursor(
            input.is_cursor_hidden,
        )))
        .is_err()
    {
        DropbearNativeError::SendError as i32
    } else {
        DropbearNativeError::Success as i32
    }
}

/// Fetches all available connected gamepads in the input state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_connected_gamepads(
    input_state_ptr: InputStatePtr,
    out_gamepads: *mut *const Gamepad,
    out_count: *mut i32,
) -> DropbearNativeReturn {
    if input_state_ptr.is_null() {
        eprintln!("[dropbear_get_connected_gamepads] [ERROR] Input state pointer provided was null");
        return DropbearNativeError::NullPointer as i32;
    }

    if out_gamepads.is_null() {
        eprintln!("[dropbear_get_connected_gamepads] [ERROR] out_gamepads pointer provided was null");
        return DropbearNativeError::NullPointer as i32;
    }

    if out_count.is_null() {
        eprintln!("[dropbear_get_connected_gamepads] [ERROR] out_count of gamepads pointer provided was null");
        return DropbearNativeError::NullPointer as i32;
    }

    let input_state = &mut *input_state_ptr;


    let mut gamepads = Vec::new();
    for g in &input_state.connected_gamepads {
        let id: usize = (*g).into();
        let left_stick = input_state.left_stick_position.get(&g);
        let right_stick = input_state.right_stick_position.get(&g);

        if let Some(l) = left_stick && let Some(r) = right_stick {
            let l = l.clone();
            let r = r.clone();

            let gamepad = Gamepad {
                id: id as i32,
                left_stick_pos: Vector2D {
                    x: l.0 as f64,
                    y: l.1 as f64,
                },
                right_stick_pos: Vector2D {
                    x: r.0 as f64,
                    y: r.1 as f64,
                },
            };
            gamepads.push(gamepad);
        }
    }

    input_state.cached_gamepads = gamepads;

    unsafe {
        *out_count = input_state.cached_gamepads.len() as i32;
        *out_gamepads = input_state.cached_gamepads.as_ptr();
    }

    DropbearNativeError::Success as i32
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_is_gamepad_button_pressed(
    input_state_ptr: InputStatePtr,
    gamepad_id: Handle,
    button_ordinal: i32,
    out_pressed: *mut Bool,
) -> DropbearNativeReturn {
    if input_state_ptr.is_null() {
        eprintln!("[dropbear_is_gamepad_button_pressed] [ERROR] Input state pointer provided was null");
        return DropbearNativeError::NullPointer as i32;
    }

    if out_pressed.is_null() {
        eprintln!("[dropbear_is_gamepad_button_pressed] [ERROR] out_pressed pointer provided was null");
        return DropbearNativeError::NullPointer as i32;
    }

    let input_state = &*input_state_ptr;

    if gamepad_id < 0 {
        eprintln!(
            "[dropbear_is_gamepad_button_pressed] [ERROR] Invalid gamepad id (negative): {}",
            gamepad_id
        );
        return DropbearNativeError::InvalidArgument as i32;
    }

    let gamepad_id_usize = gamepad_id as usize;
    let maybe_gid = input_state
        .connected_gamepads
        .iter()
        .find(|gid| {
            let id: usize = (**gid).into();
            id == gamepad_id_usize
        })
        .copied();

    let gid = match maybe_gid {
        Some(gid) => gid,
        None => {
            eprintln!(
                "[dropbear_is_gamepad_button_pressed] [ERROR] Gamepad with id {} not found",
                gamepad_id
            );
            return DropbearNativeError::GamepadNotFound as i32;
        }
    };

    let button = match button_from_ordinal(button_ordinal) {
        Ok(btn) => btn,
        Err(_) => {
            eprintln!(
                "[dropbear_is_gamepad_button_pressed] [ERROR] Invalid button ordinal: {}",
                button_ordinal
            );
            return DropbearNativeError::InvalidArgument as i32;
        }
    };

    let is_pressed = input_state.is_button_pressed(gid, button);

    *out_pressed = if is_pressed { 1 } else { 0 };

    DropbearNativeError::Success as i32
}