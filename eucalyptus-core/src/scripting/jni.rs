#![allow(non_snake_case)]
//! Deals with the Java Native Interface (JNI) with the help of the [`jni`] crate

// pub mod exports;
pub mod utils;

use crate::APP_INFO;
use crate::logging::LOG_LEVEL;
use crate::ptr::WorldPtr;
use crate::scripting::error::LastErrorMessage;
use jni::objects::{GlobalRef, JClass, JLongArray, JObject, JObjectArray, JString, JValue};
use jni::sys::jlong;
use jni::{InitArgsBuilder, JNIEnv, JNIVersion, JavaVM};
use sha2::{Digest, Sha256};
use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use once_cell::sync::OnceCell;
use crate::scripting::{AWAIT_JDB, JVM_ARGS};
use crate::scripting::DropbearContext;
use crate::scripting::jni::utils::ToJObject;
use crate::types::{CollisionEvent, ContactForceEvent};

#[derive(Default, Clone)]
pub enum RuntimeMode {
    #[default]
    None,
    Editor,
    PlayMode,
    Runtime,
}

const LIBRARY_PATH: &[u8] = include_bytes!("../../../build/libs/dropbear-1.0-SNAPSHOT-all.jar");
pub static RUNTIME_MODE: OnceCell<RuntimeMode> = OnceCell::new();

fn is_port_available(port: u16) -> bool {
    if TcpListener::bind(("0.0.0.0", port)).is_err() {
        return false;
    }
    if TcpListener::bind(("::", port)).is_err() {
        return false;
    }
    true
}

/// Provides a context for any eucalyptus-core JNI calls and JVM hosting.
pub struct JavaContext {
    pub(crate) jvm: JavaVM,
    native_engine_instance: Option<GlobalRef>,
    dropbear_engine_class: Option<GlobalRef>,
    system_manager_instance: Option<GlobalRef>,
    pub(crate) jar_path: PathBuf,
}

impl JavaContext {
    /// Creates a new JVM instance
    ///
    /// By passing in a string into the function, you can launch a VM with custom VM arguments.
    /// It does have to be one continuous string, and if it is not, then VM creation will fail.
    pub fn new(external_vm_args: Option<String>) -> anyhow::Result<Self> {
        let root = app_dirs2::app_root(app_dirs2::AppDataType::UserData, &APP_INFO)?;
        let deps = root.join("dependencies");
        let host_jar_filename = "dropbear-jvm-fat-1.0-SNAPSHOT.jar";
        let host_jar_path = deps.join(host_jar_filename);
        let hash_filename = format!("{}.sha256", host_jar_filename);
        let hash_file_path = deps.join(hash_filename);

        fs::create_dir_all(&deps)?;

        let embedded_jar_hash = {
            let mut hasher = Sha256::new();
            hasher.update(LIBRARY_PATH);
            format!("{:x}", hasher.finalize())
        };

        let stored_hash = fs::read_to_string(&hash_file_path).ok();

        let should_update = match stored_hash {
            Some(stored) => {
                if stored.trim() == embedded_jar_hash {
                    log::debug!("Host library JAR hash matches stored hash. No update needed.");
                    false
                } else {
                    log::info!("Host library JAR hash differs from stored hash. Update required.");
                    true
                }
            }
            None => {
                log::info!("Host library JAR hash file not found. Update required.");
                true
            }
        };

        if should_update {
            log::info!(
                "Writing (or updating) Host library JAR to {:?}.",
                host_jar_path
            );
            fs::write(&host_jar_path, LIBRARY_PATH)?;
            log::info!("Host library JAR written to {:?}.", host_jar_path);

            fs::write(&hash_file_path, &embedded_jar_hash)?;
            log::debug!("Host library JAR hash written to {:?}.", hash_file_path);
        } else {
            log::debug!("Host library JAR at {:?} is up-to-date.", host_jar_path);
        }

        let mut jar_paths = Vec::new();
        if deps.exists() && deps.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&deps) {
                for entry in entries.filter_map(Result::ok) {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("jar") {
                        jar_paths.push(path);
                    }
                }
            }
        }

        jar_paths.sort();

        let separator = if cfg!(target_os = "windows") {
            ";"
        } else {
            ":"
        };

        let classpath = if jar_paths.is_empty() {
            host_jar_path.display().to_string()
        } else {
            jar_paths
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(separator)
        };

        log::debug!("JVM classpath path: {}", classpath);

        let mut jvm_args = InitArgsBuilder::new()
            .version(JNIVersion::V8);
        
        let mut args_log = Vec::new();



        if let Some(args) = external_vm_args {
            args_log.push(args.clone());
            jvm_args = jvm_args.option(args);
        } else {
            let classpath_arg = format!("-Djava.class.path={}", classpath);
            args_log.push(classpath_arg.clone());
            jvm_args = jvm_args.option(classpath_arg);

            #[cfg(feature = "jvm_debug")]
            {
                let play_mode = RUNTIME_MODE.get().and_then(|b| Some(b.clone())).unwrap_or_default();
                match play_mode {
                    RuntimeMode::None => {
                        log::warn!("No runtime mode set, therefore no JWDB available");
                    }
                    RuntimeMode::Editor => {
                        log::debug!("JDB is not used in the editor (as there is no need for so)");

                        // let (start_port, end_port) = (6741, 6761);
                        // let mut port = 0000;
                        // let mut debug_arg = String::new();
                        //
                        // for p in start_port..end_port {
                        //     if is_port_available(p) {
                        //         port = p;
                        //         debug_arg = format!("-agentlib:jdwp=transport=dt_socket,server=y,suspend=n,address=*:{}", port);
                        //         break;
                        //     } else {
                        //         log::debug!("Port {} is not available", p);
                        //     }
                        // }
                        //
                        // if debug_arg.is_empty() {
                        //     log::warn!("Could not find an available port for JDWP debugger (tried 6741-6760). Debugging will be disabled.");
                        // } else {
                        //     args_log.push(debug_arg.clone());
                        //     jvm_args = jvm_args.option(debug_arg);
                        //     log::info!("JDWP debugger enabled on port {}", port);
                        // }
                    }
                    RuntimeMode::PlayMode => {
                        let (start_port, end_port) = (6751, 6771);
                        let mut port = 0000;
                        let mut debug_arg = String::new();

                        for p in start_port..end_port {
                            if is_port_available(p) {
                                port = p;
                                debug_arg = if let Some(wait) = AWAIT_JDB.get() {
                                    if *wait {
                                        format!("-agentlib:jdwp=transport=dt_socket,server=y,suspend=y,address=*:{}", port)
                                    } else {
                                        format!("-agentlib:jdwp=transport=dt_socket,server=y,suspend=y,address=*:{}", port)
                                    }
                                } else {
                                    format!("-agentlib:jdwp=transport=dt_socket,server=y,suspend=n,address=*:{}", port)
                                };
                                
                                break;
                            } else {
                                log::debug!("Port {} is not available", p);
                            }
                        }

                        if debug_arg.is_empty() {
                            log::warn!("Could not find an available port for JDWP debugger (tried 6751-6770). Debugging will be disabled.");
                        } else {
                            args_log.push(debug_arg.clone());
                            jvm_args = jvm_args.option(debug_arg);
                            log::info!("JDWP debugger enabled on port {}", port);
                        }
                    }
                    RuntimeMode::Runtime => {
                        log::warn!("Runtime mode JWDB not implemented yet...");
                    }
                }
            }

        #[cfg(feature = "jvm")]
        {
            #[allow(unused)]
            let pathbuf = std::env::current_exe()?;
            #[allow(unused)]
            let path = pathbuf
                    .parent()
                    .ok_or_else(|| anyhow::anyhow!("Unable to locate parent"))?;

                log::debug!("Libs folder at {}", path.display());
                if !path.exists() {
                    log::warn!(
                    "Libs folder ({}) does not exist; native libraries may fail to load",
                    path.display()
                );
                }

                let path_str = path.to_string_lossy();

                let (separator, default_paths) = if cfg!(target_os = "windows") {
                    (";", vec!["C:\\Windows\\System32", "C:\\Windows\\SysWOW64"])
                } else if cfg!(target_os = "macos") {
                    (
                        ":",
                        vec![
                            "/Library/Java/Extensions",
                            "/System/Library/Java/Extensions",
                            "/usr/local/lib",
                            "/usr/lib",
                            ".",
                        ],
                    )
                } else {
                    (
                        ":",
                        vec![
                            "/usr/java/packages/lib",
                            "/usr/lib64",
                            "/lib64",
                            "/lib",
                            "/usr/lib",
                            ".",
                        ],
                    )
                };

                let combined_path =
                    format!("{}{}{}", path_str, separator, default_paths.join(separator));

                log::debug!("Java library path: {}", combined_path);
                let lib_path_arg = format!("-Djava.library.path={}", combined_path);
                args_log.push(lib_path_arg.clone());
                jvm_args = jvm_args.option(lib_path_arg);
            }
        };

        let args = args_log.join(" ");
        log::info!("Current JVM args being used [{}]", args);

        let _ = JVM_ARGS.set(args);

        let jvm_init_args = jvm_args.build()?;
        let jvm = JavaVM::new(jvm_init_args)?;

        #[cfg(feature = "jvm_debug")]
        crate::success!("JDB debugger enabled on localhost:6741");

        log::info!("Created JVM instance");

        Ok(Self {
            jvm,
            native_engine_instance: None,
            dropbear_engine_class: None,
            system_manager_instance: None,
            jar_path: PathBuf::new(),
        })
    }

    pub fn init(
        &mut self,
        context: &DropbearContext,
    ) -> anyhow::Result<()> {
        let mut env = self.jvm.attach_current_thread()?;

        let result = (|| -> anyhow::Result<()> {
            let world_handle = context.world as jlong;
            let input_handle = context.input as jlong;
            let graphics_handle = context.graphics as jlong;
            let asset_handle = context.assets as jlong;
            let scene_loader_handle = context.scene_loader as jlong;
            let physics_handle = context.physics_state as jlong;

            let args = [
                JValue::Long(world_handle),
                JValue::Long(input_handle),
                JValue::Long(graphics_handle),
                JValue::Long(asset_handle),
                JValue::Long(scene_loader_handle),
                JValue::Long(physics_handle)
            ];

            let mut sig = String::from("(");
            for _ in 0..args.len() {
                sig.push('J');
            }
            sig.push(')');
            sig.push('V');

            let dropbear_context_class: JClass = env.find_class("com/dropbear/ffi/DropbearContext")?;
            let dropbear_context_obj = env.new_object(
                dropbear_context_class,
                sig,
                &args
            )?;

            log::trace!("Locating \"com/dropbear/ffi/NativeEngine\" class");
            let native_engine_class: JClass = env.find_class("com/dropbear/ffi/NativeEngine")?;

            let native_engine_obj = if let Some(ref native_engine_ref) = self.native_engine_instance {
                native_engine_ref.as_obj()
            } else {
                log::trace!("Creating new instance of NativeEngine");
                let native_engine_obj = env.new_object(native_engine_class, "()V", &[])?;
                let native_engine_global_ref = env.new_global_ref(native_engine_obj)?;
                self.native_engine_instance = Some(native_engine_global_ref);
                self.native_engine_instance
                    .as_ref()
                    .expect("NativeEngine global ref must exist")
                    .as_obj()
            };

            log::trace!("Calling NativeEngine.init() with arg [\"com/dropbear/ffi/DropbearContext\"]");
            env.call_method(
                native_engine_obj,
                "init",
                "(Lcom/dropbear/ffi/DropbearContext;)V",
                &[JValue::Object(&dropbear_context_obj)],
            )?;

            if self.dropbear_engine_class.is_none() {
                let dropbear_class: JClass = env.find_class("com/dropbear/DropbearEngine")?;
                log::trace!("Creating DropbearEngine constructor with arg (NativeEngine_object)");
                let dropbear_obj = env.new_object(
                    dropbear_class,
                    "(Lcom/dropbear/ffi/NativeEngine;)V",
                    &[JValue::Object(native_engine_obj)],
                )?;

                log::trace!("Creating new global ref for DropbearEngine");
                let engine_global_ref = env.new_global_ref(dropbear_obj)?;
                self.dropbear_engine_class = Some(engine_global_ref);
            }

            let jar_path_jstring = env.new_string(self.jar_path.to_string_lossy())?;
            let log_level_str = { LOG_LEVEL.lock().to_string() };
            let log_level_enum_class = env.find_class("com/dropbear/logging/LogLevel")?;
            let log_level_enum_instance = env
                .get_static_field(
                    log_level_enum_class,
                    log_level_str,
                    "Lcom/dropbear/logging/LogLevel;",
                )?
                .l()?;

            let std_out_writer_class = env.find_class("com/dropbear/logging/StdoutWriter")?;
            let log_writer_obj = env.new_object(std_out_writer_class, "()V", &[])?;

            if self.system_manager_instance.is_none() {
                let engine_ref = self
                    .dropbear_engine_class
                    .as_ref()
                    .expect("DropbearEngine global ref must exist")
                    .as_obj();

                log::trace!("Locating \"com/dropbear/host/SystemManager\" class");
                let system_manager_class: JClass = env.find_class("com/dropbear/host/SystemManager")?;
                log::trace!(
                "Creating SystemManager constructor with args (jar_path_string, dropbear_engine_object, log_writer_object, log_level_enum, log_target_string)"
            );

                let log_target_jstring = env.new_string("dropbear_rust_host")?;

                let system_manager_obj = env.new_object(
                    system_manager_class,
                    "(Ljava/lang/String;Lcom/dropbear/DropbearEngine;Lcom/dropbear/logging/LogWriter;Lcom/dropbear/logging/LogLevel;Ljava/lang/String;)V",
                    &[
                        JValue::Object(&jar_path_jstring),
                        JValue::Object(engine_ref),
                        JValue::Object(&log_writer_obj),
                        JValue::Object(&log_level_enum_instance),
                        JValue::Object(&log_target_jstring),
                    ],
                )?;

                log::trace!("Creating new global ref for SystemManager");
                let manager_global_ref = env.new_global_ref(system_manager_obj)?;
                self.system_manager_instance = Some(manager_global_ref);
            }

            Ok(())
        })();

        Self::get_exception(&mut env)?;

        result
    }

    pub fn get_exception(env: &mut JNIEnv) -> anyhow::Result<()> {
        if let Ok(ex) = env.exception_occurred() {
            if ex.is_null() {
                return Ok(());
            }

            env.exception_clear()?;

            let message_result = env.call_method(
                &ex,
                "toString",
                "()Ljava/lang/String;",
                &[]
            )?;
            let message_obj = message_result.l()?;
            let message_jstring = JString::from(message_obj);
            let message_str: String = env.get_string(&message_jstring)?.into();

            let stack_trace_result = env.call_method(
                &ex,
                "getStackTrace",
                "()[Ljava/lang/StackTraceElement;",
                &[]
            )?;
            let stack_trace_obj = stack_trace_result.l()?;
            let stack_trace_array = JObjectArray::from(stack_trace_obj);
            let stack_len = env.get_array_length(&stack_trace_array)?;

            let mut error_msg = format!("{}\n", message_str);

            for i in 0..stack_len {
                let element = env.get_object_array_element(&stack_trace_array, i)?;

                let element_str_result = env.call_method(
                    &element,
                    "toString",
                    "()Ljava/lang/String;",
                    &[]
                )?;
                let element_str_obj = element_str_result.l()?;
                let element_jstring = JString::from(element_str_obj);
                let element_string: String = env.get_string(&element_jstring)?.into();

                error_msg.push_str(&format!("  at {}\n", element_string));
            }

            let cause_result = env.call_method(
                &ex,
                "getCause",
                "()Ljava/lang/Throwable;",
                &[]
            )?;
            let cause_obj = cause_result.l()?;

            if !cause_obj.is_null() {
                let cause_str_result = env.call_method(
                    &cause_obj,
                    "toString",
                    "()Ljava/lang/String;",
                    &[]
                )?;
                let cause_str_obj = cause_str_result.l()?;
                let cause_jstring = JString::from(cause_str_obj);
                let cause_string: String = env.get_string(&cause_jstring)?.into();
                error_msg.push_str(&format!("Caused by: {}\n", cause_string));
            }

            return Err(anyhow::anyhow!("Java exception: {}", error_msg));
        }

        Ok(())
    }

    pub fn reload(&mut self, _world: WorldPtr) -> anyhow::Result<()> {
        log::info!(
            "Reloading JAR using SystemManager: {}",
            self.jar_path.display()
        );

        if let Some(ref manager_ref) = self.system_manager_instance {
            let mut env = self.jvm.attach_current_thread()?;

            let result = (|| -> anyhow::Result<()> {
                log::trace!("Calling SystemManager.reloadJar()");
                let jar_path_jstring = env.new_string(self.jar_path.to_string_lossy())?;
                env.call_method(
                    manager_ref,
                    "reloadJar",
                    "(Ljava/lang/String;)V",
                    &[JValue::Object(&jar_path_jstring)],
                )?;
                Ok(())
            })();

            Self::get_exception(&mut env)?;

            Ok(result?)
        } else {
            log::warn!("SystemManager instance not found during reload.");
            // self.init(world)?;
            return Err(anyhow::anyhow!("SystemManager not initialised for reload."));
        }
    }

    pub fn load_systems_for_tag(&mut self, tag: &str) -> anyhow::Result<()> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let mut env = self.jvm.attach_current_thread()?;

            let result = (|| -> anyhow::Result<()> {
                log::trace!(
                "Calling SystemManager.loadSystemsForTag() with tag: {}",
                tag
            );
                let tag_jstring = env.new_string(tag)?;
                env.call_method(
                    manager_ref,
                    "loadSystemsForTag",
                    "(Ljava/lang/String;)V",
                    &[JValue::Object(&tag_jstring)],
                )?;

                log::debug!("Loaded systems for tag: {}", tag);
                Ok(())
            })();

            Self::get_exception(&mut env)?;

            Ok(result?)
        } else {
            return Err(anyhow::anyhow!(
                "SystemManager not initialised when loading systems for tag: {}",
                tag
            ));
        }
    }

    pub fn load_systems_for_entities(&mut self, tag: &str, entity_ids: &[u64]) -> anyhow::Result<()> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let mut env = self.jvm.attach_current_thread()?;

            let result = (|| -> anyhow::Result<()> {
                log::trace!(
                    "Calling SystemManager.loadSystemsForEntities() with tag: {}, count: {}",
                    tag,
                    entity_ids.len(),
                );

                let tag_jstring = env.new_string(tag)?;
                let entity_array: JLongArray = env.new_long_array(entity_ids.len() as i32)?;

                if !entity_ids.is_empty() {
                    env.set_long_array_region(
                        &entity_array,
                        0,
                        &entity_ids.iter().map(|e| *e as i64).collect::<Vec<_>>(),
                    )?;
                }

                let entity_array_obj = JObject::from(entity_array);

                env.call_method(
                    manager_ref,
                    "loadSystemsForEntities",
                    "(Ljava/lang/String;[J)V",
                    &[
                        JValue::Object(&tag_jstring),
                        JValue::Object(&entity_array_obj),
                    ],
                )?;

                Ok(())
            })();

            Self::get_exception(&mut env)?;
            Ok(result?)
        } else {
            Err(anyhow::anyhow!(
                "SystemManager not initialised when loading systems for tag: {}",
                tag
            ))
        }
    }

    pub fn collision_event(&self, tag: &str, entity_id: u64, event: &CollisionEvent) -> anyhow::Result<()> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let mut env = self.jvm.attach_current_thread()?;

            let result = (|| -> anyhow::Result<()> {
                let tag_jstring = env.new_string(tag)?;
                let event_obj = event
                    .to_jobject(&mut env)
                    .map_err(|e| anyhow::anyhow!("Failed to marshal CollisionEvent to JVM: {e}"))?;

                env.call_method(
                    manager_ref,
                    "collisionEvent",
                    "(Ljava/lang/String;JLcom/dropbear/physics/CollisionEvent;)V",
                    &[
                        JValue::Object(&tag_jstring),
                        JValue::Long(entity_id as i64),
                        JValue::Object(&event_obj),
                    ],
                )?;

                Ok(())
            })();

            Self::get_exception(&mut env)?;
            Ok(result?)
        } else {
            Err(anyhow::anyhow!(
                "SystemManager not initialised when delivering collision events."
            ))
        }
    }

    pub fn contact_force_event(&self, tag: &str, entity_id: u64, event: &ContactForceEvent) -> anyhow::Result<()> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let mut env = self.jvm.attach_current_thread()?;

            let result = (|| -> anyhow::Result<()> {
                let tag_jstring = env.new_string(tag)?;
                let event_obj = event
                    .to_jobject(&mut env)
                    .map_err(|e| anyhow::anyhow!("Failed to marshal ContactForceEvent to JVM: {e}"))?;

                env.call_method(
                    manager_ref,
                    "collisionForceEvent",
                    "(Ljava/lang/String;JLcom/dropbear/physics/ContactForceEvent;)V",
                    &[
                        JValue::Object(&tag_jstring),
                        JValue::Long(entity_id as i64),
                        JValue::Object(&event_obj),
                    ],
                )?;

                Ok(())
            })();

            Self::get_exception(&mut env)?;
            Ok(result?)
        } else {
            Err(anyhow::anyhow!(
                "SystemManager not initialised when delivering contact force events."
            ))
        }
    }

    pub fn update_all_systems(&self, dt: f32) -> anyhow::Result<()> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let mut env = self.jvm.attach_current_thread()?;

            let result = (|| -> anyhow::Result<()> {
                log_once::trace_once!("Calling SystemManager.updateAllSystems() with dt: {}", dt);
                env.call_method(
                    manager_ref,
                    "updateAllSystems",
                    "(F)V",
                    &[JValue::Float(dt)],
                )?;

                log_once::trace_once!("Updated all systems with dt: {}", dt);

                Ok(())
            })();

            Self::get_exception(&mut env)?;

            Ok(result?)
        } else {
            return Err(anyhow::anyhow!(
                "SystemManager not initialised when updating systems."
            ));
        }
    }

    pub fn physics_update_all_systems(&self, dt: f32) -> anyhow::Result<()> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let mut env = self.jvm.attach_current_thread()?;

            let result = (|| -> anyhow::Result<()> {
                log_once::trace_once!("Calling SystemManager.physicsUpdateAllSystems() with dt: {}", dt);
                env.call_method(
                    manager_ref,
                    "physicsUpdateAllSystems",
                    "(F)V",
                    &[JValue::Float(dt)],
                )?;

                Ok(())
            })();

            Self::get_exception(&mut env)?;

            Ok(result?)
        } else {
            Err(anyhow::anyhow!(
                "SystemManager not initialised when physics updating systems."
            ))
        }
    }

    pub fn update_systems_for_tag(&self, tag: &str, dt: f32) -> anyhow::Result<()> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let mut env = self.jvm.attach_current_thread()?;

            let result = (|| -> anyhow::Result<()> {
                log::trace!(
                "Calling SystemManager.updateSystemsByTag() with tag: {}, dt: {}",
                tag,
                dt
            );
                let tag_jstring = env.new_string(tag)?;
                env.call_method(
                    manager_ref,
                    "updateSystemsByTag",
                    "(Ljava/lang/String;F)V",
                    &[JValue::Object(&tag_jstring), JValue::Float(dt)],
                )?;

                log::debug!("Updated systems for tag: {} with dt: {}", tag, dt);
                Ok(())
            })();

            Self::get_exception(&mut env)?;

            Ok(result?)
        } else {
            return Err(anyhow::anyhow!(
                "SystemManager not initialised when updating systems for tag: {}",
                tag
            ));
        }
    }

    pub fn physics_update_systems_for_tag(&self, tag: &str, dt: f32) -> anyhow::Result<()> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let mut env = self.jvm.attach_current_thread()?;

            let result = (|| -> anyhow::Result<()> {
                log::trace!(
                    "Calling SystemManager.physicsUpdateSystemsByTag() with tag: {}, dt: {}",
                    tag,
                    dt
                );
                let tag_jstring = env.new_string(tag)?;
                env.call_method(
                    manager_ref,
                    "physicsUpdateSystemsByTag",
                    "(Ljava/lang/String;F)V",
                    &[JValue::Object(&tag_jstring), JValue::Float(dt)],
                )?;

                Ok(())
            })();

            Self::get_exception(&mut env)?;

            Ok(result?)
        } else {
            Err(anyhow::anyhow!(
                "SystemManager not initialised when physics updating systems for tag: {}",
                tag
            ))
        }
    }

    pub fn update_systems_for_entities(
        &self,
        tag: &str,
        entity_ids: &[u64],
        dt: f32,
    ) -> anyhow::Result<()> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let mut env = self.jvm.attach_current_thread()?;

            let result = (|| -> anyhow::Result<()> {
                log::trace!(
                    "Calling SystemManager.updateSystemsForEntities() with tag: {}, count: {}, dt: {}",
                    tag,
                    entity_ids.len(),
                    dt
                );

                let tag_jstring = env.new_string(tag)?;
                let entity_array: JLongArray = env.new_long_array(entity_ids.len() as i32)?;
                log::trace!("u64 entity: {:?}", entity_ids);
                log::trace!(
                    "i64 entity: {:?}",
                    entity_ids.iter().map(|e| *e as i64).collect::<Vec<_>>()
                );
                if !entity_ids.is_empty() {
                    env.set_long_array_region(
                        &entity_array,
                        0,
                        &entity_ids.iter().map(|e| *e as i64).collect::<Vec<_>>(),
                    )?;
                }
                let entity_array_obj = JObject::from(entity_array);

                env.call_method(
                    manager_ref,
                    "updateSystemsForEntities",
                    "(Ljava/lang/String;[JF)V",
                    &[
                        JValue::Object(&tag_jstring),
                        JValue::Object(&entity_array_obj),
                        JValue::Float(dt),
                    ],
                )?;

                log::trace!(
                    "Updated systems for tag: {} across {} entities with dt: {}",
                    tag,
                    entity_ids.len(),
                    dt
                );
                Ok(())
            })();

            Self::get_exception(&mut env)?;

            Ok(result?)
        } else {
            Err(anyhow::anyhow!(
                "SystemManager not initialised when updating systems for tag: {}",
                tag
            ))
        }
    }

    pub fn physics_update_systems_for_entities(
        &self,
        tag: &str,
        entity_ids: &[u64],
        dt: f32,
    ) -> anyhow::Result<()> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let mut env = self.jvm.attach_current_thread()?;

            let result = (|| -> anyhow::Result<()> {
                log::trace!(
                    "Calling SystemManager.physicsUpdateSystemsForEntities() with tag: {}, count: {}, dt: {}",
                    tag,
                    entity_ids.len(),
                    dt
                );

                let tag_jstring = env.new_string(tag)?;
                let entity_array: JLongArray = env.new_long_array(entity_ids.len() as i32)?;
                if !entity_ids.is_empty() {
                    env.set_long_array_region(
                        &entity_array,
                        0,
                        &entity_ids.iter().map(|e| *e as i64).collect::<Vec<_>>(),
                    )?;
                }
                let entity_array_obj = JObject::from(entity_array);

                env.call_method(
                    manager_ref,
                    "physicsUpdateSystemsForEntities",
                    "(Ljava/lang/String;[JF)V",
                    &[
                        JValue::Object(&tag_jstring),
                        JValue::Object(&entity_array_obj),
                        JValue::Float(dt),
                    ],
                )?;

                Ok(())
            })();

            Self::get_exception(&mut env)?;

            Ok(result?)
        } else {
            Err(anyhow::anyhow!(
                "SystemManager not initialised when physics updating systems for tag: {}",
                tag
            ))
        }
    }

    pub fn unload_systems_for_tag(&self, tag: &str) -> anyhow::Result<()> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let mut env = self.jvm.attach_current_thread()?;

            let result = (|| -> anyhow::Result<()> {
                log::trace!("Calling SystemManager.unloadSystemsByTag() with tag: {}", tag);
                let tag_jstring = env.new_string(tag)?;
                env.call_method(
                    manager_ref,
                    "unloadSystemsByTag",
                    "(Ljava/lang/String;)V",
                    &[JValue::Object(&tag_jstring)],
                )?;

                Ok(())
            })();

            Self::get_exception(&mut env)?;

            Ok(result?)
        } else {
            Err(anyhow::anyhow!(
                "SystemManager not initialised when unloading systems for tag: {}",
                tag
            ))
        }
    }

    pub fn destroy_systems_for_tag(&self, tag: &str) -> anyhow::Result<()> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let mut env = self.jvm.attach_current_thread()?;

            let result = (|| -> anyhow::Result<()> {
                log::trace!(
                    "Calling SystemManager.destroySystemsByTag() with tag: {}",
                    tag
                );
                let tag_jstring = env.new_string(tag)?;
                env.call_method(
                    manager_ref,
                    "destroySystemsByTag",
                    "(Ljava/lang/String;)V",
                    &[JValue::Object(&tag_jstring)],
                )?;

                Ok(())
            })();

            Self::get_exception(&mut env)?;

            Ok(result?)
        } else {
            Err(anyhow::anyhow!(
                "SystemManager not initialised when destroying systems for tag: {}",
                tag
            ))
        }
    }

    pub fn unload_all_systems(&self) -> anyhow::Result<()> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let mut env = self.jvm.attach_current_thread()?;

            let result = (|| -> anyhow::Result<()> {
                log::trace!("Calling SystemManager.unloadAllSystems()");
                env.call_method(manager_ref, "unloadAllSystems", "()V", &[])?;
                Ok(())
            })();

            Self::get_exception(&mut env)?;

            Ok(result?)
        } else {
            Err(anyhow::anyhow!(
                "SystemManager not initialised when unloading all systems."
            ))
        }
    }

    pub fn get_system_count_for_tag(&self, tag: &str) -> anyhow::Result<i32> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let mut env = self.jvm.attach_current_thread()?;

            let result = (|| -> anyhow::Result<i32> {
                log::trace!("Calling SystemManager.getSystemCount() for tag: {}", tag);
                let tag_jstring = env.new_string(tag)?;
                let result = env.call_method(
                    manager_ref,
                    "getSystemCount",
                    "(Ljava/lang/String;)I",
                    &[JValue::Object(&tag_jstring)],
                )?;

                Ok(result.i()?)
            })();

            Self::get_exception(&mut env)?;

            Ok(result?)
        } else {
            Err(anyhow::anyhow!(
                "SystemManager not initialised when getting system count for tag: {}",
                tag
            ))
        }
    }

    pub fn has_systems_for_tag(&self, tag: &str) -> anyhow::Result<bool> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let mut env = self.jvm.attach_current_thread()?;

            let result = (|| -> anyhow::Result<bool> {
                log::trace!("Calling SystemManager.hasSystemsForTag() for tag: {}", tag);
                let tag_jstring = env.new_string(tag)?;
                let result = env.call_method(
                    manager_ref,
                    "hasSystemsForTag",
                    "(Ljava/lang/String;)Z",
                    &[JValue::Object(&tag_jstring)],
                )?;

                Ok(result.z()?)
            })();

            Self::get_exception(&mut env)?;

            Ok(result?)
        } else {
            Err(anyhow::anyhow!(
                "SystemManager not initialised when checking for systems for tag: {}",
                tag
            ))
        }
    }

    pub fn get_total_system_count(&self) -> anyhow::Result<i32> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let mut env = self.jvm.attach_current_thread()?;

            let result = (|| -> anyhow::Result<i32> {
                log::trace!("Calling SystemManager.getTotalSystemCount()");
                let result = env.call_method(manager_ref, "getTotalSystemCount", "()I", &[])?;

                Ok(result.i()?)
            })();

            Self::get_exception(&mut env)?;

            Ok(result?)
        } else {
            Err(anyhow::anyhow!(
                "SystemManager not initialised when getting total system count."
            ))
        }
    }

    pub fn clear_engine(&mut self) -> anyhow::Result<()> {
        if let Some(old_native_engine_ref) = self.native_engine_instance.take() {
            let _ = old_native_engine_ref;
        }
        if let Some(old_engine_ref) = self.dropbear_engine_class.take() {
            let _ = old_engine_ref; // drop
        }
        if let Some(old_manager_ref) = self.system_manager_instance.take() {
            let _ = old_manager_ref; // drop
        }
        Ok(())
    }
}

impl Drop for JavaContext {
    fn drop(&mut self) {
        if let Some(ref global_ref) = self.dropbear_engine_class {
            let _ = global_ref;
        }
        if let Some(old_ref) = self.system_manager_instance.take() {
            let _ = old_ref;
        }
    }
}

impl LastErrorMessage for JavaContext {
    fn get_last_error(&self) -> Option<String> {
        let mut env = self.jvm.attach_current_thread().ok()?;

        let dropbear_kt_class = env.find_class("com/dropbear/DropbearEngineKt").ok()?;

        let field_value = env
            .get_static_field(dropbear_kt_class, "lastErrorMessage", "Ljava/lang/String;")
            .ok()?;

        let jobj = field_value.l().ok()?;

        if jobj.is_null() {
            return None;
        }

        let jstring = jni::objects::JString::from(jobj);
        let rust_string = env.get_string(&jstring).ok()?;
        Some(rust_string.to_string_lossy().into_owned())
    }

    fn set_last_error(&self, err_msg: impl Into<String>) -> anyhow::Result<()> {
        let msg = err_msg.into();

        let mut env = self.jvm.attach_current_thread()?;

        let dropbear_kt_class = env.find_class("com/dropbear/DropbearEngineKt")?;

        let jstring = env.new_string(&msg)?;

        let static_field =
            env.get_static_field_id(&dropbear_kt_class, "lastErrorMessage", "Ljava/lang/String;")?;

        env.set_static_field(dropbear_kt_class, static_field, JValue::Object(&jstring))?;

        Ok(())
    }
}
