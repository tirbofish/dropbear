use jni::JNIEnv;
use jni::objects::{JClass, JObject};
use jni::sys::{jdouble, jlong, jobjectArray};

/**
 * Class:     `com_dropbear_ffi_components_RigidBodyNative`
 *
 * Method:    `applyImpulse`
 *
 * Signature: `(JLcom/dropbear/physics/Index;DDD)V`
 *
 * `JNIEXPORT void JNICALL Java_com_dropbear_ffi_components_RigidBodyNative_applyImpulse
 * (JNIEnv *, jclass, jlong, jobject, jdouble, jdouble, jdouble);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_RigidBodyNative_applyImpulse(
    mut env: JNIEnv,
    _class: JClass,
    physics_handle: jlong,
    index_obj: JObject,
    x: jdouble,
    y: jdouble,
    z: jdouble,
) {
    // TODO: Implementation
}

/**
 * Class:     `com_dropbear_ffi_components_RigidBodyNative`
 *
 * Method:    `applyTorqueImpulse`
 *
 * Signature: `(JLcom/dropbear/physics/Index;DDD)V`
 *
 * `JNIEXPORT void JNICALL Java_com_dropbear_ffi_components_RigidBodyNative_applyTorqueImpulse
 * (JNIEnv *, jclass, jlong, jobject, jdouble, jdouble, jdouble);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_RigidBodyNative_applyTorqueImpulse(
    mut env: JNIEnv,
    _class: JClass,
    physics_handle: jlong,
    index_obj: JObject,
    x: jdouble,
    y: jdouble,
    z: jdouble,
) {
    // TODO: Implementation
}

/**
 * Class:     `com_dropbear_ffi_components_RigidBodyNative`
 *
 * Method:    `setRigidBody`
 *
 * Signature: `(JJLcom/dropbear/physics/RigidBody;)V`
 *
 * `JNIEXPORT void JNICALL Java_com_dropbear_ffi_components_RigidBodyNative_setRigidBody
 * (JNIEnv *, jclass, jlong, jlong, jobject);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_RigidBodyNative_setRigidBody(
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    physics_handle: jlong,
    rigidbody_obj: JObject,
) {
    // TODO: Implementation
}

/**
 * Class:     `com_dropbear_ffi_components_RigidBodyNative`
 *
 * Method:    `getChildColliders`
 *
 * Signature: `(JJLcom/dropbear/physics/Index;)[Lcom/dropbear/physics/Collider;`
 *
 * `JNIEXPORT jobjectArray JNICALL Java_com_dropbear_ffi_components_RigidBodyNative_getChildColliders
 * (JNIEnv *, jclass, jlong, jlong, jobject);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_RigidBodyNative_getChildColliders(
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    physics_handle: jlong,
    index_obj: JObject,
) -> jobjectArray {
    // TODO: Implementation
    std::ptr::null_mut()
}