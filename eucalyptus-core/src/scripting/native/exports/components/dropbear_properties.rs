use std::ffi::{c_char, CStr};
use hecs::World;
use dropbear_engine::entity::MeshRenderer;
use crate::scripting::native::DropbearNativeError;
use crate::scripting::native::exports::dropbear_common::{Bool, DropbearNativeReturn, Handle};
use crate::states::{ModelProperties, Value};

/// Fetches the string property
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_string_property(
    world_ptr: *const World,
    entity_handle: Handle,
    label: *const c_char,
    out_value: *mut c_char,
    out_value_max_length: i32,
) -> DropbearNativeReturn {
    if world_ptr.is_null() || label.is_null() || out_value.is_null() {
        eprintln!("[dropbear_get_string_property] [ERROR] Null pointer received");
        return DropbearNativeError::NullPointer as i32;
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

/// Fetches the integer property of an entity.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_int_property(
    world_ptr: *const World,
    entity_handle: Handle,
    label: *const c_char,
    out_value: *mut i32,
) -> DropbearNativeReturn {
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

/// Fetches the long/[`f64`] property of an entity.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_long_property(
    world_ptr: *const World,
    entity_handle: Handle,
    label: *const c_char,
    out_value: *mut Handle,
) -> DropbearNativeReturn {
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

/// Fetches the double/[`f64`] (idk why its called float but cbb to change) value of an entity.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_float_property(
    world_ptr: *const World,
    entity_handle: Handle,
    label: *const c_char,
    out_value: *mut f64,
) -> DropbearNativeReturn {
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

/// Fetches the boolean property of an entity.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_bool_property(
    world_ptr: *const World,
    entity_handle: Handle,
    label: *const c_char,
    out_value: *mut Bool,
) -> DropbearNativeReturn {
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

/// Fetches the Vector3<float> property of an entity
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_vec3_property(
    world_ptr: *const World,
    entity_handle: Handle,
    label: *const c_char,
    out_x: *mut f32,
    out_y: *mut f32,
    out_z: *mut f32,
) -> DropbearNativeReturn {
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

/// Sets the string property of an entity.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_set_string_property(
    world_ptr: *const World,
    entity_handle: Handle,
    label: *const c_char,
    value: *const c_char,
) -> DropbearNativeReturn {
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

/// Sets the integer property of an entity.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_set_int_property(
    world_ptr: *const World,
    entity_handle: Handle,
    label: *const c_char,
    value: i32,
) -> DropbearNativeReturn {
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
            props.set_property(label_str, Value::Int(value as Handle));
            0
        }
        Err(_) => -2,
    }
}

/// Sets the long property of an entity.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_set_long_property(
    world_ptr: *const World,
    entity_handle: Handle,
    label: *const c_char,
    value: Handle,
) -> DropbearNativeReturn {
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

/// Sets the float property of an entity.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_set_float_property(
    world_ptr: *const World,
    entity_handle: Handle,
    label: *const c_char,
    value: f32,
) -> DropbearNativeReturn {
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

/// Sets the boolean property of an entity.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_set_bool_property(
    world_ptr: *const World,
    entity_handle: Handle,
    label: *const c_char,
    value: Bool,
) -> DropbearNativeReturn {
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

/// Sets the Vector3<float> property of an entity.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_set_vec3_property(
    world_ptr: *const World,
    entity_handle: Handle,
    label: *const c_char,
    x: f32,
    y: f32,
    z: f32,
) -> DropbearNativeReturn {
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