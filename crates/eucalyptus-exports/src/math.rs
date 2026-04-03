pub use eucalyptus_core::types::{NQuaternion, NTransform, NVector2, NVector3, NVector4};

use glam::{DQuat, DVec3};
use jni::objects::JObject;
use jni::{jni_sig, jni_str, Env, JValue};
use eucalyptus_core::scripting::native::DropbearNativeError;
use eucalyptus_core::scripting::result::DropbearNativeResult;
use crate::{FromJObject, ToJObject};

// --------------------------------------------------------------- NVector2 ---

impl ToJObject for NVector2 {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let cls = env
            .load_class(jni_str!("com/dropbear/math/Vector2d"))
            .map_err(|_| DropbearNativeError::GenericError)?;

        env.new_object(
            cls,
            jni_sig!((double, double) -> void),
            &[JValue::Double(self.x), JValue::Double(self.y)],
        )
        .map_err(|_| DropbearNativeError::GenericError)
    }
}

impl FromJObject for NVector2 {
    fn from_jobject(env: &mut Env, obj: &JObject) -> DropbearNativeResult<Self> {
        let x = env
            .get_field(obj, jni_str!("x"), jni_sig!(double))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .d()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;
        let y = env
            .get_field(obj, jni_str!("y"), jni_sig!(double))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .d()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;
        Ok(NVector2 { x, y })
    }
}

// --------------------------------------------------------------- NVector3 ---

impl FromJObject for NVector3 {
    fn from_jobject(env: &mut Env, obj: &JObject) -> DropbearNativeResult<Self> {
        let class = env
            .load_class(jni_str!("com/dropbear/math/Vector3d"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        if !env
            .is_instance_of(obj, &class)
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?
        {
            return Err(DropbearNativeError::InvalidArgument);
        }

        let x = env
            .get_field(obj, jni_str!("x"), jni_sig!(double))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .d()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;
        let y = env
            .get_field(obj, jni_str!("y"), jni_sig!(double))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .d()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;
        let z = env
            .get_field(obj, jni_str!("z"), jni_sig!(double))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .d()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        Ok(NVector3::new(x, y, z))
    }
}

impl ToJObject for NVector3 {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .load_class(jni_str!("com/dropbear/math/Vector3d"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        env.new_object(
            &class,
            jni_sig!((double, double, double) -> void),
            &[
                JValue::Double(self.x),
                JValue::Double(self.y),
                JValue::Double(self.z),
            ],
        )
        .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
    }
}

// --------------------------------------------------------------- NVector4 ---

impl FromJObject for NVector4 {
    fn from_jobject(env: &mut Env, obj: &JObject) -> DropbearNativeResult<Self> {
        let class = env
            .load_class(jni_str!("com/dropbear/math/Vector4d"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        if !env
            .is_instance_of(obj, &class)
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?
        {
            return Err(DropbearNativeError::InvalidArgument);
        }

        let x = env
            .get_field(obj, jni_str!("x"), jni_sig!(double))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .d()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;
        let y = env
            .get_field(obj, jni_str!("y"), jni_sig!(double))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .d()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;
        let z = env
            .get_field(obj, jni_str!("z"), jni_sig!(double))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .d()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;
        let w = env
            .get_field(obj, jni_str!("w"), jni_sig!(double))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .d()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        Ok(NVector4::new(x, y, z, w))
    }
}

impl ToJObject for NVector4 {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .load_class(jni_str!("com/dropbear/math/Vector3d"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        env.new_object(
            &class,
            jni_sig!((double, double, double, double) -> void),
            &[
                JValue::Double(self.x),
                JValue::Double(self.y),
                JValue::Double(self.z),
                JValue::Double(self.w),
            ],
        )
        .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
    }
}

// --------------------------------------------------------------- NQuaternion -

impl ToJObject for NQuaternion {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .load_class(jni_str!("com/dropbear/math/Quaterniond"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        env.new_object(
            &class,
            jni_sig!((double, double, double, double) -> void),
            &[
                JValue::Double(self.x),
                JValue::Double(self.y),
                JValue::Double(self.z),
                JValue::Double(self.w),
            ],
        )
        .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
    }
}

impl FromJObject for NQuaternion {
    fn from_jobject(env: &mut Env, obj: &JObject) -> DropbearNativeResult<Self> {
        let class = env
            .load_class(jni_str!("com/dropbear/math/Quaterniond"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        if !env
            .is_instance_of(obj, &class)
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?
        {
            return Err(DropbearNativeError::InvalidArgument);
        }

        let x = env
            .get_field(obj, jni_str!("x"), jni_sig!(double))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .d()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;
        let y = env
            .get_field(obj, jni_str!("y"), jni_sig!(double))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .d()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;
        let z = env
            .get_field(obj, jni_str!("z"), jni_sig!(double))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .d()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;
        let w = env
            .get_field(obj, jni_str!("w"), jni_sig!(double))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .d()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        Ok(NQuaternion { x, y, z, w })
    }
}

// --------------------------------------------------------------- NTransform --

impl FromJObject for NTransform {
    fn from_jobject(env: &mut Env, obj: &JObject) -> DropbearNativeResult<Self> {
        let pos_obj = env
            .get_field(obj, jni_str!("position"), jni_sig!(com.dropbear.math.Vector3d))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let rot_obj = env
            .get_field(obj, jni_str!("rotation"), jni_sig!(com.dropbear.math.Quaterniond))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let scale_obj = env
            .get_field(obj, jni_str!("scale"), jni_sig!(com.dropbear.math.Vector3d))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let position: DVec3 = NVector3::from_jobject(env, &pos_obj)?.into();
        let scale: DVec3 = NVector3::from_jobject(env, &scale_obj)?.into();

        let mut get_double = |field| -> DropbearNativeResult<f64> {
            env.get_field(&rot_obj, field, jni_sig!(double))
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
                .d()
                .map_err(|_| DropbearNativeError::JNIUnwrapFailed)
        };

        let rx = get_double(jni_str!("x"))?;
        let ry = get_double(jni_str!("y"))?;
        let rz = get_double(jni_str!("z"))?;
        let rw = get_double(jni_str!("w"))?;

        let rotation = DQuat::from_xyzw(rx, ry, rz, rw);

        Ok(NTransform {
            position: position.into(),
            rotation: rotation.into(),
            scale: scale.into(),
        })
    }
}

impl ToJObject for NTransform {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .load_class(jni_str!("com/dropbear/math/Transform"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let args = [
            JValue::Double(self.position.x),
            JValue::Double(self.position.y),
            JValue::Double(self.position.z),
            JValue::Double(self.rotation.x),
            JValue::Double(self.rotation.y),
            JValue::Double(self.rotation.z),
            JValue::Double(self.rotation.w),
            JValue::Double(self.scale.x),
            JValue::Double(self.scale.y),
            JValue::Double(self.scale.z),
        ];

        env.new_object(
            &class,
            jni_sig!((double, double, double, double, double, double, double, double, double, double) -> void),
            &args,
        )
        .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
    }
}
