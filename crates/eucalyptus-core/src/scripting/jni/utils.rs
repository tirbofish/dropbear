//! Utilities for JNI and JVM based code.

use crate::scripting::native::DropbearNativeError;
use crate::scripting::result::DropbearNativeResult;
use crate::types::{CollisionEvent, CollisionEventType, ContactForceEvent, IndexNative, NCollider, NVector3};
use jni::objects::{JObject, JValue};
use jni::sys::jint;
use jni::{Env, jni_sig, jni_str};

// todo: idk what to do about this module, considering about removing it

const JAVA_MOUSE_BUTTON_LEFT: jint = 0;
const JAVA_MOUSE_BUTTON_RIGHT: jint = 1;
const JAVA_MOUSE_BUTTON_MIDDLE: jint = 2;
const JAVA_MOUSE_BUTTON_BACK: jint = 3;
const JAVA_MOUSE_BUTTON_FORWARD: jint = 4;

/// Trait that defines conversion from a Java object to a Rust struct.
pub trait FromJObject {
    fn from_jobject(env: &mut Env, obj: &JObject) -> DropbearNativeResult<Self>
    where
        Self: Sized;
}

/// Converts a Rust object into a Java [JObject].
pub trait ToJObject {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>>;
}

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

impl<T> ToJObject for Vec<T>
where
    T: ToJObject,
{
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let list_class = env.load_class(jni_str!("java.util.ArrayList"))?;
        let list_obj = env.new_object(&list_class, jni_sig!(()), &[])?;

        for item in self {
            let obj = item.to_jobject(env)?;
            let _ = env.call_method(
                &list_obj,
                jni_str!("add"),
                jni_sig!((java.lang.Object) -> boolean),
                &[JValue::Object(&obj)],
            )?;
        }

        Ok(list_obj)
    }
}

impl<T> FromJObject for Vec<T>
where
    T: FromJObject,
{
    fn from_jobject(env: &mut Env, obj: &JObject) -> DropbearNativeResult<Self>
    where
        Self: Sized,
    {
        let size = env
            .call_method(obj, jni_str!("size"), jni_sig!(() -> int), &[])?
            .i()? as jint;
        let mut out = Vec::with_capacity(size as usize);

        for i in 0..size {
            let item = env
                .call_method(
                    obj,
                    jni_str!("get"),
                    jni_sig!((int) -> java.lang.Object),
                    &[JValue::Int(i)],
                )?
                .l()?;
            let value = T::from_jobject(env, &item)?;
            out.push(value);
        }

        Ok(out)
    }
}

// ─────────────────────────────────────────────────── Event ToJObject impls ──
// These are needed by scripting/jni.rs to dispatch physics events to the JVM.

impl ToJObject for NVector3 {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .load_class(jni_str!("com/dropbear/math/Vector3d"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let args = [
            JValue::Double(self.x),
            JValue::Double(self.y),
            JValue::Double(self.z),
        ];

        env.new_object(&class, jni_sig!((double, double, double) -> void), &args)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
    }
}

impl ToJObject for IndexNative {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let cls = env
            .load_class(jni_str!("com/dropbear/physics/Index"))
            .map_err(|_| DropbearNativeError::GenericError)?;

        env.new_object(
            cls,
            jni_sig!((int, int) -> void),
            &[
                JValue::Int(self.index as i32),
                JValue::Int(self.generation as i32),
            ],
        )
        .map_err(|_| DropbearNativeError::GenericError)
    }
}

impl ToJObject for NCollider {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let collider_cls = env
            .load_class(jni_str!("com/dropbear/physics/Collider"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let index_cls = env
            .load_class(jni_str!("com/dropbear/physics/Index"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let entity_cls = env
            .load_class(jni_str!("com/dropbear/EntityId"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let entity_obj = env
            .new_object(&entity_cls, jni_sig!((long) -> void), &[JValue::Long(self.entity_id as i64)])
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        let index_obj = env
            .new_object(
                &index_cls,
                jni_sig!((int, int) -> void),
                &[
                    JValue::Int(self.index.index as i32),
                    JValue::Int(self.index.generation as i32),
                ],
            )
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        env.new_object(
            collider_cls,
            jni_sig!((com.dropbear.physics.Index, com.dropbear.EntityId, int) -> void),
            &[
                JValue::Object(&index_obj),
                JValue::Object(&entity_obj),
                JValue::Int(self.id as i32),
            ],
        )
        .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
    }
}

impl ToJObject for CollisionEventType {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .load_class(jni_str!("com/dropbear/physics/CollisionEventType"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let name = match self {
            CollisionEventType::Started => "Started",
            CollisionEventType::Stopped => "Stopped",
        };
        let name_jstring = env
            .new_string(name)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        env.call_static_method(
            class,
            jni_str!("valueOf"),
            jni_sig!((java.lang.String) -> com.dropbear.physics.CollisionEventType),
            &[JValue::from(&name_jstring)],
        )
        .map_err(|_| DropbearNativeError::JNIMethodNotFound)?
        .l()
        .map_err(|_| DropbearNativeError::JNIUnwrapFailed)
    }
}

impl ToJObject for CollisionEvent {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .load_class(jni_str!("com/dropbear/physics/CollisionEvent"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let event_type = self.event_type.to_jobject(env)?;
        let collider1 = self.collider1.to_jobject(env)?;
        let collider2 = self.collider2.to_jobject(env)?;

        env.new_object(
            class,
            jni_sig!("(Lcom/dropbear/physics/CollisionEventType;Lcom/dropbear/physics/Collider;Lcom/dropbear/physics/Collider;I)V"),
            &[
                JValue::Object(&event_type),
                JValue::Object(&collider1),
                JValue::Object(&collider2),
                JValue::Int(self.flags as i32),
            ],
        )
        .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
    }
}

impl ToJObject for ContactForceEvent {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .load_class(jni_str!("com/dropbear/physics/ContactForceEvent"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let collider1 = self.collider1.to_jobject(env)?;
        let collider2 = self.collider2.to_jobject(env)?;
        let total_force = self.total_force.to_jobject(env)?;
        let max_force_direction = self.max_force_direction.to_jobject(env)?;

        env.new_object(
            class,
            jni_sig!("(Lcom/dropbear/physics/Collider;Lcom/dropbear/physics/Collider;Lcom/dropbear/math/Vector3d;DLcom/dropbear/math/Vector3d;D)V"),
            &[
                JValue::Object(&collider1),
                JValue::Object(&collider2),
                JValue::Object(&total_force),
                JValue::Double(self.total_force_magnitude),
                JValue::Object(&max_force_direction),
                JValue::Double(self.max_force_magnitude),
            ],
        )
        .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
    }
}