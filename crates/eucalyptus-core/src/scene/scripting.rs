use crate::ptr::{SceneLoaderPtr, SceneLoaderUnwrapped};
use crate::scripting::jni::utils::ToJObject;
use crate::scripting::result::DropbearNativeResult;
use crate::utils::Progress;
use ::jni as jni_ext;

pub mod shared {
    use crate::command::CommandBuffer;
    use crate::scene::loading::{SceneLoadResult, SceneLoader};
    use crate::scripting::native::DropbearNativeError;
    use crate::scripting::result::DropbearNativeResult;
    use crossbeam_channel::Sender;
    use parking_lot::Mutex;

    pub fn load_scene_async(
        command_buffer: &Sender<CommandBuffer>,
        scene_loader: &Mutex<SceneLoader>,
        scene_name: String,
        _loading_scene: Option<String>,
    ) -> DropbearNativeResult<u64> {
        let mut loader = scene_loader.lock();

        if let Some(existing_id) = loader.find_pending_id_by_name(&scene_name) {
            return Ok(existing_id);
        }

        let id = loader.register_load(scene_name.clone());

        let handle = crate::scene::loading::SceneLoadHandle {
            id,
            scene_name: scene_name.clone(),
        };

        // Send load command
        command_buffer.try_send(CommandBuffer::LoadSceneAsync(handle))
            .map_err(|_| DropbearNativeError::SendError)?;

        Ok(id)
    }

    pub fn switch_to_scene_immediate(
        command_buffer: &Sender<CommandBuffer>,
        scene_name: String,
    ) -> DropbearNativeResult<()> {
        command_buffer.try_send(CommandBuffer::SwitchSceneImmediate(scene_name))
            .map_err(|_| DropbearNativeError::SendError)?;
        Ok(())
    }

    pub fn switch_to_scene_async(
        command_buffer: &Sender<CommandBuffer>,
        scene_loader: &Mutex<SceneLoader>,
        scene_id: u64,
    ) -> DropbearNativeResult<()> {
        let loader = scene_loader.lock();

        if let Some(entry) = loader.get_entry(scene_id) {
            if matches!(entry.result, SceneLoadResult::Success) {
                let handle = crate::scene::loading::SceneLoadHandle {
                    id: scene_id,
                    scene_name: entry.scene_name.clone(),
                };

                command_buffer.try_send(CommandBuffer::SwitchToAsync(handle))
                    .map_err(|_| DropbearNativeError::SendError)?;
                Ok(())
            } else {
                Err(DropbearNativeError::PrematureSceneSwitch)
            }
        } else {
            Err(DropbearNativeError::NoSuchHandle)
        }
    }

    pub fn get_scene_load_handle_scene_name(
        scene_loader: &Mutex<SceneLoader>,
        scene_id: u64,
    ) -> DropbearNativeResult<String> {
        let loader = scene_loader.lock();

        if let Some(entry) = loader.get_entry(scene_id) {
            Ok(entry.scene_name.clone())
        } else {
            Err(DropbearNativeError::NoSuchHandle)
        }
    }

    pub fn get_scene_load_progress(
        scene_loader: &Mutex<SceneLoader>,
        scene_id: u64,
    ) -> DropbearNativeResult<crate::utils::Progress> {
        let mut loader = scene_loader.lock();

        if let Some(entry) = loader.get_entry_mut(scene_id) {
            // Update progress from status channel if available
            if let Some(ref rx) = entry.status {
                while let Ok(status) = rx.try_recv() {
                    match status {
                        crate::states::WorldLoadingStatus::Idle => {
                            entry.progress.message = "Idle".to_string();
                        },
                        crate::states::WorldLoadingStatus::LoadingEntity { index, name, total } => {
                            entry.progress.current = index;
                            entry.progress.total = total;
                            entry.progress.message = format!("Loading entity: {}", name);
                        },
                        crate::states::WorldLoadingStatus::Completed => {
                            entry.progress.current = entry.progress.total;
                            entry.progress.message = "Completed".to_string();
                        }
                    }
                }
            }

            Ok(entry.progress.clone())
        } else {
            Err(DropbearNativeError::NoSuchHandle)
        }
    }

    pub fn get_scene_load_status(
        scene_loader: &Mutex<SceneLoader>,
        scene_id: u64,
    ) -> DropbearNativeResult<u32> {
        let loader = scene_loader.lock();

        if let Some(entry) = loader.get_entry(scene_id) {
            let status = match entry.result {
                SceneLoadResult::Pending => 0,  // PENDING
                SceneLoadResult::Success => 1,  // READY
                SceneLoadResult::Error(_) => 2, // FAILED
            };
            Ok(status)
        } else {
            Err(DropbearNativeError::NoSuchHandle)
        }
    }
}

// input:
#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.scene.SceneLoadHandleNative",
        func = "getSceneLoadProgress",
        jni = "jni_ext",
    ),
    c(
        name = "dropbear_scene_get_scene_load_progress",
    )
)]
fn get_scene_load_progress(
    #[dropbear_macro::define(SceneLoaderPtr)]
    scene_loader: &mut SceneLoaderUnwrapped,
    scene_id: u64,
) -> DropbearNativeResult<Progress> {
    // in this case, a Progress is returned. Progress must use the
    // `ToJObject` trait to be valid, and must be representable by the C ABI (through #[repr(C)],
    // or a derive trait if you wish).
    shared::get_scene_load_progress(&scene_loader, scene_id)
}

pub mod jni {
    #![allow(non_snake_case)]
    use crate::command::CommandBuffer;
    use crate::ptr::{CommandBufferPtr, SceneLoaderPtr};
    use crate::scripting::jni::utils::ToJObject;
    use crate::{convert_jstring, convert_ptr};
    use jni::objects::{JClass, JString, JValue};
    use jni::sys::{jint, jlong, jobject};
    use jni::JNIEnv;

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_scene_SceneManagerNative_loadSceneAsyncNative__JJLjava_lang_String_2(
        mut env: JNIEnv,
        _: JClass,
        command_buffer_ptr: jlong,
        scene_manager_handle: jlong,
        scene_name: JString,
    ) -> jobject {
        let command_buffer = convert_ptr!(command_buffer_ptr, CommandBufferPtr => crossbeam_channel::Sender<CommandBuffer>);
        let scene_loader = convert_ptr!(scene_manager_handle, SceneLoaderPtr => parking_lot::Mutex<crate::scene::loading::SceneLoader>);

        let scene_name_str = convert_jstring!(env, scene_name);

        match super::shared::load_scene_async(command_buffer, scene_loader, scene_name_str, None) {
            Ok(scene_id) => {
                match env.find_class("java/lang/Long") {
                    Ok(long_class) => {
                        match env.new_object(long_class, "(J)V", &[JValue::Long(scene_id as i64)]) {
                            Ok(obj) => obj.into_raw(),
                            Err(e) => {
                                eprintln!("Failed to create Long object: {}", e);
                                let _ = env.throw_new("java/lang/RuntimeException",
                                                      format!("Failed to create Long object: {}", e));
                                std::ptr::null_mut()
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to find Long class: {}", e);
                        let _ = env.throw_new("java/lang/RuntimeException",
                                              format!("Failed to find Long class: {}", e));
                        std::ptr::null_mut()
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to load scene async: {}", e);
                let _ = env.throw_new("java/lang/RuntimeException",
                                      format!("Failed to load scene async: {:?}", e));
                std::ptr::null_mut()
            }
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_scene_SceneManagerNative_loadSceneAsyncNative__JJLjava_lang_String_2Ljava_lang_String_2(
        mut env: JNIEnv,
        _: JClass,
        command_buffer_ptr: jlong,
        scene_manager_handle: jlong,
        scene_name: JString,
        loading_scene: JString,
    ) -> jobject {
        let command_buffer = convert_ptr!(command_buffer_ptr, CommandBufferPtr => crossbeam_channel::Sender<CommandBuffer>);
        let scene_loader = convert_ptr!(scene_manager_handle, SceneLoaderPtr => parking_lot::Mutex<crate::scene::loading::SceneLoader>);

        let scene_name_str = convert_jstring!(env, scene_name);
        let loading_scene_str = convert_jstring!(env, loading_scene);

        match super::shared::load_scene_async(command_buffer, scene_loader, scene_name_str, Some(loading_scene_str)) {
            Ok(scene_id) => {
                match env.find_class("java/lang/Long") {
                    Ok(long_class) => {
                        match env.new_object(long_class, "(J)V", &[JValue::Long(scene_id as i64)]) {
                            Ok(obj) => obj.into_raw(),
                            Err(e) => {
                                eprintln!("Failed to create Long object: {}", e);
                                let _ = env.throw_new("java/lang/RuntimeException",
                                                      format!("Failed to create Long object: {}", e));
                                std::ptr::null_mut()
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to find Long class: {}", e);
                        let _ = env.throw_new("java/lang/RuntimeException",
                                              format!("Failed to find Long class: {}", e));
                        std::ptr::null_mut()
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to load scene async with loading scene: {}", e);
                let _ = env.throw_new("java/lang/RuntimeException",
                                      format!("Failed to load scene async: {:?}", e));
                std::ptr::null_mut()
            }
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_scene_SceneManagerNative_switchToSceneImmediateNative(
        mut env: JNIEnv,
        _: JClass,
        command_buffer_ptr: jlong,
        scene_name: JString,
    ) {
        let command_buffer = convert_ptr!(command_buffer_ptr, CommandBufferPtr => crossbeam_channel::Sender<CommandBuffer>);

        let scene_name_str = convert_jstring!(env, scene_name);

        if let Err(e) = super::shared::switch_to_scene_immediate(command_buffer, scene_name_str) {
            eprintln!("Failed to switch scene immediate: {}", e);
            let _ = env.throw_new("java/lang/RuntimeException",
                                  format!("Failed to switch scene immediate: {:?}", e));
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_scene_SceneLoadHandleNative_getSceneLoadHandleSceneName(
        mut env: JNIEnv,
        _: JClass,
        scene_loader_handle: jlong,
        scene_id: jlong,
    ) -> jobject {
        let scene_loader = convert_ptr!(scene_loader_handle, SceneLoaderPtr => parking_lot::Mutex<crate::scene::loading::SceneLoader>);

        match super::shared::get_scene_load_handle_scene_name(scene_loader, scene_id as u64) {
            Ok(scene_name) => {
                match env.new_string(scene_name) {
                    Ok(jstring) => jstring.into_raw(),
                    Err(e) => {
                        eprintln!("Failed to create Java string: {}", e);
                        let _ = env.throw_new("java/lang/RuntimeException",
                                              format!("Failed to create Java string: {}", e));
                        std::ptr::null_mut()
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to get scene name: {}", e);
                let _ = env.throw_new("java/lang/RuntimeException",
                                      format!("Failed to get scene name: {:?}", e));
                std::ptr::null_mut()
            }
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_scene_SceneLoadHandleNative_switchToSceneAsync(
        mut env: JNIEnv,
        _: JClass,
        command_buffer_ptr: jlong,
        scene_loader_handle: jlong,
        scene_id: jlong,
    ) {
        let command_buffer = convert_ptr!(command_buffer_ptr, CommandBufferPtr => crossbeam_channel::Sender<CommandBuffer>);
        let scene_loader = convert_ptr!(scene_loader_handle, SceneLoaderPtr => parking_lot::Mutex<crate::scene::loading::SceneLoader>);

        if let Err(e) = super::shared::switch_to_scene_async(command_buffer, scene_loader, scene_id as u64) {
            eprintln!("Failed to switch scene async: {}", e);

            if let crate::scripting::native::DropbearNativeError::PrematureSceneSwitch = e {
                let _ = env.throw_new("com/dropbear/exception/PrematureSceneSwitchException",
                                    "Cannot switch to scene before it has finished loading");
            } else {
                let _ = env.throw_new("java/lang/RuntimeException",
                                    format!("Failed to switch scene async: {:?}", e));
            }
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_scene_SceneLoadHandleNative_getSceneLoadProgress(
        mut env: JNIEnv,
        _: JClass,
        scene_loader_handle: jlong,
        scene_id: jlong,
    ) -> jobject {
        let scene_loader = convert_ptr!(scene_loader_handle, SceneLoaderPtr => parking_lot::Mutex<crate::scene::loading::SceneLoader>);

        match super::shared::get_scene_load_progress(scene_loader, scene_id as u64) {
            Ok(progress) => {
                match progress.to_jobject(&mut env) {
                    Ok(obj) => obj.into_raw(),
                    Err(e) => {
                        eprintln!("Failed to create Progress object: {:?}", e);
                        let _ = env.throw_new("java/lang/RuntimeException",
                                            format!("Failed to create Progress object: {:?}", e));
                        std::ptr::null_mut()
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to get scene load progress: {}", e);
                let _ = env.throw_new("java/lang/RuntimeException",
                                    format!("Failed to get scene load progress: {:?}", e));
                std::ptr::null_mut()
            }
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_scene_SceneLoadHandleNative_getSceneLoadStatus(
        mut env: JNIEnv,
        _: JClass,
        scene_loader_handle: jlong,
        scene_id: jlong,
    ) -> jint {
        let scene_loader = convert_ptr!(scene_loader_handle, SceneLoaderPtr => parking_lot::Mutex<crate::scene::loading::SceneLoader>);

        match super::shared::get_scene_load_status(scene_loader, scene_id as u64) {
            Ok(status) => status as jint,
            Err(e) => {
                eprintln!("Failed to get scene load status: {}", e);
                let _ = env.throw_new("java/lang/RuntimeException",
                                    format!("Failed to get scene load status: {:?}", e));
                -1 as jint
            }
        }
    }
}