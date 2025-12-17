#![allow(non_snake_case)]

use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jint, jlong, jobject};

use crate::ptr::CommandBufferPtr;
use crate::window::CommandBuffer;

/**
 * Class: `com_dropbear_ffi_SceneNative`
 *
 * Method: `loadSceneAsync`
 *
 * Signature: `(JLjava/lang/String;)Lcom/dropbear/scene/SceneLoadHandle;`
 *
 * `JNIEXPORT jobject JNICALL Java_com_dropbear_ffi_SceneNative_loadSceneAsync__JLjava_lang_String_2`
 * `(JNIEnv *, jclass, jlong, jstring);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_SceneNative_loadSceneAsync__JLjava_lang_String_2(
    _env: JNIEnv,
    _class: JClass,
    _command_buffer_ptr: jlong,
    _scene_name: JString,
) -> jobject {
    unimplemented!()
}

/**
 * Class: `com_dropbear_ffi_SceneNative`
 *
 * Method: `loadSceneAsync`
 *
 * Signature: `(JLjava/lang/String;Ljava/lang/String;)Lcom/dropbear/scene/SceneLoadHandle;`
 *
 * `JNIEXPORT jobject JNICALL Java_com_dropbear_ffi_SceneNative_loadSceneAsync__JLjava_lang_String_2Ljava_lang_String_2`
 * `(JNIEnv *, jclass, jlong, jstring, jstring);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_SceneNative_loadSceneAsync__JLjava_lang_String_2Ljava_lang_String_2(
    _env: JNIEnv,
    _class: JClass,
    _command_buffer_ptr: jlong,
    _scene_name: JString,
    _loading_scene_name: JString,
) -> jobject {
    unimplemented!()
}

/**
 * Class: `com_dropbear_ffi_SceneNative`
 *
 * Method: `switchToSceneAsync`
 *
 * Signature: `(JLcom/dropbear/scene/SceneLoadHandle;)I`
 *
 * `JNIEXPORT jint JNICALL Java_com_dropbear_ffi_SceneNative_switchToSceneAsync`
 * `(JNIEnv *, jclass, jlong, jobject);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_SceneNative_switchToSceneAsync(
    _env: JNIEnv,
    _class: JClass,
    _command_buffer_ptr: jlong,
    _handle: jobject,
) -> jint {
    unimplemented!()
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
    let graphics = command_buffer_ptr as CommandBufferPtr;
    if graphics.is_null() {
        eprintln!(
            "[Java_com_dropbear_ffi_SceneNative_switchToSceneImmediate] [ERROR] Graphics pointer is null"
        );
        return;
    }

    let scene_name = match env.get_string(&scene_name) {
        Ok(s) => s.to_string_lossy().to_string(),
        Err(e) => {
            eprintln!(
                "[Java_com_dropbear_ffi_SceneNative_switchToSceneImmediate] [ERROR] Failed to read scene name: {}",
                e
            );
            return;
        }
    };

    let graphics = unsafe { &*graphics };
    if let Err(e) = graphics.send(CommandBuffer::SwitchScene(scene_name)) {
        eprintln!(
            "[Java_com_dropbear_ffi_SceneNative_switchToSceneImmediate] [ERROR] Failed to send SwitchScene: {}",
            e
        );
    }
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
    _env: JNIEnv,
    _class: JClass,
    _command_buffer_ptr: jlong,
    _handle: jobject, // SceneLoadHandle
) -> jobject {
    unimplemented!()
}