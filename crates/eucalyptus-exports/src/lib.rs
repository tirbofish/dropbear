use jni::Env;
use jni::objects::JObject;
use eucalyptus_core::scripting::result::DropbearNativeResult;

// Re-export macros so the dropbear_macro::export expansion can find them at crate::convert_ptr! etc.
pub use eucalyptus_core::convert_ptr;
pub use eucalyptus_core::ffi_error_return;
pub use eucalyptus_core::convert_jstring;
pub use eucalyptus_core::convert_jlong_to_entity;

pub mod animation;
pub mod asset;
pub mod camera;
pub mod component;
pub mod debug;
pub mod entity;
pub mod input;
pub mod lighting;
pub mod math;
pub mod mesh;
pub mod physics;
pub mod primitives;
pub mod properties;
pub mod scene;
pub mod scripting;
pub mod transform;
pub mod utils;
pub mod engine;

pub mod ptr {
    pub use eucalyptus_core::ptr::*;
}

/// Trait that defines conversion from a Java object to a Rust struct.
pub trait FromJObject {
    fn from_jobject(env: &mut Env, obj: &JObject) -> DropbearNativeResult<Self>
    where
        Self: Sized;
}

/// Converts a Rust object (struct or enum) into a java [JObject].
pub trait ToJObject {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>>;
}
