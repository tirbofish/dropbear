use crate::ptr::{AssetRegistryPtr, AssetRegistryUnwrapped};
use crate::scripting::native::DropbearNativeError;
use crate::scripting::result::DropbearNativeResult;
use dropbear_engine::asset::Handle;

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.asset.TextureNative", func = "getLabel"),
    c(name = "dropbear_asset_texture_get_label")
)]
fn get_texture_label(
    #[dropbear_macro::define(AssetRegistryPtr)] asset_manager: &AssetRegistryUnwrapped,
    texture_handle: u64,
) -> DropbearNativeResult<Option<String>> {
    Ok(asset_manager
        .read()
        .get_label_from_texture_handle(Handle::new(texture_handle)))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.asset.TextureNative", func = "getWidth"),
    c(name = "dropbear_asset_texture_get_width")
)]
fn get_texture_width(
    #[dropbear_macro::define(AssetRegistryPtr)] asset_manager: &AssetRegistryUnwrapped,
    texture_handle: u64,
) -> DropbearNativeResult<u32> {
    asset_manager
        .read()
        .get_texture(Handle::new(texture_handle))
        .map(|v| v.size.width)
        .ok_or(DropbearNativeError::AssetNotFound)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.asset.TextureNative", func = "getHeight"),
    c(name = "dropbear_asset_texture_get_height")
)]
fn get_texture_height(
    #[dropbear_macro::define(AssetRegistryPtr)] asset_manager: &AssetRegistryUnwrapped,
    texture_handle: u64,
) -> DropbearNativeResult<u32> {
    asset_manager
        .read()
        .get_texture(Handle::new(texture_handle))
        .map(|v| v.size.height)
        .ok_or(DropbearNativeError::AssetNotFound)
}
