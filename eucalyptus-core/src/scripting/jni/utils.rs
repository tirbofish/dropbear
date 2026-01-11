//! Utilities for JNI and JVM based code.

use crate::scripting::result::DropbearNativeResult;
use jni::objects::JObject;
use jni::sys::jint;
use jni::JNIEnv;

const JAVA_MOUSE_BUTTON_LEFT: jint = 0;
const JAVA_MOUSE_BUTTON_RIGHT: jint = 1;
const JAVA_MOUSE_BUTTON_MIDDLE: jint = 2;
const JAVA_MOUSE_BUTTON_BACK: jint = 3;
const JAVA_MOUSE_BUTTON_FORWARD: jint = 4;

pub fn Java_button_to_rust(button_code: jint) -> Option<winit::event::MouseButton> {
    match button_code {
        JAVA_MOUSE_BUTTON_LEFT => Some(winit::event::MouseButton::Left),
        JAVA_MOUSE_BUTTON_RIGHT => Some(winit::event::MouseButton::Right),
        JAVA_MOUSE_BUTTON_MIDDLE => Some(winit::event::MouseButton::Middle),
        JAVA_MOUSE_BUTTON_BACK => Some(winit::event::MouseButton::Back),
        JAVA_MOUSE_BUTTON_FORWARD => Some(winit::event::MouseButton::Forward),
        other if other >= 0 => Some(winit::event::MouseButton::Other(other as u16)),
        _ => None,
    }
}

/// Trait that defines conversion from a Java object to a Rust struct.
pub trait FromJObject {
    /// Converts a Java object to a Rust struct.
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self>
    where
        Self: Sized;
}

/// Converts a Rust object (struct or enum) into a java [JObject]
pub trait ToJObject {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>>;
}