//! Deals with Kotlin/Native library loading for different platforms.
#![allow(clippy::missing_safety_doc)]

pub mod exports;
pub mod sig;
pub mod types;

use crate::ptr::{AssetRegistryPtr, GraphicsPtr, InputStatePtr, WorldPtr};
use crate::scripting::error::LastErrorMessage;
use crate::scripting::native::sig::{DestroyAll, DestroyTagged, Init, LoadTagged, UpdateAll, UpdateTagged, UpdateWithEntities};
use anyhow::anyhow;
use libloading::{Library, Symbol};
use std::ffi::CString;
use std::path::Path;

pub struct NativeLibrary {
    #[allow(dead_code)]
    /// The libloading library that is currently loaded
    library: Library,
    init_fn: Symbol<'static, Init>,
    load_systems_fn: Symbol<'static, LoadTagged>,
    update_all_fn: Symbol<'static, UpdateAll>,
    update_tag_fn: Symbol<'static, UpdateTagged>,
    update_with_entities_fn: Symbol<'static, UpdateWithEntities>,
    destroy_all_fn: Symbol<'static, DestroyAll>,
    destroy_tagged_fn: Symbol<'static, DestroyTagged>,

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
            let library: Library = Library::new(lib_path)
                .map_err(|err| enhance_library_error(lib_path, err))?;

            let init_fn = load_symbol(&library, &[b"dropbear_init\0"], "dropbear_init")?;
            let load_systems_fn = load_symbol(
                &library,
                &[b"dropbear_load_systems\0", b"dropbear_load_tagged\0"],
                "dropbear_load_systems",
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
                update_all_fn,
                update_tag_fn,
                update_with_entities_fn,
                destroy_all_fn,
                destroy_tagged_fn,
                get_last_err_msg_fn,
                set_last_err_msg_fn,
            })
        }
    }

    /// Initialises the NativeLibrary by populating it with context.
    pub fn init(
        &mut self,
        world_ptr: WorldPtr,
        input_state_ptr: InputStatePtr,
        graphics_ptr: GraphicsPtr,
        asset_ptr: AssetRegistryPtr,
    ) -> anyhow::Result<()> {
        unsafe {
            let result = (self.init_fn)(world_ptr, input_state_ptr, graphics_ptr, asset_ptr);
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

    pub fn update_all(&mut self, dt: f32) -> anyhow::Result<()> {
        unsafe {
            let result = (self.update_all_fn)(dt);
            self.handle_result(result, "update_all")
        }
    }

    pub fn update_tagged(&mut self, tag: &String, dt: f32) -> anyhow::Result<()> {
        unsafe {
            let c_string: CString = CString::new(tag.clone())?;
            let result = (self.update_tag_fn)(c_string.as_ptr(), dt);
            self.handle_result(result, "update_tagged")
        }
    }

    pub fn update_systems_for_entities(
        &self,
        tag: &str,
        entity_ids: &[u64],
        dt: f32,
    ) -> anyhow::Result<()> {
        unsafe {
            let c_string = CString::new(tag)?;
            let result = (self.update_with_entities_fn)(
                c_string.as_ptr(),
                entity_ids.as_ptr(),
                entity_ids.len() as i32,
                dt
            );
            self.handle_result(result, "update_systems_for_entities")
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
}

impl NativeLibrary {
    /// Translates native return codes into rich errors, preferring the last error message when available.
    fn handle_result(&self, result: i32, operation: &str) -> anyhow::Result<()> {
        if result == 0 {
            return Ok(());
        }

        let code_label = DropbearNativeError::code_to_string(result);
        let last_error = self
            .get_last_error()
            .map(|msg| format!(": {msg}"))
            .unwrap_or_default();

        anyhow::bail!("Native script {} failed ({}{})", operation, code_label, last_error);
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
        match unsafe { library.get::<T>(candidate) } {
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

/// Displays the types of errors that can be returned by the native library.
pub enum DropbearNativeError {
    /// An error in the case the function returns an unsigned value.
    ///
    /// Subtract [`DropbearNativeError::UnsignedGenericError`] with another value
    /// to get the alternative unsigned error.
    UnsignedGenericError = 65535,
    Success = 0,
    NullPointer = -1,
    QueryFailed = -2,
    EntityNotFound = -3,
    NoSuchComponent = -4,
    NoSuchEntity = -5,
    WorldInsertError = -6,
    /// When the graphics queue fails to send its message to the receiver
    SendError = -7,
    /// Error while creating a new CString
    CStringError = -8,
    BufferTooSmall = -9,
    PrematureSceneSwitch = -10,
    
    InvalidUTF8 = -108,
    /// A generic error when the library doesn't know what happened or cannot find a
    /// suitable error code.
    ///
    /// The number `1274` comes from the total sum of the word "UnknownError" in decimal
    UnknownError = -1274,
}

impl DropbearNativeError {
    /// Attempts to convert an [`i32`] numerical code to a [`String`] for better error displaying
    pub fn code_to_string(code: i32) -> String {
        match code {
            x if x == DropbearNativeError::NullPointer as i32 => "NullPointer (-1)".to_string(),
            x if x == DropbearNativeError::QueryFailed as i32 => "QueryFailed (-2)".to_string(),
            x if x == DropbearNativeError::EntityNotFound as i32 => "EntityNotFound (-3)".to_string(),
            x if x == DropbearNativeError::NoSuchComponent as i32 => "NoSuchComponent (-4)".to_string(),
            x if x == DropbearNativeError::NoSuchEntity as i32 => "NoSuchEntity (-5)".to_string(),
            x if x == DropbearNativeError::WorldInsertError as i32 => "WorldInsertError (-6)".to_string(),
            x if x == DropbearNativeError::SendError as i32 => "SendError (-7)".to_string(),
            x if x == DropbearNativeError::InvalidUTF8 as i32 => "InvalidUTF8 (-108)".to_string(),
            x if x == DropbearNativeError::UnknownError as i32 => "UnknownError (-1274)".to_string(),
            x if x == DropbearNativeError::UnsignedGenericError as i32 => {
                "UnsignedGenericError (65535)".to_string()
            }
            _ => format!("code {code}"),
        }
    }
}