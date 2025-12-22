use std::ffi::{c_char, CStr};
use hecs::World;
use dropbear_engine::utils::ResourceReference;
use crate::ptr::{AssetRegistryPtr, CommandBufferPtr};
use crate::scripting::native::DropbearNativeError;
use crate::scripting::native::exports::dropbear_common::{DropbearNativeReturn, Handle};
use crate::states::Label;
use crate::command::CommandBuffer;

/// Fetches an entity from the world/current scene by its label.
///
/// Returns the entity's id.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_entity(
    label: *const c_char,
    world_ptr: *const World,
    out_entity: *mut Handle,
) -> DropbearNativeReturn {
    if label.is_null() || world_ptr.is_null() || out_entity.is_null() {
        eprintln!("[dropbear_get_entity] [ERROR] received null pointer");
        return DropbearNativeError::NullPointer as i32;
    }

    let world = unsafe { &*world_ptr };

    let label_str = match unsafe { CStr::from_ptr(label) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            eprintln!("[dropbear_get_entity] [ERROR] invalid UTF-8 in label");
            return DropbearNativeError::InvalidUTF8 as i32;
        }
    };

    for (id, entity_label) in world.query::<&Label>().iter() {
        if entity_label.as_str() == label_str {
            unsafe { *out_entity = id.id() as i64 };
            return DropbearNativeError::Success as i32;
        }
    }

    eprintln!(
        "[dropbear_get_entity] [ERROR] Entity with label '{}' not found",
        label_str
    );
    DropbearNativeError::EntityNotFound as i32
}

/// Fetches an asset from the asset registry as by its name.
///
/// Returns the asset's handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_asset(
    asset_ptr: AssetRegistryPtr,
    uri: *const c_char,
    out_asset_id: *mut i64,
) -> DropbearNativeReturn {
    if asset_ptr.is_null() || uri.is_null() || out_asset_id.is_null() {
        eprintln!("[dropbear_get_asset] [ERROR] Null pointer received");
        return DropbearNativeError::NullPointer as i32;
    }

    let asset = &*asset_ptr;
    let uri_str = match CStr::from_ptr(uri).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            eprintln!("[dropbear_get_asset] [ERROR] Invalid UTF-8 in URI");
            return DropbearNativeError::InvalidUTF8 as i32;
        }
    };

    let reference = match ResourceReference::from_euca_uri(uri_str) {
        Ok(reference) => reference,
        Err(err) => {
            eprintln!(
                "[dropbear_get_asset] [ERROR] Failed to parse resource reference: {}",
                err
            );
            return DropbearNativeError::UnknownError as i32;
        }
    };

    if let Some(handle) = asset.get_handle_from_reference(&reference) {
        unsafe {
            *out_asset_id = handle.raw() as i64;
        }
        DropbearNativeError::Success as i32
    } else {
        eprintln!("[dropbear_get_asset] [ERROR] Asset not found");
        DropbearNativeError::EntityNotFound as i32
    }
}

/// Quits the currently running app or game.
///
/// Does not return anything. If any issues occur, it will panic.
///
/// # Behaviours
/// - eucalyptus-editor - When called, this exits your Play Mode session and brings you back to
///                       `EditorState::Editing`
/// - redback-runtime - When called, this will exit your current process and kill the app as is. It will
///                     also drop any pointers and do any additional clean-up.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_quit(
    command_ptr: CommandBufferPtr,
) {
    if command_ptr.is_null() {
        panic!("NullPointer (-1) while quitting, better off to shoot with gun than to nicely ask...")
    }

    let graphics = unsafe { &*(command_ptr as CommandBufferPtr) };

    if graphics
        .send(CommandBuffer::Quit)
        .is_err()
    {
        panic!("SendError (-7) while sending a quit signal to the CommandBuffer, \
        better off to shoot with gun than to nicely ask...")
    }
}