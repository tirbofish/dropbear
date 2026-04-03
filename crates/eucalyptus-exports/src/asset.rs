pub mod model;
pub mod texture;

use jni::{jni_sig, jni_str, Env};
use jni::objects::JObject;
use dropbear_engine::asset::AssetKind;
use eucalyptus_core::ptr::{AssetRegistryPtr, AssetRegistryUnwrapped};
use eucalyptus_core::scripting::native::DropbearNativeError;
use eucalyptus_core::scripting::result::DropbearNativeResult;
use crate::FromJObject;

impl FromJObject for AssetKind {
    fn from_jobject(env: &mut Env, obj: &JObject) -> DropbearNativeResult<Self>
    where
        Self: Sized,
    {
        let ordinal = env
            .call_method(obj, jni_str!("ordinal"), jni_sig!(() -> i32), &[])?
            .i()?;

        match ordinal {
            0 => Ok(AssetKind::Texture),
            1 => Ok(AssetKind::Model),
            _ => Err(DropbearNativeError::InvalidEnumOrdinal),
        }
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.DropbearEngineNative", func = "getAsset"),
    c(name = "dropbear_engine_get_asset")
)]
fn dropbear_asset_get_asset(
    #[dropbear_macro::define(AssetRegistryPtr)] asset: &AssetRegistryUnwrapped,
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

