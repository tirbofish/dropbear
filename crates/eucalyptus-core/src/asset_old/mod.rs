pub mod texture;
pub mod model;

use dropbear_engine::asset::AssetKind;
use crate::pointer_convert;
use crate::ptr::AssetRegistryUnwrapped;
use crate::scripting::result::DropbearNativeResult;

/**
 * Fetches the asset_old information from the internal AssetRegistry (located in
 * `dropbear_engine::asset_old::AssetRegistry`) from the provided label.
 */
#[uniffi::export]
pub fn dropbear_asset_get_asset(
    asset_ptr: u64,
    label: String,
    kind: AssetKind,
) -> DropbearNativeResult<Option<u64>> {
    let asset = pointer_convert!(asset_ptr => AssetRegistryUnwrapped);
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
