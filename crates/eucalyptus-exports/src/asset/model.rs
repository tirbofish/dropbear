use dropbear_engine::asset::Handle;
use eucalyptus_core::ptr::{AssetRegistryPtr, AssetRegistryUnwrapped};
use eucalyptus_core::scripting::native::DropbearNativeError;
use eucalyptus_core::scripting::result::DropbearNativeResult;
use crate::asset::model::ty::{NAnimation, NMaterial, NMesh, NNode, NSkin};

mod ty;

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.asset.ModelNative", func = "getLabel"),
    c(name = "dropbear_asset_model_get_label")
)]
fn dropbear_asset_model_get_label(
    #[dropbear_macro::define(AssetRegistryPtr)] asset: &AssetRegistryUnwrapped,
    model_handle: u64,
) -> DropbearNativeResult<String> {
    let label = asset
        .read()
        .get_label_from_model_handle(Handle::new(model_handle))
        .ok_or_else(|| DropbearNativeError::InvalidHandle)?;
    Ok(label)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.asset.ModelNative", func = "getMeshes"),
    c(name = "dropbear_asset_model_get_meshes")
)]
fn dropbear_asset_model_get_meshes(
    #[dropbear_macro::define(AssetRegistryPtr)] asset: &AssetRegistryUnwrapped,
    model_handle: u64,
) -> DropbearNativeResult<Vec<NMesh>> {
    let reader = asset.read();
    let model = reader
        .get_model(Handle::new(model_handle))
        .ok_or(DropbearNativeError::InvalidHandle)?;

    Ok(model.meshes.iter().map(|v| v.into()).collect())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.asset.ModelNative", func = "getMaterials"),
    c(name = "dropbear_asset_model_get_materials")
)]
fn dropbear_asset_model_get_materials(
    #[dropbear_macro::define(AssetRegistryPtr)] asset: &AssetRegistryUnwrapped,
    model_handle: u64,
) -> DropbearNativeResult<Vec<NMaterial>> {
    let reader = asset.read();
    let model = reader
        .get_model(Handle::new(model_handle))
        .ok_or(DropbearNativeError::InvalidHandle)?;

    Ok(model
        .materials
        .iter()
        .map(|v| v.clone().into())
        .collect())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.asset.ModelNative", func = "getSkins"),
    c(name = "dropbear_asset_model_get_skins")
)]
pub fn dropbear_asset_model_get_skins(
    #[dropbear_macro::define(AssetRegistryPtr)] asset: &AssetRegistryUnwrapped,
    model_handle: u64,
) -> DropbearNativeResult<Vec<NSkin>> {
    let reader = asset.read();
    let model = reader
        .get_model(Handle::new(model_handle))
        .ok_or(DropbearNativeError::InvalidHandle)?;

    Ok(model.skins.iter().map(|v| v.clone().into()).collect())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.asset.ModelNative", func = "getAnimations"),
    c(name = "dropbear_asset_model_get_animations")
)]
pub fn dropbear_asset_model_get_animations(
    #[dropbear_macro::define(AssetRegistryPtr)] asset: &AssetRegistryUnwrapped,
    model_handle: u64,
) -> DropbearNativeResult<Vec<NAnimation>> {
    let reader = asset.read();
    let model = reader
        .get_model(Handle::new(model_handle))
        .ok_or(DropbearNativeError::InvalidHandle)?;

    Ok(model.animations.iter().map(|v| v.clone().into()).collect())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.asset.ModelNative", func = "getNodes"),
    c(name = "dropbear_asset_model_get_nodes")
)]
pub fn dropbear_asset_model_get_nodes(
    #[dropbear_macro::define(AssetRegistryPtr)] asset: &AssetRegistryUnwrapped,
    model_handle: u64,
) -> DropbearNativeResult<Vec<NNode>> {
    let reader = asset.read();
    let model = reader
        .get_model(Handle::new(model_handle))
        .ok_or(DropbearNativeError::InvalidHandle)?;

    Ok(model.nodes.iter().map(|v| v.clone().into()).collect())
}
