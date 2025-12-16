use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use hecs::World;
use jni::JNIEnv;
use jni::objects::{JClass, JObject, JString};
use jni::sys::{jboolean, jlong, jobjectArray, jstring};
use parking_lot::Mutex;
use dropbear_engine::asset::{AssetHandle, AssetRegistry, ASSET_REGISTRY};
use dropbear_engine::asset::PointerKind::Const;
use dropbear_engine::entity::MeshRenderer;
use dropbear_engine::model::Model;
use crate::{convert_jlong_to_entity, convert_jstring, convert_ptr};
use crate::ptr::{AssetRegistryPtr, WorldPtr};

/**
 * Class:     `com_dropbear_ffi_components_MeshRendererNative`
 *
 * Method:    `getModel`
 *
 * Signature: `(JJ)J`
 *
 * JNIEXPORT jlong JNICALL Java_com_dropbear_ffi_components_MeshRendererNative_getModel
 * (JNIEnv *, jclass, jlong, jlong);
*/
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_MeshRendererNative_getModel(
    _env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    entity_id: jlong,
) -> jlong {
    let world = world_handle as WorldPtr;
    if world.is_null() {
        println!("[Java_com_dropbear_ffi_components_MeshRendererNative_getModel] [ERROR] World pointer is null");
        return -1;
    }

    let world = unsafe { &*world };
    let entity = convert_jlong_to_entity!(entity_id);

    if let Ok(mut q) = world.query_one::<&MeshRenderer>(entity)
        && let Some(model) = q.get()
    {
        let handle = model.asset_handle();
        handle.raw() as jlong
    } else {
        println!(
            "[Java_com_dropbear_ffi_components_MeshRendererNative_getModel] [ERROR] Unable to find entity in world"
        );
        -1
    }
}

/**
 * Class:     `com_dropbear_ffi_components_MeshRendererNative`
 *
 * Method:    `setModel`
 *
 * Signature: `(JJJJ)V`
 *
 * `JNIEXPORT void JNICALL Java_com_dropbear_ffi_components_MeshRendererNative_setModel
 * (JNIEnv *, jclass, jlong, jlong, jlong, jlong);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_MeshRendererNative_setModel(
    _env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    asset_handle: jlong,
    entity_id: jlong,
    model_handle: jlong,
) {
    let world = world_handle as WorldPtr;
    if world.is_null() {
        println!("[Java_com_dropbear_ffi_components_MeshRendererNative_setModel] [ERROR] World pointer is null");
        return;
    }

    let asset = asset_handle as AssetRegistryPtr;
    if asset.is_null() {
        println!(
            "[Java_com_dropbear_ffi_components_MeshRendererNative_setModel] [ERROR] Asset registry pointer is null"
        );
        return;
    }

    let world = unsafe { &*world };
    let asset = unsafe { &*asset };
    let entity = convert_jlong_to_entity!(entity_id);

    if let Ok(mut q) = world.query_one::<&mut MeshRenderer>(entity)
        && let Some(model) = q.get()
    {
        let asset_handle = AssetHandle::new(model_handle as u64);
        if !asset.contains_handle(asset_handle) {
            println!("[Java_com_dropbear_ffi_components_MeshRendererNative_setModel] [ERROR] Invalid model handle");
            return;
        }
        if let Err(e) = model.set_asset_handle_raw(asset, asset_handle) {
            println!(
                "[Java_com_dropbear_ffi_components_MeshRendererNative_setModel] [ERROR] Unable to set model: {}",
                e
            );
        }
    }
}

/**
 * Class:     `com_dropbear_ffi_components_MeshRendererNative`
 *
 * Method:    `isModelHandle`
 *
 * Signature: `(JJ)Z`
 *
 * `JNIEXPORT jboolean JNICALL Java_com_dropbear_ffi_components_MeshRendererNative_isModelHandle
 * (JNIEnv *, jclass, jlong, jlong);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_MeshRendererNative_isModelHandle(
    _env: JNIEnv,
    _class: JClass,
    asset_handle: jlong,
    model_id: jlong,
) -> jboolean {
    let asset = convert_ptr!(asset_handle, AssetRegistryPtr => AssetRegistry);

    asset
        .contains_handle(AssetHandle::new(model_id as u64))
        .into()
}

/**
 * Class:     `com_dropbear_ffi_components_MeshRendererNative`
 *
 * Method:    `isUsingModel`
 *
 * Signature: `(JJJ)Z`
 * `JNIEXPORT jboolean JNICALL Java_com_dropbear_ffi_components_MeshRendererNative_isUsingModel
 * (JNIEnv *, jclass, jlong, jlong, jlong);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_MeshRendererNative_isUsingModel(
    _env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    entity_id: jlong,
    model_handle: jlong,
) -> jboolean {
    let world = convert_ptr!(world_handle, WorldPtr => World);
    let entity = convert_jlong_to_entity!(entity_id);

    let handle = AssetHandle::new(model_handle as u64);
    if let Ok(mut q) = world.query_one::<&MeshRenderer>(entity)
        && let Some(model) = q.get()
    {
        if model.asset_handle() == handle {
            true.into()
        } else {
            false.into()
        }
    } else {
        println!(
            "[Java_com_dropbear_ffi_components_MeshRendererNative_isUsingModel] [ERROR] Unable to find entity in world"
        );
        false.into()
    }
}


/**
 * Class:     `com_dropbear_ffi_components_MeshRendererNative`
 *
 * Method:    `getTexture`
 *
 * Signature: `(JJJLjava/lang/String;)J`
 *
 * `JNIEXPORT jlong JNICALL Java_com_dropbear_ffi_components_MeshRendererNative_getTexture
 * (JNIEnv *, jclass, jlong, jlong, jlong, jstring);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_MeshRendererNative_getTexture(
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    asset_handle: jlong,
    entity_id: jlong,
    name: JString,
) -> jlong {
    let world = world_handle as WorldPtr;
    if world.is_null() {
        println!("[Java_com_dropbear_ffi_components_MeshRendererNative_getTexture] [ERROR] World pointer is null");
        return -1;
    }

    let world = unsafe { &*world };

    let asset = convert_ptr!(asset_handle, AssetRegistryPtr => AssetRegistry);
    let entity = convert_jlong_to_entity!(entity_id);

    if let Ok(mut q) = world.query_one::<&MeshRenderer>(entity)
        && let Some(mesh) = q.get()
    {
        let str = convert_jstring!(env, name);
        if let Some(handle) = mesh.material_handle_raw(asset, str.as_str()) {
            handle.raw() as jlong
        } else {
            println!("[Java_com_dropbear_ffi_components_MeshRendererNative_getTexture] [ERROR] Invalid texture handle");
            0
        }
    } else {
        println!(
            "[Java_com_dropbear_ffi_components_MeshRendererNative_getTexture] [ERROR] Unable to find entity in world"
        );
        0
    }
}

/**
 * Class:     `com_dropbear_ffi_components_MeshRendererNative`
 *
 * Method:    `getTextureName`
 *
 * Signature: `(JJ)Ljava/lang/String;`
 *
 * `JNIEXPORT jstring JNICALL Java_com_dropbear_ffi_components_MeshRendererNative_getTextureName
 * (JNIEnv *, jclass, jlong, jlong);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_MeshRendererNative_getTextureName(
    env: JNIEnv,
    _class: JClass,
    asset_handle: jlong,
    texture_id: jlong,
) -> jstring {
    let asset = convert_ptr!(asset_handle, AssetRegistryPtr => AssetRegistry);

    let texture_id = AssetHandle::new(texture_id as u64);
    asset.get_material(texture_id).map_or_else(
        || {
            println!(
                "[Java_com_dropbear_ffi_components_MeshRendererNative_getTextureName] [ERROR] Invalid texture handle"
            );
            return std::ptr::null_mut();
        },
        |material| {
            let Ok(str) = env.new_string(material.name.as_str()) else {
                return std::ptr::null_mut();
            };

            str.into_raw()
        },
    )
}

/**
 * Class:     `com_dropbear_ffi_components_MeshRendererNative`
 *
 * Method:    `setTexture`
 *
 * Signature: `(JJJLjava/lang/String;J)V`
 *
 *
 * `JNIEXPORT void JNICALL Java_com_dropbear_ffi_components_MeshRendererNative_setTexture
 * (JNIEnv *, jclass, jlong, jlong, jlong, jstring, jlong);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_MeshRendererNative_setTexture(
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    asset_handle: jlong,
    entity_id: jlong,
    old_material_name: JString,
    new_texture_handle: jlong,
) {
    let world = world_handle as WorldPtr;
    if world.is_null() {
        println!("[Java_com_dropbear_ffi_components_MeshRendererNative_setTexture] [ERROR] World pointer is null");
        return;
    }

    let asset = asset_handle as AssetRegistryPtr;
    if asset.is_null() {
        println!(
            "[Java_com_dropbear_ffi_components_MeshRendererNative_setTexture] [ERROR] Asset registry pointer is null"
        );
        return;
    }

    let asset = unsafe { &*asset };

    let world = unsafe { &*world };
    let entity = convert_jlong_to_entity!(entity_id);

    match world.query_one::<&mut MeshRenderer>(entity) {
        Ok(mut query) => {
            let Some(renderer) = query.get() else {
                println!(
                    "[Java_com_dropbear_ffi_components_MeshRendererNative_setTexture] [ERROR] Entity does not have a MeshRenderer component"
                );
                return;
            };

            let Some(cache) = asset.get_pointer(Const("model_cache")) else {
                println!(
                    "[Java_com_dropbear_ffi_components_MeshRendererNative_setTexture] [ERROR] Asset registry does not contain model cache"
                );
                return;
            };

            let cache = cache as *const Mutex<HashMap<String, Arc<Model>>>;
            let cache = unsafe { &*cache };

            let jni_result = env.get_string(&old_material_name);
            let target_identifier = match jni_result {
                Ok(java_string) => match java_string.to_str() {
                    Ok(rust_str) => rust_str.to_string(),
                    Err(e) => {
                        println!(
                            "[Java_com_dropbear_ffi_components_MeshRendererNative_setTexture] [ERROR] Failed to convert Java string to Rust string: {}",
                            e
                        );
                        return;
                    }
                },
                Err(e) => {
                    println!(
                        "[Java_com_dropbear_ffi_components_MeshRendererNative_setTexture] [ERROR] Failed to get string from JNI: {}",
                        e
                    );
                    return;
                }
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
                let message = format!(
                    "[Java_com_dropbear_ffi_components_MeshRendererNative_setTexture] [ERROR] Unable to resolve material '{}' on model '{}'",
                    target_identifier,
                    renderer.model().label
                );
                println!("{}", message);
                return;
            };

            let handle = AssetHandle::new(new_texture_handle as u64);

            if !asset.contains_handle(handle) {
                println!(
                    "[Java_com_dropbear_ffi_components_MeshRendererNative_setTexture] [ERROR] Invalid texture handle: {}",
                    new_texture_handle
                );
                return;
            }

            if !asset.is_material(handle) {
                println!(
                    "[Java_com_dropbear_ffi_components_MeshRendererNative_setTexture] [ERROR] Handle {} does not refer to a material",
                    new_texture_handle
                );
                return;
            }

            let Some(material) = asset.get_material(handle) else {
                println!(
                    "[Java_com_dropbear_ffi_components_MeshRendererNative_setTexture] [ERROR] Invalid texture handle"
                );
                return;
            };

            let Some(owner_model_id) = asset.material_owner(handle) else {
                println!(
                    "[Java_com_dropbear_ffi_components_MeshRendererNative_setTexture] [ERROR] Unable to determine owning model for material handle {}",
                    new_texture_handle
                );
                return;
            };

            let Some(owner_model_handle) = asset.model_handle_from_id(owner_model_id) else {
                println!(
                    "[Java_com_dropbear_ffi_components_MeshRendererNative_setTexture] [ERROR] Unable to resolve model handle for owner id {:?}",
                    owner_model_id
                );
                return;
            };

            let Some(source_reference) = asset.model_reference_for_handle(owner_model_handle)
            else {
                println!(
                    "[Java_com_dropbear_ffi_components_MeshRendererNative_setTexture] [ERROR] Unable to resolve model reference for handle {}",
                    owner_model_handle.raw()
                );
                return;
            };

            if let Err(e) = renderer.apply_material_override_raw(
                asset,
                cache,
                target_material.as_str(),
                source_reference,
                material.name.as_str(),
            ) {
                println!(
                    "[Java_com_dropbear_ffi_components_MeshRendererNative_setTexture] [ERROR] Failed to apply material override: {}",
                    e
                );
            }
        }
        Err(err) => {
            println!(
                "[Java_com_dropbear_ffi_components_MeshRendererNative_setTexture] [ERROR] Unable to query MeshRenderer: {}",
                err
            );
        }
    }
}

/**
 * Class:     `com_dropbear_ffi_components_MeshRendererNative`
 *
 * Method:    `isTextureHandle`
 *
 * Signature: `(JJ)Z`
 *
 *
 * `JNIEXPORT jboolean JNICALL Java_com_dropbear_ffi_components_MeshRendererNative_isTextureHandle
 * (JNIEnv *, jclass, jlong, jlong);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_MeshRendererNative_isTextureHandle(
    _env: JNIEnv,
    _class: JClass,
    asset_handle: jlong,
    texture_id: jlong,
) -> jboolean {
    let asset = convert_ptr!(asset_handle, AssetRegistryPtr => AssetRegistry);
    let texture_id = AssetHandle::new(texture_id as u64);
    if asset.is_material(texture_id) {
        true.into()
    } else {
        false.into()
    }
}

/**
 * Class:     `com_dropbear_ffi_components_MeshRendererNative`
 *
 * Method:    `isUsingTexture`
 *
 * Signature: `(JJJ)Z`
 *
 * `JNIEXPORT jboolean JNICALL Java_com_dropbear_ffi_components_MeshRendererNative_isUsingTexture
 * (JNIEnv *, jclass, jlong, jlong, jlong);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_MeshRendererNative_isUsingTexture(
    _env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    entity_id: jlong,
    texture_handle: jlong,
) -> jboolean {
    let world = convert_ptr!(world_handle, WorldPtr => World);
    let entity = convert_jlong_to_entity!(entity_id);

    if let Ok(mut q) = world.query_one::<&MeshRenderer>(entity)
        && let Some(mesh) = q.get()
    {
        mesh.contains_material_handle(AssetHandle::new(texture_handle as u64))
            .into()
    } else {
        println!(
            "[Java_com_dropbear_ffi_components_MeshRendererNative_isUsingTexture] [ERROR] Unable to find entity in world"
        );
        false.into()
    }
}

/**
 * Class:     `com_dropbear_ffi_components_MeshRendererNative`
 * 
 * Method:    `getAllTextures`
 * 
 * Signature: `(JJ)[Ljava/lang/String;`
 *
 * `JNIEXPORT jobjectArray JNICALL Java_com_dropbear_ffi_components_MeshRendererNative_getAllTextures
 * (JNIEnv *, jclass, jlong, jlong);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_MeshRendererNative_getAllTextures(
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    entity_id: jlong,
) -> jobjectArray {
    let world = convert_ptr!(world_handle, WorldPtr => World);
    let entity = convert_jlong_to_entity!(entity_id);

    let mut query = match world.query_one::<&mut MeshRenderer>(entity) {
        Ok(query) => query,
        Err(e) => {
            let message = format!(
                "[Java_com_dropbear_ffi_components_MeshRendererNative_getAllTextures] [ERROR] Failed to query entity: {}",
                e
            );
            println!("{}", message);
            return std::ptr::null_mut();
        }
    };

    let renderer = match query.get() {
        Some(renderer) => renderer,
        None => {
            let message = "[Java_com_dropbear_ffi_components_MeshRendererNative_getAllTextures] [ERROR] Entity does not have a MeshRenderer component";
            println!("{}", message);
            return std::ptr::null_mut();
        }
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

    let string_class = match env.find_class("java/lang/String") {
        Ok(class) => class,
        Err(e) => {
            let message = format!(
                "[Java_com_dropbear_ffi_components_MeshRendererNative_getAllTextures] [ERROR] Failed to locate java/lang/String: {}",
                e
            );
            println!("{}", message);
            return std::ptr::null_mut();
        }
    };

    let array = match env.new_object_array(textures.len() as i32, string_class, JObject::null()) {
        Ok(array) => array,
        Err(e) => {
            let message = format!(
                "[Java_com_dropbear_ffi_components_MeshRendererNative_getAllTextures] [ERROR] Failed to allocate string array: {}",
                e
            );
            println!("{}", message);
            return std::ptr::null_mut();
        }
    };

    for (index, value) in textures.iter().enumerate() {
        let java_string = match env.new_string(value) {
            Ok(string) => string,
            Err(e) => {
                let message = format!(
                    "[Java_com_dropbear_ffi_components_MeshRendererNative_getAllTextures] [ERROR] Failed to create Java string: {}",
                    e
                );
                println!("{}", message);
                return std::ptr::null_mut();
            }
        };

        if let Err(e) =
            env.set_object_array_element(&array, index as i32, JObject::from(java_string))
        {
            let message = format!(
                "[Java_com_dropbear_ffi_components_MeshRendererNative_getAllTextures] [ERROR] Failed to set array element: {}",
                e
            );
            println!("{}", message);
            return std::ptr::null_mut();
        }
    }

    array.into_raw()
}