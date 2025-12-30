use std::fmt;
use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use dropbear_macro::SerializableComponent;
use dropbear_traits::SerializableComponent;
use egui::Ui;
use crate::states::Property;

/// Properties for an entity, typically queries with `entity.getProperty<Float>` and `entity.setProperty(21)`
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, SerializableComponent)]
pub struct CustomProperties {
    pub custom_properties: Vec<Property>,
    pub next_id: u64,
}

impl CustomProperties {
    /// Creates a new [CustomProperties]
    pub fn new() -> Self {
        Self {
            custom_properties: Vec::new(),
            next_id: 0,
        }
    }

    /// Sets the property based on the [Value] (type) and its key.
    ///
    /// If the value does NOT exist, it will be created.
    /// If the value does exist, it will replace the contents of that item.
    pub fn set_property(&mut self, key: String, value: Value) {
        if let Some(prop) = self.custom_properties.iter_mut().find(|p| p.key == key) {
            prop.value = value;
        } else {
            self.custom_properties.push(Property {
                id: self.next_id,
                key,
                value,
            });
            self.next_id += 1;
        }
    }

    /// Fetches the property by its key.
    pub fn get_property(&self, key: &str) -> Option<&Value> {
        self.custom_properties
            .iter()
            .find(|p| p.key == key)
            .map(|p| &p.value)
    }

    /// Fetches the float property
    pub fn get_float(&self, key: &str) -> Option<f64> {
        match self.get_property(key)? {
            Value::Double(f) => Some(*f),
            _ => None,
        }
    }

    /// Fetches the integer property
    pub fn get_int(&self, key: &str) -> Option<i64> {
        match self.get_property(key)? {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// Creates a new property based on a key and a value.
    ///
    /// It will push that value again to the property vector.
    pub fn add_property(&mut self, key: String, value: Value) {
        self.custom_properties.push(Property {
            id: self.next_id,
            key,
            value,
        });
        self.next_id += 1;
    }

    /// Shows a template of the different values when inspected as a component in the editor.
    pub fn show_value_editor(ui: &mut Ui, value: &mut Value) -> bool {
        match value {
            Value::String(s) => ui.text_edit_singleline(s).changed(),
            Value::Int(i) => ui
                .add(egui::Slider::new(i, -1000..=1000).text(""))
                .changed(),
            Value::Double(f) => ui
                .add(egui::Slider::new(f, -100.0..=100.0).text(""))
                .changed(),
            Value::Bool(b) => ui.checkbox(b, "").changed(),
            Value::Vec3(vec) => {
                let mut changed = false;
                ui.horizontal(|ui| {
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut vec[0], -10.0..=10.0)
                                .text("X")
                                .fixed_decimals(2),
                        )
                        .changed();
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut vec[1], -10.0..=10.0)
                                .text("Y")
                                .fixed_decimals(2),
                        )
                        .changed();
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut vec[2], -10.0..=10.0)
                                .text("Z")
                                .fixed_decimals(2),
                        )
                        .changed();
                });
                changed
            }
        }
    }
}

impl Default for CustomProperties {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Value {
    String(String),
    Int(i64),
    Double(f64),
    Bool(bool),
    Vec3([f64; 3]),
}

impl Default for Value {
    fn default() -> Self {
        Self::String(String::new())
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let string: String = match self {
            Value::String(_) => "String".into(),
            Value::Int(_) => "Int".into(),
            Value::Double(_) => "Double".into(),
            Value::Bool(_) => "Bool".into(),
            Value::Vec3(_) => "Vec3".into(),
        };
        write!(f, "{}", string)
    }
}

pub mod shared {
    use std::ffi::CStr;
    use std::os::raw::c_char;
    use hecs::World;
    use crate::properties::CustomProperties;
    use crate::scripting::native::DropbearNativeError;
    use crate::scripting::result::DropbearNativeResult;

    pub fn custom_properties_exists_for_entity(world: &World, entity: hecs::Entity) -> bool {
        world.get::<&CustomProperties>(entity).is_ok()
    }

    pub(crate) unsafe fn read_key(ptr: *const c_char) -> DropbearNativeResult<String> {
        if ptr.is_null() {
            return DropbearNativeResult::Err(DropbearNativeError::NullPointer);
        }
        match { CStr::from_ptr(ptr) }.to_str() {
            Ok(s) => DropbearNativeResult::Ok(s.to_string()),
            Err(_) => DropbearNativeResult::Err(DropbearNativeError::InvalidUTF8),
        }
    }
}

pub mod jni {
    #![allow(non_snake_case)]
    use hecs::World;
    use jni::JNIEnv;
    use jni::objects::{JClass, JObject, JString, JValue};
    use jni::sys::{jboolean, jdouble, jfloat, jint, jlong, jobject, jstring};
    use glam::DVec3;

    use crate::properties::{CustomProperties, Value};
    use crate::scripting::jni::utils::{FromJObject, ToJObject};

    /// Returns a primitive that is boxed (long => java.lang.Long)
    ///
    /// ```
    /// return_boxed!(&mut env, Some(JValue::Int(21 as jint)), "(I)Ljava/lang/Integer;", "java/lang/Integer")
    /// ```
    #[macro_export]
    macro_rules! return_boxed {
        ($env:expr, $val:expr, $sig:expr, $wrapper:expr) => {
            match $val {
                Some(v) => {
                    let result = |env: &mut jni::JNIEnv| -> jni::errors::Result<jni::sys::jobject> {
                        let cls = env.find_class($wrapper)?;

                        let param: jni::objects::JValue = v.into();
                        let ret = env.call_static_method(cls, "valueOf", $sig, &[param])?;

                        Ok(ret.l()?.into_raw())
                    }($env);

                    match result {
                        Ok(ptr) => ptr,
                        Err(e) => {
                            eprintln!("return_boxed failed for {}: {:?}", $wrapper, e);

                            let _ = $env.throw_new("java/lang/RuntimeException", format!("Boxing failed: {:?}", e));

                            std::ptr::null_mut()
                        }
                    }
                }
                None => std::ptr::null_mut(),
            }
        };
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_components_CustomPropertiesNative_customPropertiesExistsForEntity(
        _env: JNIEnv,
        _class: JClass,
        world_handle: jlong,
        entity_id: jlong,
    ) -> jboolean {
        let world = crate::convert_ptr!(world_handle => World);
        let entity = crate::convert_jlong_to_entity!(entity_id);

        if super::shared::custom_properties_exists_for_entity(&world, entity) {
            1
        } else {
            0
        }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_components_CustomPropertiesNative_getStringProperty(
        mut env: JNIEnv,
        _class: JClass,
        world_handle: jlong,
        entity_id: jlong,
        key: JString,
    ) -> jstring {
        let world = crate::convert_ptr!(world_handle => World);
        let entity = crate::convert_jlong_to_entity!(entity_id);
        let key_str = crate::convert_jstring!(env, key);

        if let Ok(props) = world.get::<&CustomProperties>(entity) {
            if let Some(Value::String(s)) = props.get_property(&key_str) {
                return env.new_string(s).map(|s| s.into_raw()).unwrap_or(std::ptr::null_mut());
            }
        }
        std::ptr::null_mut()
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_components_CustomPropertiesNative_getIntProperty(
        mut env: JNIEnv,
        _class: JClass,
        world_handle: jlong,
        entity_id: jlong,
        key: JString,
    ) -> jobject {
        let world = crate::convert_ptr!(world_handle => World);
        let entity = crate::convert_jlong_to_entity!(entity_id);
        let key_str = crate::convert_jstring!(env, key);

        let val = if let Ok(props) = world.get::<&CustomProperties>(entity) {
            if let Some(Value::Int(v)) = props.get_property(&key_str) {
                Some(JValue::Int(*v as jint))
            } else { None }
        } else { None };

        return_boxed!(&mut env, val, "(I)Ljava/lang/Integer;", "java/lang/Integer")
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_components_CustomPropertiesNative_getLongProperty(
        mut env: JNIEnv,
        _class: JClass,
        world_handle: jlong,
        entity_id: jlong,
        key: JString,
    ) -> jobject {
        let world = crate::convert_ptr!(world_handle => World);
        let entity = crate::convert_jlong_to_entity!(entity_id);
        let key_str = crate::convert_jstring!(env, key);

        let val = if let Ok(props) = world.get::<&CustomProperties>(entity) {
            if let Some(Value::Int(v)) = props.get_property(&key_str) {
                Some(JValue::Long(*v))
            } else { None }
        } else { None };

        return_boxed!(&mut env, val, "(J)Ljava/lang/Long;", "java/lang/Long")
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_components_CustomPropertiesNative_getDoubleProperty(
        mut env: JNIEnv,
        _class: JClass,
        world_handle: jlong,
        entity_id: jlong,
        key: JString,
    ) -> jobject {
        let world = crate::convert_ptr!(world_handle => World);
        let entity = crate::convert_jlong_to_entity!(entity_id);
        let key_str = crate::convert_jstring!(env, key);

        let val = if let Ok(props) = world.get::<&CustomProperties>(entity) {
            if let Some(Value::Double(v)) = props.get_property(&key_str) {
                Some(JValue::Double(*v))
            } else { None }
        } else { None };

        return_boxed!(&mut env, val, "(D)Ljava/lang/Double;", "java/lang/Double")
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_components_CustomPropertiesNative_getFloatProperty(
        mut env: JNIEnv,
        _class: JClass,
        world_handle: jlong,
        entity_id: jlong,
        key: JString,
    ) -> jobject {
        let world = crate::convert_ptr!(world_handle => World);
        let entity = crate::convert_jlong_to_entity!(entity_id);
        let key_str = crate::convert_jstring!(env, key);

        let val = if let Ok(props) = world.get::<&CustomProperties>(entity) {
            if let Some(Value::Double(v)) = props.get_property(&key_str) {
                Some(JValue::Float(*v as jfloat))
            } else { None }
        } else { None };

        return_boxed!(&mut env, val, "(F)Ljava/lang/Float;", "java/lang/Float")
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_components_CustomPropertiesNative_getBoolProperty(
        mut env: JNIEnv,
        _class: JClass,
        world_handle: jlong,
        entity_id: jlong,
        key: JString,
    ) -> jobject {
        let world = crate::convert_ptr!(world_handle => World);
        let entity = crate::convert_jlong_to_entity!(entity_id);
        let key_str = crate::convert_jstring!(env, key);

        let val = if let Ok(props) = world.get::<&CustomProperties>(entity) {
            if let Some(Value::Bool(v)) = props.get_property(&key_str) {
                Some(JValue::Bool(if *v { 1 } else { 0 }))
            } else { None }
        } else { None };

        return_boxed!(&mut env, val, "(Z)Ljava/lang/Boolean;", "java/lang/Boolean")
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_components_CustomPropertiesNative_getVec3Property(
        mut env: JNIEnv,
        _class: JClass,
        world_handle: jlong,
        entity_id: jlong,
        key: JString,
    ) -> jobject {
        let world = crate::convert_ptr!(world_handle => World);
        let entity = crate::convert_jlong_to_entity!(entity_id);
        let key_str = crate::convert_jstring!(env, key);

        if let Ok(props) = world.get::<&CustomProperties>(entity) {
            if let Some(Value::Vec3(v)) = props.get_property(&key_str) {
                return match DVec3::from_array(*v).to_jobject(&mut env) {
                    Ok(obj) => obj.into_raw(),
                    Err(_) => std::ptr::null_mut()
                };
            }
        }
        std::ptr::null_mut()
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_components_CustomPropertiesNative_setStringProperty(
        mut env: JNIEnv,
        _class: JClass,
        world_handle: jlong,
        entity_id: jlong,
        key: JString,
        value: JString,
    ) {
        let world = crate::convert_ptr!(world_handle => World);
        let entity = crate::convert_jlong_to_entity!(entity_id);
        let key_str = crate::convert_jstring!(env, key);
        let val_str = crate::convert_jstring!(env, value);

        if let Ok(mut props) = world.get::<&mut CustomProperties>(entity) {
            props.set_property(key_str, Value::String(val_str));
        }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_components_CustomPropertiesNative_setIntProperty(
        mut env: JNIEnv,
        _class: JClass,
        world_handle: jlong,
        entity_id: jlong,
        key: JString,
        value: jint,
    ) {
        let world = crate::convert_ptr!(world_handle => World);
        let entity = crate::convert_jlong_to_entity!(entity_id);
        let key_str = crate::convert_jstring!(env, key);

        if let Ok(mut props) = world.get::<&mut CustomProperties>(entity) {
            props.set_property(key_str, Value::Int(value as i64));
        }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_components_CustomPropertiesNative_setLongProperty(
        mut env: JNIEnv,
        _class: JClass,
        world_handle: jlong,
        entity_id: jlong,
        key: JString,
        value: jlong,
    ) {
        let world = crate::convert_ptr!(world_handle => World);
        let entity = crate::convert_jlong_to_entity!(entity_id);
        let key_str = crate::convert_jstring!(env, key);

        if let Ok(mut props) = world.get::<&mut CustomProperties>(entity) {
            props.set_property(key_str, Value::Int(value));
        }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_components_CustomPropertiesNative_setFloatProperty(
        mut env: JNIEnv,
        _class: JClass,
        world_handle: jlong,
        entity_id: jlong,
        key: JString,
        value: jfloat,
    ) {
        let world = crate::convert_ptr!(world_handle => World);
        let entity = crate::convert_jlong_to_entity!(entity_id);
        let key_str = crate::convert_jstring!(env, key);

        if let Ok(mut props) = world.get::<&mut CustomProperties>(entity) {
            props.set_property(key_str, Value::Double(value as f64));
        }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_components_CustomPropertiesNative_setDoubleProperty(
        mut env: JNIEnv,
        _class: JClass,
        world_handle: jlong,
        entity_id: jlong,
        key: JString,
        value: jdouble,
    ) {
        let world = crate::convert_ptr!(world_handle => World);
        let entity = crate::convert_jlong_to_entity!(entity_id);
        let key_str = crate::convert_jstring!(env, key);

        if let Ok(mut props) = world.get::<&mut CustomProperties>(entity) {
            props.set_property(key_str, Value::Double(value));
        }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_components_CustomPropertiesNative_setBoolProperty(
        mut env: JNIEnv,
        _class: JClass,
        world_handle: jlong,
        entity_id: jlong,
        key: JString,
        value: jboolean,
    ) {
        let world = crate::convert_ptr!(world_handle => World);
        let entity = crate::convert_jlong_to_entity!(entity_id);
        let key_str = crate::convert_jstring!(env, key);

        if let Ok(mut props) = world.get::<&mut CustomProperties>(entity) {
            props.set_property(key_str, Value::Bool(value != 0));
        }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_components_CustomPropertiesNative_setVec3Property(
        mut env: JNIEnv,
        _class: JClass,
        world_handle: jlong,
        entity_id: jlong,
        key: JString,
        value: JObject,
    ) {
        let world = crate::convert_ptr!(world_handle => World);
        let entity = crate::convert_jlong_to_entity!(entity_id);
        let key_str = crate::convert_jstring!(env, key);

        let vec_val = match DVec3::from_jobject(&mut env, &value) {
            Ok(v) => v,
            Err(_) => return,
        };

        if let Ok(mut props) = world.get::<&mut CustomProperties>(entity) {
            props.set_property(key_str, Value::Vec3(vec_val.to_array()));
        }
    }
}

#[dropbear_macro::impl_c_api]
pub mod native {
    use hecs::{Entity, World};
    use std::ffi::CString;
    use std::os::raw::c_char;

    use crate::convert_ptr;
    use crate::properties::shared::read_key;
    use crate::properties::{CustomProperties, Value};
    use crate::ptr::WorldPtr;
    use crate::scripting::native::DropbearNativeError;
    use crate::scripting::result::DropbearNativeResult;
    use crate::types::Vector3;
    
    pub fn dropbear_custom_properties_exists_for_entity(
        world_ptr: WorldPtr,
        entity_id: u64,
    ) -> DropbearNativeResult<bool> {
        let world = convert_ptr!(world_ptr => World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        DropbearNativeResult::Ok(super::shared::custom_properties_exists_for_entity(world, entity))
    }


    pub fn dropbear_get_string_property(
        world_ptr: WorldPtr,
        entity_id: u64,
        key: *const c_char
    ) -> DropbearNativeResult<*mut c_char> {
        let world = convert_ptr!(world_ptr => World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;
        let key_str = unsafe { read_key(key)? };

        if let Ok(props) = world.get::<&CustomProperties>(entity) {
            if let Some(Value::String(s)) = props.get_property(&key_str) {
                return match CString::new(s.clone()) {
                    Ok(c) => DropbearNativeResult::Ok(c.into_raw()),
                    Err(_) => DropbearNativeResult::Err(DropbearNativeError::CStringError),
                };
            }
        }
        DropbearNativeResult::Err(DropbearNativeError::InvalidArgument)
    }

    pub fn dropbear_get_int_property(
        world_ptr: WorldPtr,
        entity_id: u64,
        key: *const c_char
    ) -> DropbearNativeResult<i32> {
        let world = convert_ptr!(world_ptr => World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;
        let key_str = unsafe { read_key(key)? };

        if let Ok(props) = world.get::<&CustomProperties>(entity) {
            if let Some(Value::Int(v)) = props.get_property(&key_str) {
                return DropbearNativeResult::Ok(*v as i32);
            }
        }
        DropbearNativeResult::Err(DropbearNativeError::InvalidArgument)
    }

    pub fn dropbear_get_long_property(
        world_ptr: WorldPtr,
        entity_id: u64,
        key: *const c_char
    ) -> DropbearNativeResult<i64> {
        let world = convert_ptr!(world_ptr => World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;
        let key_str = unsafe { read_key(key)? };

        if let Ok(props) = world.get::<&CustomProperties>(entity) {
            if let Some(Value::Int(v)) = props.get_property(&key_str) {
                return DropbearNativeResult::Ok(*v);
            }
        }
        DropbearNativeResult::Err(DropbearNativeError::InvalidArgument)
    }

    pub fn dropbear_get_double_property(
        world_ptr: WorldPtr,
        entity_id: u64,
        key: *const c_char
    ) -> DropbearNativeResult<f64> {
        let world = convert_ptr!(world_ptr => World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;
        let key_str = unsafe { read_key(key)? };

        if let Ok(props) = world.get::<&CustomProperties>(entity) {
            if let Some(Value::Double(v)) = props.get_property(&key_str) {
                return DropbearNativeResult::Ok(*v);
            }
        }
        DropbearNativeResult::Err(DropbearNativeError::InvalidArgument)
    }

    pub fn dropbear_get_float_property(
        world_ptr: WorldPtr,
        entity_id: u64,
        key: *const c_char
    ) -> DropbearNativeResult<f32> {
        let world = convert_ptr!(world_ptr => World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;
        let key_str = unsafe { read_key(key)? };

        if let Ok(props) = world.get::<&CustomProperties>(entity) {
            if let Some(Value::Double(v)) = props.get_property(&key_str) {
                return DropbearNativeResult::Ok(*v as f32);
            }
        }
        DropbearNativeResult::Err(DropbearNativeError::InvalidArgument)
    }

    pub fn dropbear_get_bool_property(
        world_ptr: WorldPtr,
        entity_id: u64,
        key: *const c_char
    ) -> DropbearNativeResult<bool> {
        let world = convert_ptr!(world_ptr => World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;
        let key_str = unsafe { read_key(key)? };

        if let Ok(props) = world.get::<&CustomProperties>(entity) {
            if let Some(Value::Bool(v)) = props.get_property(&key_str) {
                return DropbearNativeResult::Ok(*v);
            }
        }
        DropbearNativeResult::Err(DropbearNativeError::InvalidArgument)
    }

    pub fn dropbear_get_vec3_property(
        world_ptr: WorldPtr,
        entity_id: u64,
        key: *const c_char
    ) -> DropbearNativeResult<Vector3> {
        let world = convert_ptr!(world_ptr => World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;
        let key_str = unsafe { read_key(key)? };

        if let Ok(props) = world.get::<&CustomProperties>(entity) {
            if let Some(Value::Vec3(v)) = props.get_property(&key_str) {
                return DropbearNativeResult::Ok(Vector3::from(*v));
            }
        }
        DropbearNativeResult::Err(DropbearNativeError::InvalidArgument)
    }

    pub fn dropbear_set_string_property(
        world_ptr: WorldPtr,
        entity_id: u64,
        key: *const c_char,
        value: *const c_char
    ) -> DropbearNativeResult<()> {
        let world = convert_ptr!(world_ptr => World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;
        let key_str = unsafe { read_key(key)? };
        let val_str = unsafe { read_key(value)? };

        if let Ok(mut props) = world.get::<&mut CustomProperties>(entity) {
            props.set_property(key_str, Value::String(val_str));
            DropbearNativeResult::Ok(())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_set_int_property(
        world_ptr: WorldPtr,
        entity_id: u64,
        key: *const c_char,
        value: i32
    ) -> DropbearNativeResult<()> {
        let world = convert_ptr!(world_ptr => World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;
        let key_str = unsafe { read_key(key)? };

        if let Ok(mut props) = world.get::<&mut CustomProperties>(entity) {
            props.set_property(key_str, Value::Int(value as i64));
            DropbearNativeResult::Ok(())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_set_long_property(
        world_ptr: WorldPtr,
        entity_id: u64,
        key: *const c_char,
        value: i64
    ) -> DropbearNativeResult<()> {
        let world = convert_ptr!(world_ptr => World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;
        let key_str = unsafe { read_key(key)? };

        if let Ok(mut props) = world.get::<&mut CustomProperties>(entity) {
            props.set_property(key_str, Value::Int(value));
            DropbearNativeResult::Ok(())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_set_double_property(
        world_ptr: WorldPtr,
        entity_id: u64,
        key: *const c_char,
        value: f64
    ) -> DropbearNativeResult<()> {
        let world = convert_ptr!(world_ptr => World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;
        let key_str = unsafe { read_key(key)? };

        if let Ok(mut props) = world.get::<&mut CustomProperties>(entity) {
            props.set_property(key_str, Value::Double(value));
            DropbearNativeResult::Ok(())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_set_float_property(
        world_ptr: WorldPtr,
        entity_id: u64,
        key: *const c_char,
        value: f32
    ) -> DropbearNativeResult<()> {
        let world = convert_ptr!(world_ptr => World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;
        let key_str = unsafe { read_key(key)? };

        if let Ok(mut props) = world.get::<&mut CustomProperties>(entity) {
            props.set_property(key_str, Value::Double(value as f64));
            DropbearNativeResult::Ok(())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_set_bool_property(
        world_ptr: WorldPtr,
        entity_id: u64,
        key: *const c_char,
        value: bool
    ) -> DropbearNativeResult<()> {
        let world = convert_ptr!(world_ptr => World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;
        let key_str = unsafe { read_key(key)? };

        if let Ok(mut props) = world.get::<&mut CustomProperties>(entity) {
            props.set_property(key_str, Value::Bool(value));
            DropbearNativeResult::Ok(())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_set_vec3_property(
        world_ptr: WorldPtr,
        entity_id: u64,
        key: *const c_char,
        value: Vector3
    ) -> DropbearNativeResult<()> {
        let world = convert_ptr!(world_ptr => World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;
        let key_str = unsafe { read_key(key)? };

        if let Ok(mut props) = world.get::<&mut CustomProperties>(entity) {
            props.set_property(key_str, Value::Vec3(value.to_array()));
            DropbearNativeResult::Ok(())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }
}