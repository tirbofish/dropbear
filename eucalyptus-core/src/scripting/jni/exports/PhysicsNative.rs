use hecs::World;
use jni::JNIEnv;
use jni::objects::{JClass, JObject};
use jni::sys::{jboolean, jlong, jobject, jobjectArray};
use crate::{convert_jlong_to_entity, convert_ptr};
use crate::physics::PhysicsState;
use crate::physics::rigidbody::RigidBody;

/**
 * Class:     `com_dropbear_ffi_PhysicsNative`
 *
 * Method:    `setPhysicsEnabled`
 *
 * Signature: `(JJJZ)V`
 *
 * `JNIEXPORT void JNICALL Java_com_dropbear_ffi_PhysicsNative_setPhysicsEnabled
 * (JNIEnv *, jclass, jlong, jlong, jlong, jboolean);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_PhysicsNative_setPhysicsEnabled(
    mut env: JNIEnv,
    _class: JClass,
    world_ptr: jlong,
    physics_handle: jlong,
    entity_id: jlong,
    enabled: jboolean,
) {
    let world = convert_ptr!(mut world_ptr => World);
    let physics = convert_ptr!(mut physics_handle => PhysicsState);
    let entity = convert_jlong_to_entity!(entity_id);

    todo!()
}

/**
 * Class:     `com_dropbear_ffi_PhysicsNative`
 *
 * Method:    `isPhysicsEnabled`
 *
 * Signature: `(JJJ)Z`
 *
 * `JNIEXPORT jboolean JNICALL Java_com_dropbear_ffi_PhysicsNative_isPhysicsEnabled
 * (JNIEnv *, jclass, jlong, jlong, jlong);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_PhysicsNative_isPhysicsEnabled(
    _env: JNIEnv,
    _class: JClass,
    world_ptr: jlong,
    physics_handle: jlong,
    entity_id: jlong,
) -> jboolean {
    let world = convert_ptr!(mut world_ptr => World);
    let physics = convert_ptr!(mut physics_handle => PhysicsState);
    let entity = convert_jlong_to_entity!(entity_id);

    if let Ok(rb) = world.get::<&RigidBody>(entity) {
        if rb.disable_physics {
            false.into()
        } else {
            true.into()
        }
    } else {
        false.into()
    }
}

/**
 * Class:     `com_dropbear_ffi_PhysicsNative`
 *
 * Method:    `getRigidBody`
 *
 * Signature: `(JJJ)Lcom/dropbear/physics/RigidBody;`
 *
 * `JNIEXPORT jobject JNICALL Java_com_dropbear_ffi_PhysicsNative_getRigidBody
 * (JNIEnv *, jclass, jlong, jlong, jlong);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_PhysicsNative_getRigidBody(
    mut env: JNIEnv,
    _class: JClass,
    world_ptr: jlong,
    physics_handle: jlong,
    entity_id: jlong,
) -> jobject {
    let world = convert_ptr!(mut world_ptr => World);
    let physics = convert_ptr!(mut physics_handle => PhysicsState);
    let entity = convert_jlong_to_entity!(entity_id);

    todo!()
}

/**
 * Class:     `com_dropbear_ffi_PhysicsNative`
 *
 * Method:    `getAllColliders`
 *
 * Signature: `(JJ)[Lcom/dropbear/physics/Collider;`
 *
 * `JNIEXPORT jobjectArray JNICALL Java_com_dropbear_ffi_PhysicsNative_getAllColliders
 * (JNIEnv *, jclass, jlong, jlong, jlong);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_PhysicsNative_getAllColliders(
    mut env: JNIEnv,
    _class: JClass,
    world_ptr: jlong,
    physics_handle: jlong,
    entity_id: jlong,
) -> jobjectArray {
    let world = convert_ptr!(mut world_ptr => World);
    let physics = convert_ptr!(mut physics_handle => PhysicsState);
    let entity = convert_jlong_to_entity!(entity_id);

    todo!()
}