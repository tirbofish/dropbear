//! Deals with Kotlin/Native library loading for different platforms.
#![allow(clippy::missing_safety_doc)]

// pub mod exports;
pub mod sig;
pub mod utils;

use crate::scripting::error::LastErrorMessage;
use crate::scripting::native::sig::{DestroyAll, DestroyInScopeTagged, DestroyTagged, Init, LoadTagged, PhysicsUpdateAll, PhysicsUpdateTagged, PhysicsUpdateWithEntities, UpdateAll, UpdateTagged, UpdateWithEntities};
use anyhow::anyhow;
use libloading::{Library, Symbol};
use std::ffi::CString;
use std::fmt::{Display, Formatter};
use std::path::Path;
use crate::scripting::DropbearContext;

pub struct NativeLibrary {
    #[allow(dead_code)]
    /// The libloading library that is currently loaded
    library: Library,
    init_fn: Symbol<'static, Init>,
    load_systems_fn: Symbol<'static, LoadTagged>,
    update_all_fn: Symbol<'static, UpdateAll>,
    update_tag_fn: Symbol<'static, UpdateTagged>,
    update_with_entities_fn: Symbol<'static, UpdateWithEntities>,
    physics_update_all_fn: Symbol<'static, PhysicsUpdateAll>,
    physics_update_tag_fn: Symbol<'static, PhysicsUpdateTagged>,
    physics_update_with_entities_fn: Symbol<'static, PhysicsUpdateWithEntities>,
    destroy_all_fn: Symbol<'static, DestroyAll>,
    destroy_tagged_fn: Symbol<'static, DestroyTagged>,
    destroy_in_scope_tagged_fn: Symbol<'static, DestroyInScopeTagged>,

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
                physics_update_all_fn,
                physics_update_tag_fn,
                physics_update_with_entities_fn,
                destroy_all_fn,
                destroy_tagged_fn,
                destroy_in_scope_tagged_fn,
                get_last_err_msg_fn,
                set_last_err_msg_fn,
            })
        }
    }

    /// Initialises the NativeLibrary by populating it with context.
    pub fn init(
        &mut self,
        dropbear_context: &DropbearContext
    ) -> anyhow::Result<()> {
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

    pub fn update_all(&mut self, dt: f32) -> anyhow::Result<()> {
        unsafe {
            let result = (self.update_all_fn)(dt);
            self.handle_result(result, "update_all")
        }
    }

    pub fn physics_update_all(&mut self, dt: f32) -> anyhow::Result<()> {
        unsafe {
            let result = (self.physics_update_all_fn)(dt);
            self.handle_result(result, "physics_update_all")
        }
    }

    pub fn update_tagged(&mut self, tag: &String, dt: f32) -> anyhow::Result<()> {
        unsafe {
            let c_string: CString = CString::new(tag.clone())?;
            let result = (self.update_tag_fn)(c_string.as_ptr(), dt);
            self.handle_result(result, "update_tagged")
        }
    }

    pub fn physics_update_tagged(&mut self, tag: &String, dt: f32) -> anyhow::Result<()> {
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

    pub fn physics_update_systems_for_entities(
        &self,
        tag: &str,
        entity_ids: &[u64],
        dt: f32,
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

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Displays the types of errors that can be returned by the native library.
pub enum DropbearNativeError {
    /// An error in the case the function returns an unsigned value.
    ///
    /// Subtract [`DropbearNativeError::UnsignedGenericError`] with another value
    /// to get the alternative unsigned error.
    UnsignedGenericError = 65535,
    /// An error that is thrown, but doesn't have any attached context.
    GenericError = 1,
    /// The default return code for a successful FFI operation.
    Success = 0,
    /// A null pointer was provided into the function, and cannot be read.
    NullPointer = -1,
    /// Attempting to query the current world failed for some cause.
    QueryFailed = -2,
    /// The entity does not exist in the world.
    EntityNotFound = -3,

    /// No such component exists **or** the component type id is not the same as the one in the
    /// [`hecs::World`] database.
    ///
    /// # Causes
    /// There are two potential causes for this error:
    /// - If the component that the world is locating is not available within the entity, it will
    ///   throw this.
    /// - Due to Rust's compilation methods and the weird architecture of the dropbear project,
    ///   if the `eucalyptus_core` library is not compiled with an executable
    ///   (such as `eucalyptus-editor` or `redback-runtime`), it will throw this error.
    ///
    /// The querying system of `eucalyptus-core` is done with a [`hecs::World`] (which stored all the
    /// entities), the component registry ([`dropbear_traits::registry::ComponentRegistry`]) that
    /// stores all the potential component names (including ones from external plugins) and the
    /// [`std::any::TypeId`] (which generates a hash of the components/types).
    ///
    /// If `eucalyptus-core` is externally compiled as its own thing (and not bundled with any executable),
    /// a query will lead to a fail due to the hashes being completely different.
    ///
    /// When originally stumped with the issue of DLL's and EXE constantly throwing this error, a user
    /// on the Rust discord provided me with this:
    ///
    /// ```txt
    /// in short, the rules are as follows:
    /// 1. If the compilers that produced two Rust binaries are different, or they were produced by
    ///    compiling for different targets, or if one of them is a compilation root (not of crate
    ///    type rlib or dylib), the two binaries have different Rust ABI
    /// 2. If two binaries have different Rust ABIs, one cannot be used to satisfy a crate dependency
    ///    of another (and thus, absent extern blocks and the associated unsafe, they cannot call each other's functions)
    /// 3. If two binaries have different Rust ABI, their TypeIds will not be consistent
    /// 4. If two Rust binaries are built from different source code, one cannot be substituted for
    ///    another to satisfy the dependency of some other crate
    /// ```
    ///
    /// Yeah, so likely if this error is thrown at you, either the compilation is wrong or you
    /// **genuinely** messed up and didn't include the component.
    ///
    /// Anyhow, if you are able to confirm it was a compilation error, please open an issue
    /// on the dropbear GitHub.
    NoSuchComponent = -4,

    /// No such entity uses the specific component.
    NoSuchEntity = -5,
    /// Inserting something (like a component) into the world failed
    WorldInsertError = -6,
    /// When the graphics queue fails to send its message to the receiver
    SendError = -7,
    /// Error while creating a new CString
    CStringError = -8,
    BufferTooSmall = -9,
    /// Attempting to switch scenes before the world is loaded will throw this error.
    PrematureSceneSwitch = -10,
    /// When a gamepad is not found while querying the input state for so.
    GamepadNotFound = -11,
    /// When the argument is invalid
    InvalidArgument = -12,
    /// The handle provided does not exist. Could be for an asset, entity, or other handle type.
    NoSuchHandle = -13,
    /// Failed to create a Java object via JNI.
    JNIFailedToCreateObject = -14,
    /// Failed to get a field from a Java object via JNI.
    JNIFailedToGetField = -15,
    /// Failed to find a Java class via JNI.
    JNIClassNotFound = -16,
    /// Failed to find a Java method via JNI.
    JNIMethodNotFound = -17,
    /// Failed to unwrap a Java object via JNI.
    JNIUnwrapFailed = -18,
    /// Generic asset error. There was an error thrown, however there is no context attached. 
    GenericAssetError = -19,
    /// The provided uri (either euca:// or https) was invalid and formatted wrong.
    InvalidURI = -20,
    /// The asset provided by the handle is wrong.
    AssetNotFound = -21,
    /// When a handle has been inputted wrongly.
    InvalidHandle = -22,
    PhysicsObjectNotFound = -23,
    
    /// The entity provided was invalid, likely not from [hecs::Entity::from_bits].
    InvalidEntity = -100,


    /// The CString (or `*const c_char`) contained invalid UTF-8 while being decoded.
    InvalidUTF8 = -108,
    /// A generic error when the library doesn't know what happened or cannot find a
    /// suitable error code.
    ///
    /// The number `1274` comes from the total sum of the word "UnknownError" in decimal
    UnknownError = -1274,
}

impl Display for DropbearNativeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            DropbearNativeError::NullPointer => "NullPointer (-1)",
            DropbearNativeError::QueryFailed => "QueryFailed (-2)",
            DropbearNativeError::EntityNotFound => "EntityNotFound (-3)",
            DropbearNativeError::NoSuchComponent => "NoSuchComponent (-4)",
            DropbearNativeError::NoSuchEntity => "NoSuchEntity (-5)",
            DropbearNativeError::WorldInsertError => "WorldInsertError (-6)",
            DropbearNativeError::SendError => "SendError (-7)",
            DropbearNativeError::InvalidUTF8 => "InvalidUTF8 (-108)",
            DropbearNativeError::UnknownError => "UnknownError (-1274)",
            DropbearNativeError::UnsignedGenericError => "UnsignedGenericError (65535)",
            DropbearNativeError::Success => "Success (0) [should not be displayed]",
            DropbearNativeError::CStringError => "CStringError (-8)",
            DropbearNativeError::BufferTooSmall => "BufferTooSmall (-9)",
            DropbearNativeError::PrematureSceneSwitch => "PrematureSceneSwitch (-10)",
            DropbearNativeError::GamepadNotFound => "GamepadNotFound (-11)",
            DropbearNativeError::InvalidArgument => "InvalidArgument (-12)",
            DropbearNativeError::NoSuchHandle => "NoSuchHandle (-13)",
            DropbearNativeError::JNIFailedToCreateObject => "JNIFailedToCreateObject (-14)",
            DropbearNativeError::JNIFailedToGetField => "JNIFailedToGetField (-15)",
            DropbearNativeError::JNIClassNotFound => "JNIClassNotFound (-16)",
            DropbearNativeError::JNIMethodNotFound => "JNIMethodNotFound (-17)",
            DropbearNativeError::JNIUnwrapFailed => "JNIUnwrapFailed (-18)",
            DropbearNativeError::InvalidEntity => "InvalidEntity (-100)",
            DropbearNativeError::GenericAssetError => "GenericAssetError (-19)",
            DropbearNativeError::InvalidURI => "InvalidURI (-20)",
            DropbearNativeError::AssetNotFound => "AssetNotFound (-21)",
            DropbearNativeError::InvalidHandle => "InvalidHandle (-22)",
            DropbearNativeError::GenericError => "GenericError (1)",
            DropbearNativeError::PhysicsObjectNotFound => "PhysicsObjectNotFound (-23)",
        })
    }
}