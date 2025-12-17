use std::collections::{HashMap, HashSet};
use std::ffi::{c_char, CStr, CString};
use std::ptr;
use std::sync::{Arc};
use hecs::World;
use parking_lot::Mutex;
use dropbear_engine::asset::AssetHandle;
use dropbear_engine::asset::PointerKind::Const;
use dropbear_engine::entity::MeshRenderer;
use dropbear_engine::model::Model;
use crate::ptr::AssetRegistryPtr;
use crate::scripting::native::DropbearNativeError;
use crate::scripting::native::exports::dropbear_common::{Bool, DropbearNativeReturn, Handle};

/// Fetches the model that is currently being used by the entity being queried. 
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_model(
    world_ptr: *const World,
    asset_ptr: AssetRegistryPtr,
    entity_handle: Handle,
    out_model_id: *mut Handle,
) -> DropbearNativeReturn {
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
                DropbearNativeError::Success as i32
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

/// Sets the MeshRenderer's model as that of the provided asset in the form of a ModelHandle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_set_model(
    world_ptr: *const World,
    asset_ptr: AssetRegistryPtr,
    entity_handle: Handle,
    model_id: Handle,
) -> DropbearNativeReturn {
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
                    DropbearNativeError::Success as i32
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

/// Checks if a handle provided is that of a Model. Returns false if not or true if is.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_is_model_handle(
    asset_ptr: AssetRegistryPtr,
    handle: Handle,
    out_is_model: *mut Bool,
) -> DropbearNativeReturn {
    if asset_ptr.is_null() || out_is_model.is_null() {
        eprintln!("[dropbear_is_model_handle] [ERROR] Null pointer received");
        return DropbearNativeError::NullPointer as i32;
    }

    let asset = &*asset_ptr;
    let handle = AssetHandle::new(handle as u64);
    unsafe {
        *out_is_model = if asset.is_model(handle) { 1 } else { 0 };
    }
    DropbearNativeError::Success as i32
}

/// Checks if the entity is currently using a specific asset (in the form of a ModelHandle) to render
/// its meshes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_is_using_model(
    world_ptr: *const World,
    entity_handle: Handle,
    model_handle: Handle,
    out_is_using: *mut Bool,
) -> DropbearNativeReturn {
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
                DropbearNativeError::Success as i32
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

/// Fetches the texture (in the form of a TextureHandle) from a specific entity that is currently
/// rendering the model. 
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_texture(
    world_ptr: *const World,
    asset_ptr: AssetRegistryPtr,
    entity_handle: Handle,
    name: *const c_char,
    out_texture_id: *mut Handle,
) -> DropbearNativeReturn {
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
                    DropbearNativeError::Success as i32
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

/// Fetches the texture name. 
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_texture_name(
    asset_ptr: AssetRegistryPtr,
    texture_handle: Handle,
    out_name: *mut *const c_char,
) -> DropbearNativeReturn {
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
                DropbearNativeError::Success as i32
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

/// Replaces the texture of a specific texture of the model currently being rendered on an entity. 
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_set_texture(
    world_ptr: *const World,
    asset_ptr: AssetRegistryPtr,
    entity_handle: Handle,
    old_material_name: *const c_char,
    texture_id: Handle,
) -> DropbearNativeReturn {
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

                        let registry_reference = asset
                            .material_handle(model_id, &material.name)
                            .and_then(|handle| asset.material_reference_for_handle(handle))
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
                Ok(()) => DropbearNativeError::Success as i32,
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

/// Checks if a general AssetHandle is linked to a texture, therefore making it a TextureHandle. 
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_is_texture_handle(
    asset_ptr: AssetRegistryPtr,
    handle: Handle,
    out_is_texture: *mut Bool,
) -> DropbearNativeReturn {
    if asset_ptr.is_null() || out_is_texture.is_null() {
        eprintln!("[dropbear_is_texture_handle] [ERROR] Null pointer received");
        return DropbearNativeError::NullPointer as i32;
    }

    let asset = &*asset_ptr;
    let handle = AssetHandle::new(handle as u64);
    unsafe {
        *out_is_texture = if asset.is_material(handle) { 1 } else { 0 };
    }
    DropbearNativeError::Success as i32
}

/// Checks if an entity is currently using a texture as specified by the TextureId. 
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_is_using_texture(
    world_ptr: *const World,
    entity_handle: Handle,
    texture_handle: Handle,
    out_is_using: *mut Bool,
) -> DropbearNativeReturn {
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
                DropbearNativeError::Success as i32
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


// todo: change from returning texture names to return TextureHandle
/// Fetches all the textures currently being rendered by an entity
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_all_textures(
    world_ptr: *const World,
    asset_registry_ptr: AssetRegistryPtr,
    entity_handle: Handle,
    out_textures: *mut *mut *const c_char,
    out_count: *mut usize,
) -> DropbearNativeReturn {
    if world_ptr.is_null() || out_textures.is_null() || out_count.is_null() || asset_registry_ptr.is_null() {
        eprintln!("[dropbear_get_all_textures] [ERROR] Null pointer received");
        return DropbearNativeError::NullPointer as i32;
    }

    let world = &mut *(world_ptr as *mut World);
    let asset = &*asset_registry_ptr;
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

        let reference = asset
            .material_handle(model_id, &material.name)
            .and_then(|handle| asset.material_reference_for_handle(handle))
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
        return DropbearNativeError::Success as i32;
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
    DropbearNativeError::Success as i32
}