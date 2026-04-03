pub use eucalyptus_core::types::NColour;

use jni::objects::JObject;
use jni::{jni_sig, jni_str, Env, JValue};
use eucalyptus_core::scripting::native::DropbearNativeError;
use eucalyptus_core::scripting::result::DropbearNativeResult;
use crate::{FromJObject, ToJObject};

impl FromJObject for NColour {
    fn from_jobject(env: &mut Env, obj: &JObject) -> DropbearNativeResult<Self> {
        let class = env
            .load_class(jni_str!("com/dropbear/utils/Colour"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        if !env
            .is_instance_of(obj, &class)
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?
        {
            return Err(DropbearNativeError::InvalidArgument);
        }

        let mut get_byte = |field| -> DropbearNativeResult<u8> {
            let v = env
                .get_field(obj, field, jni_sig!(byte))
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
                .b()
                .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;
            Ok(v as u8)
        };

        Ok(Self {
            r: get_byte(jni_str!("r"))?,
            g: get_byte(jni_str!("g"))?,
            b: get_byte(jni_str!("b"))?,
            a: get_byte(jni_str!("a"))?,
        })
    }
}

impl ToJObject for NColour {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .load_class(jni_str!("com/dropbear/utils/Colour"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        env.new_object(
            &class,
            jni_sig!((byte, byte, byte, byte) -> void),
            &[
                JValue::Byte(self.r as i8),
                JValue::Byte(self.g as i8),
                JValue::Byte(self.b as i8),
                JValue::Byte(self.a as i8),
            ],
        )
        .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
    }
}
