#![allow(non_snake_case)]
//! Deals with the Java Native Interface (JNI) with the help of the [`jni`] crate

pub mod primitives;
pub mod utils;

use crate::APP_INFO;
use crate::logging::LOG_LEVEL;
use crate::ptr::WorldPtr;
use crate::scripting::DropbearContext;
use crate::scripting::JVM_ARGS;
use crate::scripting::error::LastErrorMessage;
use crate::scripting::jni::utils::ToJObject;
use crate::types::{CollisionEvent, ContactForceEvent};
use jni::objects::{Global, JClass, JLongArray, JObject, JString, JValue};
use jni::signature::RuntimeMethodSignature;
use jni::strings::JNIString;
use jni::sys::jlong;
use jni::{InitArgsBuilder, JNIVersion, JavaVM, jni_sig, jni_str};
use once_cell::sync::OnceCell;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

#[cfg(feature = "jvm_debug")]
use crate::scripting::AWAIT_JDB;

#[derive(Default, Clone)]
pub enum RuntimeMode {
    #[default]
    None,
    Editor,
    PlayMode,
    Runtime,
}

const LIBRARY_PATH: &[u8] = include_bytes!("../../../../build/libs/dropbear-1.0-SNAPSHOT-all.jar");
pub static RUNTIME_MODE: OnceCell<RuntimeMode> = OnceCell::new();

#[cfg(feature = "jvm_debug")]
fn is_port_available(port: u16) -> bool {
    if std::net::TcpListener::bind(("0.0.0.0", port)).is_err() {
        return false;
    }
    if std::net::TcpListener::bind(("::", port)).is_err() {
        return false;
    }
    true
}

/// Provides a context for any eucalyptus-core JNI calls and JVM hosting.

pub struct JavaContext {
    native_engine_instance: Option<Global<JObject<'static>>>,
    dropbear_engine_class: Option<Global<JObject<'static>>>,
    system_manager_instance: Option<Global<JObject<'static>>>,
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
            hasher.finalize()
        };

        let stored_hash = fs::read_to_string(&hash_file_path).ok();

        let should_update = match stored_hash {
            Some(stored) => {
                if stored.trim().as_bytes() == embedded_jar_hash.as_slice() {
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

        let mut jvm_args = InitArgsBuilder::new().version(JNIVersion::V21);

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
                let play_mode = RUNTIME_MODE
                    .get()
                    .and_then(|b| Some(b.clone()))
                    .unwrap_or_default();
                match play_mode {
                    RuntimeMode::None => {
                        log::warn!("No runtime mode set, therefore no JWDB available");
                    }
                    RuntimeMode::Editor => {
                        log::debug!("JDB is not used in the editor (as there is no need for so)");
                    }
                    RuntimeMode::PlayMode | RuntimeMode::Runtime => {
                        let (start_port, end_port) = (6751, 6771);
                        let mut port = 0000;
                        let mut debug_arg = String::new();

                        for p in start_port..end_port {
                            if is_port_available(p) {
                                port = p;
                                debug_arg = if let Some(wait) = AWAIT_JDB.get() {
                                    if *wait {
                                        format!(
                                            "-agentlib:jdwp=transport=dt_socket,server=y,suspend=y,address=*:{}",
                                            port
                                        )
                                    } else {
                                        format!(
                                            "-agentlib:jdwp=transport=dt_socket,server=y,suspend=y,address=*:{}",
                                            port
                                        )
                                    }
                                } else {
                                    format!(
                                        "-agentlib:jdwp=transport=dt_socket,server=y,suspend=n,address=*:{}",
                                        port
                                    )
                                };

                                break;
                            } else {
                                log::debug!("Port {} is not available", p);
                            }
                        }

                        if debug_arg.is_empty() {
                            log::warn!(
                                "Could not find an available port for JDWP debugger (tried 6751-6770). Debugging will be disabled."
                            );
                        } else {
                            args_log.push(debug_arg.clone());
                            jvm_args = jvm_args.option(debug_arg);
                            log::info!("JDWP debugger enabled on port {}", port);
                        }
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

                let lib_name = if cfg!(target_os = "windows") {
                    "eucalyptus_core.dll"
                } else if cfg!(target_os = "macos") {
                    "libeucalyptus_core.dylib"
                } else {
                    "libeucalyptus_core.so"
                };
                let core_lib_path = path.join(lib_name);
                let core_lib_arg = format!("-Deucalyptus.core.lib={}", core_lib_path.display());
                args_log.push(core_lib_arg.clone());
                jvm_args = jvm_args.option(core_lib_arg);
            }
        };

        let args = args_log.join(" ");
        log::info!("Current JVM args being used [{}]", args);

        let _ = JVM_ARGS.set(args);

        let jvm_init_args = jvm_args.build()?;
        JavaVM::new(jvm_init_args)?;

        log::info!("Created JVM instance");

        Ok(Self {
            native_engine_instance: None,
            dropbear_engine_class: None,
            system_manager_instance: None,
            jar_path: PathBuf::new(),
        })
    }

    pub fn init(&mut self, context: &DropbearContext) -> anyhow::Result<()> {
        let jvm = JavaVM::singleton()?;
        jvm.attach_current_thread(|env| {
            let world_handle = context.world as jlong;
            let input_handle = context.input as jlong;
            let graphics_handle = context.command_buffer as jlong;
            let graphics_context_handle = context.graphics_context as jlong;
            let asset_handle = context.assets as jlong;
            let scene_loader_handle = context.scene_loader as jlong;
            let physics_handle = context.physics_state as jlong;
            let ui_handle = context.ui_buffer as jlong;

            let args = [
                JValue::Long(world_handle),
                JValue::Long(input_handle),
                JValue::Long(graphics_handle),
                JValue::Long(graphics_context_handle),
                JValue::Long(asset_handle),
                JValue::Long(scene_loader_handle),
                JValue::Long(physics_handle),
                JValue::Long(ui_handle),
            ];

            let mut sig = String::from("(");
            for _ in 0..args.len() {
                sig.push('J');
            }
            sig.push(')');
            sig.push('V');

            let dropbear_context_class: JClass =
                env.load_class(jni_str!("com.dropbear.ffi.DropbearContext"))?;
            let runtime_sig = RuntimeMethodSignature::from_str(&sig)
                .map_err(|e| jni::errors::Error::MethodNotFound { name: sig.clone(), sig: e.to_string() })?;
            let dropbear_context_obj = env.new_object(dropbear_context_class, runtime_sig.method_signature(), &args)?;

            log::trace!("Locating \"com/dropbear/ffi/NativeEngine\" class");
            let native_engine_class: JClass = env.load_class(jni_str!("com.dropbear.ffi.NativeEngine"))?;

            let native_engine_obj = if let Some(ref native_engine_ref) = self.native_engine_instance
            {
                native_engine_ref.as_obj()
            } else {
                log::trace!("Creating new instance of NativeEngine");
                let native_engine_obj = env.new_object(native_engine_class, jni_sig!(()), &[])?;
                let native_engine_global_ref = env.new_global_ref(native_engine_obj)?;
                self.native_engine_instance = Some(native_engine_global_ref);
                self.native_engine_instance
                    .as_ref()
                    .expect("NativeEngine global ref must exist")
                    .as_obj()
            };

            log::trace!(
                "Calling NativeEngine.init() with arg [\"com.dropbear.ffi.DropbearContext\"]"
            );

            env.call_method(
                native_engine_obj,
                jni_str!("init"),
                jni_sig!((context: com.dropbear.ffi.DropbearContext) -> ()),
                &[JValue::Object(&dropbear_context_obj)],
            )?;

            if self.dropbear_engine_class.is_none() {
                let dropbear_class: JClass = env.load_class(jni_str!("com.dropbear.DropbearEngine"))?;
                log::trace!("Creating DropbearEngine constructor with arg (NativeEngine_object)");
                let dropbear_obj = env.new_object(
                    dropbear_class,
                    jni_sig!((native: com.dropbear.ffi.NativeEngine) -> ()),
                    &[JValue::Object(native_engine_obj)],
                )?;

                log::trace!("Creating new global ref for DropbearEngine");
                let engine_global_ref = env.new_global_ref(dropbear_obj)?;
                self.dropbear_engine_class = Some(engine_global_ref);
            }

            let jar_path_jstring = env.new_string(self.jar_path.to_string_lossy())?;
            let log_level_str = { LOG_LEVEL.lock().to_string() };
            let log_level_enum_class = env.load_class(jni_str!("com.dropbear.logging.LogLevel"))?;
            let log_level_enum_instance = env
                .get_static_field(
                    log_level_enum_class,
                    JNIString::from(log_level_str),
                    jni_sig!(com.dropbear.logging.LogLevel),
                )?
                .l()?;

            let log_writer_obj = match RUNTIME_MODE.get() {
                Some(RuntimeMode::Editor) | Some(RuntimeMode::PlayMode) => {
                    let port = 56624;
                    if std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok() {
                        let socket_writer_class =
                            env.load_class(jni_str!("com.dropbear.logging.SocketWriter"))?;
                        env.new_object(socket_writer_class, jni_sig!(()), &[])?
                    } else {
                        log::debug!(
                            "Editor console not reachable at 127.0.0.1:{}. Falling back to StdoutWriter.",
                            port
                        );
                        let std_out_writer_class =
                            env.load_class(jni_str!("com.dropbear.logging.StdoutWriter"))?;
                        env.new_object(std_out_writer_class, jni_sig!(()), &[])?
                    }
                }
                _ => {
                    let std_out_writer_class =
                        env.load_class(jni_str!("com.dropbear.logging.StdoutWriter"))?;
                    env.new_object(std_out_writer_class, jni_sig!(()), &[])?
                }
            };

            if self.system_manager_instance.is_none() {
                let engine_ref = self
                    .dropbear_engine_class
                    .as_ref()
                    .expect("DropbearEngine global ref must exist")
                    .as_obj();

                log::trace!("Locating \"com/dropbear/host/SystemManager\" class");
                let system_manager_class: JClass =
                    env.load_class(jni_str!("com.dropbear.host.SystemManager"))?;
                log::trace!(
                    "Creating SystemManager constructor with args (jar_path_string, dropbear_engine_object, log_writer_object, log_level_enum, log_target_string)"
                );

                let log_target_jstring = env.new_string("dropbear_rust_host")?;

                let system_manager_obj = env.new_object(
                    system_manager_class,
                    jni_sig!((java.lang.String, com.dropbear.DropbearEngine, com.dropbear.logging.LogWriter, com.dropbear.logging.LogLevel, java.lang.String) -> ()),
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

            Self::register_components()?;

            Ok(())
        })
    }

    pub fn reload(&mut self, _world: WorldPtr) -> anyhow::Result<()> {
        log::info!(
            "Reloading JAR using SystemManager: {}",
            self.jar_path.display()
        );

        if let Some(ref manager_ref) = self.system_manager_instance {
            let jvm = JavaVM::singleton()?;
            jvm.attach_current_thread(|env| -> anyhow::Result<()> {
                log::trace!("Calling SystemManager.reloadJar()");
                let jar_path_jstring = env.new_string(self.jar_path.to_string_lossy())?;
                env.call_method(
                    manager_ref,
                    jni_str!("reloadJar"),
                    jni_sig!((java.lang.String) -> ()),
                    &[JValue::Object(&jar_path_jstring)],
                )?;
                Self::register_components()?;
                Ok(())
            })?;
            Ok(())
        } else {
            log::warn!("SystemManager instance not found during reload.");
            // self.init(world)?;
            Err(anyhow::anyhow!("SystemManager not initialised for reload."))
        }
    }

    pub fn load_systems_for_tag(&mut self, tag: &str) -> anyhow::Result<()> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let jvm = JavaVM::singleton()?;

            jvm.attach_current_thread(|env| -> anyhow::Result<()> {
                log::trace!(
                    "Calling SystemManager.loadSystemsForTag() with tag: {}",
                    tag
                );
                let tag_jstring = env.new_string(tag)?;
                env.call_method(
                    manager_ref,
                    jni_str!("loadSystemsForTag"),
                    jni_sig!((java.lang.String) -> ()),
                    &[JValue::Object(&tag_jstring)],
                )?;

                log::debug!("Loaded systems for tag: {}", tag);
                Ok(())
            })?;
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "SystemManager not initialised when loading systems for tag: {}",
                tag
            ))
        }
    }

    pub fn load_systems_for_entities(
        &mut self,
        tag: &str,
        entity_ids: &[u64],
    ) -> anyhow::Result<()> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let jvm = JavaVM::singleton()?;
            jvm.attach_current_thread(|env| -> anyhow::Result<()> {
                log::trace!(
                    "Calling SystemManager.loadSystemsForEntities() with tag: {}, count: {}",
                    tag,
                    entity_ids.len(),
                );

                let tag_jstring = env.new_string(tag)?;
                let entity_array: JLongArray = env.new_long_array(entity_ids.len())?;

                if !entity_ids.is_empty() {
                    entity_array.set_region(
                        env,
                        0,
                        &entity_ids.iter().map(|e| *e as i64).collect::<Vec<_>>(),
                    )?;
                }

                let entity_array_obj = JObject::from(entity_array);

                env.call_method(
                    manager_ref,
                    jni_str!("loadSystemsForEntities"),
                    jni_sig!((java.lang.String, [long]) -> ()),
                    &[
                        JValue::Object(&tag_jstring),
                        JValue::Object(&entity_array_obj),
                    ],
                )?;

                Ok(())
            })?;

            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "SystemManager not initialised when loading systems for tag: {}",
                tag
            ))
        }
    }

    pub fn collision_event(
        &self,
        tag: &str,
        entity_id: u64,
        event: &CollisionEvent,
    ) -> anyhow::Result<()> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let jvm = JavaVM::singleton()?;
            jvm.attach_current_thread(|env| -> anyhow::Result<()> {
                let tag_jstring = env.new_string(tag)?;
                let event_obj = event
                    .to_jobject(env)
                    .map_err(|e| anyhow::anyhow!("Failed to marshal CollisionEvent to JVM: {e}"))?;

                env.call_method(
                    manager_ref,
                    jni_str!("collisionEvent"),
                    jni_sig!((java.lang.String, long, com.dropbear.physics.CollisionEvent) -> ()),
                    &[
                        JValue::Object(&tag_jstring),
                        JValue::Long(entity_id as i64),
                        JValue::Object(&event_obj),
                    ],
                )?;

                Ok(())
            })
        } else {
            Err(anyhow::anyhow!(
                "SystemManager not initialised when delivering collision events."
            ))
        }
    }

    pub fn contact_force_event(
        &self,
        tag: &str,
        entity_id: u64,
        event: &ContactForceEvent,
    ) -> anyhow::Result<()> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let jvm = JavaVM::singleton()?;
            jvm.attach_current_thread(|env| -> anyhow::Result<()> {
                let tag_jstring = env.new_string(tag)?;
                let event_obj = event.to_jobject(env).map_err(|e| {
                    anyhow::anyhow!("Failed to marshal ContactForceEvent to JVM: {e}")
                })?;

                env.call_method(
                    manager_ref,
                    jni_str!("collisionForceEvent"),
                    jni_sig!((java.lang.String, long, com.dropbear.physics.ContactForceEvent) -> ()),
                    &[
                        JValue::Object(&tag_jstring),
                        JValue::Long(entity_id as i64),
                        JValue::Object(&event_obj),
                    ],
                )?;

                Ok(())
            })
        } else {
            Err(anyhow::anyhow!(
                "SystemManager not initialised when delivering contact force events."
            ))
        }
    }

    pub fn update_all_systems(&self, dt: f64) -> anyhow::Result<()> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let jvm = JavaVM::singleton()?;
            jvm.attach_current_thread(|env| -> anyhow::Result<()> {
                log_once::trace_once!("Calling SystemManager.updateAllSystems() with dt: {}", dt);
                env.call_method(
                    manager_ref,
                    jni_str!("updateAllSystems"),
                    jni_sig!((f64) -> ()),
                    &[JValue::Double(dt)],
                )?;

                log_once::trace_once!("Updated all systems with dt: {}", dt);

                Ok(())
            })
        } else {
            Err(anyhow::anyhow!(
                "SystemManager not initialised when updating systems."
            ))
        }
    }

    pub fn physics_update_all_systems(&self, dt: f64) -> anyhow::Result<()> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let jvm = JavaVM::singleton()?;
            jvm.attach_current_thread(|env| -> anyhow::Result<()> {
                log_once::trace_once!(
                    "Calling SystemManager.physicsUpdateAllSystems() with dt: {}",
                    dt
                );
                env.call_method(
                    manager_ref,
                    jni_str!("physicsUpdateAllSystems"),
                    jni_sig!((f64) -> ()),
                    &[JValue::Double(dt)],
                )?;

                Ok(())
            })
        } else {
            Err(anyhow::anyhow!(
                "SystemManager not initialised when physics updating systems."
            ))
        }
    }

    pub fn update_systems_for_tag(&self, tag: &str, dt: f64) -> anyhow::Result<()> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let jvm = JavaVM::singleton()?;
            jvm.attach_current_thread(|env| -> anyhow::Result<()> {
                log::trace!(
                    "Calling SystemManager.updateSystemsForTag() with tag: {}, dt: {}",
                    tag,
                    dt
                );
                let tag_jstring = env.new_string(tag)?;
                env.call_method(
                    manager_ref,
                    jni_str!("updateSystemsForTag"),
                    jni_sig!((java.lang.String, f64) -> ()),
                    &[JValue::Object(&tag_jstring), JValue::Double(dt)],
                )?;
                Ok(())
            })
        } else {
            Err(anyhow::anyhow!(
                "SystemManager not initialised when updating systems for tag: {}",
                tag
            ))
        }
    }

    pub fn physics_update_systems_for_tag(&self, tag: &str, dt: f64) -> anyhow::Result<()> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let jvm = JavaVM::singleton()?;
            jvm.attach_current_thread(|env| -> anyhow::Result<()> {
                log::trace!(
                    "Calling SystemManager.physicsUpdateSystemsForTag() with tag: {}, dt: {}",
                    tag,
                    dt
                );
                let tag_jstring = env.new_string(tag)?;
                env.call_method(
                    manager_ref,
                    jni_str!("physicsUpdateSystemsForTag"),
                    jni_sig!((java.lang.String, f64) -> ()),
                    &[JValue::Object(&tag_jstring), JValue::Double(dt)],
                )?;

                Ok(())
            })
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
        dt: f64,
    ) -> anyhow::Result<()> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let jvm = JavaVM::singleton()?;
            jvm.attach_current_thread(|env| -> anyhow::Result<()> {
                log::trace!(
                    "Calling SystemManager.updateSystemsForEntities() with tag: {}, count: {}, dt: {}",
                    tag,
                    entity_ids.len(),
                    dt
                );

                let tag_jstring = env.new_string(tag)?;
                let entity_array: JLongArray = env.new_long_array(entity_ids.len())?;
                log::trace!("u64 entity: {:?}", entity_ids);
                log::trace!(
                    "i64 entity: {:?}",
                    entity_ids.iter().map(|e| *e as i64).collect::<Vec<_>>()
                );
                if !entity_ids.is_empty() {
                    entity_array.set_region(
                        env,
                        0,
                        &entity_ids.iter().map(|e| *e as i64).collect::<Vec<_>>(),
                    )?;
                }
                let entity_array_obj = JObject::from(entity_array);

                env.call_method(
                    manager_ref,
                    jni_str!("updateSystemsForEntities"),
                    jni_sig!((java.lang.String, [long], f64) -> ()),
                    &[
                        JValue::Object(&tag_jstring),
                        JValue::Object(&entity_array_obj),
                        JValue::Double(dt),
                    ],
                )?;

                log::trace!(
                    "Updated systems for tag: {} across {} entities with dt: {}",
                    tag,
                    entity_ids.len(),
                    dt
                );
                Ok(())
            })
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
        dt: f64,
    ) -> anyhow::Result<()> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let jvm = JavaVM::singleton()?;
            jvm.attach_current_thread(|env| -> anyhow::Result<()> {
                log::trace!(
                    "Calling SystemManager.physicsUpdateSystemsForEntities() with tag: {}, count: {}, dt: {}",
                    tag,
                    entity_ids.len(),
                    dt
                );

                let tag_jstring = env.new_string(tag)?;
                let entity_array: JLongArray = env.new_long_array(entity_ids.len())?;
                if !entity_ids.is_empty() {
                    entity_array.set_region(
                        env,
                        0,
                        &entity_ids.iter().map(|e| *e as i64).collect::<Vec<_>>(),
                    )?;
                }
                let entity_array_obj = JObject::from(entity_array);

                env.call_method(
                    manager_ref,
                    jni_str!("physicsUpdateSystemsForEntities"),
                    jni_sig!((java.lang.String, [long], f64) -> ()),
                    &[
                        JValue::Object(&tag_jstring),
                        JValue::Object(&entity_array_obj),
                        JValue::Double(dt),
                    ],
                )?;

                Ok(())
            })
        } else {
            Err(anyhow::anyhow!(
                "SystemManager not initialised when physics updating systems for tag: {}",
                tag
            ))
        }
    }

    pub fn unload_systems_for_tag(&self, tag: &str) -> anyhow::Result<()> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let jvm = JavaVM::singleton()?;
            jvm.attach_current_thread(|env| -> anyhow::Result<()> {
                log::trace!(
                    "Calling SystemManager.unloadSystemsByTag() with tag: {}",
                    tag
                );
                let tag_jstring = env.new_string(tag)?;
                env.call_method(
                    manager_ref,
                    jni_str!("unloadSystemsByTag"),
                    jni_sig!((java.lang.String) -> ()),
                    &[JValue::Object(&tag_jstring)],
                )?;

                Ok(())
            })
        } else {
            Err(anyhow::anyhow!(
                "SystemManager not initialised when unloading systems for tag: {}",
                tag
            ))
        }
    }

    pub fn destroy_systems_for_tag(&self, tag: &str) -> anyhow::Result<()> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let jvm = JavaVM::singleton()?;
            jvm.attach_current_thread(|env| -> anyhow::Result<()> {
                log::trace!(
                    "Calling SystemManager.destroySystemsByTag() with tag: {}",
                    tag
                );
                let tag_jstring = env.new_string(tag)?;
                env.call_method(
                    manager_ref,
                    jni_str!("destroySystemsByTag"),
                    jni_sig!((java.lang.String) -> ()),
                    &[JValue::Object(&tag_jstring)],
                )?;

                Ok(())
            })
        } else {
            Err(anyhow::anyhow!(
                "SystemManager not initialised when destroying systems for tag: {}",
                tag
            ))
        }
    }

    pub fn unload_all_systems(&self) -> anyhow::Result<()> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let jvm = JavaVM::singleton()?;
            jvm.attach_current_thread(|env| -> anyhow::Result<()> {
                log::trace!("Calling SystemManager.unloadAllSystems()");
                env.call_method(manager_ref, jni_str!("unloadAllSystems"), jni_sig!(()), &[])?;
                Ok(())
            })
        } else {
            Err(anyhow::anyhow!(
                "SystemManager not initialised when unloading all systems."
            ))
        }
    }

    pub fn get_system_count_for_tag(&self, tag: &str) -> anyhow::Result<i32> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let jvm = JavaVM::singleton()?;
            jvm.attach_current_thread(|env| -> anyhow::Result<i32> {
                log::trace!("Calling SystemManager.getSystemCount() for tag: {}", tag);
                let tag_jstring = env.new_string(tag)?;
                let result = env.call_method(
                    manager_ref,
                    jni_str!("getSystemCount"),
                    jni_sig!((java.lang.String) -> i32),
                    &[JValue::Object(&tag_jstring)],
                )?;

                Ok(result.i()?)
            })
        } else {
            Err(anyhow::anyhow!(
                "SystemManager not initialised when getting system count for tag: {}",
                tag
            ))
        }
    }

    pub fn has_systems_for_tag(&self, tag: &str) -> anyhow::Result<bool> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let jvm = JavaVM::singleton()?;
            jvm.attach_current_thread(|env| -> anyhow::Result<bool> {
                log::trace!("Calling SystemManager.hasSystemsForTag() for tag: {}", tag);
                let tag_jstring = env.new_string(tag)?;
                let result = env.call_method(
                    manager_ref,
                    jni_str!("hasSystemsForTag"),
                    jni_sig!((java.lang.String) -> boolean),
                    &[JValue::Object(&tag_jstring)],
                )?;

                Ok(result.z()?)
            })
        } else {
            Err(anyhow::anyhow!(
                "SystemManager not initialised when checking for systems for tag: {}",
                tag
            ))
        }
    }

    pub fn get_total_system_count(&self) -> anyhow::Result<i32> {
        if let Some(ref manager_ref) = self.system_manager_instance {
            let jvm = JavaVM::singleton()?;
            jvm.attach_current_thread(|env| -> anyhow::Result<i32> {
                log::trace!("Calling SystemManager.getTotalSystemCount()");
                let result = env.call_method(
                    manager_ref,
                    jni_str!("getTotalSystemCount"),
                    jni_sig!(() -> i32),
                    &[],
                )?;

                Ok(result.i()?)
            })
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
        let jvm = JavaVM::singleton().ok()?;
        jvm.attach_current_thread(|env| -> anyhow::Result<Option<String>> {
            let dropbear_kt_class = env.load_class(jni_str!("com.dropbear.DropbearEngineKt"))?;

            let field_value = env.get_static_field(
                dropbear_kt_class,
                jni_str!("lastErrorMessage"),
                jni_sig!(java.lang.String),
            )?;

            let jobj = field_value.l()?;

            if jobj.is_null() {
                return Ok(None);
            }
            let rust_string = JString::cast_local(env, jobj)
                .map_err(|_| anyhow::anyhow!("Failed to cast JString"))?;
            Ok(Some(rust_string.to_string()))
        })
        .ok()
        .flatten()
    }

    fn set_last_error(&self, err_msg: impl Into<String>) -> anyhow::Result<()> {
        let msg = err_msg.into();
        let jvm = JavaVM::singleton()?;
        jvm.attach_current_thread(|env| -> anyhow::Result<()> {
            let dropbear_kt_class = env.load_class(jni_str!("com.dropbear.DropbearEngineKt"))?;

            let jstring = env.new_string(&msg)?;

            env.set_static_field(
                dropbear_kt_class,
                jni_str!("lastErrorMessage"),
                jni_sig!(java.lang.String),
                JValue::Object(&jstring),
            )?;

            Ok(())
        })
    }
}
