use crate::scripting::jni::utils::{FromJObject, ToJObject};
use crate::scripting::native::DropbearNativeError;
use crate::scripting::result::DropbearNativeResult;
use jni::JNIEnv;
use jni::objects::{JObject, JValue};
use jni::sys::{jdouble, jint, jlong};

impl ToJObject for Option<i32> {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        match self {
            Some(value) => {
                let class = env
                    .find_class("java/lang/Integer")
                    .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

                env.new_object(&class, "(I)V", &[JValue::Int(*value)])
                    .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
            }
            None => Ok(JObject::null()),
        }
    }
}

impl FromJObject for Option<i32> {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self> {
        if obj.is_null() {
            return Ok(None);
        }

        let class = env
            .find_class("java/lang/Integer")
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        if !env
            .is_instance_of(obj, &class)
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?
        {
            return Err(DropbearNativeError::InvalidArgument);
        }

        let value = env
            .call_method(obj, "intValue", "()I", &[])
            .map_err(|_| DropbearNativeError::JNIMethodNotFound)?
            .i()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        Ok(Some(value as i32))
    }
}

impl ToJObject for Vec<i32> {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        self.as_slice().to_jobject(env)
    }
}

impl ToJObject for &[i32] {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let array = env
            .new_int_array(self.len() as i32)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;
        let buf: Vec<jint> = self.iter().map(|v| *v as jint).collect();
        env.set_int_array_region(&array, 0, &buf)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;
        Ok(JObject::from(array))
    }
}

impl ToJObject for &[Vec<i32>] {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let list = new_array_list(env)?;
        for value in self.iter() {
            let boxed = value.as_slice().to_jobject(env)?;
            array_list_add(env, &list, &boxed)?;
        }
        Ok(list)
    }
}

impl ToJObject for Option<f32> {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        match self {
            Some(value) => {
                let class = env
                    .find_class("java/lang/Float")
                    .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

                env.new_object(&class, "(F)V", &[JValue::Float(*value)])
                    .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
            }
            None => Ok(JObject::null()),
        }
    }
}

impl ToJObject for Option<f64> {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        match self {
            Some(value) => {
                let class = env
                    .find_class("java/lang/Double")
                    .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

                env.new_object(&class, "(D)V", &[JValue::Double(*value)])
                    .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
            }
            None => Ok(JObject::null()),
        }
    }
}

impl ToJObject for Vec<f64> {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        self.as_slice().to_jobject(env)
    }
}

impl ToJObject for &[f64] {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let array = env
            .new_double_array(self.len() as i32)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;
        let buf: Vec<jdouble> = self.iter().map(|v| *v as jdouble).collect();
        env.set_double_array_region(&array, 0, &buf)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;
        Ok(JObject::from(array))
    }
}

impl ToJObject for &[Vec<f64>] {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let list = new_array_list(env)?;
        for value in self.iter() {
            let array = value.as_slice().to_jobject(env)?;
            array_list_add(env, &list, &array)?;
        }
        Ok(list)
    }
}

fn new_array_list<'a>(env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
    let class = env
        .find_class("java/util/ArrayList")
        .map_err(|_| DropbearNativeError::JNIClassNotFound)?;
    env.new_object(&class, "()V", &[])
        .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
}

fn array_list_add(env: &mut JNIEnv, list: &JObject, item: &JObject) -> DropbearNativeResult<()> {
    env.call_method(
        list,
        "add",
        "(Ljava/lang/Object;)Z",
        &[JValue::Object(item)],
    )
    .map_err(|_| DropbearNativeError::JNIMethodNotFound)?;
    Ok(())
}

impl ToJObject for Vec<u64> {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        self.as_slice().to_jobject(env)
    }
}

impl ToJObject for &[u64] {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let array = env.new_long_array(self.len() as i32)?;
        let buf: Vec<jlong> = self.iter().map(|v| *v as jlong).collect();
        env.set_long_array_region(&array, 0, &buf)?;
        Ok(JObject::from(array))
    }
}

impl ToJObject for String {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let result = JObject::from(env.new_string(self)?);
        Ok(result)
    }
}
