use jni::JNIEnv;
use jni::objects::{JClass, JObject};
use jni::sys::jlong;

/**
 * Class:     `com_dropbear_ffi_components_ColliderNative`
 *
 * Method:    `setCollider`
 *
 * Signature: `(JLcom/dropbear/physics/Collider;)V`
 *
 * `JNIEXPORT void JNICALL Java_com_dropbear_ffi_components_ColliderNative_setCollider
 * (JNIEnv *, jclass, jlong, jlong, jobject);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_ColliderNative_setCollider(
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    physics_handle: jlong,
    collider_obj: JObject,
) {
    // TODO: Implementation
}