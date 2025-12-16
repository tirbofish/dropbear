use hecs::World;
use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jlong, jlongArray};
use crate::{convert_jlong_to_entity, convert_jstring, convert_ptr};
use crate::hierarchy::{Children, Parent};
use crate::ptr::WorldPtr;
use crate::states::Label;

/**
 * Class:     `com_dropbear_ffi_components_HierarchyNative`
 *
 * Method:    `getChildren`
 *
 * Signature: `(JJ)[J`
 *
 * `JNIEXPORT jlongArray JNICALL Java_com_dropbear_ffi_components_HierarchyNative_getChildren
 * (JNIEnv *, jclass, jlong, jlong);`
*/
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_HierarchyNative_getChildren(
    env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    entity_id: jlong,
) -> jlongArray {
    let world = convert_ptr!(world_handle, WorldPtr => World);
    let entity = convert_jlong_to_entity!(entity_id);

    let entities = if let Ok(mut q) = world.query_one::<&Children>(entity)
        && let Some(children) = q.get()
    {
        let children = children.children();
        let mut array = vec![];
        for child in children {
            array.push(child.to_bits().get() as i64);
        }
        array
    } else {
        vec![]
    };

    let array = match env.new_long_array(entities.len() as i32) {
        Ok(array) => array,
        Err(e) => {
            return crate::ffi_error_return!("Unable to create a new long array: {}", e);
        }
    };

    if let Err(e) = env.set_long_array_region(&array, 0, &entities) {
        return crate::ffi_error_return!("Unable to populate long array: {}", e);
    }

    array.into_raw()
}

/**
 * Class:     `com_dropbear_ffi_components_HierarchyNative`
 *
 * Method:    `getChildByLabel`
 *
 * Signature: `(JJLjava/lang/String;)J`
 *
 * `JNIEXPORT jlong JNICALL Java_com_dropbear_ffi_components_HierarchyNative_getChildByLabel
 * (JNIEnv *, jclass, jlong, jlong, jstring);`
*/
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_HierarchyNative_getChildByLabel(
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    entity_id: jlong,
    label: JString,
) -> jlong {
    let world = convert_ptr!(world_handle, WorldPtr => World);
    let entity = convert_jlong_to_entity!(entity_id);
    let target = convert_jstring!(env, label);

    if let Ok(mut q) = world.query_one::<&Children>(entity)
        && let Some(children) = q.get()
    {
        for child in children.children() {
            if let Ok(label) = world.get::<&Label>(entity) {
                if label.as_str() == target {
                    return child.to_bits().get() as jlong;
                }
            } else {
                // skip if error or no entity
                continue;
            }
        }
    } else {
        // no children exist for the entity
        return -2 as jlong;
    };

    // no children exist with that label
    -2 as jlong
}

/**
 * Class:     com_dropbear_ffi_components_HierarchyNative
 *
 * Method:    getParent
 *
 * Signature: (JJ)J
 * 
 * `JNIEXPORT jlong JNICALL Java_com_dropbear_ffi_components_HierarchyNative_getParent
 * (JNIEnv *, jclass, jlong, jlong);`
*/
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_HierarchyNative_getParent(
    _env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    entity_id: jlong,
) -> jlong {
    let world = convert_ptr!(world_handle, WorldPtr => World);
    let entity = convert_jlong_to_entity!(entity_id);

    if let Ok(mut q) = world.query_one::<&Parent>(entity) {
        if let Some(parent) = q.get() {
            parent.parent().to_bits().get() as jlong
        } else {
            -2 as jlong
        }
    } else {
        crate::ffi_error_return!("No entity exists")
    }
}