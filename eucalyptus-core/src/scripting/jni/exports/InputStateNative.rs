#![allow(non_snake_case)]

use crate::ptr::{GraphicsPtr, InputStatePtr};
use crate::scripting::jni::utils::{java_button_to_rust, new_float_array};
use crate::utils::keycode_from_ordinal;
use crate::window::{CommandBuffer, WindowCommand};
use jni::objects::JClass;
use jni::sys::{jboolean, jfloatArray, jint, jlong, JNI_FALSE};
use jni::JNIEnv;

/// **Class:** `com_dropbear_ffi_InputStateNative`  
/// **Method:** `printInputState`  
/// **Signature:** `(J)V`  
///  
/// `JNIEXPORT void JNICALL Java_com_dropbear_ffi_InputStateNative_printInputState`  
/// `(JNIEnv *, jclass, jlong);`
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_InputStateNative_printInputState(
    _env: JNIEnv,
    _class: JClass,
    input_handle: jlong,
) {
    let input = input_handle as InputStatePtr;

    if input.is_null() {
        println!(
            "[Java_com_dropbear_ffi_InputStateNative_printInputState] [ERROR] Input state pointer is null"
        );
        return;
    }

    let input = unsafe { &*input };
    println!("{:#?}", input);
}

/// **Class:** `com_dropbear_ffi_InputStateNative`  
/// **Method:** `isKeyPressed`  
/// **Signature:** `(JI)Z`  
///  
/// `JNIEXPORT jboolean JNICALL Java_com_dropbear_ffi_InputStateNative_isKeyPressed`  
/// `(JNIEnv *, jclass, jlong, jint);`
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_InputStateNative_isKeyPressed(
    _env: JNIEnv,
    _class: JClass,
    input_handle: jlong,
    key: jint,
) -> jboolean {
    let input = input_handle as InputStatePtr;
    if input.is_null() {
        println!(
            "[Java_com_dropbear_ffi_InputStateNative_isKeyPressed] [ERROR] Input state pointer is null"
        );
        return false.into();
    }
    let input = unsafe { &*input };

    match keycode_from_ordinal(key) {
        Some(k) => input.pressed_keys.contains(&k).into(),
        None => {
            println!(
                "[Java_com_dropbear_ffi_InputStateNative_isKeyPressed] [WARN] Ordinal keycode is invalid"
            );
            false.into()
        }
    }
}

/// **Class:** `com_dropbear_ffi_InputStateNative`  
/// **Method:** `getMousePosition`  
/// **Signature:** `(J)[F`  
///  
/// `JNIEXPORT jfloatArray JNICALL Java_com_dropbear_ffi_InputStateNative_getMousePosition`  
/// `(JNIEnv *, jclass, jlong);`
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_InputStateNative_getMousePosition(
    mut env: JNIEnv,
    _class: JClass,
    input_handle: jlong,
) -> jfloatArray {
    let input = input_handle as InputStatePtr;
    if input.is_null() {
        println!(
            "[Java_com_dropbear_ffi_InputStateNative_getMousePosition] [ERROR] Input state pointer is null"
        );
        return new_float_array(&mut env, -1.0, -1.0);
    }

    let input = unsafe { &*input };
    new_float_array(&mut env, input.mouse_pos.0 as f32, input.mouse_pos.1 as f32)
}

/// **Class:** `com_dropbear_ffi_InputStateNative`  
/// **Method:** `isMouseButtonPressed`  
/// **Signature:** `(JI)Z`  
///  
/// `JNIEXPORT jboolean JNICALL Java_com_dropbear_ffi_InputStateNative_isMouseButtonPressed`  
/// `(JNIEnv *, jclass, jlong, jint);`
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_InputStateNative_isMouseButtonPressed(
    _env: JNIEnv,
    _class: JClass,
    input_handle: jlong,
    button: jint,
) -> jboolean {
    let input_ptr = input_handle as InputStatePtr;

    if input_ptr.is_null() {
        println!(
            "[Java_com_dropbear_ffi_InputStateNative_isMouseButtonPressed] [ERROR] Input state pointer is null"
        );
        return false as jboolean;
    }

    let input = unsafe { &*input_ptr };

    if let Some(rust_button) = java_button_to_rust(button) {
        input.mouse_button.contains(&rust_button) as jboolean
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_InputStateNative_isMouseButtonPressed] [ERROR] Invalid button code: {}",
            button
        );
        false as jboolean
    }
}

/// **Class:** `com_dropbear_ffi_InputStateNative`  
/// **Method:** `getMouseDelta`  
/// **Signature:** `(J)[F`  
///  
/// `JNIEXPORT jfloatArray JNICALL Java_com_dropbear_ffi_InputStateNative_getMouseDelta`  
/// `(JNIEnv *, jclass, jlong);`
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_InputStateNative_getMouseDelta(
    mut env: JNIEnv,
    _class: JClass,
    input_handle: jlong,
) -> jfloatArray {
    let input = input_handle as InputStatePtr;
    if input.is_null() {
        println!(
            "[Java_com_dropbear_ffi_InputStateNative_getMouseDelta] [ERROR] Input state pointer is null"
        );
        return new_float_array(&mut env, 0.0, 0.0);
    }

    let input = unsafe { &mut *input };

    if let Some(pos) = input.mouse_delta.take() {
        new_float_array(&mut env, pos.0 as f32, pos.1 as f32)
    } else {
        new_float_array(&mut env, 0.0, 0.0)
    }
}

/// **Class:** `com_dropbear_ffi_InputStateNative`  
/// **Method:** `isCursorLocked`  
/// **Signature:** `(J)Z`  
///  
/// `JNIEXPORT jboolean JNICALL Java_com_dropbear_ffi_InputStateNative_isCursorLocked`  
/// `(JNIEnv *, jclass, jlong);`
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_InputStateNative_isCursorLocked(
    _env: JNIEnv,
    _class: JClass,
    input_handle: jlong,
) -> jboolean {
    let input = input_handle as InputStatePtr;
    if input.is_null() {
        println!(
            "[Java_com_dropbear_ffi_InputStateNative_isCursorLocked] [ERROR] Input state pointer is null"
        );
        return false as jboolean;
    }

    let input = unsafe { &*input };
    input.is_cursor_locked as jboolean
}

/// **Class:** `com_dropbear_ffi_InputStateNative`  
/// **Method:** `setCursorLocked`  
/// **Signature:** `(JJZ)V`  
///  
/// `JNIEXPORT void JNICALL Java_com_dropbear_ffi_InputStateNative_setCursorLocked`  
/// `(JNIEnv *, jclass, jlong, jlong, jboolean);`
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_InputStateNative_setCursorLocked(
    _env: JNIEnv,
    _class: JClass,
    input_handle: jlong,
    command_handle: jlong,
    locked: jboolean,
) {
    let input = input_handle as InputStatePtr;
    if input.is_null() {
        eprintln!(
            "[Java_com_dropbear_ffi_InputStateNative_setCursorLocked] [ERROR] Input state pointer is null"
        );
        return;
    }

    let graphics = command_handle as GraphicsPtr;
    if graphics.is_null() {
        eprintln!(
            "[Java_com_dropbear_ffi_InputStateNative_setCursorLocked] [ERROR] Graphics pointer is null"
        );
        return;
    }

    let input = unsafe { &mut *input };
    let graphics = unsafe { &*graphics };

    let is_locked = locked != 0;

    if let Err(e) = graphics.send(CommandBuffer::WindowCommand(WindowCommand::WindowGrab(is_locked))) {
        eprintln!(
            "[Java_com_dropbear_ffi_InputStateNative_setCursorLocked] [ERROR] Unable to send window command: {}",
            e
        );
        return;
    }

    input.is_cursor_locked = is_locked;
}

/// **Class:** `com_dropbear_ffi_InputStateNative`  
/// **Method:** `getLastMousePos`  
/// **Signature:** `(J)[F`  
///  
/// `JNIEXPORT jfloatArray JNICALL Java_com_dropbear_ffi_InputStateNative_getLastMousePos`  
/// `(JNIEnv *, jclass, jlong);`
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_InputStateNative_getLastMousePos(
    mut env: JNIEnv,
    _class: JClass,
    input_handle: jlong,
) -> jfloatArray {
    let input = input_handle as InputStatePtr;
    if input.is_null() {
        println!(
            "[Java_com_dropbear_ffi_InputStateNative_getLastMousePos] [ERROR] Input state pointer is null"
        );
        return new_float_array(&mut env, 0.0, 0.0);
    }

    let input = unsafe { &*input };
    if let Some(pos) = input.last_mouse_pos {
        new_float_array(&mut env, pos.0 as f32, pos.1 as f32)
    } else {
        new_float_array(&mut env, 0.0, 0.0)
    }
}

/// **Class:** `com_dropbear_ffi_InputStateNative`  
/// **Method:** `isCursorHidden`  
/// **Signature:** `(J)Z`  
///  
/// `JNIEXPORT jboolean JNICALL Java_com_dropbear_ffi_InputStateNative_isCursorHidden`  
/// `(JNIEnv *, jclass, jlong);`
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_InputStateNative_isCursorHidden(
    _env: JNIEnv,
    _class: JClass,
    input_handle: jlong,
) -> jboolean {
    let input = input_handle as InputStatePtr;
    if input.is_null() {
        println!(
            "[Java_com_dropbear_ffi_InputStateNative_isCursorHidden] [ERROR] Input state pointer is null"
        );
        return false.into();
    }
    let input = unsafe { &*input };

    input.is_cursor_hidden.into()
}

/// **Class:** `com_dropbear_ffi_InputStateNative`  
/// **Method:** `setCursorHidden`  
/// **Signature:** `(JJZ)V`  
///  
/// `JNIEXPORT void JNICALL Java_com_dropbear_ffi_InputStateNative_setCursorHidden`  
/// `(JNIEnv *, jclass, jlong, jlong, jboolean);`
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_InputStateNative_setCursorHidden(
    _env: JNIEnv,
    _class: JClass,
    input_handle: jlong,
    command_handle: jlong,
    hide: jboolean,
) {
    let input = input_handle as InputStatePtr;
    if input.is_null() {
        println!(
            "[Java_com_dropbear_ffi_InputStateNative_setCursorHidden] [ERROR] Input state pointer is null"
        );
        return;
    }
    let input = unsafe { &mut *input };

    let graphics = command_handle as GraphicsPtr;
    if graphics.is_null() {
        println!(
            "[Java_com_dropbear_ffi_InputStateNative_setCursorHidden] [ERROR] Graphics pointer is null"
        );
        return;
    }
    let graphics = unsafe { &*graphics };

    let hide = hide != JNI_FALSE;

    if let Err(e) = graphics.send(CommandBuffer::WindowCommand(WindowCommand::HideCursor(hide))) {
        println!(
            "[Java_com_dropbear_ffi_InputStateNative_setCursorHidden] [ERROR] Unable to send hide cursor command: {}",
            e
        );
    }

    input.is_cursor_hidden = hide;
}