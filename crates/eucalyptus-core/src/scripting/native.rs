//! Deals with Kotlin/Native library loading for different platforms.
#![allow(clippy::missing_safety_doc)]

// pub mod exports;
pub mod sig;
pub mod utils;

use crate::scripting::error::LastErrorMessage;
use crate::scripting::native::sig::{
    CollisionEvent, ContactForceEvent, DestroyAll, DestroyInScopeTagged, DestroyTagged, Init,
    LoadTagged, LoadWithEntities, PhysicsUpdateAll, PhysicsUpdateTagged, PhysicsUpdateWithEntities,
    UpdateAll, UpdateTagged, UpdateWithEntities,
};
use anyhow::anyhow;
use libloading::{Library, Symbol};
use std::ffi::CString;
// use std::fmt::{Display, Formatter}; // Display derived by thiserror
use crate::scripting::DropbearContext;
use crate::types::{
    CollisionEvent as CollisionEventFFI, ContactForceEvent as ContactForceEventFFI,
};
use hecs::ComponentError;
use jni::errors::JniError;
use jni::signature::TypeSignature;
use std::path::Path;
use thiserror::Error;

pub struct NativeLibrary {
    #[allow(dead_code)]
    /// The libloading library that is currently loaded
    library: Library,
    init_fn: Symbol<'static, Init>,
    load_systems_fn: Symbol<'static, LoadTagged>,
    load_systems_with_entities_fn: Symbol<'static, LoadWithEntities>,
    update_all_fn: Symbol<'static, UpdateAll>,
    update_tag_fn: Symbol<'static, UpdateTagged>,
    update_with_entities_fn: Symbol<'static, UpdateWithEntities>,
    physics_update_all_fn: Symbol<'static, PhysicsUpdateAll>,
    physics_update_tag_fn: Symbol<'static, PhysicsUpdateTagged>,
    physics_update_with_entities_fn: Symbol<'static, PhysicsUpdateWithEntities>,
    destroy_all_fn: Symbol<'static, DestroyAll>,
    destroy_tagged_fn: Symbol<'static, DestroyTagged>,
    destroy_in_scope_tagged_fn: Symbol<'static, DestroyInScopeTagged>,

    collision_event_fn: Symbol<'static, CollisionEvent>,
    contact_force_event_fn: Symbol<'static, ContactForceEvent>,

    // err msg
    #[allow(dead_code)]
    pub(crate) get_last_err_msg_fn: Symbol<'static, sig::GetLastErrorMessage>,
    #[allow(dead_code)]
    pub(crate) set_last_err_msg_fn: Symbol<'static, sig::SetLastErrorMessage>,
}

impl NativeLibrary {
    /// Creates a new instance of [`NativeLibrary`]
    pub fn new(lib_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let lib_path = lib_path.as_ref();
        if !lib_path.exists() {
            anyhow::bail!(
                "Native script library missing at '{}'. Expected this file to be copied next to the runtime executable or inside its 'libs' directory.",
                lib_path.display()
            );
        }
        unsafe {
            let library: Library =
                Library::new(lib_path).map_err(|err| enhance_library_error(lib_path, err))?;

            let init_fn = load_symbol(&library, &[b"dropbear_init\0"], "dropbear_init")?;
            let load_systems_fn = load_symbol(
                &library,
                &[b"dropbear_load_systems\0", b"dropbear_load_tagged\0"],
                "dropbear_load_systems",
            )?;

            let load_systems_with_entities_fn = load_symbol(
                &library,
                &[b"dropbear_load_with_entities\0"],
                "dropbear_load_with_entities",
            )?;
            let update_all_fn =
                load_symbol(&library, &[b"dropbear_update_all\0"], "dropbear_update_all")?;
            let update_tag_fn = load_symbol(
                &library,
                &[b"dropbear_update_tagged\0"],
                "dropbear_update_tagged",
            )?;
            let update_with_entities_fn = load_symbol(
                &library,
                &[b"dropbear_update_with_entities\0"],
                "dropbear_update_with_entities",
            )?;

            let physics_update_all_fn = load_symbol(
                &library,
                &[b"dropbear_physics_update_all\0"],
                "dropbear_physics_update_all",
            )?;
            let physics_update_tag_fn = load_symbol(
                &library,
                &[b"dropbear_physics_update_tagged\0"],
                "dropbear_physics_update_tagged",
            )?;
            let physics_update_with_entities_fn = load_symbol(
                &library,
                &[b"dropbear_physics_update_with_entities\0"],
                "dropbear_physics_update_with_entities",
            )?;
            let destroy_all_fn = load_symbol(
                &library,
                &[b"dropbear_destroy_all\0"],
                "dropbear_destroy_all",
            )?;
            let destroy_tagged_fn = load_symbol(
                &library,
                &[b"dropbear_destroy_tagged\0"],
                "dropbear_destroy_tagged",
            )?;
            let destroy_in_scope_tagged_fn = load_symbol(
                &library,
                &[b"dropbear_destroy_in_scope_tagged\0"],
                "dropbear_destroy_in_scope_tagged",
            )?;

            let collision_event_fn = load_symbol(
                &library,
                &[b"dropbear_collision_event\0"],
                "dropbear_collision_event",
            )?;

            let contact_force_event_fn = load_symbol(
                &library,
                &[b"dropbear_contact_force_event\0"],
                "dropbear_contact_force_event",
            )?;
            let get_last_err_msg_fn = load_symbol(
                &library,
                &[
                    b"dropbear_get_last_error_message\0",
                    b"dropbear_get_last_error\0",
                ],
                "dropbear_get_last_error_message",
            )?;
            let set_last_err_msg_fn = load_symbol(
                &library,
                &[
                    b"dropbear_set_last_error_message\0",
                    b"dropbear_set_last_error\0",
                ],
                "dropbear_set_last_error_message",
            )?;

            Ok(Self {
                library,
                init_fn,
                load_systems_fn,
                load_systems_with_entities_fn,
                update_all_fn,
                update_tag_fn,
                update_with_entities_fn,
                physics_update_all_fn,
                physics_update_tag_fn,
                physics_update_with_entities_fn,
                destroy_all_fn,
                destroy_tagged_fn,
                destroy_in_scope_tagged_fn,

                collision_event_fn,
                contact_force_event_fn,
                get_last_err_msg_fn,
                set_last_err_msg_fn,
            })
        }
    }

    /// Initialises the NativeLibrary by populating it with context.
    pub fn init(&mut self, dropbear_context: &DropbearContext) -> anyhow::Result<()> {
        unsafe {
            let result = (self.init_fn)(dropbear_context as *const DropbearContext);
            self.handle_result(result, "init")
        }
    }

    pub fn load_systems(&mut self, tag: String) -> anyhow::Result<()> {
        unsafe {
            let c_string: CString = CString::new(tag)?;
            let result = (self.load_systems_fn)(c_string.as_ptr());
            self.handle_result(result, "load_systems")
        }
    }

    pub fn load_systems_for_entities(
        &mut self,
        tag: &str,
        entity_ids: &[u64],
    ) -> anyhow::Result<()> {
        unsafe {
            let c_string = CString::new(tag)?;
            let result = (self.load_systems_with_entities_fn)(
                c_string.as_ptr(),
                entity_ids.as_ptr(),
                entity_ids.len() as i32,
            );
            self.handle_result(result, "load_systems_for_entities")
        }
    }

    pub fn collision_event(
        &self,
        tag: &str,
        current_entity_id: u64,
        event: &CollisionEventFFI,
    ) -> anyhow::Result<()> {
        unsafe {
            let c_string = CString::new(tag)?;
            let event_type = match event.event_type {
                crate::types::CollisionEventType::Started => 0,
                crate::types::CollisionEventType::Stopped => 1,
            };

            let result = (self.collision_event_fn)(
                c_string.as_ptr(),
                current_entity_id,
                event_type,
                event.collider1.index.index as i32,
                event.collider1.index.generation as i32,
                event.collider1.entity_id,
                event.collider1.id as i32,
                event.collider2.index.index as i32,
                event.collider2.index.generation as i32,
                event.collider2.entity_id,
                event.collider2.id as i32,
                event.flags,
            );
            self.handle_result(result, "collision_event")
        }
    }

    pub fn contact_force_event(
        &self,
        tag: &str,
        current_entity_id: u64,
        event: &ContactForceEventFFI,
    ) -> anyhow::Result<()> {
        unsafe {
            let c_string = CString::new(tag)?;
            let result = (self.contact_force_event_fn)(
                c_string.as_ptr(),
                current_entity_id,
                event.collider1.index.index as i32,
                event.collider1.index.generation as i32,
                event.collider1.entity_id,
                event.collider1.id as i32,
                event.collider2.index.index as i32,
                event.collider2.index.generation as i32,
                event.collider2.entity_id,
                event.collider2.id as i32,
                event.total_force.x,
                event.total_force.y,
                event.total_force.z,
                event.total_force_magnitude,
                event.max_force_direction.x,
                event.max_force_direction.y,
                event.max_force_direction.z,
                event.max_force_magnitude,
            );
            self.handle_result(result, "contact_force_event")
        }
    }

    pub fn update_all(&mut self, dt: f64) -> anyhow::Result<()> {
        unsafe {
            let result = (self.update_all_fn)(dt);
            self.handle_result(result, "update_all")
        }
    }

    pub fn physics_update_all(&mut self, dt: f64) -> anyhow::Result<()> {
        unsafe {
            let result = (self.physics_update_all_fn)(dt);
            self.handle_result(result, "physics_update_all")
        }
    }

    pub fn update_tagged(&mut self, tag: &String, dt: f64) -> anyhow::Result<()> {
        unsafe {
            let c_string: CString = CString::new(tag.clone())?;
            let result = (self.update_tag_fn)(c_string.as_ptr(), dt);
            self.handle_result(result, "update_tagged")
        }
    }

    pub fn physics_update_tagged(&mut self, tag: &String, dt: f64) -> anyhow::Result<()> {
        unsafe {
            let c_string: CString = CString::new(tag.clone())?;
            let result = (self.physics_update_tag_fn)(c_string.as_ptr(), dt);
            self.handle_result(result, "physics_update_tagged")
        }
    }

    pub fn update_systems_for_entities(
        &self,
        tag: &str,
        entity_ids: &[u64],
        dt: f64,
    ) -> anyhow::Result<()> {
        unsafe {
            let c_string = CString::new(tag)?;
            let result = (self.update_with_entities_fn)(
                c_string.as_ptr(),
                entity_ids.as_ptr(),
                entity_ids.len() as i32,
                dt,
            );
            self.handle_result(result, "update_systems_for_entities")
        }
    }

    pub fn physics_update_systems_for_entities(
        &self,
        tag: &str,
        entity_ids: &[u64],
        dt: f64,
    ) -> anyhow::Result<()> {
        unsafe {
            let c_string = CString::new(tag)?;
            let result = (self.physics_update_with_entities_fn)(
                c_string.as_ptr(),
                entity_ids.as_ptr(),
                entity_ids.len() as i32,
                dt,
            );
            self.handle_result(result, "physics_update_with_entities")
        }
    }

    pub fn destroy_all(&mut self) -> anyhow::Result<()> {
        unsafe {
            let result = (self.destroy_all_fn)();
            self.handle_result(result, "destroy_all")
        }
    }

    pub fn destroy_tagged(&mut self, tag: String) -> anyhow::Result<()> {
        unsafe {
            let c_string: CString = CString::new(tag)?;
            let result = (self.destroy_tagged_fn)(c_string.as_ptr());
            self.handle_result(result, "destroy_tagged")
        }
    }

    pub fn destroy_in_scope_tagged(&mut self, tag: String) -> anyhow::Result<()> {
        unsafe {
            let c_string: CString = CString::new(tag)?;
            let result = (self.destroy_in_scope_tagged_fn)(c_string.as_ptr());
            self.handle_result(result, "destroy_in_scope_tagged")
        }
    }
}

impl NativeLibrary {
    /// Translates native return codes into rich errors, preferring the last error message when available.
    fn handle_result(&self, result: i32, operation: &str) -> anyhow::Result<()> {
        if result == 0 {
            return Ok(());
        }

        let last_error = self
            .get_last_error()
            .map(|msg| format!(": {msg}"))
            .unwrap_or_default();

        anyhow::bail!("Native script {} failed ({})", operation, last_error);
    }
}

fn enhance_library_error(path: &Path, err: libloading::Error) -> anyhow::Error {
    #[cfg(windows)]
    {
        let err_str = err.to_string();
        if err_str.contains("os error 126") {
            return anyhow!(
                "Failed to load native script library '{}': {}. Windows error 126 means a dependent DLL is missing—copy every *.dll (and matching *.dll.lib) produced by your Gradle native build next to the runtime or into its 'libs' folder.",
                path.display(),
                err
            );
        }
    }

    anyhow!(
        "Failed to load native script library '{}': {}",
        path.display(),
        err
    )
}

fn load_symbol<T>(
    library: &Library,
    candidates: &[&[u8]],
    label: &str,
) -> anyhow::Result<Symbol<'static, T>> {
    let mut last_err = None;

    for (idx, candidate) in candidates.iter().enumerate() {
        match unsafe { library.get::<T>(*candidate) } {
            Ok(symbol) => {
                if idx > 0 {
                    log::warn!(
                        "Resolved native symbol '{}' via compatibility fallback for {}",
                        format_symbol_name(candidate),
                        label
                    );
                }
                let symbol = unsafe { std::mem::transmute(symbol) };
                return Ok(symbol);
            }
            Err(err) => last_err = Some(err),
        }
    }

    let requested = candidates
        .iter()
        .map(|bytes| format_symbol_name(bytes))
        .collect::<Vec<_>>()
        .join("', '");
    let last_err = last_err
        .map(|err| err.to_string())
        .unwrap_or_else(|| "symbol missing".to_string());

    anyhow::bail!(
        "Unable to locate any of the symbols ['{}'] for {} (last error: {})",
        requested,
        label,
        last_err
    );
}

fn format_symbol_name(bytes: &[u8]) -> String {
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..len]).into_owned()
}

impl LastErrorMessage for NativeLibrary {
    fn get_last_error(&self) -> Option<String> {
        unsafe {
            let msg_ptr = (self.get_last_err_msg_fn)();
            if msg_ptr.is_null() {
                return None;
            }

            let c_str = std::ffi::CStr::from_ptr(msg_ptr);
            c_str.to_str().ok().map(|s| s.to_string())
        }
    }

    fn set_last_error(&self, msg: impl Into<String>) -> anyhow::Result<()> {
        let msg = msg.into();
        unsafe {
            let c_string = CString::new(msg)?;
            (self.set_last_err_msg_fn)(c_string.as_ptr());
            Ok(())
        }
    }
}

#[derive(Debug, Error)]
/// Displays the types of errors that can be returned by the native library.
pub enum DropbearNativeError {
    /// An error in the case the function returns an unsigned value.
    #[error("Unsigned generic error")]
    UnsignedGenericError,
    /// An error that is thrown, but doesn't have any attached context.
    #[error("Generic error")]
    GenericError,
    /// The default return code for a successful FFI operation.
    #[error("Success")]
    Success,
    /// A null pointer was provided into the function, and cannot be read.
    #[error("Null pointer")]
    NullPointer,
    /// Attempting to query the current world failed for some cause.
    #[error("Query failed")]
    QueryFailed,
    /// The entity does not exist in the world.
    #[error("Entity not found")]
    EntityNotFound,
    /// No such component exists.
    #[error("No such component")]
    NoSuchComponent,
    /// No such entity uses the specific component.
    #[error("No such entity")]
    NoSuchEntity,
    /// Inserting something (like a component) into the world failed
    #[error("World insert error")]
    WorldInsertError,
    /// When the graphics queue fails to send its message to the receiver
    #[error("Send error")]
    SendError,
    /// Error while creating a new CString
    #[error("CString error")]
    CStringError,
    #[error("Buffer too small")]
    BufferTooSmall,
    /// Attempting to switch scenes before the world is loaded will throw this error.
    #[error("Premature scene switch")]
    PrematureSceneSwitch,
    /// When a gamepad is not found while querying the input state for so.
    #[error("Gamepad not found")]
    GamepadNotFound,
    /// When the argument is invalid
    #[error("Invalid argument")]
    InvalidArgument,
    /// The handle provided does not exist. Could be for an asset, entity, or other handle type.
    #[error("No such handle")]
    NoSuchHandle,
    /// Failed to create a Java object via JNI.
    #[error("JNI failed to create object")]
    JNIFailedToCreateObject,
    /// Failed to get a field from a Java object via JNI.
    #[error("JNI failed to get field")]
    JNIFailedToGetField,
    /// Failed to find a Java class via JNI.
    #[error("JNI class not found")]
    JNIClassNotFound,
    /// Failed to find a Java method via JNI.
    #[error("JNI method not found")]
    JNIMethodNotFound,
    /// Failed to unwrap a Java object via JNI.
    #[error("JNI unwrap failed")]
    JNIUnwrapFailed,
    /// Generic asset error. There was an error thrown, however there is no context attached.
    #[error("Generic asset error")]
    GenericAssetError,
    /// The provided uri (either euca:// or https) was invalid and formatted wrong.
    #[error("Invalid URI")]
    InvalidURI,
    /// The asset provided by the handle is wrong.
    #[error("Asset not found")]
    AssetNotFound,
    /// When a handle has been inputted wrongly.
    #[error("Invalid handle")]
    InvalidHandle,
    /// When a physics object is not found
    #[error("Physics object not found")]
    PhysicsObjectNotFound,
    /// When parsing through the JObject, the enum ordinal provided was invalid.
    #[error("Invalid enum ordinal")]
    InvalidEnumOrdinal,
    /// The entity did not have a requested component
    #[error("Missing component")]
    MissingComponent,
    /// The entity provided was invalid.
    #[error("Invalid entity")]
    InvalidEntity,
    /// The CString contained invalid UTF-8.
    #[error("Invalid UTF-8")]
    InvalidUTF8,
    /// A generic error when the library doesn't know what happened.
    #[error("Unknown error")]
    UnknownError,

    // JNI Errors impl
    #[error("Invalid JValue type cast: {0}. Actual type: {1}")]
    WrongJValueType(&'static str, &'static str),
    #[error("Invalid constructor return type (must be void)")]
    InvalidCtorReturn,
    #[error("Invalid number or type of arguments passed to java method: {0}")]
    InvalidArgList(TypeSignature),
    #[error("Method not found: {name} {sig}")]
    MethodNotFound { name: String, sig: String },
    #[error("Field not found: {name} {sig}")]
    FieldNotFound { name: String, sig: String },
    #[error("Java exception was thrown")]
    JavaException,
    #[error("JNIEnv null method pointer for {0}")]
    JNIEnvMethodNotFound(&'static str),
    #[error("Null pointer in {0}")]
    NullPtr(&'static str),
    #[error("Null pointer deref in {0}")]
    NullDeref(&'static str),
    #[error("Mutex already locked")]
    TryLock,
    #[error("JavaVM null method pointer for {0}")]
    JavaVMMethodNotFound(&'static str),
    #[error("Field already set: {0}")]
    FieldAlreadySet(String),
    #[error("Throw failed with error code {0}")]
    ThrowFailed(i32),
    #[error("Parse failed for input: {1}")]
    ParseFailed(#[source] combine::error::StringStreamError, String),
    #[error("JNI call failed")]
    JniCall(#[source] JniError),
}

impl DropbearNativeError {
    pub fn code(&self) -> i32 {
        match self {
            DropbearNativeError::Success => 0,
            DropbearNativeError::UnsignedGenericError => 65535,
            DropbearNativeError::GenericError => 1,
            DropbearNativeError::NullPointer => -1,
            DropbearNativeError::QueryFailed => -2,
            DropbearNativeError::EntityNotFound => -3,
            DropbearNativeError::NoSuchComponent => -4,
            DropbearNativeError::NoSuchEntity => -5,
            DropbearNativeError::WorldInsertError => -6,
            DropbearNativeError::SendError => -7,
            DropbearNativeError::CStringError => -8,
            DropbearNativeError::BufferTooSmall => -9,
            DropbearNativeError::PrematureSceneSwitch => -10,
            DropbearNativeError::GamepadNotFound => -11,
            DropbearNativeError::InvalidArgument => -12,
            DropbearNativeError::NoSuchHandle => -13,
            DropbearNativeError::JNIFailedToCreateObject => -14,
            DropbearNativeError::JNIFailedToGetField => -15,
            DropbearNativeError::JNIClassNotFound => -16,
            DropbearNativeError::JNIMethodNotFound => -17,
            DropbearNativeError::JNIUnwrapFailed => -18,
            DropbearNativeError::GenericAssetError => -19,
            DropbearNativeError::InvalidURI => -20,
            DropbearNativeError::AssetNotFound => -21,
            DropbearNativeError::InvalidHandle => -22,
            DropbearNativeError::PhysicsObjectNotFound => -23,
            DropbearNativeError::InvalidEnumOrdinal => -24,
            DropbearNativeError::MissingComponent => -25,
            DropbearNativeError::InvalidEntity => -100,
            DropbearNativeError::InvalidUTF8 => -108,
            DropbearNativeError::UnknownError => -1274,
            // New JNI errors start from -200 to separate them
            DropbearNativeError::WrongJValueType(_, _) => -200,
            DropbearNativeError::InvalidCtorReturn => -201,
            DropbearNativeError::InvalidArgList(_) => -202,
            DropbearNativeError::MethodNotFound { .. } => -203,
            DropbearNativeError::FieldNotFound { .. } => -204,
            DropbearNativeError::JavaException => -205,
            DropbearNativeError::JNIEnvMethodNotFound(_) => -206,
            DropbearNativeError::NullPtr(_) => -207,
            DropbearNativeError::NullDeref(_) => -208,
            DropbearNativeError::TryLock => -209,
            DropbearNativeError::JavaVMMethodNotFound(_) => -210,
            DropbearNativeError::FieldAlreadySet(_) => -211,
            DropbearNativeError::ThrowFailed(_) => -212,
            DropbearNativeError::ParseFailed(_, _) => -213,
            DropbearNativeError::JniCall(_) => -214,
        }
    }
}

impl From<jni::errors::Error> for DropbearNativeError {
    fn from(err: jni::errors::Error) -> Self {
        match err {
            jni::errors::Error::WrongJValueType(a, b) => DropbearNativeError::WrongJValueType(a, b),
            jni::errors::Error::InvalidCtorReturn => DropbearNativeError::InvalidCtorReturn,
            jni::errors::Error::InvalidArgList(s) => DropbearNativeError::InvalidArgList(s),
            jni::errors::Error::MethodNotFound { name, sig } => {
                DropbearNativeError::MethodNotFound { name, sig }
            }
            jni::errors::Error::FieldNotFound { name, sig } => {
                DropbearNativeError::FieldNotFound { name, sig }
            }
            jni::errors::Error::JavaException => DropbearNativeError::JavaException,
            jni::errors::Error::JNIEnvMethodNotFound(s) => {
                DropbearNativeError::JNIEnvMethodNotFound(s)
            }
            jni::errors::Error::NullPtr(s) => DropbearNativeError::NullPtr(s),
            jni::errors::Error::NullDeref(s) => DropbearNativeError::NullDeref(s),
            jni::errors::Error::TryLock => DropbearNativeError::TryLock,
            jni::errors::Error::JavaVMMethodNotFound(s) => {
                DropbearNativeError::JavaVMMethodNotFound(s)
            }
            jni::errors::Error::FieldAlreadySet(s) => DropbearNativeError::FieldAlreadySet(s),
            jni::errors::Error::ThrowFailed(i) => DropbearNativeError::ThrowFailed(i),
            jni::errors::Error::ParseFailed(e, s) => DropbearNativeError::ParseFailed(e, s),
            jni::errors::Error::JniCall(e) => DropbearNativeError::JniCall(e),
        }
    }
}

impl From<hecs::ComponentError> for DropbearNativeError {
    fn from(e: hecs::ComponentError) -> Self {
        match e {
            ComponentError::NoSuchEntity => DropbearNativeError::NoSuchEntity,
            ComponentError::MissingComponent(_) => DropbearNativeError::MissingComponent,
        }
    }
}
