pub mod texture;
pub mod model;

use dropbear_engine::asset::AssetKind;
use crate::ptr::{AssetRegistryPtr, AssetRegistryUnwrapped};
use crate::scripting::result::DropbearNativeResult;

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.DropbearEngineNative", func = "getAsset"),
    c(name = "dropbear_engine_get_asset")
)]
fn dropbear_asset_get_asset(
    #[dropbear_macro::define(AssetRegistryPtr)]
    asset: &AssetRegistryUnwrapped,
    label: String,
    kind: &AssetKind,
) -> DropbearNativeResult<Option<u64>> {
    let reader = asset.read();
    match kind {
        AssetKind::Texture => {
            let result = reader.get_texture_handle_from_label(&label);
            if let Some(handle) = result {
                Ok(Some(handle.id))
            } else {
                Ok(None)
            }
        }
        AssetKind::Model => {
            let result = reader.get_model_handle_from_label(&label);
            if let Some(handle) = result {
                Ok(Some(handle.id))
            } else {
                Ok(None)
            }
        }
    }
}