use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jint, jlong, jobject};

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
    p0: jlong,
    p1: JString,
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
    p0: jlong,
    p1: JString,
    p2: JString,
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
    p0: jlong,
    p1: jobject,
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
    _env: JNIEnv,
    _class: JClass,
    p0: jlong,
    p1: JString,
) {

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
    p0: jlong,
    p1: jobject,
) -> jobject {
    unimplemented!()
}