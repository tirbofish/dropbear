use hecs::World;
use dropbear_engine::entity::EntityTransform;
use crate::hierarchy::EntityTransformExt;
use crate::scripting::native::{utils, DropbearNativeError};
use crate::scripting::native::exports::dropbear_common::{DropbearNativeReturn, Handle};
use crate::scripting::native::exports::dropbear_math::{NativeEntityTransform, NativeTransform};

/// Fetches the [`EntityTransform`] component of an entity.
///
/// The [`EntityTransform`] returns a `world` and `local` transform.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_transform(
    world_ptr: *const World,
    entity_handle: i64,
    out_transform: *mut NativeEntityTransform,
) -> DropbearNativeReturn {
    if world_ptr.is_null() || out_transform.is_null() {
        eprintln!("[dropbear_get_transform] [ERROR] Null pointer received");
        return DropbearNativeError::NullPointer as i32;
    }

    let world = &*world_ptr;
    let entity = world.find_entity_from_id(entity_handle as u32);

    match world.query_one::<&EntityTransform>(entity) {
        Ok(mut q) => {
            if let Some(transform) = q.get() {
                unsafe {
                    utils::write_native_transform(&mut (*out_transform).local, transform.local());
                    utils::write_native_transform(&mut (*out_transform).world, transform.world());
                }
                DropbearNativeError::Success as i32
            } else {
                eprintln!("[dropbear_get_transform] [ERROR] Entity has no transform component");
                DropbearNativeError::NoSuchComponent as i32
            }
        }
        Err(_) => {
            eprintln!("[dropbear_get_transform] [ERROR] Failed to query entity");
            DropbearNativeError::QueryFailed as i32
        }
    }
}

/// Propagates a transform.
///
/// To propagate a transform is to collect all the [`EntityTransform`] of the parent and self of the
/// querying entity. It then calculates the positions by summing them up (position, rotation and scale),
/// then returns that resulting [`Transform`]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_propagate_transform(
    world_ptr: *const World,
    entity_handle: i64,
    out_transform: *mut NativeTransform,
) -> DropbearNativeReturn {
    if world_ptr.is_null() || out_transform.is_null() {
        eprintln!("[dropbear_propagate_transform] [ERROR] Null pointer received");
        return DropbearNativeError::NullPointer as i32;
    }

    let world = &mut *(world_ptr as *mut World);
    let entity = world.find_entity_from_id(entity_handle as u32);

    match world.query_one::<&mut EntityTransform>(entity) {
        Ok(mut q) => {
            if let Some(transform) = q.get() {
                let propagated = transform.propagate(world, entity);
                unsafe {
                    utils::write_native_transform(&mut *out_transform, &propagated);
                }
                DropbearNativeError::Success as i32
            } else {
                eprintln!(
                    "[dropbear_propagate_transform] [ERROR] Entity has no transform component"
                );
                DropbearNativeError::NoSuchComponent as i32
            }
        }
        Err(_) => {
            eprintln!("[dropbear_propagate_transform] [ERROR] Failed to query entity");
            DropbearNativeError::QueryFailed as i32
        }
    }
}

/// Sets the [`EntityTransform`] component for a specific entity.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_set_transform(
    world_ptr: *const World,
    entity_handle: Handle,
    transform: NativeEntityTransform,
) -> DropbearNativeReturn {
    if world_ptr.is_null() {
        eprintln!("[dropbear_set_transform] [ERROR] Null pointer received");
        return DropbearNativeError::NullPointer as i32;
    }

    let world = &mut *(world_ptr as *mut World);
    let entity = world.find_entity_from_id(entity_handle as u32);

    match world.query_one::<&mut EntityTransform>(entity) {
        Ok(mut q) => {
            if let Some(entity_transform) = q.get() {
                *entity_transform.local_mut() = utils::native_transform_to_transform(&transform.local);
                *entity_transform.world_mut() = utils::native_transform_to_transform(&transform.world);
                DropbearNativeError::Success as i32
            } else {
                eprintln!("[dropbear_set_transform] [ERROR] Entity has no transform component");
                DropbearNativeError::NoSuchComponent as i32
            }
        }
        Err(_) => {
            eprintln!("[dropbear_set_transform] [ERROR] Failed to query entity");
            DropbearNativeError::QueryFailed as i32
        }
    }
}