pub mod texture;
pub mod model;

use jni::JNIEnv;
use jni::objects::JObject;
use dropbear_engine::asset::AssetKind;
use crate::ptr::{AssetRegistryPtr, AssetRegistryUnwrapped};
use crate::scripting::jni::utils::{FromJObject};
use crate::scripting::native::DropbearNativeError;
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

impl FromJObject for AssetKind {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self>
    where
        Self: Sized
    {
        let ordinal = env
            .call_method(obj, "ordinal", "()I", &[])?
            .i()?;

        match ordinal {
            0 => Ok(AssetKind::Texture),
            1 => Ok(AssetKind::Model),
            _ => Err(DropbearNativeError::InvalidEnumOrdinal)
        }
    }
}