//! Utilities for JNI and JVM based code.

use crate::scripting::native::DropbearNativeError;
use crate::scripting::result::DropbearNativeResult;
use glam::DVec3;
use jni::objects::{JObject, JValue};
use jni::sys::jint;
use jni::JNIEnv;

const JAVA_MOUSE_BUTTON_LEFT: jint = 0;
const JAVA_MOUSE_BUTTON_RIGHT: jint = 1;
const JAVA_MOUSE_BUTTON_MIDDLE: jint = 2;
const JAVA_MOUSE_BUTTON_BACK: jint = 3;
const JAVA_MOUSE_BUTTON_FORWARD: jint = 4;

pub fn java_button_to_rust(button_code: jint) -> Option<winit::event::MouseButton> {
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

impl FromJObject for DVec3 {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self>
    where
        Self: Sized
    {
        let x_obj = env
            .get_field(obj, "x", "Ljava/lang/Number;").map_err(|_| DropbearNativeError::JNIFailedToGetField)?.l().map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;
        let y_obj = env
            .get_field(obj, "y", "Ljava/lang/Number;").map_err(|_| DropbearNativeError::JNIFailedToGetField)?.l().map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;
        let z_obj = env
            .get_field(obj, "z", "Ljava/lang/Number;").map_err(|_| DropbearNativeError::JNIFailedToGetField)?.l().map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let x = env
            .call_method(&x_obj, "doubleValue", "()D", &[]).map_err(|_| DropbearNativeError::JNIMethodNotFound)?.d().map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;
        let y = env
            .call_method(&y_obj, "doubleValue", "()D", &[]).map_err(|_| DropbearNativeError::JNIMethodNotFound)?.d().map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;
        let z = env
            .call_method(&z_obj, "doubleValue", "()D", &[]).map_err(|_| DropbearNativeError::JNIMethodNotFound)?.d().map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        Ok(DVec3::new(x, y, z))
    }
}

/// Converts a Rust object (struct or enum) into a java [JObject]
pub trait ToJObject {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>>;
}

impl ToJObject for DVec3 {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let cls = env.find_class("com/dropbear/math/Vector3d")
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let obj = env.new_object(
            cls,
            "(DDD)V",
            &[
                JValue::Double(self.x),
                JValue::Double(self.y),
                JValue::Double(self.z)
            ]
        ).map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        Ok(obj)
    }
}