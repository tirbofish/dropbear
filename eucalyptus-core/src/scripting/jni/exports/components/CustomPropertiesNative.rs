use hecs::World;
use jni::JNIEnv;
use jni::objects::{JClass, JPrimitiveArray, JString};
use jni::sys::{jboolean, jdouble, jfloatArray, jint, jlong, jstring};
use dropbear_engine::entity::MeshRenderer;
use crate::convert_jlong_to_entity;
use crate::states::{ModelProperties, Value};

/**
 * Class:     `com_dropbear_ffi_components_CustomPropertiesNative`
 *
 * Method:    `getStringProperty`
 *
 * Signature: `(JJLjava/lang/String;)Ljava/lang/String;`
 *
 * `JNIEXPORT jstring JNICALL Java_com_dropbear_ffi_components_CustomPropertiesNative_getStringProperty
 * (JNIEnv *, jclass, jlong, jlong, jstring);`
*/
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_CustomPropertiesNative_getStringProperty(
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    entity_id: jlong,
    property_name: JString,
) -> jstring {
    let world = world_handle as *mut World;
    if world.is_null() {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getStringProperty] [ERROR] World pointer is null"
        );
        return std::ptr::null_mut();
    }

    let world = unsafe { &mut *world };
    let entity = convert_jlong_to_entity!(entity_id);
    if let Ok(mut q) = world.query_one::<(&MeshRenderer, &ModelProperties)>(entity)
        && let Some((_, props)) = q.get()
    {
        let string = env.get_string(&property_name);
        let value: String = if let Ok(str) = string {
            let value = str.to_string_lossy();
            value.to_string()
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getStringProperty] [ERROR] Failed to get property name"
            );
            return std::ptr::null_mut();
        };
        let output = props.get_property(&value);
        if let Some(output) = output {
            match output {
                Value::String(val) => match env.new_string(val) {
                    Ok(string) => string.as_raw(),
                    Err(e) => {
                        eprintln!(
                            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getStringProperty] [ERROR] Failed to create string: {}",
                            e
                        );
                        std::ptr::null_mut()
                    }
                },
                _ => {
                    println!(
                        "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getStringProperty] [WARN] Property is not a string"
                    );
                    std::ptr::null_mut()
                }
            }
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getStringProperty] [WARN] Property not found"
            );
            std::ptr::null_mut()
        }
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getStringProperty] [ERROR] Failed to query entity for model properties"
        );
        std::ptr::null_mut()
    }
}


/**
 * Fetches a [`jint`]/[`i32`] value from a key value.
 *
 * If the value does not exist, it will return `650911`, a randomly generated number
 * that is extremely specific that no one would be sane enough to use this as a property.
 *
 * Class: `com_dropbear_ffi_components_CustomPropertiesNative`
 *
 * Method: `getIntProperty`
 *
 * Signature: `(JJLjava/lang/String;)I`
 *
 * `JNIEXPORT jint JNICALL Java_com_dropbear_ffi_components_CustomPropertiesNative_getIntProperty
 * (JNIEnv *, jclass, jlong, jlong, jstring);`
*/
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_CustomPropertiesNative_getIntProperty(
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    entity_id: jlong,
    property_name: JString,
) -> jint {
    let world = world_handle as *mut World;
    if world.is_null() {
        eprintln!("[Java_com_dropbear_ffi_components_CustomPropertiesNative_getIntProperty] [ERROR] World pointer is null");
        return 650911;
    }

    let world = unsafe { &mut *world };
    let entity = convert_jlong_to_entity!(entity_id);
    if let Ok(mut q) = world.query_one::<(&MeshRenderer, &ModelProperties)>(entity)
        && let Some((_, props)) = q.get()
    {
        let string = env.get_string(&property_name);
        let value: String = if let Ok(str) = string {
            let value = str.to_string_lossy();
            value.to_string()
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getIntProperty] [ERROR] Failed to get property name"
            );
            return 650911;
        };
        let output = props.get_property(&value);
        if let Some(output) = output {
            match output {
                Value::Int(val) => *val as jint,
                _ => {
                    eprintln!(
                        "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getIntProperty] [WARN] Property is not an int"
                    );
                    650911
                }
            }
        } else {
            eprintln!("[Java_com_dropbear_ffi_components_CustomPropertiesNative_getIntProperty] [WARN] Property not found");
            650911
        }
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getIntProperty] [ERROR] Failed to query entity for model properties"
        );
        650911
    }
}


/**
 * Gets a [`jlong`]/[`i64`] property.
 *
 * If the value doesn't exist, it will return this value: `6509112938`. This is a random number
 * from a generator I got, and it is such a specific number that no one would ever have this number
 * in one of their properties.
 *
 * Class:     `com_dropbear_ffi_components_CustomPropertiesNative`
 *
 * Method:    `getLongProperty`
 *
 * Signature: `(JJLjava/lang/String;)J`
 *
 * `JNIEXPORT jlong JNICALL Java_com_dropbear_ffi_components_CustomPropertiesNative_getLongProperty
 * (JNIEnv *, jclass, jlong, jlong, jstring);`
*/
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_CustomPropertiesNative_getLongProperty(
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    entity_id: jlong,
    property_name: JString,
) -> jlong {
    let world = world_handle as *mut World;
    if world.is_null() {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getLongProperty] [ERROR] World pointer is null"
        );
        return 6509112938;
    }

    let world = unsafe { &mut *world };
    let entity = convert_jlong_to_entity!(entity_id);
    if let Ok(mut q) = world.query_one::<(&MeshRenderer, &ModelProperties)>(entity)
        && let Some((_, props)) = q.get()
    {
        let string = env.get_string(&property_name);
        let value: String = if let Ok(str) = string {
            let value = str.to_string_lossy();
            value.to_string()
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getLongProperty] [ERROR] Failed to get property name"
            );
            return 0;
        };
        let output = props.get_property(&value);
        if let Some(output) = output {
            match output {
                Value::Int(val) => *val as jlong,
                _ => {
                    eprintln!(
                        "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getLongProperty] [WARN] Property is not a long"
                    );
                    6509112938
                }
            }
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getLongProperty] [WARN] Property not found"
            );
            6509112938
        }
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getLongProperty] [ERROR] Failed to query entity for model properties"
        );
        6509112938
    }
}

/**
 * Class:     `com_dropbear_ffi_components_CustomPropertiesNative`
 *
 * Method:    `getFloatProperty`
 *
 * Signature: `(JJLjava/lang/String;)D`
 *
 * `JNIEXPORT jdouble JNICALL Java_com_dropbear_ffi_components_CustomPropertiesNative_getFloatProperty
 * (JNIEnv *, jclass, jlong, jlong, jstring);`
*/
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_CustomPropertiesNative_getFloatProperty(
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    entity_id: jlong,
    property_name: JString,
) -> jdouble {
    let world = world_handle as *mut World;
    if world.is_null() {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getFloatProperty] [ERROR] World pointer is null"
        );
        return f64::NAN;
    }

    let world = unsafe { &mut *world };
    let entity = convert_jlong_to_entity!(entity_id);
    if let Ok(mut q) = world.query_one::<(&MeshRenderer, &ModelProperties)>(entity)
        && let Some((_, props)) = q.get()
    {
        let string = env.get_string(&property_name);
        let value: String = if let Ok(str) = string {
            let value = str.to_string_lossy();
            value.to_string()
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getFloatProperty] [ERROR] Failed to get property name"
            );
            return f64::NAN;
        };
        let output = props.get_property(&value);
        if let Some(output) = output {
            match output {
                Value::Float(val) => *val as jdouble,
                _ => {
                    eprintln!(
                        "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getFloatProperty] [WARN] Property is not a float"
                    );
                    f64::NAN
                }
            }
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getFloatProperty] [WARN] Property not found"
            );
            f64::NAN
        }
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getFloatProperty] [ERROR] Failed to query entity for model properties"
        );
        f64::NAN
    }
}


/**
 * Class:     `com_dropbear_ffi_components_CustomPropertiesNative`
 *
 * Method:    `getBoolProperty`
 *
 * Signature: `(JJLjava/lang/String;)Z`
 *
 * `JNIEXPORT jboolean JNICALL Java_com_dropbear_ffi_components_CustomPropertiesNative_getBoolProperty
 * (JNIEnv *, jclass, jlong, jlong, jstring);`
*/
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_CustomPropertiesNative_getBoolProperty(
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    entity_id: jlong,
    property_name: JString,
) -> jboolean {
    let world = world_handle as *mut World;
    if world.is_null() {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getBoolProperty] [ERROR] World pointer is null"
        );
        return 0;
    }

    let world = unsafe { &mut *world };
    let entity = convert_jlong_to_entity!(entity_id);
    if let Ok(mut q) = world.query_one::<(&MeshRenderer, &ModelProperties)>(entity)
        && let Some((_, props)) = q.get()
    {
        let string = env.get_string(&property_name);
        let value: String = if let Ok(str) = string {
            let value = str.to_string_lossy();
            value.to_string()
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getBoolProperty] [ERROR] Failed to get property name"
            );
            return 0;
        };
        let output = props.get_property(&value);
        if let Some(output) = output {
            match output {
                Value::Bool(val) => {
                    if *val {
                        1
                    } else {
                        0
                    }
                }
                _ => {
                    eprintln!(
                        "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getBoolProperty] [WARN] Property is not a bool"
                    );
                    0
                }
            }
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getBoolProperty] [WARN] Property not found"
            );
            0
        }
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getBoolProperty] [ERROR] Failed to query entity for model properties"
        );
        0
    }
}

/**
 * Class:     `com_dropbear_ffi_components_CustomPropertiesNative`
 *
 * Method:    `getVec3Property`
 *
 * Signature: `(JJLjava/lang/String;)[F`
 *
 * `JNIEXPORT jfloatArray JNICALL Java_com_dropbear_ffi_components_CustomPropertiesNative_getVec3Property
 * (JNIEnv *, jclass, jlong, jlong, jstring);`
*/
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_CustomPropertiesNative_getVec3Property(
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    entity_id: jlong,
    property_name: JString,
) -> jfloatArray {
    let world = world_handle as *mut World;
    if world.is_null() {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getVec3Property] [ERROR] World pointer is null"
        );
        return std::ptr::null_mut();
    }

    let world = unsafe { &mut *world };
    let entity = convert_jlong_to_entity!(entity_id);
    if let Ok(mut q) = world.query_one::<(&MeshRenderer, &ModelProperties)>(entity)
        && let Some((_, props)) = q.get()
    {
        let string = env.get_string(&property_name);
        let value: String = if let Ok(str) = string {
            let value = str.to_string_lossy();
            value.to_string()
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getVec3Property] [ERROR] Failed to get property name"
            );
            return std::ptr::null_mut();
        };
        let output = props.get_property(&value);
        if let Some(output) = output {
            match output {
                Value::Vec3([x, y, z]) => {
                    let arr = env.new_float_array(3);
                    if let Ok(arr) = arr {
                        let values = [*x, *y, *z];
                        if env.set_float_array_region(&arr, 0, &values).is_ok() {
                            arr.into_raw()
                        } else {
                            eprintln!(
                                "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getVec3Property] [ERROR] Failed to set array region"
                            );
                            std::ptr::null_mut()
                        }
                    } else {
                        eprintln!(
                            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getVec3Property] [ERROR] Failed to create float array"
                        );
                        std::ptr::null_mut()
                    }
                }
                _ => {
                    eprintln!(
                        "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getVec3Property] [WARN] Property is not a vec3"
                    );
                    std::ptr::null_mut()
                }
            }
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getVec3Property] [WARN] Property not found"
            );
            std::ptr::null_mut()
        }
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_getVec3Property] [ERROR] Failed to query entity for model properties"
        );
        std::ptr::null_mut()
    }
}


/**
 * Class:     `com_dropbear_ffi_components_CustomPropertiesNative`
 *
 * Method:    `setStringProperty`
 *
 * Signature: `(JJLjava/lang/String;Ljava/lang/String;)V`
 *
 * `JNIEXPORT void JNICALL Java_com_dropbear_ffi_components_CustomPropertiesNative_setStringProperty
 * (JNIEnv *, jclass, jlong, jlong, jstring, jstring);`
*/
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_CustomPropertiesNative_setStringProperty(
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    entity_id: jlong,
    property_name: JString,
    value: JString,
) {
    let world = world_handle as *mut World;
    if world.is_null() {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_setStringProperty] [ERROR] World pointer is null"
        );
        return;
    }

    let world = unsafe { &mut *world };
    let entity = convert_jlong_to_entity!(entity_id);

    let key = env.get_string(&property_name);
    let key: String = if let Ok(str) = key {
        let value = str.to_string_lossy();
        value.to_string()
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_setStringProperty] [ERROR] Failed to get property name"
        );
        return;
    };

    let string = env.get_string(&value);
    let value: String = if let Ok(str) = string {
        let value = str.to_string_lossy();
        value.to_string()
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_setStringProperty] [ERROR] Failed to get property name"
        );
        return;
    };

    if let Ok((_, props)) = world.query_one_mut::<(&MeshRenderer, &mut ModelProperties)>(entity) {
        props.set_property(key, Value::String(value));
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_setStringProperty] [ERROR] Failed to query entity for model properties"
        );
    }
}


/**
 * Class:     `com_dropbear_ffi_components_CustomPropertiesNative`
 *
 * Method:    `setIntProperty`
 *
 * Signature: `(JJLjava/lang/String;I)V`
 *
 * `JNIEXPORT void JNICALL Java_com_dropbear_ffi_components_CustomPropertiesNative_setIntProperty
 * (JNIEnv *, jclass, jlong, jlong, jstring, jint);`
*/
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_CustomPropertiesNative_setIntProperty(
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    entity_id: jlong,
    property_name: JString,
    value: jint,
) {
    let world = world_handle as *mut World;
    if world.is_null() {
        eprintln!("[Java_com_dropbear_ffi_components_CustomPropertiesNative_setIntProperty] [ERROR] World pointer is null");
        return;
    }

    let world = unsafe { &mut *world };
    let entity = convert_jlong_to_entity!(entity_id);

    let key = env.get_string(&property_name);
    let key: String = if let Ok(str) = key {
        let value = str.to_string_lossy();
        value.to_string()
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_setIntProperty] [ERROR] Failed to get property name"
        );
        return;
    };

    if let Ok((_, props)) = world.query_one_mut::<(&MeshRenderer, &mut ModelProperties)>(entity) {
        props.set_property(key, Value::Int(value as i64));
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_setIntProperty] [ERROR] Failed to query entity for model properties"
        );
    }
}

/**
 * Class:     `com_dropbear_ffi_components_CustomPropertiesNative`
 *
 * Method:    `setLongProperty`
 *
 * Signature: `(JJLjava/lang/String;J)V`
 *
 * `JNIEXPORT void JNICALL Java_com_dropbear_ffi_components_CustomPropertiesNative_setLongProperty
 * (JNIEnv *, jclass, jlong, jlong, jstring, jlong);`
*/
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_CustomPropertiesNative_setLongProperty(
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    entity_id: jlong,
    property_name: JString,
    value: jlong,
) {
    let world = world_handle as *mut World;
    if world.is_null() {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_setLongProperty] [ERROR] World pointer is null"
        );
        return;
    }

    let world = unsafe { &mut *world };
    let entity = convert_jlong_to_entity!(entity_id);

    let key = env.get_string(&property_name);
    let key: String = if let Ok(str) = key {
        let value = str.to_string_lossy();
        value.to_string()
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_setLongProperty] [ERROR] Failed to get property name"
        );
        return;
    };

    if let Ok((_, props)) = world.query_one_mut::<(&MeshRenderer, &mut ModelProperties)>(entity) {
        props.set_property(key, Value::Int(value));
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_setLongProperty] [ERROR] Failed to query entity for model properties"
        );
    }
}

/**
 * Class:     `com_dropbear_ffi_components_CustomPropertiesNative`
 *
 * Method:    `setFloatProperty`
 *
 * Signature: `(JJLjava/lang/String;D)V`
 *
 * `JNIEXPORT void JNICALL Java_com_dropbear_ffi_components_CustomPropertiesNative_setFloatProperty
 * (JNIEnv *, jclass, jlong, jlong, jstring, jdouble);`
*/
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_CustomPropertiesNative_setFloatProperty(
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    entity_id: jlong,
    property_name: JString,
    value: jdouble,
) {
    let world = world_handle as *mut World;
    if world.is_null() {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_setFloatProperty] [ERROR] World pointer is null"
        );
        return;
    }

    let world = unsafe { &mut *world };
    let entity = convert_jlong_to_entity!(entity_id);

    let key = env.get_string(&property_name);
    let key: String = if let Ok(str) = key {
        let value = str.to_string_lossy();
        value.to_string()
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_setFloatProperty] [ERROR] Failed to get property name"
        );
        return;
    };

    if let Ok((_, props)) = world.query_one_mut::<(&MeshRenderer, &mut ModelProperties)>(entity) {
        props.set_property(key, Value::Float(value));
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_setFloatProperty] [ERROR] Failed to query entity for model properties"
        );
    }
}

/**
 * Class:     `com_dropbear_ffi_components_CustomPropertiesNative`
 *
 * Method:    `setBoolProperty`
 *
 * Signature: `(JJLjava/lang/String;Z)V`
 *
 * `JNIEXPORT void JNICALL Java_com_dropbear_ffi_components_CustomPropertiesNative_setBoolProperty
 * (JNIEnv *, jclass, jlong, jlong, jstring, jboolean);`
*/
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_CustomPropertiesNative_setBoolProperty(
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    entity_id: jlong,
    property_name: JString,
    value: jboolean,
) {
    let world = world_handle as *mut World;
    if world.is_null() {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_setBoolProperty] [ERROR] World pointer is null"
        );
        return;
    }

    let world = unsafe { &mut *world };
    let entity = convert_jlong_to_entity!(entity_id);

    let key = env.get_string(&property_name);
    let key: String = if let Ok(str) = key {
        let value = str.to_string_lossy();
        value.to_string()
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_setBoolProperty] [ERROR] Failed to get property name"
        );
        return;
    };

    let bool_value = value != 0;

    if let Ok((_, props)) = world.query_one_mut::<(&MeshRenderer, &mut ModelProperties)>(entity) {
        props.set_property(key, Value::Bool(bool_value));
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_setBoolProperty] [ERROR] Failed to query entity for model properties"
        );
    }
}


/**
 * Class:     `com_dropbear_ffi_components_CustomPropertiesNative`
 *
 * Method:    `setVec3Property`
 *
 * Signature: `(JJLjava/lang/String;[F)V`
 *
 * `JNIEXPORT void JNICALL Java_com_dropbear_ffi_components_CustomPropertiesNative_setVec3Property
 * (JNIEnv *, jclass, jlong, jlong, jstring, jfloatArray);`
*/
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_CustomPropertiesNative_setVec3Property(
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    entity_id: jlong,
    property_name: JString,
    value: jfloatArray,
) {
    let world = world_handle as *mut World;
    if world.is_null() {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_setVec3Property] [ERROR] World pointer is null"
        );
        return;
    }

    if value.is_null() {
        eprintln!("[Java_com_dropbear_ffi_components_CustomPropertiesNative_setVec3Property] [ERROR] Value array is null");
        return;
    }

    let world = unsafe { &mut *world };
    let entity = convert_jlong_to_entity!(entity_id);
    #[allow(unused_unsafe)]
    let val = unsafe { value };
    let array = unsafe { JPrimitiveArray::from_raw(val) };

    let key = env.get_string(&property_name);
    let key: String = if let Ok(str) = key {
        let value = str.to_string_lossy();
        value.to_string()
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_setVec3Property] [ERROR] Failed to get property name"
        );
        return;
    };

    let length = env.get_array_length(&array);

    if let Ok(length) = length {
        if length != 3 {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CustomPropertiesNative_setVec3Property] [ERROR] Vec3 array must have exactly 3 elements, got {}",
                length
            );
            return;
        }
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_setVec3Property] [ERROR] Failed to get array length"
        );
        return;
    }

    let mut values = [0.0f32; 3];
    if env.get_float_array_region(&array, 0, &mut values).is_err() {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_setVec3Property] [ERROR] Failed to get array region"
        );
        return;
    }

    if let Ok((_, props)) = world.query_one_mut::<(&MeshRenderer, &mut ModelProperties)>(entity) {
        props.set_property(key, Value::Vec3(values));
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CustomPropertiesNative_setVec3Property] [ERROR] Failed to query entity for model properties"
        );
    }
}