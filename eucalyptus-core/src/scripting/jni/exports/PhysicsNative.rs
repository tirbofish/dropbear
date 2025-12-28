use jni::JNIEnv;
use jni::objects::{JClass, JObject};
use jni::sys::{jboolean, jlong, jobject, jobjectArray};

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
    physics_ptr: jlong,
    entity_id: jlong,
    enabled: jboolean,
) {
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
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    physics_handle: jlong,
    entity_id: jlong,
) -> jboolean {
    // TODO: Implementation
    0 // false
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
    world_handle: jlong,
    physics_handle: jlong,
    entity_id: jlong,
) -> jobject {
    // TODO: Implementation
    JObject::null().into_raw()
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
    world_handle: jlong,
    physics_handle: jlong,
    entity_id: jlong,
) -> jobjectArray {
    // TODO: Implementation
    std::ptr::null_mut()
}