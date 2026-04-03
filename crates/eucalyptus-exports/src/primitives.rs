use crate::{FromJObject, ToJObject};
use eucalyptus_core::scripting::native::DropbearNativeError;
use eucalyptus_core::scripting::result::DropbearNativeResult;
use jni::objects::{JObject, JValue};
use jni::sys::{jdouble, jint, jlong};
use jni::{Env, jni_sig, jni_str};

impl ToJObject for Option<i32> {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        match self {
            Some(value) => {
                let class = env
                    .load_class(jni_str!("java.lang.Integer"))
                    .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

                env.new_object(&class, jni_sig!((int) -> void), &[JValue::Int(*value)])
                    .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
            }
            None => Ok(JObject::null()),
        }
    }
}

impl FromJObject for Option<i32> {
    fn from_jobject(env: &mut Env, obj: &JObject) -> DropbearNativeResult<Self> {
        if obj.is_null() {
            return Ok(None);
        }

        let class = env
            .load_class(jni_str!("java.lang.Integer"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        if !env
            .is_instance_of(obj, &class)
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?
        {
            return Err(DropbearNativeError::InvalidArgument);
        }

        let value = env
            .call_method(obj, jni_str!("intValue"), jni_sig!(() -> i32), &[])
            .map_err(|_| DropbearNativeError::JNIMethodNotFound)?
            .i()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        Ok(Some(value as i32))
    }
}

impl ToJObject for Vec<i32> {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        self.as_slice().to_jobject(env)
    }
}

impl ToJObject for &[i32] {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let array = env
            .new_int_array(self.len())
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;
        let buf: Vec<jint> = self.iter().map(|v| *v as jint).collect();
        array
            .set_region(env, 0, &buf)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;
        Ok(JObject::from(array))
    }
}

impl ToJObject for &[Vec<i32>] {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let list = new_array_list(env)?;
        for value in self.iter() {
            let boxed = value.as_slice().to_jobject(env)?;
            array_list_add(env, &list, &boxed)?;
        }
        Ok(list)
    }
}

impl ToJObject for Option<f32> {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        match self {
            Some(value) => {
                let class = env
                    .load_class(jni_str!("java.lang.Float"))
                    .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

                env.new_object(&class, jni_sig!((f32) -> ()), &[JValue::Float(*value)])
                    .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
            }
            None => Ok(JObject::null()),
        }
    }
}

impl ToJObject for Option<f64> {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        match self {
            Some(value) => {
                let class = env
                    .load_class(jni_str!("java.lang.Double"))
                    .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

                env.new_object(&class, jni_sig!((f64) -> ()), &[JValue::Double(*value)])
                    .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
            }
            None => Ok(JObject::null()),
        }
    }
}

impl ToJObject for Vec<f64> {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        self.as_slice().to_jobject(env)
    }
}

impl ToJObject for &[f64] {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let array = env
            .new_double_array(self.len())
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;
        let buf: Vec<jdouble> = self.iter().map(|v| *v as jdouble).collect();
        array
            .set_region(env, 0, &buf)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;
        Ok(JObject::from(array))
    }
}

impl ToJObject for &[Vec<f64>] {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let list = new_array_list(env)?;
        for value in self.iter() {
            let array = value.as_slice().to_jobject(env)?;
            array_list_add(env, &list, &array)?;
        }
        Ok(list)
    }
}

fn new_array_list<'a>(env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
    let class = env
        .load_class(jni_str!("java.util.ArrayList"))
        .map_err(|_| DropbearNativeError::JNIClassNotFound)?;
    env.new_object(&class, jni_sig!(() -> ()), &[])
        .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
}

fn array_list_add(env: &mut Env, list: &JObject, item: &JObject) -> DropbearNativeResult<()> {
    env.call_method(
        list,
        jni_str!("add"),
        jni_sig!((java.lang.Object) -> boolean),
        &[JValue::Object(item)],
    )
    .map_err(|_| DropbearNativeError::JNIMethodNotFound)?;
    Ok(())
}

impl ToJObject for Vec<u64> {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        self.as_slice().to_jobject(env)
    }
}

impl ToJObject for &[u64] {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let array = env.new_long_array(self.len())?;
        let buf: Vec<jlong> = self.iter().map(|v| *v as jlong).collect();
        array.set_region(env, 0, &buf)?;
        Ok(JObject::from(array))
    }
}

impl ToJObject for String {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let result = JObject::from(env.new_string(self)?);
        Ok(result)
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
