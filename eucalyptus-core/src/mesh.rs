pub mod shared {
    use dropbear_engine::entity::MeshRenderer;
    use hecs::{Entity, World};

    pub fn mesh_renderer_exists_for_entity(world: &World, entity: Entity) -> bool {
        world.get::<&MeshRenderer>(entity).is_ok()
    }

    pub fn resolve_target_material_name(
        renderer: &MeshRenderer,
        target_identifier: &str
    ) -> Option<String> {
        if let Some(cached) = renderer.resolve_texture_identifier(target_identifier) {
            return Some(cached.to_string());
        }

        let model = renderer.model();

        model.materials.iter().find_map(|mat| {
            if mat.name == target_identifier { return Some(mat.name.clone()); }
            if let Some(tag) = &mat.texture_tag { if tag == target_identifier { return Some(mat.name.clone()); } }
            None
        })
    }
}

pub mod jni {
    #![allow(non_snake_case)]

    use crate::return_boxed;
    use dropbear_engine::asset::PointerKind::Const;
    use dropbear_engine::asset::{AssetHandle, AssetRegistry};
    use dropbear_engine::entity::MeshRenderer;
    use dropbear_engine::model::Model;
    use hecs::World;
    use jni::objects::{JClass, JString};
    use jni::sys::{jboolean, jlong, jlongArray, jobject, jsize};
    use jni::JNIEnv;
    use parking_lot::Mutex;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_components_MeshRendererNative_meshRendererExistsForEntity(
        _env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
    ) -> jboolean {
        let world = crate::convert_ptr!(world_ptr => World);
        let entity = crate::convert_jlong_to_entity!(entity_id);
        super::shared::mesh_renderer_exists_for_entity(&world, entity) as jboolean
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_components_MeshRendererNative_getModel(
        mut env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
    ) -> jlong {
        let world = crate::convert_ptr!(world_ptr => World);
        let entity = crate::convert_jlong_to_entity!(entity_id);

        if let Ok(mesh) = world.get::<&MeshRenderer>(entity) {
            mesh.model_id().raw() as jlong
        } else {
            let _ = env.throw_new("java/lang/RuntimeException", "Entity does not have a MeshRenderer component");
            -1
        }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_components_MeshRendererNative_setModel(
        mut env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        asset_ptr: jlong,
        entity_id: jlong,
        model_id: jlong,
    ) {
        let world = crate::convert_ptr!(world_ptr => World);
        let asset = crate::convert_ptr!(asset_ptr => AssetRegistry);
        let entity = crate::convert_jlong_to_entity!(entity_id);

        if let Ok(mut mesh) = world.get::<&mut MeshRenderer>(entity) {
            match mesh.set_asset_handle_raw(asset, AssetHandle::new(model_id as u32)) {
                Ok(_) => {}
                Err(e) => {
                    let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to set model: {}", e));
                }
            }
        } else {
            let _ = env.throw_new("java/lang/RuntimeException", "Entity does not have a MeshRenderer component");
        }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_components_MeshRendererNative_getAllTextureIds(
        mut env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        asset_ptr: jlong,
        entity_id: jlong,
    ) -> jlongArray {
        let world = crate::convert_ptr!(world_ptr => World);
        let entity = crate::convert_jlong_to_entity!(entity_id);
        let asset = crate::convert_ptr!(asset_ptr => AssetRegistry);

        if let Ok(mut renderer) = world.get::<&mut MeshRenderer>(entity) {
            let handles = renderer.collect_all_material_handles_raw(&asset);
            let jarray = match env.new_long_array(handles.len() as jsize) {
                Ok(val) => val,
                Err(_) => {
                    let _ = env.throw_new("java/lang/OutOfMemoryError", "Could not allocate texture ID array");
                    return std::ptr::null_mut();
                }
            };

            if let Err(e) = env.set_long_array_region(&jarray, 0, handles.iter().map(|v| v.raw() as jlong).collect::<Vec<_>>().as_slice()) {
                let _ = env.throw_new("java/lang/RuntimeException", format!("{:?}", e));
                return std::ptr::null_mut();
            }

            jarray.into_raw()
        } else {
            let _ = env.throw_new("java/lang/RuntimeException", "Entity does not have a MeshRenderer component");
            std::ptr::null_mut()
        }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_components_MeshRendererNative_getTexture(
        mut env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        asset_ptr: jlong,
        entity_id: jlong,
        material_name: JString,
    ) -> jobject {
        let world = crate::convert_ptr!(world_ptr => hecs::World);
        let asset = crate::convert_ptr!(asset_ptr => dropbear_engine::asset::AssetRegistry);
        let entity = crate::convert_jlong_to_entity!(entity_id);
        let name = crate::convert_jstring!(env, material_name);

        let handle_value: Option<i64> = if let Ok(renderer) = world.get::<&dropbear_engine::entity::MeshRenderer>(entity) {
            renderer.material_handle_raw(asset, &name)
                .map(|h| h.raw() as i64)
        } else {
            None
        };

        return_boxed!(&mut env, handle_value, "(J)Ljava/lang/Long;", "java/lang/Long")
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_components_MeshRendererNative_setTextureOverride(
        mut env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        asset_handle: jlong,
        entity_id: jlong,
        material_name: JString,
        texture_handle: jlong,
    ) {
        let world = crate::convert_ptr!(world_ptr => hecs::World);
        let asset = crate::convert_ptr!(asset_handle => dropbear_engine::asset::AssetRegistry);
        let entity = crate::convert_jlong_to_entity!(entity_id);
        let target_identifier = crate::convert_jstring!(env, material_name);
        let new_texture_handle = dropbear_engine::asset::AssetHandle::new(texture_handle as u64);

        if let Ok(mut mesh) = world.get::<&mut dropbear_engine::entity::MeshRenderer>(entity) {
            let resolved_target = super::shared::resolve_target_material_name(&mesh, &target_identifier);

            let Some(target_material_name) = resolved_target else {
                let _ = env.throw_new(
                    "java/lang/IllegalArgumentException",
                    format!(
                        "Could not resolve material identifier '{}' on model '{}'",
                        target_identifier,
                        mesh.model().label
                    ),
                );
                return;
            };

            if !asset.contains_handle(new_texture_handle) {
                let _ = env.throw_new("java/lang/IllegalArgumentException", "Invalid texture handle provided");
                return;
            }

            if !asset.is_material(new_texture_handle) {
                let _ = env.throw_new("java/lang/IllegalArgumentException", "Handle provided is not a material/texture");
                return;
            }

            let Some(source_material) = asset.get_material(new_texture_handle) else { return; };

            let Some(owner_model_id) = asset.material_owner(new_texture_handle) else {
                let _ = env.throw_new("java/lang/IllegalStateException", "Texture handle has no owner model in registry");
                return;
            };

            let Some(owner_model_handle) = asset.model_handle_from_id(owner_model_id) else {
                let _ = env.throw_new("java/lang/IllegalStateException", "Texture owner model not found in registry");
                return;
            };

            let Some(source_reference) = asset.model_reference_for_handle(owner_model_handle) else {
                let _ = env.throw_new("java/lang/IllegalStateException", "Texture owner model has no resource reference");
                return;
            };

            let Some(model_cache) = asset.get_pointer(Const("model_cache")) else {
                let _ = env.throw_new("java/lang/IllegalArgumentException", "Unable to locate model_cache pointer within the asset registry");
                return;
            };

            let model_cache = model_cache as *const Mutex<HashMap<String, Arc<Model>>>;
            if model_cache.is_null() {
                let _ = env.throw_new("java/lang/IllegalStateException", "Model cache cannot be null");
                return;
            }

            let model_cache = unsafe { &*model_cache };

            if let Err(e) = mesh.apply_material_override_raw(
                asset,
                model_cache,
                &target_material_name,
                source_reference,
                &source_material.name,
            ) {
                let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to apply texture override: {}", e));
            }

        } else {
            let _ = env.throw_new("java/lang/RuntimeException", "Entity does not have a MeshRenderer component");
        }
    }
}

#[dropbear_macro::impl_c_api]
pub mod native {
    use parking_lot::Mutex;
    use std::collections::HashMap;
    use std::os::raw::c_char;
    use std::sync::Arc;

    use hecs::{Entity, World};

    use dropbear_engine::asset::{AssetHandle, AssetRegistry, PointerKind};
    use dropbear_engine::entity::MeshRenderer;
    use dropbear_engine::model::Model;

    use crate::convert_ptr;
    use crate::engine::shared::read_str;
    use crate::ptr::{AssetRegistryPtr, WorldPtr};
    use crate::scripting::native::DropbearNativeError;
    use crate::scripting::result::DropbearNativeResult;

    pub fn dropbear_mesh_renderer_exists_for_entity(
        world_ptr: WorldPtr,
        entity_id: u64,
    ) -> DropbearNativeResult<bool> {
        let world = convert_ptr!(world_ptr => World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;
        DropbearNativeResult::Ok(super::shared::mesh_renderer_exists_for_entity(world, entity))
    }
    
    pub fn dropbear_get_model(
        world_ptr: WorldPtr,
        entity_id: u64,
    ) -> DropbearNativeResult<u64> {
        let world = convert_ptr!(world_ptr => World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(mesh) = world.get::<&MeshRenderer>(entity) {
            DropbearNativeResult::Ok(mesh.model_id().raw())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }
    
    pub fn dropbear_set_model(
        world_ptr: WorldPtr,
        asset_ptr: AssetRegistryPtr,
        entity_id: u64,
        model_id: u64,
    ) -> DropbearNativeResult<()> {
        let world = convert_ptr!(world_ptr => World);
        let asset = convert_ptr!(asset_ptr => AssetRegistry);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(mut mesh) = world.get::<&mut MeshRenderer>(entity) {
            match mesh.set_asset_handle_raw(asset, AssetHandle::new(model_id)) {
                Ok(_) => DropbearNativeResult::Ok(()),
                Err(_) => DropbearNativeResult::Err(DropbearNativeError::UnknownError),
            }
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }
    
    pub fn dropbear_get_all_texture_ids(
        world_ptr: WorldPtr,
        asset_ptr: AssetRegistryPtr,
        entity_id: u64,
        out_count: *mut usize,
    ) -> DropbearNativeResult<*mut u64> {
        let world = convert_ptr!(world_ptr => World);
        let asset = convert_ptr!(asset_ptr => AssetRegistry);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if out_count.is_null() {
            return DropbearNativeResult::Err(DropbearNativeError::NullPointer);
        }

        if let Ok(renderer) = world.get::<&MeshRenderer>(entity) {
            let handles = renderer.collect_all_material_handles_raw(asset);

            let mut raw_ids: Vec<u64> = handles.iter().map(|h| h.raw()).collect();

            unsafe { *out_count = raw_ids.len(); }

            let ptr = raw_ids.as_mut_ptr();
            std::mem::forget(raw_ids);

            DropbearNativeResult::Ok(ptr)
        } else {
            unsafe { *out_count = 0; }
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }
    
    pub fn dropbear_get_texture(
        world_ptr: WorldPtr,
        asset_ptr: AssetRegistryPtr,
        entity_id: u64,
        material_name: *const c_char,
    ) -> DropbearNativeResult<u64> {
        let world = convert_ptr!(world_ptr => World);
        let asset = convert_ptr!(asset_ptr => AssetRegistry);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;
        let target = unsafe { read_str(material_name)? };

        if let Ok(renderer) = world.get::<&MeshRenderer>(entity) {
            match renderer.material_handle_raw(asset, &target) {
                Some(handle) => DropbearNativeResult::Ok(handle.raw()),
                None => DropbearNativeResult::Err(DropbearNativeError::UnknownError),
            }
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }
    
    pub fn dropbear_set_texture_override(
        world_ptr: WorldPtr,
        asset_ptr: AssetRegistryPtr,
        entity_id: u64,
        material_name: *const c_char,
        texture_handle: u64,
    ) -> DropbearNativeResult<()> {
        let world = convert_ptr!(world_ptr => World);
        let asset = convert_ptr!(asset_ptr => AssetRegistry);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;
        let target_identifier = unsafe { read_str(material_name)? };
        let new_texture_handle = AssetHandle::new(texture_handle);

        if let Ok(mut mesh) = world.get::<&mut MeshRenderer>(entity) {
            let resolved_target = super::shared::resolve_target_material_name(&mesh, &target_identifier);
            let target_material_name = match resolved_target {
                Some(name) => name,
                None => return DropbearNativeResult::Err(DropbearNativeError::InvalidArgument),
            };

            if !asset.contains_handle(new_texture_handle) || !asset.is_material(new_texture_handle) {
                return DropbearNativeResult::Err(DropbearNativeError::InvalidArgument);
            }

            let source_material = match asset.get_material(new_texture_handle) {
                Some(m) => m,
                None => return DropbearNativeResult::Err(DropbearNativeError::UnknownError),
            };

            let owner_model_id = match asset.material_owner(new_texture_handle) {
                Some(id) => id,
                None => return DropbearNativeResult::Err(DropbearNativeError::QueryFailed),
            };

            let owner_model_handle = match asset.model_handle_from_id(owner_model_id) {
                Some(h) => h,
                None => return DropbearNativeResult::Err(DropbearNativeError::QueryFailed),
            };

            let source_reference = match asset.model_reference_for_handle(owner_model_handle) {
                Some(r) => r,
                None => return DropbearNativeResult::Err(DropbearNativeError::QueryFailed),
            };

            let model_cache_ptr = match asset.get_pointer(PointerKind::Const("model_cache")) {
                Some(ptr) => ptr as *const Mutex<HashMap<String, Arc<Model>>>,
                None => return DropbearNativeResult::Err(DropbearNativeError::NullPointer), // Cache not found
            };

            if model_cache_ptr.is_null() {
                return DropbearNativeResult::Err(DropbearNativeError::NullPointer);
            }

            let model_cache = unsafe { &*model_cache_ptr };

            match mesh.apply_material_override_raw(
                asset,
                model_cache,
                &target_material_name,
                source_reference,
                &source_material.name,
            ) {
                Ok(_) => DropbearNativeResult::Ok(()),
                Err(_) => DropbearNativeResult::Err(DropbearNativeError::WorldInsertError),
            }

        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }
}