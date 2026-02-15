pub mod shared {
    use dropbear_engine::asset::AssetRegistry;
    use dropbear_engine::entity::MeshRenderer;
    use dropbear_engine::model::Model;
    use hecs::{Entity, World};

    pub fn mesh_renderer_exists_for_entity(world: &World, entity: Entity) -> bool {
        world.get::<&MeshRenderer>(entity).is_ok()
    }

    fn matches_material_label(material: &dropbear_engine::model::Material, target: &str) -> bool {
        material.texture_tag.as_deref() == Some(target) || material.name == target
    }

    pub fn resolve_target_material_index(
        model: &Model,
        target_identifier: &str,
    ) -> Option<usize> {
        model
            .materials
            .iter()
            .position(|mat| matches_material_label(mat, target_identifier))
    }

    pub fn resolve_target_material_name(
        model: &Model,
        target_identifier: &str,
    ) -> Option<String> {
        model
            .materials
            .iter()
            .find(|mat| matches_material_label(mat, target_identifier))
            .map(|mat| mat.name.clone())
    }

    pub fn model_for_renderer<'a>(
        registry: &'a AssetRegistry,
        renderer: &MeshRenderer,
    ) -> Option<&'a Model> {
        registry.get_model(renderer.model())
    }
}

use crate::ptr::{AssetRegistryPtr, AssetRegistryUnwrapped, GraphicsContextPtr, WorldPtr};
use crate::scripting::native::DropbearNativeError;
use crate::scripting::result::DropbearNativeResult;
use dropbear_engine::asset::Handle;
use dropbear_engine::entity::MeshRenderer;
use dropbear_engine::graphics::SharedGraphicsContext;
use dropbear_engine::texture::Texture;
use std::collections::HashSet;

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.MeshRendererNative", func = "meshRendererExistsForEntity")
)]
fn mesh_renderer_exists_for_entity(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::entity]
    entity: hecs::Entity,
) -> DropbearNativeResult<bool> {
    Ok(shared::mesh_renderer_exists_for_entity(world, entity))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.MeshRendererNative", func = "getModel")
)]
fn get_model(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::entity]
    entity: hecs::Entity,
) -> DropbearNativeResult<u64> {
    if let Ok(mesh) = world.get::<&MeshRenderer>(entity) {
        Ok(mesh.model().id)
    } else {
        Err(DropbearNativeError::NoSuchComponent)
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.MeshRendererNative", func = "setModel")
)]
fn set_model(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::define(AssetRegistryPtr)]
    asset: &AssetRegistryUnwrapped,
    #[dropbear_macro::entity]
    entity: hecs::Entity,
    model_handle: u64,
) -> DropbearNativeResult<()> {
    let handle = Handle::new(model_handle);
    if asset.read().get_model(handle).is_none() {
        return Err(DropbearNativeError::InvalidHandle);
    }

    if let Ok(mut mesh) = world.get::<&mut MeshRenderer>(entity) {
        mesh.set_model(handle);
        Ok(())
    } else {
        Err(DropbearNativeError::NoSuchComponent)
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.MeshRendererNative", func = "getAllTextureIds")
)]
fn get_all_texture_ids(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::define(AssetRegistryPtr)]
    asset: &AssetRegistryUnwrapped,
    #[dropbear_macro::entity]
    entity: hecs::Entity,
) -> DropbearNativeResult<Vec<u64>> {
    let reader = asset.read();
    let renderer = world
        .get::<&MeshRenderer>(entity)
        .map_err(|_| DropbearNativeError::NoSuchComponent)?;
    let model = shared::model_for_renderer(&reader, &renderer)
        .ok_or(DropbearNativeError::AssetNotFound)?;

    let mut ids = HashSet::new();
    let mut push_handle = |texture: &Texture| {
        if let Some(hash) = texture.hash {
            if let Some(handle) = reader.texture_handle_by_hash(hash) {
                ids.insert(handle.id);
            }
        }
    };

    for material in &model.materials {
        push_handle(&material.diffuse_texture);
        push_handle(&material.normal_texture);
        if let Some(tex) = &material.emissive_texture {
            push_handle(tex);
        }
        if let Some(tex) = &material.metallic_roughness_texture {
            push_handle(tex);
        }
        if let Some(tex) = &material.occlusion_texture {
            push_handle(tex);
        }
    }

    Ok(ids.into_iter().collect())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.MeshRendererNative", func = "getTexture"),
    c
)]
fn get_texture(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::define(AssetRegistryPtr)]
    asset: &AssetRegistryUnwrapped,
    #[dropbear_macro::entity]
    entity: hecs::Entity,
    material_name: String,
) -> DropbearNativeResult<Option<u64>> {
    let reader = asset.read();
    let renderer = world
        .get::<&MeshRenderer>(entity)
        .map_err(|_| DropbearNativeError::NoSuchComponent)?;
    let model = shared::model_for_renderer(&reader, &renderer)
        .ok_or(DropbearNativeError::AssetNotFound)?;
    let idx = match shared::resolve_target_material_index(model, &material_name) {
        Some(value) => value,
        None => return Ok(None),
    };
    let material = model
        .materials
        .get(idx)
        .ok_or(DropbearNativeError::InvalidArgument)?;

    Ok(material
        .diffuse_texture
        .hash
        .and_then(|hash| reader.texture_handle_by_hash(hash))
        .map(|handle| handle.id))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.MeshRendererNative", func = "setTextureOverride"),
    c
)]
fn set_texture_override(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::define(AssetRegistryPtr)]
    asset: &AssetRegistryUnwrapped,
    #[dropbear_macro::entity]
    entity: hecs::Entity,
    material_name: String,
    texture_handle: u64,
) -> DropbearNativeResult<()> {
    let reader = asset.read();
    let renderer = world
        .get::<&MeshRenderer>(entity)
        .map_err(|_| DropbearNativeError::NoSuchComponent)?;
    let model = shared::model_for_renderer(&reader, &renderer)
        .ok_or(DropbearNativeError::AssetNotFound)?;
    let _ = shared::resolve_target_material_name(model, &material_name)
        .ok_or(DropbearNativeError::InvalidArgument)?;

    let handle = Handle::<dropbear_engine::texture::Texture>::new(texture_handle);
    if reader.get_texture(handle).is_none() {
        return Err(DropbearNativeError::InvalidHandle);
    }

    drop(reader);

    let mut renderer = world
        .get::<&mut MeshRenderer>(entity)
        .map_err(|_| DropbearNativeError::NoSuchComponent)?;
    renderer.set_texture_override(handle);
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.MeshRendererNative", func = "setMaterialTint"),
    c
)]
fn set_material_tint(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::define(AssetRegistryPtr)]
    asset: &AssetRegistryUnwrapped,
    #[dropbear_macro::define(GraphicsContextPtr)]
    graphics: &SharedGraphicsContext,
    #[dropbear_macro::entity]
    entity: hecs::Entity,
    material_name: String,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) -> DropbearNativeResult<()> {
    let renderer = world
        .get::<&MeshRenderer>(entity)
        .map_err(|_| DropbearNativeError::NoSuchComponent)?;
    let handle = renderer.model();

    let mut registry = asset.write();
    let model = registry
        .get_model(handle)
        .cloned()
        .ok_or(DropbearNativeError::AssetNotFound)?;
    let mut model = model;

    let index = shared::resolve_target_material_index(&model, &material_name)
        .ok_or(DropbearNativeError::InvalidArgument)?;

    if let Some(material) = model.materials.get_mut(index) {
        material.tint = [r, g, b, a];
        material.sync_uniform(graphics);
    }

    registry.update_model(handle, model);
    Ok(())
}