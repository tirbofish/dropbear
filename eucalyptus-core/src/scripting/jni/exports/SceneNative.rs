#![allow(non_snake_case, dead_code)]

use jni::JNIEnv;
use jni::objects::{JClass, JObject, JString, JThrowable, JValue};
use jni::sys::{jlong, jobject};
use crate::command::CommandBuffer;
use crate::ptr::{CommandBufferPtr, SceneLoaderPtr};
use crate::scene::loading::{SceneLoadHandle as CoreSceneLoadHandle};
use crate::states::WorldLoadingStatus;

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
 * Signature: `(JJLjava/lang/String;Lcom/dropbear/ffi/NativeEngine;)Lcom/dropbear/scene/SceneLoadHandle;`
 *
 * `JNIEXPORT jobject JNICALL Java_com_dropbear_ffi_SceneNative_loadSceneAsync__JJLjava_lang_String_2Lcom_dropbear_ffi_NativeEngine_2`
 * `(JNIEnv *, jclass, jlong, jstring, jobject);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_SceneNative_loadSceneAsync__JJLjava_lang_String_2Lcom_dropbear_ffi_NativeEngine_2(
    mut env: JNIEnv,
    _class: JClass,
    command_buffer_handle: jlong,
    scene_loader_ptr: jlong,
    scene_name: JString,
    native_engine: JObject,
) -> jobject {
    let command_buffer_ptr = command_buffer_handle as CommandBufferPtr;
    let scene_loader_ptr = scene_loader_ptr as SceneLoaderPtr;

    if command_buffer_ptr.is_null() || scene_loader_ptr.is_null() {
        return JObject::null().into_raw();
    }

    let command_buffer = unsafe { &*command_buffer_ptr };
    let scene_loader = unsafe { &*scene_loader_ptr };

    let scene_name_str: String = match env.get_string(&scene_name) {
        Ok(s) => s.into(),
        Err(_) => return JObject::null().into_raw(),
    };

    // removes need for double scene loading
    let id = {
        let loader_guard = scene_loader.lock();
        if let Some(existing_id) = loader_guard.find_pending_id_by_name(&scene_name_str) {
            existing_id
        } else {
            drop(loader_guard);
            let mut loader_guard = scene_loader.lock();
            let new_id = loader_guard.register_load(scene_name_str.clone());

            let core_handle = CoreSceneLoadHandle {
                id: new_id,
                scene_name: scene_name_str.clone(),
            };

            if command_buffer.try_send(CommandBuffer::LoadSceneAsync(core_handle)).is_err() {
                return JObject::null().into_raw();
            }

            new_id
        }
    };

    let cls = match env.find_class("com/dropbear/scene/SceneLoadHandle") {
        Ok(c) => c,
        Err(_) => return JObject::null().into_raw(),
    };

    let obj = match env.new_object(cls, "(JLjava/lang/String;Lcom/dropbear/ffi/NativeEngine;)V", &[
        JValue::Long(id as jlong),
        JValue::Object(&scene_name),
        JValue::Object(&native_engine),
    ]) {
        Ok(o) => o,
        Err(_) => return JObject::null().into_raw(),
    };

    obj.into_raw()
}

/**
 * Class: `com_dropbear_ffi_SceneNative`
 *
 * Method: `loadSceneAsync`
 *
 * Signature: `(JJLjava/lang/String;Ljava/lang/String;Lcom/dropbear/ffi/NativeEngine;)Lcom/dropbear/scene/SceneLoadHandle;`
 *
 * `JNIEXPORT jobject JNICALL Java_com_dropbear_ffi_SceneNative_loadSceneAsync__JJLjava_lang_String_2Ljava_lang_String_2Lcom_dropbear_ffi_NativeEngine_2`
 * `(JNIEnv *, jclass, jlong, jstring, jstring, jobject);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_SceneNative_loadSceneAsync__JJLjava_lang_String_2Ljava_lang_String_2Lcom_dropbear_ffi_NativeEngine_2(
    _env: JNIEnv,
    _class: JClass,
    _command_buffer_handle: jlong,
    _scene_loader_ptr: jlong,
    _scene_name: JString,
    _loading_scene_name: JString,
    _native_engine: JObject,
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
    handle: JObject,
) {
    let command_buffer_ptr = command_buffer_ptr as CommandBufferPtr;
    if command_buffer_ptr.is_null() {
        return;
    }
    let command_buffer = unsafe { &*command_buffer_ptr };

    let id = match env.get_field(&handle, "id", "J") {
        Ok(v) => v.j().unwrap(),
        Err(_) => return,
    };
    
    let name_obj = match env.get_field(&handle, "sceneName", "Ljava/lang/String;") {
        Ok(v) => v.l().unwrap(),
        Err(_) => return,
    };
    
    let name_str: String = match env.get_string(&JString::from(name_obj)) {
        Ok(s) => s.into(),
        Err(_) => return,
    };

    let core_handle = CoreSceneLoadHandle {
        id: id as u64,
        scene_name: name_str,
    };

    let _ = command_buffer.try_send(CommandBuffer::SwitchToAsync(core_handle));
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
    let command_buffer_ptr = command_buffer_ptr as CommandBufferPtr;
    if command_buffer_ptr.is_null() {
        return;
    }
    let command_buffer = unsafe { &*command_buffer_ptr };

    let scene_name_str: String = match env.get_string(&scene_name) {
        Ok(s) => s.into(),
        Err(_) => return,
    };

    let _ = command_buffer.try_send(CommandBuffer::SwitchSceneImmediate(scene_name_str));
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
    handle: JObject, // SceneLoadHandle
) -> jobject {
    let scene_loader_ptr = scene_loader_ptr as SceneLoaderPtr;
    if scene_loader_ptr.is_null() {
        return JObject::null().into_raw();
    }
    let scene_loader = unsafe { &*scene_loader_ptr };
    
    let id = match env.get_field(&handle, "id", "J") {
        Ok(v) => v.j().unwrap(),
        Err(_) => return JObject::null().into_raw(),
    };

    let mut loader = scene_loader.lock();
    if let Some(entry) = loader.get_entry_mut(id as u64) {
        if let Some(rx) = &entry.status {
            while let Ok(status) = rx.try_recv() {
                match status {
                    WorldLoadingStatus::Idle => {
                        entry.progress.message = "Idle".to_string();
                    },
                    WorldLoadingStatus::LoadingEntity { index, name, total } => {
                        entry.progress.current = index;
                        entry.progress.total = total;
                        entry.progress.message = format!("Loading entity: {}", name);
                    },
                    WorldLoadingStatus::Completed => {
                        entry.progress.current = entry.progress.total;
                        entry.progress.message = "Completed".to_string();
                    }
                }
            }
        }

        let p = &entry.progress;
        
        let cls = match env.find_class("com/dropbear/utils/Progress") {
            Ok(c) => c,
            Err(_) => return JObject::null().into_raw(),
        };
        
        let msg = match env.new_string(&p.message) {
            Ok(s) => s,
            Err(_) => return JObject::null().into_raw(),
        };

        let obj = match env.new_object(cls, "(DDLjava/lang/String;)V", &[
            JValue::Double(p.current as f64),
            JValue::Double(p.total as f64),
            JValue::Object(&msg),
        ]) {
            Ok(o) => o,
            Err(_) => return JObject::null().into_raw(),
        };
        
        obj.into_raw()
    } else {
        JObject::null().into_raw()
    }
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
    handle: JObject, // SceneLoadHandle
) -> jobject {
    let scene_loader_ptr = scene_loader_ptr as SceneLoaderPtr;
    if scene_loader_ptr.is_null() {
        return JObject::null().into_raw();
    }
    let scene_loader = unsafe { &*scene_loader_ptr };
    
    let id = match env.get_field(&handle, "id", "J") {
        Ok(v) => v.j().unwrap(),
        Err(_) => return JObject::null().into_raw(),
    };

    let loader = scene_loader.lock();
    if let Some(entry) = loader.get_entry(id as u64) {
        let status_str = match entry.result {
            crate::scene::loading::SceneLoadResult::Pending => "PENDING",
            crate::scene::loading::SceneLoadResult::Success => "READY",
            crate::scene::loading::SceneLoadResult::Error(_) => "FAILED",
        };
        
        let cls = match env.find_class("com/dropbear/scene/SceneLoadStatus") {
            Ok(c) => c,
            Err(_) => return JObject::null().into_raw(),
        };
        
        let field_id = match env.get_static_field_id(&cls, status_str, "Lcom/dropbear/scene/SceneLoadStatus;") {
            Ok(f) => f,
            Err(_) => return JObject::null().into_raw(),
        };
        
        let obj = match env.get_static_field_unchecked(&cls, field_id, jni::signature::JavaType::Object("com/dropbear/scene/SceneLoadStatus".to_string())) {
             Ok(v) => v.l().unwrap(),
             Err(_) => return JObject::null().into_raw(),
        };
        
        obj.into_raw()
    } else {
        JObject::null().into_raw()
    }
}
