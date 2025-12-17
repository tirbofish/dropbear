use std::ffi::c_char;
use crate::ptr::CommandBufferPtr;
use crate::scripting::native::exports::dropbear_common::{DropbearNativeReturn, Handle};
use crate::scripting::native::exports::dropbear_utils::Progress;

/// The sister to [`crate::scene::loading::SceneLoadResult`], which provides C-compatible enum values.
#[repr(C)]
#[derive(Default)]
pub enum SceneLoadResult {
    Pending = 0,
    Success = 1,
    #[default]
    Error = -1,
}

impl From<SceneLoadResult> for i32 {
    fn from(value: SceneLoadResult) -> Self {
        value as i32
    }
}

impl From<i32> for SceneLoadResult {
    fn from(value: i32) -> Self {
        match value {
            -1 => SceneLoadResult::Error,
            0 => SceneLoadResult::Pending,
            1 => SceneLoadResult::Success,
            _ => SceneLoadResult::Error,
        }
    }
}

impl From<crate::scene::loading::SceneLoadResult> for SceneLoadResult {
    fn from(value: crate::scene::loading::SceneLoadResult) -> Self {
        match value {
            crate::scene::loading::SceneLoadResult::Pending => SceneLoadResult::Pending,
            crate::scene::loading::SceneLoadResult::Success => SceneLoadResult::Success,
            crate::scene::loading::SceneLoadResult::Error(_) => SceneLoadResult::Error,
        }
    }
}

/// The sister handle to [`crate::scene::loading::SceneLoadHandle`], which provides C-compatible values.
#[repr(C)]
pub struct SceneLoadHandle {
    pub id: Handle,
    pub scene_name: *mut c_char,
    pub result: SceneLoadResult,
}

/// Loads a scene asynchronously.
///
/// Returns a handle (in the form of an integer)
/// to the scene load operation.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_load_scene_async_1(
    _command_buffer_ptr: CommandBufferPtr,
    _scene_name: *const c_char,
    _out_scene_handle: *mut SceneLoadHandle,
) -> DropbearNativeReturn {
    unimplemented!()
}

/// Loads a scene asynchronously. Allows you to include a loading_scene_name,
/// which will be displayed while the scene is loading.
///
/// Returns a handle (in the form of an integer)
/// to the scene load operation.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_load_scene_async_2(
    _command_buffer_ptr: CommandBufferPtr,
    _scene_name: *const c_char,
    _loading_scene_name: *const c_char,
    _out_scene_handle: *mut SceneLoadHandle,
) -> DropbearNativeReturn {
    unimplemented!()
}

/// Switches to a scene asynchronously.
///
/// This must be run after you initialise the scene loading (using [`dropbear_load_scene_async_1`]
/// or [`dropbear_load_scene_async_2`]). If this function is called before you have checked the progress
/// (with the `SceneLoadHandle.result`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_switch_to_scene_async(
    _command_buffer_ptr: CommandBufferPtr,
    _scene_handle: SceneLoadHandle,
) -> DropbearNativeReturn {
    unimplemented!()
}

/// Switches to a scene immediately. 
/// 
/// # Warning
/// This will block your main thread and freeze the window. It will be extremely inconvenient for
/// your players, and is recommended to use [`dropbear_load_scene_async_1`] or 
/// [`dropbear_load_scene_async_2`]. 
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_switch_to_scene_immediate(
    _command_buffer_ptr: CommandBufferPtr,
    _scene_name: *const c_char,
) -> DropbearNativeReturn {
    unimplemented!()
}

/// Gets the progress of a scene load operation. 
/// 
/// Returns a [`Progress`] and a [`DropbearNativeReturn`]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_scene_load_progress(
    _command_buffer_ptr: CommandBufferPtr,
    _scene_handle: SceneLoadHandle,
    _out_progress: *mut Progress,
) -> DropbearNativeReturn {
    unimplemented!()
}