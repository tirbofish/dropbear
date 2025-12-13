#![allow(unsafe_op_in_unsafe_fn)]

use crate::camera::{CameraComponent, CameraType};
use crate::hierarchy::{Children, EntityTransformExt, Parent};
use crate::ptr::{AssetRegistryPtr, GraphicsPtr, InputStatePtr};
use crate::scripting::native::DropbearNativeError;
use crate::scripting::native::types::{
    NativeCamera,
    NativeEntityTransform,
    NativeTransform,
    Vector3D,
};
use crate::states::{Label, ModelProperties, Value};
use crate::utils::keycode_from_ordinal;
use crate::window::{CommandBuffer, WindowCommand};
use dropbear_engine::asset::{PointerKind::Const, ASSET_REGISTRY, AssetHandle};
use dropbear_engine::camera::Camera;
use dropbear_engine::entity::{EntityTransform, MeshRenderer, Transform};
use dropbear_engine::model::Model;
use dropbear_engine::utils::ResourceReference;
use glam::{DQuat, DVec3};
use hecs::World;
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use std::ffi::{CStr, CString, c_char};
use std::ptr;
use std::sync::Arc;

fn write_native_transform(target: &mut NativeTransform, transform: &Transform) {
    target.position_x = transform.position.x;
    target.position_y = transform.position.y;
    target.position_z = transform.position.z;
    target.rotation_x = transform.rotation.x;
    target.rotation_y = transform.rotation.y;
    target.rotation_z = transform.rotation.z;
    target.rotation_w = transform.rotation.w;
    target.scale_x = transform.scale.x;
    target.scale_y = transform.scale.y;
    target.scale_z = transform.scale.z;
}

fn native_transform_to_transform(native: &NativeTransform) -> Transform {
    Transform {
        position: DVec3::new(native.position_x, native.position_y, native.position_z),
        rotation: DQuat::from_xyzw(
            native.rotation_x,
            native.rotation_y,
            native.rotation_z,
            native.rotation_w,
        ),
        scale: DVec3::new(native.scale_x, native.scale_y, native.scale_z),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_entity(
    label: *const c_char,
    world_ptr: *const World,
    out_entity: *mut i64,
) -> i32 {
    if label.is_null() || world_ptr.is_null() || out_entity.is_null() {
        eprintln!("[dropbear_get_entity] [ERROR] received null pointer");
        return -1;
    }

    let world = unsafe { &*world_ptr };

    let label_str = match unsafe { CStr::from_ptr(label) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            eprintln!("[dropbear_get_entity] [ERROR] invalid UTF-8 in label");
            return -108;
        }
    };

    for (id, entity_label) in world.query::<&Label>().iter() {
        if entity_label.as_str() == label_str {
            unsafe { *out_entity = id.id() as i64 };
            return 0;
        }
    }

    eprintln!(
        "[dropbear_get_entity] [ERROR] Entity with label '{}' not found",
        label_str
    );
    -3
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_entity_name(
    world_ptr: *const World,
    entity_id: i64,
    out_name: *mut c_char,
    max_len: usize,
) -> i32 {
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_world_transform(
    world_ptr: *const World,
    entity_id: i64,
    out_transform: *mut NativeTransform,
) -> i32 {
    if world_ptr.is_null() || out_transform.is_null() {
        eprintln!("[dropbear_get_world_transform] [ERROR] Null pointer received");
        return -1;
    }

    let world = unsafe { &*world_ptr };
    let entity = unsafe { world.find_entity_from_id(entity_id as u32) };

    match world.query_one::<&EntityTransform>(entity) {
        Ok(mut q) => {
            if let Some(transform) = q.get() {
                let transform = transform.world();
                unsafe {
                    (*out_transform).position_x = transform.position.x;
                    (*out_transform).position_y = transform.position.y;
                    (*out_transform).position_z = transform.position.z;
                    (*out_transform).rotation_x = transform.rotation.x;
                    (*out_transform).rotation_y = transform.rotation.y;
                    (*out_transform).rotation_z = transform.rotation.z;
                    (*out_transform).rotation_w = transform.rotation.w;
                    (*out_transform).scale_x = transform.scale.x;
                    (*out_transform).scale_y = transform.scale.y;
                    (*out_transform).scale_z = transform.scale.z;
                }
                0
            } else {
                eprintln!(
                    "[dropbear_get_transform] [ERROR] Entity has no WorldTransform component"
                );
                DropbearNativeError::NoSuchComponent as i32
            }
        }
        Err(_) => {
            eprintln!("[dropbear_get_transform] [ERROR] Failed to query entity");
            DropbearNativeError::QueryFailed as i32
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_local_transform(
    world_ptr: *const World,
    entity_id: i64,
    out_transform: *mut NativeTransform,
) -> i32 {
    if world_ptr.is_null() || out_transform.is_null() {
        eprintln!("[dropbear_get_local_transform] [ERROR] Null pointer received");
        return DropbearNativeError::NullPointer as i32;
    }

    let world = unsafe { &*world_ptr };
    let entity = unsafe { world.find_entity_from_id(entity_id as u32) };

    match world.query_one::<&EntityTransform>(entity) {
        Ok(mut q) => {
            if let Some(transform) = q.get() {
                let transform = transform.local();
                unsafe {
                    (*out_transform).position_x = transform.position.x;
                    (*out_transform).position_y = transform.position.y;
                    (*out_transform).position_z = transform.position.z;
                    (*out_transform).rotation_x = transform.rotation.x;
                    (*out_transform).rotation_y = transform.rotation.y;
                    (*out_transform).rotation_z = transform.rotation.z;
                    (*out_transform).rotation_w = transform.rotation.w;
                    (*out_transform).scale_x = transform.scale.x;
                    (*out_transform).scale_y = transform.scale.y;
                    (*out_transform).scale_z = transform.scale.z;
                }
                0
            } else {
                eprintln!(
                    "[dropbear_get_local_transform] [ERROR] Entity has no LocalTransform component"
                );
                DropbearNativeError::NoSuchComponent as i32
            }
        }
        Err(_) => {
            eprintln!("[dropbear_get_local_transform] [ERROR] Failed to query entity");
            DropbearNativeError::QueryFailed as i32
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_transform(
    world_ptr: *const World,
    entity_handle: i64,
    out_transform: *mut NativeEntityTransform,
) -> i32 {
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
                    write_native_transform(&mut (*out_transform).local, transform.local());
                    write_native_transform(&mut (*out_transform).world, transform.world());
                }
                0
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_propagate_transform(
    world_ptr: *const World,
    entity_handle: i64,
    out_transform: *mut NativeTransform,
) -> i32 {
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
                    write_native_transform(&mut *out_transform, &propagated);
                }
                0
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_set_transform(
    world_ptr: *const World,
    entity_handle: i64,
    transform: NativeEntityTransform,
) -> i32 {
    if world_ptr.is_null() {
        eprintln!("[dropbear_set_transform] [ERROR] Null pointer received");
        return DropbearNativeError::NullPointer as i32;
    }

    let world = &mut *(world_ptr as *mut World);
    let entity = world.find_entity_from_id(entity_handle as u32);

    match world.query_one::<&mut EntityTransform>(entity) {
        Ok(mut q) => {
            if let Some(entity_transform) = q.get() {
                *entity_transform.local_mut() = native_transform_to_transform(&transform.local);
                *entity_transform.world_mut() = native_transform_to_transform(&transform.world);
                0
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_string_property(
    world_ptr: *const World,
    entity_handle: i64,
    label: *const c_char,
    out_value: *mut c_char,
    out_value_max_length: i32,
) -> i32 {
    if world_ptr.is_null() || label.is_null() || out_value.is_null() {
        eprintln!("[dropbear_get_string_property] [ERROR] Null pointer received");
        return -1;
    }

    let world = unsafe { &*world_ptr };
    let entity = unsafe { world.find_entity_from_id(entity_handle as u32) };

    let label_str = match unsafe { CStr::from_ptr(label) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            eprintln!("[dropbear_get_string_property] [ERROR] Invalid UTF-8 in label");
            return -108;
        }
    };

    match world.query_one::<(&MeshRenderer, &ModelProperties)>(entity) {
        Ok(mut q) => {
            if let Some((_, props)) = q.get() {
                if let Some(Value::String(val)) = props.get_property(label_str) {
                    let bytes = val.as_bytes();
                    let copy_len = std::cmp::min(bytes.len(), (out_value_max_length - 1) as usize);
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            bytes.as_ptr(),
                            out_value as *mut u8,
                            copy_len,
                        );
                        *out_value.add(copy_len) = 0; // null terminator
                    }
                    0
                } else {
                    eprintln!(
                        "[dropbear_get_string_property] [WARN] Property not found or wrong type"
                    );
                    -3
                }
            } else {
                -4
            }
        }
        Err(_) => {
            eprintln!("[dropbear_get_string_property] [ERROR] Failed to query entity");
            -2
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_int_property(
    world_ptr: *const World,
    entity_handle: i64,
    label: *const c_char,
    out_value: *mut i32,
) -> i32 {
    if world_ptr.is_null() || label.is_null() || out_value.is_null() {
        eprintln!("[dropbear_get_int_property] [ERROR] Null pointer received");
        return -1;
    }

    let world = unsafe { &*world_ptr };
    let entity = unsafe { world.find_entity_from_id(entity_handle as u32) };

    let label_str = match unsafe { CStr::from_ptr(label) }.to_str() {
        Ok(s) => s,
        Err(_) => return -108,
    };

    match world.query_one::<(&MeshRenderer, &ModelProperties)>(entity) {
        Ok(mut q) => {
            if let Some((_, props)) = q.get() {
                if let Some(Value::Int(val)) = props.get_property(label_str) {
                    unsafe { *out_value = *val as i32 };
                    0
                } else {
                    -3
                }
            } else {
                -4
            }
        }
        Err(_) => -2,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_long_property(
    world_ptr: *const World,
    entity_handle: i64,
    label: *const c_char,
    out_value: *mut i64,
) -> i32 {
    if world_ptr.is_null() || label.is_null() || out_value.is_null() {
        return -1;
    }

    let world = unsafe { &*world_ptr };
    let entity = unsafe { world.find_entity_from_id(entity_handle as u32) };

    let label_str = match unsafe { CStr::from_ptr(label) }.to_str() {
        Ok(s) => s,
        Err(_) => return -108,
    };

    match world.query_one::<(&MeshRenderer, &ModelProperties)>(entity) {
        Ok(mut q) => {
            if let Some((_, props)) = q.get() {
                if let Some(Value::Int(val)) = props.get_property(label_str) {
                    unsafe { *out_value = *val };
                    0
                } else {
                    -3
                }
            } else {
                -4
            }
        }
        Err(_) => -2,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_float_property(
    world_ptr: *const World,
    entity_handle: i64,
    label: *const c_char,
    out_value: *mut f64,
) -> i32 {
    if world_ptr.is_null() || label.is_null() || out_value.is_null() {
        return -1;
    }

    let world = unsafe { &*world_ptr };
    let entity = unsafe { world.find_entity_from_id(entity_handle as u32) };

    let label_str = match unsafe { CStr::from_ptr(label) }.to_str() {
        Ok(s) => s,
        Err(_) => return -108,
    };

    match world.query_one::<(&MeshRenderer, &ModelProperties)>(entity) {
        Ok(mut q) => {
            if let Some((_, props)) = q.get() {
                if let Some(Value::Float(val)) = props.get_property(label_str) {
                    unsafe { *out_value = *val as f64 };
                    0
                } else {
                    -3
                }
            } else {
                -4
            }
        }
        Err(_) => -2,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_double_property(
    world_ptr: *const World,
    entity_handle: i64,
    label: *const c_char,
    out_value: *mut f64,
) -> i32 {
    if world_ptr.is_null() || label.is_null() || out_value.is_null() {
        return -1;
    }

    let world = unsafe { &*world_ptr };
    let entity = unsafe { world.find_entity_from_id(entity_handle as u32) };

    let label_str = match unsafe { CStr::from_ptr(label) }.to_str() {
        Ok(s) => s,
        Err(_) => return -108,
    };

    match world.query_one::<(&MeshRenderer, &ModelProperties)>(entity) {
        Ok(mut q) => {
            if let Some((_, props)) = q.get() {
                if let Some(Value::Float(val)) = props.get_property(label_str) {
                    unsafe { *out_value = *val };
                    0
                } else {
                    -3
                }
            } else {
                -4
            }
        }
        Err(_) => -2,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_bool_property(
    world_ptr: *const World,
    entity_handle: i64,
    label: *const c_char,
    out_value: *mut i32,
) -> i32 {
    if world_ptr.is_null() || label.is_null() || out_value.is_null() {
        return -1;
    }

    let world = unsafe { &*world_ptr };
    let entity = unsafe { world.find_entity_from_id(entity_handle as u32) };

    let label_str = match unsafe { CStr::from_ptr(label) }.to_str() {
        Ok(s) => s,
        Err(_) => return -108,
    };

    match world.query_one::<(&MeshRenderer, &ModelProperties)>(entity) {
        Ok(mut q) => {
            if let Some((_, props)) = q.get() {
                if let Some(Value::Bool(val)) = props.get_property(label_str) {
                    unsafe { *out_value = if *val { 1 } else { 0 } };
                    0
                } else {
                    -3
                }
            } else {
                -4
            }
        }
        Err(_) => -2,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_vec3_property(
    world_ptr: *const World,
    entity_handle: i64,
    label: *const c_char,
    out_x: *mut f32,
    out_y: *mut f32,
    out_z: *mut f32,
) -> i32 {
    if world_ptr.is_null()
        || label.is_null()
        || out_x.is_null()
        || out_y.is_null()
        || out_z.is_null()
    {
        return -1;
    }

    let world = unsafe { &*world_ptr };
    let entity = unsafe { world.find_entity_from_id(entity_handle as u32) };

    let label_str = match unsafe { CStr::from_ptr(label) }.to_str() {
        Ok(s) => s,
        Err(_) => return -108,
    };

    match world.query_one::<(&MeshRenderer, &ModelProperties)>(entity) {
        Ok(mut q) => {
            if let Some((_, props)) = q.get() {
                if let Some(Value::Vec3([x, y, z])) = props.get_property(label_str) {
                    unsafe {
                        *out_x = *x;
                        *out_y = *y;
                        *out_z = *z;
                    }
                    0
                } else {
                    -3
                }
            } else {
                -4
            }
        }
        Err(_) => -2,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_set_string_property(
    world_ptr: *const World,
    entity_handle: i64,
    label: *const c_char,
    value: *const c_char,
) -> i32 {
    if world_ptr.is_null() || label.is_null() || value.is_null() {
        return -1;
    }

    let world = unsafe { &mut *(world_ptr as *mut World) };
    let entity = unsafe { world.find_entity_from_id(entity_handle as u32) };

    let label_str = match unsafe { CStr::from_ptr(label) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return -108,
    };

    let value_str = match unsafe { CStr::from_ptr(value) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return -108,
    };

    match world.query_one_mut::<(&MeshRenderer, &mut ModelProperties)>(entity) {
        Ok((_, props)) => {
            props.set_property(label_str, Value::String(value_str));
            0
        }
        Err(_) => -2,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_set_int_property(
    world_ptr: *const World,
    entity_handle: i64,
    label: *const c_char,
    value: i32,
) -> i32 {
    if world_ptr.is_null() || label.is_null() {
        return -1;
    }

    let world = unsafe { &mut *(world_ptr as *mut World) };
    let entity = unsafe { world.find_entity_from_id(entity_handle as u32) };

    let label_str = match unsafe { CStr::from_ptr(label) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return -108,
    };

    match world.query_one_mut::<(&MeshRenderer, &mut ModelProperties)>(entity) {
        Ok((_, props)) => {
            props.set_property(label_str, Value::Int(value as i64));
            0
        }
        Err(_) => -2,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_set_long_property(
    world_ptr: *const World,
    entity_handle: i64,
    label: *const c_char,
    value: i64,
) -> i32 {
    if world_ptr.is_null() || label.is_null() {
        return -1;
    }

    let world = unsafe { &mut *(world_ptr as *mut World) };
    let entity = unsafe { world.find_entity_from_id(entity_handle as u32) };

    let label_str = match unsafe { CStr::from_ptr(label) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return -108,
    };

    match world.query_one_mut::<(&MeshRenderer, &mut ModelProperties)>(entity) {
        Ok((_, props)) => {
            props.set_property(label_str, Value::Int(value));
            0
        }
        Err(_) => -2,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_set_float_property(
    world_ptr: *const World,
    entity_handle: i64,
    label: *const c_char,
    value: f32,
) -> i32 {
    if world_ptr.is_null() || label.is_null() {
        return -1;
    }

    let world = unsafe { &mut *(world_ptr as *mut World) };
    let entity = unsafe { world.find_entity_from_id(entity_handle as u32) };

    let label_str = match unsafe { CStr::from_ptr(label) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return -108,
    };

    match world.query_one_mut::<(&MeshRenderer, &mut ModelProperties)>(entity) {
        Ok((_, props)) => {
            props.set_property(label_str, Value::Float(value as f64));
            0
        }
        Err(_) => -2,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_set_double_property(
    world_ptr: *const World,
    entity_handle: i64,
    label: *const c_char,
    value: f64,
) -> i32 {
    if world_ptr.is_null() || label.is_null() {
        return -1;
    }

    let world = unsafe { &mut *(world_ptr as *mut World) };
    let entity = unsafe { world.find_entity_from_id(entity_handle as u32) };

    let label_str = match unsafe { CStr::from_ptr(label) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return -108,
    };

    match world.query_one_mut::<(&MeshRenderer, &mut ModelProperties)>(entity) {
        Ok((_, props)) => {
            props.set_property(label_str, Value::Float(value));
            0
        }
        Err(_) => -2,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_set_bool_property(
    world_ptr: *const World,
    entity_handle: i64,
    label: *const c_char,
    value: i32,
) -> i32 {
    if world_ptr.is_null() || label.is_null() {
        return -1;
    }

    let world = unsafe { &mut *(world_ptr as *mut World) };
    let entity = unsafe { world.find_entity_from_id(entity_handle as u32) };

    let label_str = match unsafe { CStr::from_ptr(label) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return -108,
    };

    match world.query_one_mut::<(&MeshRenderer, &mut ModelProperties)>(entity) {
        Ok((_, props)) => {
            props.set_property(label_str, Value::Bool(value != 0));
            0
        }
        Err(_) => -2,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_set_vec3_property(
    world_ptr: *const World,
    entity_handle: i64,
    label: *const c_char,
    x: f32,
    y: f32,
    z: f32,
) -> i32 {
    if world_ptr.is_null() || label.is_null() {
        return -1;
    }

    let world = unsafe { &mut *(world_ptr as *mut World) };
    let entity = unsafe { world.find_entity_from_id(entity_handle as u32) };

    let label_str = match unsafe { CStr::from_ptr(label) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return -108,
    };

    match world.query_one_mut::<(&MeshRenderer, &mut ModelProperties)>(entity) {
        Ok((_, props)) => {
            props.set_property(label_str, Value::Vec3([x, y, z]));
            0
        }
        Err(_) => -2,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_print_input_state(input_state_ptr: InputStatePtr) {
    if input_state_ptr.is_null() {
        eprintln!("[dropbear_print_input_state] [ERROR] Input state pointer is null");
        return;
    }

    let input_state = unsafe { &*input_state_ptr };
    println!("{:#?}", input_state);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_is_key_pressed(
    input_state_ptr: InputStatePtr,
    keycode: i32,
    out_value: *mut i32,
) -> i32 {
    if input_state_ptr.is_null() || out_value.is_null() {
        return -1;
    }

    let input = unsafe { &*input_state_ptr };

    match keycode_from_ordinal(keycode) {
        Some(k) => {
            let is_pressed = input.pressed_keys.contains(&k);
            unsafe { *out_value = if is_pressed { 1 } else { 0 } };
            0
        }
        None => {
            eprintln!("[dropbear_is_key_pressed] [WARN] Invalid keycode");
            unsafe { *out_value = 0 };
            0
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_mouse_position(
    input_state_ptr: InputStatePtr,
    out_x: *mut f32,
    out_y: *mut f32,
) -> i32 {
    if input_state_ptr.is_null() || out_x.is_null() || out_y.is_null() {
        return -1;
    }

    let input = unsafe { &*input_state_ptr };

    unsafe {
        *out_x = input.mouse_pos.0 as f32;
        *out_y = input.mouse_pos.1 as f32;
    }

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_is_mouse_button_pressed(
    input_state_ptr: InputStatePtr,
    button_code: i32,
    out_pressed: *mut i32,
) -> i32 {
    if input_state_ptr.is_null() || out_pressed.is_null() {
        return -1;
    }

    let input = unsafe { &*input_state_ptr };

    match keycode_from_ordinal(button_code) {
        None => {
            eprintln!("[dropbear_is_mouse_button_pressed] [WARN] Invalid button code");
            unsafe { *out_pressed = 0 };
            return 0;
        }
        Some(key) => {
            if input.pressed_keys.contains(&key) {
                unsafe { *out_pressed = 1 };
            } else {
                unsafe { *out_pressed = 0 };
            }
        }
    }

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_mouse_delta(
    input_state_ptr: InputStatePtr,
    out_delta_x: *mut f32,
    out_delta_y: *mut f32,
) -> i32 {
    if input_state_ptr.is_null() || out_delta_x.is_null() || out_delta_y.is_null() {
        return -1;
    }

    let input = unsafe { &mut *(input_state_ptr as InputStatePtr) };

    if let Some(pos) = input.mouse_delta.take() {
        unsafe {
            *out_delta_x = pos.0 as f32;
            *out_delta_y = pos.1 as f32;
        }
    } else {
        unsafe {
            *out_delta_x = 0.0;
            *out_delta_y = 0.0;
        }
    }

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_is_cursor_locked(
    input_state_ptr: InputStatePtr,
    out_locked: *mut i32,
) -> i32 {
    if input_state_ptr.is_null() || out_locked.is_null() {
        return -1;
    }

    let input = unsafe { &*input_state_ptr };

    unsafe { *out_locked = if input.is_cursor_locked { 1 } else { 0 } };

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_set_cursor_locked(
    input_state_ptr: InputStatePtr,
    queue_ptr: GraphicsPtr,
    locked: i32,
) -> i32 {
    if input_state_ptr.is_null() || queue_ptr.is_null() {
        return DropbearNativeError::NullPointer as i32;
    }

    let input = unsafe { &mut *(input_state_ptr as InputStatePtr) };

    let graphics = unsafe { &*(queue_ptr as GraphicsPtr) };

    input.is_cursor_locked = locked != 0;

    if graphics
        .send(CommandBuffer::WindowCommand(WindowCommand::WindowGrab(
            input.is_cursor_locked,
        )))
        .is_err()
    {
        DropbearNativeError::SendError as i32
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_camera(
    world_ptr: *const World,
    label: *const c_char,
    out_camera: *mut NativeCamera,
) -> i32 {
    if world_ptr.is_null() || label.is_null() || out_camera.is_null() {
        eprintln!("[dropbear_get_camera] [ERROR] Null pointer received");
        return -1;
    }

    let world = unsafe { &*world_ptr };

    let label_str = match unsafe { CStr::from_ptr(label) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            eprintln!("[dropbear_get_camera] [ERROR] Invalid UTF-8 in label");
            return DropbearNativeError::InvalidUTF8 as i32;
        }
    };

    if let Some((id, (cam, comp))) = world
        .query::<(&Camera, &CameraComponent)>()
        .iter()
        .find(|(_, (cam, _))| cam.label == label_str)
    {
        if matches!(comp.camera_type, CameraType::Debug) {
            eprintln!("[dropbear_get_camera] [WARN] Querying a CameraType::Debug is illegal");
            return -5;
        }

        let label_cstring = CString::new(cam.label.as_str()).unwrap();

        unsafe {
            (*out_camera).label = label_cstring.into_raw();
            (*out_camera).entity_id = id.id() as i64;

            (*out_camera).eye = Vector3D {
                x: cam.eye.x as f32,
                y: cam.eye.y as f32,
                z: cam.eye.z as f32,
            };

            (*out_camera).target = Vector3D {
                x: cam.target.x as f32,
                y: cam.target.y as f32,
                z: cam.target.z as f32,
            };

            (*out_camera).up = Vector3D {
                x: cam.up.x as f32,
                y: cam.up.y as f32,
                z: cam.up.z as f32,
            };

            (*out_camera).aspect = cam.aspect;
            (*out_camera).fov_y = cam.settings.fov_y;
            (*out_camera).znear = cam.znear;
            (*out_camera).zfar = cam.zfar;
            (*out_camera).yaw = cam.yaw;
            (*out_camera).pitch = cam.pitch;
            (*out_camera).speed = cam.settings.speed;
            (*out_camera).sensitivity = cam.settings.sensitivity;
        }

        return 0;
    }

    eprintln!(
        "[dropbear_get_camera] [ERROR] Camera with label '{}' not found",
        label_str
    );
    -3
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_attached_camera(
    world_ptr: *const World,
    id: i64,
    out_camera: *mut NativeCamera,
) -> i32 {
    if world_ptr.is_null() || out_camera.is_null() {
        eprintln!("[dropbear_get_attached_camera] [ERROR] Null pointer received");
        return -1;
    }

    let world = unsafe { &*world_ptr };
    let entity = unsafe { world.find_entity_from_id(id as u32) };

    match world.query_one::<(&Camera, &CameraComponent)>(entity) {
        Ok(mut q) => {
            if let Some((cam, comp)) = q.get() {
                if matches!(comp.camera_type, CameraType::Debug) {
                    eprintln!(
                        "[dropbear_get_attached_camera] [WARN] Querying a CameraType::Debug is illegal"
                    );
                    return -5;
                }

                let label_cstring = std::ffi::CString::new(cam.label.as_str()).unwrap();

                unsafe {
                    (*out_camera).label = label_cstring.into_raw();
                    (*out_camera).entity_id = id;

                    (*out_camera).eye = Vector3D {
                        x: cam.eye.x as f32,
                        y: cam.eye.y as f32,
                        z: cam.eye.z as f32,
                    };

                    (*out_camera).target = Vector3D {
                        x: cam.target.x as f32,
                        y: cam.target.y as f32,
                        z: cam.target.z as f32,
                    };

                    (*out_camera).up = Vector3D {
                        x: cam.up.x as f32,
                        y: cam.up.y as f32,
                        z: cam.up.z as f32,
                    };

                    (*out_camera).aspect = cam.aspect;
                    (*out_camera).fov_y = cam.settings.fov_y;
                    (*out_camera).znear = cam.znear;
                    (*out_camera).zfar = cam.zfar;
                    (*out_camera).yaw = cam.yaw;
                    (*out_camera).pitch = cam.pitch;
                    (*out_camera).speed = cam.settings.speed;
                    (*out_camera).sensitivity = cam.settings.sensitivity;
                }

                0
            } else {
                eprintln!("[dropbear_get_attached_camera] [ERROR] Entity has no Camera component");
                -4
            }
        }
        Err(_) => {
            eprintln!("[dropbear_get_attached_camera] [ERROR] Failed to query entity");
            -2
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_set_camera(
    world_ptr: *mut World,
    camera: *const NativeCamera,
) -> i32 {
    if world_ptr.is_null() || camera.is_null() {
        eprintln!("[dropbear_set_camera] [ERROR] Null pointer received");
        return -1;
    }

    let world = unsafe { &mut *(world_ptr) };
    let cam_data = unsafe { &*camera };

    let entity = unsafe { world.find_entity_from_id(cam_data.entity_id as u32) };

    match world.query_one_mut::<&mut Camera>(entity) {
        Ok(cam) => {
            cam.eye = DVec3::new(
                cam_data.eye.x as f64,
                cam_data.eye.y as f64,
                cam_data.eye.z as f64,
            );

            cam.target = DVec3::new(
                cam_data.target.x as f64,
                cam_data.target.y as f64,
                cam_data.target.z as f64,
            );

            cam.up = DVec3::new(
                cam_data.up.x as f64,
                cam_data.up.y as f64,
                cam_data.up.z as f64,
            );

            cam.aspect = cam_data.aspect;
            cam.settings.fov_y = cam_data.fov_y;
            cam.znear = cam_data.znear;
            cam.zfar = cam_data.zfar;
            cam.yaw = cam_data.yaw;
            cam.pitch = cam_data.pitch;
            cam.settings.speed = cam_data.speed;
            cam.settings.sensitivity = cam_data.sensitivity;

            0
        }
        Err(_) => {
            eprintln!("[dropbear_set_camera] [ERROR] Unable to query camera component");
            -2
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_last_mouse_pos(
    input_state_ptr: InputStatePtr,
    out_x: *mut f32,
    out_y: *mut f32,
) -> i32 {
    if input_state_ptr.is_null() || out_x.is_null() || out_y.is_null() {
        return -1;
    }

    let input = unsafe { &*input_state_ptr };

    if let Some(pos) = input.last_mouse_pos {
        unsafe {
            *out_x = pos.0 as f32;
            *out_y = pos.1 as f32;
        }
    } else {
        unsafe {
            *out_x = 0.0;
            *out_y = 0.0;
        }
    }

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_is_cursor_hidden(
    input_state_ptr: InputStatePtr,
    out_hidden: *mut i32,
) -> i32 {
    if input_state_ptr.is_null() || out_hidden.is_null() {
        return -1;
    }

    let input = unsafe { &*input_state_ptr };

    unsafe { *out_hidden = if input.is_cursor_hidden { 1 } else { 0 } };

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_set_cursor_hidden(
    input_state_ptr: InputStatePtr,
    queue_ptr: GraphicsPtr,
    hidden: i32,
) -> i32 {
    if input_state_ptr.is_null() || queue_ptr.is_null() {
        return DropbearNativeError::NullPointer as i32;
    }

    let input = unsafe { &mut *(input_state_ptr as InputStatePtr) };
    let graphics = unsafe { &*(queue_ptr as GraphicsPtr) };
    input.is_cursor_hidden = hidden != 0;

    if graphics
        .send(CommandBuffer::WindowCommand(WindowCommand::HideCursor(
            input.is_cursor_hidden,
        )))
        .is_err()
    {
        DropbearNativeError::SendError as i32
    } else {
        DropbearNativeError::Success as i32
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_model(
    world_ptr: *const World,
    asset_ptr: AssetRegistryPtr,
    entity_handle: i64,
    out_model_id: *mut i64,
) -> i32 {
    if world_ptr.is_null() || asset_ptr.is_null() || out_model_id.is_null() {
        eprintln!("[dropbear_get_model] [ERROR] Null pointer received");
        return DropbearNativeError::NullPointer as i32;
    }

    let world = &*world_ptr;
    let _asset = &*asset_ptr;
    let entity = world.find_entity_from_id(entity_handle as u32);

    match world.query_one::<&MeshRenderer>(entity) {
        Ok(mut q) => {
            if let Some(renderer) = q.get() {
                unsafe { *out_model_id = renderer.asset_handle().raw() as i64; }
                0
            } else {
                eprintln!("[dropbear_get_model] [ERROR] Entity missing MeshRenderer component");
                DropbearNativeError::NoSuchComponent as i32
            }
        }
        Err(_) => {
            eprintln!("[dropbear_get_model] [ERROR] Failed to query entity");
            DropbearNativeError::QueryFailed as i32
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_set_model(
    world_ptr: *const World,
    asset_ptr: AssetRegistryPtr,
    entity_handle: i64,
    model_id: i64,
) -> i32 {
    if world_ptr.is_null() || asset_ptr.is_null() {
        eprintln!("[dropbear_set_model] [ERROR] Null pointer received");
        return DropbearNativeError::NullPointer as i32;
    }

    let world = &*world_ptr;
    let asset = &*asset_ptr;
    let entity = world.find_entity_from_id(entity_handle as u32);

    match world.query_one::<&mut MeshRenderer>(entity) {
        Ok(mut q) => {
            if let Some(renderer) = q.get() {
                let handle = AssetHandle::new(model_id as u64);
                if let Err(err) = renderer.set_asset_handle_raw(asset, handle) {
                    eprintln!(
                        "[dropbear_set_model] [ERROR] Failed to set model handle {}: {}",
                        model_id,
                        err
                    );
                    DropbearNativeError::UnknownError as i32
                } else {
                    0
                }
            } else {
                eprintln!("[dropbear_set_model] [ERROR] Entity missing MeshRenderer component");
                DropbearNativeError::NoSuchComponent as i32
            }
        }
        Err(_) => {
            eprintln!("[dropbear_set_model] [ERROR] Failed to query entity");
            DropbearNativeError::QueryFailed as i32
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_is_model_handle(
    asset_ptr: AssetRegistryPtr,
    handle: i64,
    out_is_model: *mut i32,
) -> i32 {
    if asset_ptr.is_null() || out_is_model.is_null() {
        eprintln!("[dropbear_is_model_handle] [ERROR] Null pointer received");
        return DropbearNativeError::NullPointer as i32;
    }

    let asset = &*asset_ptr;
    let handle = AssetHandle::new(handle as u64);
    unsafe {
        *out_is_model = if asset.is_model(handle) { 1 } else { 0 };
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_is_using_model(
    world_ptr: *const World,
    entity_handle: i64,
    model_handle: i64,
    out_is_using: *mut i32,
) -> i32 {
    if world_ptr.is_null() || out_is_using.is_null() {
        eprintln!("[dropbear_is_using_model] [ERROR] Null pointer received");
        return DropbearNativeError::NullPointer as i32;
    }

    let world = &*world_ptr;
    let entity = world.find_entity_from_id(entity_handle as u32);
    let handle = AssetHandle::new(model_handle as u64);

    match world.query_one::<&MeshRenderer>(entity) {
        Ok(mut q) => {
            if let Some(renderer) = q.get() {
                unsafe {
                    *out_is_using = if renderer.uses_model_handle(handle) { 1 } else { 0 };
                }
                0
            } else {
                eprintln!(
                    "[dropbear_is_using_model] [ERROR] Entity missing MeshRenderer component"
                );
                DropbearNativeError::NoSuchComponent as i32
            }
        }
        Err(_) => {
            eprintln!("[dropbear_is_using_model] [ERROR] Failed to query entity");
            DropbearNativeError::QueryFailed as i32
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_texture(
    world_ptr: *const World,
    asset_ptr: AssetRegistryPtr,
    entity_handle: i64,
    name: *const c_char,
    out_texture_id: *mut i64,
) -> i32 {
    if world_ptr.is_null() || asset_ptr.is_null() || name.is_null() || out_texture_id.is_null() {
        eprintln!("[dropbear_get_texture] [ERROR] Null pointer received");
        return DropbearNativeError::NullPointer as i32;
    }

    let world = &*world_ptr;
    let asset = &*asset_ptr;
    let entity = world.find_entity_from_id(entity_handle as u32);
    let label = match CStr::from_ptr(name).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            eprintln!("[dropbear_get_texture] [ERROR] Invalid UTF-8 in material name");
            return DropbearNativeError::InvalidUTF8 as i32;
        }
    };

    match world.query_one::<&MeshRenderer>(entity) {
        Ok(mut q) => {
            if let Some(renderer) = q.get() {
                if let Some(handle) = renderer.material_handle_raw(asset, label.as_str()) {
                    unsafe { *out_texture_id = handle.raw() as i64; }
                    0
                } else {
                    eprintln!(
                        "[dropbear_get_texture] [ERROR] Material '{}' not found on entity",
                        label
                    );
                    DropbearNativeError::EntityNotFound as i32
                }
            } else {
                eprintln!("[dropbear_get_texture] [ERROR] Entity missing MeshRenderer component");
                DropbearNativeError::NoSuchComponent as i32
            }
        }
        Err(_) => {
            eprintln!("[dropbear_get_texture] [ERROR] Failed to query entity");
            DropbearNativeError::QueryFailed as i32
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_texture_name(
    asset_ptr: AssetRegistryPtr,
    texture_handle: i64,
    out_name: *mut *const c_char,
) -> i32 {
    if asset_ptr.is_null() || out_name.is_null() {
        eprintln!("[dropbear_get_texture_name] [ERROR] Null pointer received");
        return DropbearNativeError::NullPointer as i32;
    }

    let asset = &*asset_ptr;
    let handle = AssetHandle::new(texture_handle as u64);

    if let Some(material) = asset.get_material(handle) {
        match CString::new(material.name.as_str()) {
            Ok(c_string) => {
                unsafe {
                    *out_name = c_string.into_raw();
                }
                0
            }
            Err(err) => {
                eprintln!(
                    "[dropbear_get_texture_name] [ERROR] Failed to allocate string: {}",
                    err
                );
                DropbearNativeError::UnknownError as i32
            }
        }
    } else {
        eprintln!(
            "[dropbear_get_texture_name] [ERROR] Invalid texture handle {}",
            texture_handle
        );
        DropbearNativeError::EntityNotFound as i32
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_set_texture(
    world_ptr: *const World,
    asset_ptr: AssetRegistryPtr,
    entity_handle: i64,
    old_material_name: *const c_char,
    texture_id: i64,
) -> i32 {
    if world_ptr.is_null() || asset_ptr.is_null() || old_material_name.is_null() {
        eprintln!("[dropbear_set_texture] [ERROR] Null pointer received");
        return DropbearNativeError::NullPointer as i32;
    }

    let world = &*world_ptr;
    let asset = &*asset_ptr;
    let entity = world.find_entity_from_id(entity_handle as u32);
    let target_identifier = match CStr::from_ptr(old_material_name).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            eprintln!("[dropbear_set_texture] [ERROR] Invalid UTF-8 in material identifier");
            return DropbearNativeError::InvalidUTF8 as i32;
        }
    };

    match world.query_one::<&mut MeshRenderer>(entity) {
        Ok(mut query) => {
            let Some(renderer) = query.get() else {
                eprintln!(
                    "[dropbear_set_texture] [ERROR] Entity missing MeshRenderer component"
                );
                return DropbearNativeError::NoSuchComponent as i32;
            };

            let cache_ptr = match asset.get_pointer(Const("model_cache")) {
                Some(ptr) => ptr as *const Mutex<HashMap<String, Arc<Model>>>,
                None => {
                    eprintln!(
                        "[dropbear_set_texture] [ERROR] Asset registry missing model cache pointer"
                    );
                    return DropbearNativeError::UnknownError as i32;
                }
            };

            let Some(cache) = (unsafe { cache_ptr.as_ref() }) else {
                eprintln!("[dropbear_set_texture] [ERROR] Model cache pointer is null");
                return DropbearNativeError::NullPointer as i32;
            };

            let resolved_target_name = renderer
                .resolve_texture_identifier(&target_identifier)
                .map(|name| name.to_string())
                .or_else(|| {
                    let model = renderer.model();
                    let model_id = renderer.model_id();

                    if model
                        .materials
                        .iter()
                        .any(|material| material.name == target_identifier)
                    {
                        return Some(target_identifier.clone());
                    }

                    model.materials.iter().find_map(|material| {
                        if material.name == target_identifier {
                            return Some(material.name.clone());
                        }

                        let registry_reference = ASSET_REGISTRY
                            .material_handle(model_id, &material.name)
                            .and_then(|handle| ASSET_REGISTRY.material_reference_for_handle(handle))
                            .and_then(|reference| reference.as_uri().map(|uri| uri.to_string()));

                        if registry_reference
                            .as_ref()
                            .map(|value| value == &target_identifier)
                            .unwrap_or(false)
                        {
                            return Some(material.name.clone());
                        }

                        if material
                            .texture_tag
                            .as_ref()
                            .map(|tag| tag == &target_identifier)
                            .unwrap_or(false)
                        {
                            return Some(material.name.clone());
                        }

                        None
                    })
                });

            let Some(target_material) = resolved_target_name else {
                eprintln!(
                    "[dropbear_set_texture] [ERROR] Unable to resolve material '{}' on model '{}'",
                    target_identifier,
                    renderer.model().label
                );
                return DropbearNativeError::EntityNotFound as i32;
            };

            let handle = AssetHandle::new(texture_id as u64);

            if !asset.contains_handle(handle) || !asset.is_material(handle) {
                eprintln!(
                    "[dropbear_set_texture] [ERROR] Handle {} is not a valid material",
                    texture_id
                );
                return DropbearNativeError::EntityNotFound as i32;
            }

            let Some(material) = asset.get_material(handle) else {
                eprintln!("[dropbear_set_texture] [ERROR] Material handle not found");
                return DropbearNativeError::EntityNotFound as i32;
            };

            let Some(owner_model_id) = asset.material_owner(handle) else {
                eprintln!(
                    "[dropbear_set_texture] [ERROR] Unable to resolve owner model for material"
                );
                return DropbearNativeError::UnknownError as i32;
            };

            let Some(owner_model_handle) = asset.model_handle_from_id(owner_model_id) else {
                eprintln!(
                    "[dropbear_set_texture] [ERROR] Unable to resolve model handle for owner"
                );
                return DropbearNativeError::UnknownError as i32;
            };

            let Some(source_reference) = asset.model_reference_for_handle(owner_model_handle) else {
                eprintln!(
                    "[dropbear_set_texture] [ERROR] Unable to resolve model reference for owner"
                );
                return DropbearNativeError::UnknownError as i32;
            };

            match renderer.apply_material_override_raw(
                asset,
                cache,
                target_material.as_str(),
                source_reference,
                material.name.as_str(),
            ) {
                Ok(()) => 0,
                Err(err) => {
                    eprintln!(
                        "[dropbear_set_texture] [ERROR] Failed to apply material override: {}",
                        err
                    );
                    DropbearNativeError::UnknownError as i32
                }
            }
        }
        Err(err) => {
            eprintln!(
                "[dropbear_set_texture] [ERROR] Unable to query MeshRenderer: {}",
                err
            );
            DropbearNativeError::QueryFailed as i32
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_is_texture_handle(
    asset_ptr: AssetRegistryPtr,
    handle: i64,
    out_is_texture: *mut i32,
) -> i32 {
    if asset_ptr.is_null() || out_is_texture.is_null() {
        eprintln!("[dropbear_is_texture_handle] [ERROR] Null pointer received");
        return DropbearNativeError::NullPointer as i32;
    }

    let asset = &*asset_ptr;
    let handle = AssetHandle::new(handle as u64);
    unsafe {
        *out_is_texture = if asset.is_material(handle) { 1 } else { 0 };
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_is_using_texture(
    world_ptr: *const World,
    entity_handle: i64,
    texture_handle: i64,
    out_is_using: *mut i32,
) -> i32 {
    if world_ptr.is_null() || out_is_using.is_null() {
        eprintln!("[dropbear_is_using_texture] [ERROR] Null pointer received");
        return DropbearNativeError::NullPointer as i32;
    }

    let world = &*world_ptr;
    let entity = world.find_entity_from_id(entity_handle as u32);
    let handle = AssetHandle::new(texture_handle as u64);

    match world.query_one::<&MeshRenderer>(entity) {
        Ok(mut q) => {
            if let Some(renderer) = q.get() {
                unsafe {
                    *out_is_using = if renderer.contains_material_handle(handle) { 1 } else { 0 };
                }
                0
            } else {
                eprintln!(
                    "[dropbear_is_using_texture] [ERROR] Entity missing MeshRenderer component"
                );
                DropbearNativeError::NoSuchComponent as i32
            }
        }
        Err(_) => {
            eprintln!("[dropbear_is_using_texture] [ERROR] Failed to query entity");
            DropbearNativeError::QueryFailed as i32
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_asset(
    asset_ptr: AssetRegistryPtr,
    uri: *const c_char,
    out_asset_id: *mut i64,
) -> i32 {
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
        0
    } else {
        eprintln!("[dropbear_get_asset] [ERROR] Asset not found");
        DropbearNativeError::EntityNotFound as i32
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_all_textures(
    world_ptr: *const World,
    entity_handle: i64,
    out_textures: *mut *mut *const c_char,
    out_count: *mut usize,
) -> i32 {
    if world_ptr.is_null() || out_textures.is_null() || out_count.is_null() {
        eprintln!("[dropbear_get_all_textures] [ERROR] Null pointer received");
        return DropbearNativeError::NullPointer as i32;
    }

    let world = &mut *(world_ptr as *mut World);
    let entity = world.find_entity_from_id(entity_handle as u32);

    let mut query = match world.query_one::<&mut MeshRenderer>(entity) {
        Ok(query) => query,
        Err(err) => {
            eprintln!(
                "[dropbear_get_all_textures] [ERROR] Failed to query entity: {}",
                err
            );
            return DropbearNativeError::QueryFailed as i32;
        }
    };

    let Some(renderer) = query.get() else {
        eprintln!(
            "[dropbear_get_all_textures] [ERROR] Entity missing MeshRenderer component"
        );
        return DropbearNativeError::NoSuchComponent as i32;
    };

    renderer.clear_texture_identifier_cache();
    let model = renderer.model();
    let model_id = renderer.model_id();

    let mut seen = HashSet::new();
    let mut textures = Vec::new();

    for material in &model.materials {
        renderer.register_texture_identifier(material.name.clone(), material.name.clone());
        if let Some(tag) = &material.texture_tag {
            renderer.register_texture_identifier(tag.clone(), material.name.clone());
        }

        let reference = ASSET_REGISTRY
            .material_handle(model_id, &material.name)
            .and_then(|handle| ASSET_REGISTRY.material_reference_for_handle(handle))
            .and_then(|reference| reference.as_uri().map(|uri| uri.to_string()))
            .or_else(|| material.texture_tag.clone())
            .unwrap_or_else(|| material.name.clone());

        if seen.insert(reference.clone()) {
            renderer.register_texture_identifier(reference.clone(), material.name.clone());
            textures.push(reference);
        }
    }

    unsafe {
        *out_count = textures.len();
    }

    if textures.is_empty() {
        unsafe { *out_textures = ptr::null_mut(); }
        return 0;
    }

    let mut c_pointers: Vec<*const c_char> = Vec::with_capacity(textures.len());
    for value in textures {
        match CString::new(value) {
            Ok(c_string) => c_pointers.push(c_string.into_raw()),
            Err(err) => {
                eprintln!(
                    "[dropbear_get_all_textures] [ERROR] Failed to allocate string: {}",
                    err
                );
                return DropbearNativeError::UnknownError as i32;
            }
        }
    }

    let mut boxed = c_pointers.into_boxed_slice();
    unsafe {
        *out_textures = boxed.as_mut_ptr();
    }
    Box::leak(boxed);
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_children(
    world_ptr: *const World,
    entity_id: i64,
    out_children: *mut *mut i64,
    out_count: *mut usize,
) -> i32 {
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
        return 0;
    }

    let mut boxed = children.into_boxed_slice();
    unsafe {
        *out_children = boxed.as_mut_ptr();
    }
    Box::leak(boxed);
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_child_by_label(
    world_ptr: *const World,
    entity_id: i64,
    label: *const c_char,
    out_child: *mut i64,
) -> i32 {
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
                            return 0;
                        }
                    }
                }
            }
            DropbearNativeError::EntityNotFound as i32
        }
        Err(_) => DropbearNativeError::QueryFailed as i32,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_parent(
    world_ptr: *const World,
    entity_id: i64,
    out_parent: *mut i64,
) -> i32 {
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
            0
        }
        Err(_) => {
            eprintln!("[dropbear_get_parent] [WARN] Parent component not found");
            DropbearNativeError::EntityNotFound as i32
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_quit(
    command_ptr: GraphicsPtr,
) {
    if command_ptr.is_null() {
        panic!("NullPointer (-1) while quitting GraphicsCommand, better off to shoot with gun than to nicely ask...")
    }

    let graphics = unsafe { &*(command_ptr as GraphicsPtr) };

    if graphics
        .send(CommandBuffer::Quit)
        .is_err()
    {
        panic!("SendError (-7) while quitting GraphicsCommand, better off to shoot with gun than to nicely ask...")
    }
}