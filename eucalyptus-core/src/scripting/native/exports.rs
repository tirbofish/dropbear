#![allow(unsafe_op_in_unsafe_fn)]

pub mod dropbear_common;
pub mod dropbear_math;
pub mod dropbear_utils;
pub mod components;
pub mod dropbear_engine;
pub mod dropbear_input;
pub mod dropbear_scene;

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