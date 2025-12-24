#![allow(non_snake_case)]

use crossbeam_channel::Sender;
use jni::JNIEnv;
use jni::objects::{JClass, JObject, JString, JThrowable, JValue};
use jni::sys::{jlong, jobject};
use parking_lot::Mutex;
use crate::ptr::{CommandBufferPtr, SceneLoaderPtr};
use crate::command::CommandBuffer;
use crate::ffi_error_return;
use crate::scene::loading::{SceneLoader, SceneLoadHandle, SceneLoadResult};

fn throw_premature_scene_switch(env: &mut JNIEnv, message: &str) -> Result<(), jni::errors::Error> {
    let ex_class = env.find_class("com/dropbear/exception/PrematureSceneSwitchException")?;
    let msg = env.new_string(message)?;
    let ex_obj = env.new_object(
        ex_class,
        "(Ljava/lang/String;Ljava/lang/Throwable;)V",
        &[JValue::Object(&JObject::from(msg)), JValue::Object(&JObject::null())],
    )?;
    let ex_throwable = JThrowable::from(ex_obj);
    env.throw(ex_throwable)?;
    Ok(())
}

/**
 * Class: `com_dropbear_ffi_SceneNative`
 *
 * Method: `loadSceneAsync`
 *
 * Signature: `(JJLjava/lang/String;)Lcom/dropbear/scene/SceneLoadHandle;`
 *
 * `JNIEXPORT jobject JNICALL Java_com_dropbear_ffi_SceneNative_loadSceneAsync__JJLjava_lang_String_2`
 * `(JNIEnv *, jclass, jlong, jstring);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_SceneNative_loadSceneAsync__JJLjava_lang_String_2(
    mut env: JNIEnv,
    _class: JClass,
    command_buffer_handle: jlong,
    scene_loader_ptr: jlong,
    scene_name: JString,
) -> jobject {
    todo!("Not implemented yet")
}

/**
 * Class: `com_dropbear_ffi_SceneNative`
 *
 * Method: `loadSceneAsync`
 *
 * Signature: `(JJLjava/lang/String;Ljava/lang/String;)Lcom/dropbear/scene/SceneLoadHandle;`
 *
 * `JNIEXPORT jobject JNICALL Java_com_dropbear_ffi_SceneNative_loadSceneAsync__JJLjava_lang_String_2Ljava_lang_String_2`
 * `(JNIEnv *, jclass, jlong, jstring, jstring);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_SceneNative_loadSceneAsync__JJLjava_lang_String_2Ljava_lang_String_2(
    mut env: JNIEnv,
    _class: JClass,
    command_buffer_handle: jlong,
    scene_loader_ptr: jlong,
    scene_name: JString,
    loading_scene_name: JString,
) -> jobject {
    todo!("Not implemented yet")
}

/**
 * Class: `com_dropbear_ffi_SceneNative`
 *
 * Method: `switchToSceneAsync`
 *
 * Signature: `(JLcom/dropbear/scene/SceneLoadHandle;)V`
 *
 * `JNIEXPORT void JNICALL Java_com_dropbear_ffi_SceneNative_switchToSceneAsync`
 * `(JNIEnv *, jclass, jlong, jobject);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_SceneNative_switchToSceneAsync(
    mut env: JNIEnv,
    _class: JClass,
    command_buffer_ptr: jlong,
    handle: jobject,
) {
    todo!("Not implemented yet")
}

/**
 * Class: `com_dropbear_ffi_SceneNative`
 *
 * Method: `switchToSceneImmediate`
 *
 * Signature: `(JLjava/lang/String;)V`
 *
 * `JNIEXPORT void JNICALL Java_com_dropbear_ffi_SceneNative_switchToSceneImmediate`
 * `(JNIEnv *, jclass, jlong, jstring);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_SceneNative_switchToSceneImmediate(
    mut env: JNIEnv,
    _class: JClass,
    command_buffer_ptr: jlong,
    scene_name: JString,
) {
    todo!("Not implemented yet")
}

/**
 * Class: `com_dropbear_ffi_SceneNative`
 *
 * Method: `getSceneLoadProgress`
 *
 * Signature: `(JLcom/dropbear/scene/SceneLoadHandle;)Lcom/dropbear/utils/Progress;`
 *
 * `JNIEXPORT jobject JNICALL Java_com_dropbear_ffi_SceneNative_getSceneLoadProgress`
 * `(JNIEnv *, jclass, jlong, jobject);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_SceneNative_getSceneLoadProgress(
    mut env: JNIEnv,
    _class: JClass,
    scene_loader_ptr: jlong,
    handle: jobject, // SceneLoadHandle
) -> jobject {
    todo!("Not implemented yet")
}

/**
 * Class: `com_dropbear_ffi_SceneNative`
 *
 * Method: `getSceneLoadStatus`
 *
 * Signature: `(JLcom/dropbear/scene/SceneLoadHandle;)Lcom/dropbear/scene/SceneLoadStatus;`
 *
 * `JNIEXPORT jobject JNICALL Java_com_dropbear_ffi_SceneNative_getSceneLoadStatus`
 * `(JNIEnv *, jclass, jlong, jobject);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_SceneNative_getSceneLoadStatus(
    mut env: JNIEnv,
    _class: JClass,
    scene_loader_ptr: jlong,
    handle: jobject, // SceneLoadHandle
) -> jobject {
    todo!("Not implemented yet")
}