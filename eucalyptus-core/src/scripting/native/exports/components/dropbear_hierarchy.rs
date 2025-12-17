use std::ffi::{c_char, CStr};
use std::ptr;
use hecs::World;
use crate::hierarchy::{Children, Parent};
use crate::scripting::native::DropbearNativeError;
use crate::scripting::native::exports::dropbear_common::{DropbearNativeReturn, Handle};
use crate::states::Label;

/// Fetches the children of the given entity.
///
/// Returns an array of [`Handle`], or an [`ptr::null_mut()`] if no children of the entity.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_children(
    world_ptr: *const World,
    entity_id: Handle,
    out_children: *mut *mut Handle, // array of handles
    out_count: *mut usize,
) -> DropbearNativeReturn {
    if world_ptr.is_null() || out_children.is_null() || out_count.is_null() {
        eprintln!("[dropbear_get_children] [ERROR] Null pointer received");
        return DropbearNativeError::NullPointer as i32;
    }

    let world = &*world_ptr;
    let entity = world.find_entity_from_id(entity_id as u32);

    let children: Vec<i64> = match world.query_one::<&Children>(entity) {
        Ok(mut q) => q
            .get()
            .map(|component| {
                component
                    .children()
                    .iter()
                    .map(|child| child.id() as i64)
                    .collect()
            })
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    };

    unsafe {
        *out_count = children.len();
    }

    if children.is_empty() {
        unsafe { *out_children = ptr::null_mut(); }
        return DropbearNativeError::Success as i32;
    }

    let mut boxed = children.into_boxed_slice();
    unsafe {
        *out_children = boxed.as_mut_ptr();
    }
    Box::leak(boxed);
    DropbearNativeError::Success as i32
}

/// Fetches the child of the entity by a String label.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_child_by_label(
    world_ptr: *const World,
    entity_id: Handle,
    label: *const c_char,
    out_child: *mut Handle,
) -> DropbearNativeReturn {
    if world_ptr.is_null() || label.is_null() || out_child.is_null() {
        eprintln!("[dropbear_get_child_by_label] [ERROR] Null pointer received");
        return DropbearNativeError::NullPointer as i32;
    }

    let world = &*world_ptr;
    let entity = world.find_entity_from_id(entity_id as u32);
    let target = match CStr::from_ptr(label).to_str() {
        Ok(s) => s,
        Err(_) => {
            eprintln!("[dropbear_get_child_by_label] [ERROR] Invalid UTF-8 in label");
            return DropbearNativeError::InvalidUTF8 as i32;
        }
    };

    match world.query_one::<&Children>(entity) {
        Ok(mut q) => {
            if let Some(children) = q.get() {
                for child in children.children() {
                    if let Ok(label_comp) = world.get::<&Label>(*child) {
                        if label_comp.as_str() == target {
                            unsafe {
                                *out_child = child.id() as i64;
                            }
                            return DropbearNativeError::Success as i32;
                        }
                    }
                }
            }
            DropbearNativeError::EntityNotFound as i32
        }
        Err(_) => DropbearNativeError::QueryFailed as i32,
    }
}

/// Fetches the parent entity ID of the given entity.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_parent(
    world_ptr: *const World,
    entity_id: Handle,
    out_parent: *mut Handle,
) -> DropbearNativeReturn {
    if world_ptr.is_null() || out_parent.is_null() {
        eprintln!("[dropbear_get_parent] [ERROR] Null pointer received");
        return DropbearNativeError::NullPointer as i32;
    }

    let world = &*world_ptr;
    let entity = world.find_entity_from_id(entity_id as u32);

    match world.get::<&Parent>(entity) {
        Ok(parent) => {
            unsafe {
                *out_parent = parent.parent().id() as i64;
            }
            DropbearNativeError::Success as i32
        }
        Err(_) => {
            eprintln!("[dropbear_get_parent] [WARN] Parent component not found");
            DropbearNativeError::EntityNotFound as i32
        }
    }
}