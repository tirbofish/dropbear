#![allow(non_snake_case)]

use glam::{DQuat, DVec3};
use hecs::World;
use jni::JNIEnv;
use jni::objects::{JClass, JObject, JValue};
use jni::sys::{jclass, jlong, jobject};
use dropbear_engine::entity::{EntityTransform, Transform};
use crate::convert_jlong_to_entity;
use crate::hierarchy::EntityTransformExt;

/**
 * Class:     `com_dropbear_ffi_components_EntityTransformNative`
 *
 * Method:    `getTransform`
 *
 * Signature: `(JJ)Lcom/dropbear/EntityTransform;`
 *
 * `JNIEXPORT jobject JNICALL Java_com_dropbear_ffi_components_EntityTransformNative_getTransform
 * (JNIEnv *, jclass, jlong, jlong);`
*/
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_EntityTransformNative_getTransform(
    mut env: JNIEnv,
    _class: jclass,
    world_handle: jlong,
    entity_id: jlong,
) -> JObject {
    let world = world_handle as *mut World;

    if world.is_null() {
        println!("[Java_com_dropbear_ffi_components_EntityTransformNative_getTransform] [ERROR] World pointer is null");
        return JObject::null();
    }

    let world = unsafe { &mut *world };

    let entity = convert_jlong_to_entity!(entity_id);

    if let Ok(mut q) = world.query_one::<&EntityTransform>(entity)
        && let Some(transform) = q.get()
    {
        let wt = *transform.world();

        let transform_class = match env.find_class("com/dropbear/math/Transform") {
            Ok(c) => c,
            Err(_) => return JObject::null(),
        };

        let world_transform_java = match env.new_object(
            &transform_class,
            "(DDDDDDDDDD)V",
            &[
                wt.position.x.into(),
                wt.position.y.into(),
                wt.position.z.into(),
                wt.rotation.x.into(),
                wt.rotation.y.into(),
                wt.rotation.z.into(),
                wt.rotation.w.into(),
                wt.scale.x.into(),
                wt.scale.y.into(),
                wt.scale.z.into(),
            ],
        ) {
            Ok(java_transform) => java_transform,
            Err(_) => {
                println!(
                    "[Java_com_dropbear_ffi_components_EntityTransformNative_getTransform] [ERROR] Failed to create world transform object"
                );
                return JObject::null();
            }
        };

        let lt = *transform.local();

        let local_transform_java = match env.new_object(
            &transform_class,
            "(DDDDDDDDDD)V",
            &[
                lt.position.x.into(),
                lt.position.y.into(),
                lt.position.z.into(),
                lt.rotation.x.into(),
                lt.rotation.y.into(),
                lt.rotation.z.into(),
                lt.rotation.w.into(),
                lt.scale.x.into(),
                lt.scale.y.into(),
                lt.scale.z.into(),
            ],
        ) {
            Ok(java_transform) => java_transform,
            Err(_) => {
                println!(
                    "[Java_com_dropbear_ffi_components_EntityTransformNative_getTransform] [ERROR] Failed to create local transform object"
                );
                return JObject::null();
            }
        };

        let entity_transform_class = match env.find_class("com/dropbear/EntityTransform") {
            Ok(c) => c,
            Err(_) => return JObject::null(),
        };

        return match env.new_object(
            &entity_transform_class,
            "(Lcom/dropbear/math/Transform;Lcom/dropbear/math/Transform;)V",
            &[
                JValue::Object(&local_transform_java),
                JValue::Object(&world_transform_java),
            ],
        ) {
            Ok(java_transform) => java_transform,
            Err(e) => {
                println!(
                    "[Java_com_dropbear_ffi_components_EntityTransformNative_getTransform] [ERROR] Failed to create Transform object: {}",
                    e
                );
                JObject::null()
            }
        };
    }

    println!(
        "[Java_com_dropbear_ffi_components_EntityTransformNative_getTransform] [ERROR] Failed to query for transform value for entity: {}",
        entity_id
    );
    JObject::null()
}

/**
 * Class:     com_dropbear_ffi_components_EntityTransformNative
 *
 * Method:    propagateTransform
 *
 * Signature: (JJ)Lcom/dropbear/math/Transform;
 *
 * `JNIEXPORT jobject JNICALL Java_com_dropbear_ffi_components_EntityTransformNative_propagateTransform
 * (JNIEnv *, jclass, jlong, jlong);`
*/
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_EntityTransformNative_propagateTransform(
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    entity_id: jlong,
) -> jobject {
    let world = world_handle as *mut World;

    if world.is_null() {
        println!(
            "[Java_com_dropbear_ffi_components_EntityTransformNative_propagateTransform] [ERROR] World pointer is null"
        );
        return std::ptr::null_mut();
    }

    let world = unsafe { &mut *world };
    let entity = convert_jlong_to_entity!(entity_id);

    if let Ok(mut q) = world.query_one::<&mut EntityTransform>(entity) {
        if let Some(entity_transform) = q.get() {
            let new_world = entity_transform.propagate(world, entity);

            let transform_class = match env.find_class("com/dropbear/math/Transform") {
                Ok(c) => c,
                Err(e) => {
                    println!(
                        "[Java_com_dropbear_ffi_components_EntityTransformNative_propagateTransform] [ERROR] Failed to find Transform class: {:?}",
                        e
                    );
                    return std::ptr::null_mut();
                }
            };

            let transform_obj = match env.new_object(
                &transform_class,
                "(DDDDDDDDDD)V",
                &[
                    new_world.position.x.into(),
                    new_world.position.y.into(),
                    new_world.position.z.into(),
                    new_world.rotation.x.into(),
                    new_world.rotation.y.into(),
                    new_world.rotation.z.into(),
                    new_world.rotation.w.into(),
                    new_world.scale.x.into(),
                    new_world.scale.y.into(),
                    new_world.scale.z.into(),
                ],
            ) {
                Ok(obj) => obj,
                Err(e) => {
                    println!(
                        "[Java_com_dropbear_ffi_components_EntityTransformNative_propagateTransform] [ERROR] Failed to create Transform object: {:?}",
                        e
                    );
                    return std::ptr::null_mut();
                }
            };

            transform_obj.into_raw()
        } else {
            println!(
                "[Java_com_dropbear_ffi_components_EntityTransformNative_propagateTransform] [ERROR] Failed to get entity transform"
            );
            std::ptr::null_mut()
        }
    } else {
        println!(
            "[Java_com_dropbear_ffi_components_EntityTransformNative_propagateTransform] [ERROR] Entity does not have EntityTransform component"
        );
        std::ptr::null_mut()
    }
}

/**
 * Class:     `com_dropbear_ffi_components_EntityTransformNative`
 *
 * Method:    `setTransform`
 *
 * Signature: `(JJLcom/dropbear/EntityTransform;)V`
 *
 * `JNIEXPORT void JNICALL Java_com_dropbear_ffi_components_EntityTransformNative_setTransform
 * (JNIEnv *, jclass, jlong, jlong, jobject);`
*/
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_EntityTransformNative_setTransform(
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    entity_id: jlong,
    entity_transform_obj: JObject,
) {
    let world = world_handle as *mut World;

    if world.is_null() {
        println!("[Java_com_dropbear_ffi_components_EntityTransformNative_setTransform] [ERROR] World pointer is null");
        return;
    }

    let world = unsafe { &mut *world };
    let entity = convert_jlong_to_entity!(entity_id);

    let extract_transform = |env: &mut JNIEnv, transform_obj: &JObject| -> Option<Transform> {
        let get_number_field = |env: &mut JNIEnv, obj: &JObject, field_name: &str| -> f64 {
            match env.get_field(obj, field_name, "Ljava/lang/Number;") {
                Ok(v) => match v.l() {
                    Ok(num_obj) => match env.call_method(&num_obj, "doubleValue", "()D", &[]) {
                        Ok(result) => result.d().unwrap_or(0.0),
                        Err(_) => 0.0,
                    },
                    Err(_) => 0.0,
                },
                Err(_) => 0.0,
            }
        };

        let position_obj: JObject =
            match env.get_field(transform_obj, "position", "Lcom/dropbear/math/Vector3;") {
                Ok(v) => v.l().ok()?,
                Err(_) => return None,
            };

        let rotation_obj: JObject =
            match env.get_field(transform_obj, "rotation", "Lcom/dropbear/math/Quaternion;") {
                Ok(v) => v.l().ok()?,
                Err(_) => return None,
            };

        let scale_obj: JObject =
            match env.get_field(transform_obj, "scale", "Lcom/dropbear/math/Vector3;") {
                Ok(v) => v.l().ok()?,
                Err(_) => return None,
            };

        let px = get_number_field(env, &position_obj, "x");
        let py = get_number_field(env, &position_obj, "y");
        let pz = get_number_field(env, &position_obj, "z");

        let rx = get_number_field(env, &rotation_obj, "x");
        let ry = get_number_field(env, &rotation_obj, "y");
        let rz = get_number_field(env, &rotation_obj, "z");
        let rw = get_number_field(env, &rotation_obj, "w");

        let sx = get_number_field(env, &scale_obj, "x");
        let sy = get_number_field(env, &scale_obj, "y");
        let sz = get_number_field(env, &scale_obj, "z");

        Some(Transform {
            position: DVec3::new(px, py, pz),
            rotation: DQuat::from_xyzw(rx, ry, rz, rw),
            scale: DVec3::new(sx, sy, sz),
        })
    };

    let local_obj: JObject = match env.get_field(
        &entity_transform_obj,
        "local",
        "Lcom/dropbear/math/Transform;",
    ) {
        Ok(v) => v.l().unwrap_or_else(|_| JObject::null()),
        Err(_) => {
            println!(
                "[Java_com_dropbear_ffi_components_EntityTransformNative_setTransform] [ERROR] Failed to get local transform field"
            );
            return;
        }
    };

    let world_obj: JObject = match env.get_field(
        &entity_transform_obj,
        "world",
        "Lcom/dropbear/math/Transform;",
    ) {
        Ok(v) => v.l().unwrap_or_else(|_| JObject::null()),
        Err(_) => {
            println!(
                "[Java_com_dropbear_ffi_components_EntityTransformNative_setTransform] [ERROR] Failed to get world transform field"
            );
            return;
        }
    };

    if local_obj.is_null() || world_obj.is_null() {
        println!(
            "[Java_com_dropbear_ffi_components_EntityTransformNative_setTransform] [ERROR] local or world transform is null"
        );
        return;
    }

    let local_transform = match extract_transform(&mut env, &local_obj) {
        Some(t) => t,
        None => {
            println!(
                "[Java_com_dropbear_ffi_components_EntityTransformNative_setTransform] [ERROR] Failed to extract local transform"
            );
            return;
        }
    };

    let world_transform = match extract_transform(&mut env, &world_obj) {
        Some(t) => t,
        None => {
            println!(
                "[Java_com_dropbear_ffi_components_EntityTransformNative_setTransform] [ERROR] Failed to extract world transform"
            );
            return;
        }
    };

    if let Ok(mut q) = world.query_one::<&mut EntityTransform>(entity) {
        if let Some(entity_transform) = q.get() {
            *entity_transform.local_mut() = local_transform;
            *entity_transform.world_mut() = world_transform;
        } else {
            println!(
                "[Java_com_dropbear_ffi_components_EntityTransformNative_setTransform] [ERROR] Failed to get entity transform"
            );
        }
    } else {
        println!(
            "[Java_com_dropbear_ffi_components_EntityTransformNative_setTransform] [ERROR] Entity does not have EntityTransform component"
        );
    }
}
