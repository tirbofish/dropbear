use std::ffi::{c_char, CStr, CString};
use crate::command::CommandBuffer;
use crate::ptr::{CommandBufferPtr, SceneLoaderPtr};
use crate::scripting::native::DropbearNativeError;
use crate::scripting::native::exports::dropbear_common::{DropbearNativeReturn, Handle};
use crate::scripting::native::exports::dropbear_utils::Progress;
use crate::scene::loading::{SceneLoadHandle as CoreSceneLoadHandle};

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
    if command_buffer_ptr.is_null() || scene_loader_ptr.is_null() || scene_name.is_null() || out_scene_handle.is_null() {
        return DropbearNativeError::NullPointer as i32;
    }

    let command_buffer = &*command_buffer_ptr;
    let scene_loader = &*scene_loader_ptr;
    
    let Ok(scene_name_str) = CStr::from_ptr(scene_name).to_str() else {
        return DropbearNativeError::InvalidUTF8 as i32;
    };
    let scene_name_string = scene_name_str.to_string();

    let mut loader = scene_loader.lock();
    let id = loader.register_load(scene_name_string.clone());
    
    let c_string = match CString::new(scene_name_string.clone()) {
        Ok(s) => s,
        Err(_) => return DropbearNativeError::InvalidUTF8 as i32,
    };
    
    *out_scene_handle = SceneLoadHandle {
        id: id as Handle,
        name: c_string.into_raw(),
    };

    let core_handle = CoreSceneLoadHandle {
        id,
        scene_name: scene_name_string,
    };
    
    if command_buffer.try_send(CommandBuffer::LoadSceneAsync(core_handle)).is_err() {
        return DropbearNativeError::SendError as i32;
    }

    DropbearNativeError::Success as i32
}

/// Loads a scene asynchronously. Allows you to include a loading_scene_name,
/// which will be displayed while the scene is loading.
///
/// Returns a handle (in the form of an integer)
/// to the scene load operation.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_load_scene_async_2(
    _command_buffer_ptr: CommandBufferPtr,
    _scene_loader_ptr: SceneLoaderPtr,
    _scene_name: *const c_char,
    _loading_scene_name: *const c_char,
    _out_scene_handle: *mut SceneLoadHandle,
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
    if command_buffer_ptr.is_null() {
        return DropbearNativeError::NullPointer as i32;
    }
    
    let command_buffer = &*command_buffer_ptr;
    
    if scene_handle.name.is_null() {
         return DropbearNativeError::NullPointer as i32;
    }
    
    let Ok(scene_name) = CStr::from_ptr(scene_handle.name).to_str() else {
        return DropbearNativeError::InvalidUTF8 as i32;
    };

    let core_handle = CoreSceneLoadHandle {
        id: scene_handle.id as u64,
        scene_name: scene_name.to_string(),
    };

    if command_buffer.try_send(CommandBuffer::SwitchToAsync(core_handle)).is_err() {
        return DropbearNativeError::SendError as i32;
    }

    DropbearNativeError::Success as i32
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
    if command_buffer_ptr.is_null() {
        eprintln!("[dropbear_switch_to_scene_immediate] [ERROR] Null command buffer pointer received");
        return DropbearNativeError::NullPointer as i32;
    }

    if scene_name.is_null() {
        eprintln!("[dropbear_switch_to_scene_immediate] [ERROR] Null scene name received");
        return DropbearNativeError::NullPointer as i32;
    }

    let command_buffer =  &*command_buffer_ptr;
    let Ok(scene_name) = CStr::from_ptr(scene_name).to_str() else {
        return DropbearNativeError::InvalidUTF8 as i32;
    };
    
    if command_buffer.try_send(CommandBuffer::SwitchSceneImmediate(scene_name.to_string())).is_err() {
        return DropbearNativeError::SendError as i32;
    }
    
    DropbearNativeError::Success as i32
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
    if scene_loader_ptr.is_null() || out_progress.is_null() {
        return DropbearNativeError::NullPointer as i32;
    }
    
    let scene_loader = &*scene_loader_ptr;
    let loader = scene_loader.lock();
    
    if let Some(entry) = loader.get_entry(scene_handle.id as u64) {
        let p = &entry.progress;
        let c_message = CString::new(p.message.clone()).unwrap_or_default();
        *out_progress = Progress {
            current: p.current as f64,
            total: p.total as f64,
            message: c_message.into_raw(),
        };
        DropbearNativeError::Success as i32
    } else {
        DropbearNativeError::GenericError as i32
    }
}

/// Gets the status of a scene load operation
///
/// Returns a [`SceneLoadResult`] and a [`DropbearNativeReturn`]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_scene_load_status(
    scene_loader_ptr: SceneLoaderPtr,
    scene_handle: SceneLoadHandle,
    out_status: *mut SceneLoadResult,
) -> DropbearNativeReturn {
    if scene_loader_ptr.is_null() || out_status.is_null() {
        return DropbearNativeError::NullPointer as i32;
    }
    
    let scene_loader = &*scene_loader_ptr;
    let loader = scene_loader.lock();
    
    if let Some(entry) = loader.get_entry(scene_handle.id as u64) {
        *out_status = entry.result.clone().into();
        DropbearNativeError::Success as i32
    } else {
        DropbearNativeError::GenericError as i32
    }
}