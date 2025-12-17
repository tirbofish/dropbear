use crate::ptr::{CommandBufferPtr, InputStatePtr};
use crate::scripting::native::DropbearNativeError;
use crate::scripting::native::exports::dropbear_common::{Bool, DropbearNativeReturn};
use crate::utils::keycode_from_ordinal;
use crate::window::{CommandBuffer, WindowCommand};

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