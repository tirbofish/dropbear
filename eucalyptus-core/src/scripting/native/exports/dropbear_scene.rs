use std::cell::RefCell;
use std::ffi::{c_char, CStr, CString};
use crate::ptr::{CommandBufferPtr, SceneLoaderPtr};
use crate::scripting::native::DropbearNativeError;
use crate::scripting::native::exports::dropbear_common::{DropbearNativeReturn, Handle};
use crate::scripting::native::exports::dropbear_utils::Progress;
use crate::command::CommandBuffer;
use crate::scene::loading::{SCENE_LOADER, SceneLoadHandle as DropbearSceneLoadHandle, SceneLoadResult as DropbearSceneLoadResult};

/// The sister to [`crate::scene::loading::SceneLoadResult`], which provides C-compatible enum values.
#[repr(C)]
#[derive(Default)]
pub enum SceneLoadResult {
    Pending = 0,
    Success = 1,
    #[default]
    Error = 2,
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
    pub name: *mut c_char,
}

/// Loads a scene asynchronously.
///
/// Returns a handle (in the form of an integer)
/// to the scene load operation.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_load_scene_async_1(
    command_buffer_ptr: CommandBufferPtr,
    scene_loader_ptr: SceneLoaderPtr,
    scene_name: *const c_char,
    out_scene_handle: *mut SceneLoadHandle,
) -> DropbearNativeReturn {
    todo!("Not implemented yet")
}

/// Loads a scene asynchronously. Allows you to include a loading_scene_name,
/// which will be displayed while the scene is loading.
///
/// Returns a handle (in the form of an integer)
/// to the scene load operation.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_load_scene_async_2(
    command_buffer_ptr: CommandBufferPtr,
    scene_loader_ptr: SceneLoaderPtr,
    scene_name: *const c_char,
    loading_scene_name: *const c_char,
    out_scene_handle: *mut SceneLoadHandle,
) -> DropbearNativeReturn {
    todo!("Not implemented yet")
}

/// Switches to a scene asynchronously.
///
/// This must be run after you initialise the scene loading (using [`dropbear_load_scene_async_1`]
/// or [`dropbear_load_scene_async_2`]). If this function is called before you have checked the progress
/// (with the [`dropbear_get_scene_load_status`] function), it will return -10 or [`DropbearNativeError::PrematureSceneSwitch`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_switch_to_scene_async(
    command_buffer_ptr: CommandBufferPtr,
    scene_handle: SceneLoadHandle,
) -> DropbearNativeReturn {
    todo!("Not implemented yet")
}

/// Switches to a scene immediately.
///
/// # Warning
/// This will block your main thread and freeze the window. It will be extremely inconvenient for
/// your players, and is recommended to use [`dropbear_load_scene_async_1`] or
/// [`dropbear_load_scene_async_2`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_switch_to_scene_immediate(
    command_buffer_ptr: CommandBufferPtr,
    scene_name: *const c_char,
) -> DropbearNativeReturn {
    todo!("Not implemented yet")
}

/// Gets the progress of a scene load operation.
///
/// Returns a [`Progress`] and a [`DropbearNativeReturn`]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_scene_load_progress(
    scene_loader_ptr: SceneLoaderPtr,
    scene_handle: SceneLoadHandle,
    out_progress: *mut Progress,
) -> DropbearNativeReturn {
    todo!("Not implemented yet")
}

/// Gets the status of a scene load operation
///
/// Returns a [`SceneLoadResult`] and a [`DropbearNativeReturn`]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_scene_load_status(
    scene_loader_ptr: SceneLoaderPtr,
    scene_handle: SceneLoadHandle,
    out_progress: *mut SceneLoadResult,
) -> DropbearNativeReturn {
    todo!("Not implemented yet")
}