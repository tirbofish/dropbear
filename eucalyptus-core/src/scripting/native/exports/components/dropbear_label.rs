use std::ffi::{c_char, CString};
use hecs::World;
use crate::scripting::native::DropbearNativeError;
use crate::scripting::native::exports::dropbear_common::{DropbearNativeReturn, Handle};
use crate::states::Label;

/// Fetches the [`Label`] (or name) component of an entity. 
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_entity_name(
    world_ptr: *const World,
    entity_id: Handle,
    out_name: *mut c_char,
    max_len: usize,
) -> DropbearNativeReturn {
    if world_ptr.is_null() || out_name.is_null() {
        eprintln!("[dropbear_get_entity_name] [ERROR] Null pointer received");
        return DropbearNativeError::NullPointer as i32;
    }

    let world = unsafe { &*world_ptr };
    let entity = unsafe { world.find_entity_from_id(entity_id as u32) };

    if let Ok(mut q) = world.query_one::<&Label>(entity) {
        if let Some(label) = q.get() {
            let label_str = label.as_str();
            let Ok(c_str) = CString::new(label_str) else {
                return DropbearNativeError::CStringError as i32;
            };

            let bytes = c_str.as_bytes_with_nul();
            if bytes.len() > max_len {
                return DropbearNativeError::BufferTooSmall as i32;
            }

            unsafe {
                std::ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    out_name as *mut u8,
                    bytes.len()
                );
            }

            return DropbearNativeError::Success as i32;
        }
    }

    DropbearNativeError::QueryFailed as i32
}