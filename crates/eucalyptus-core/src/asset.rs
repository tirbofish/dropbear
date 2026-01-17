pub mod texture;

pub mod shared {
    use dropbear_engine::asset::{AssetHandle, AssetRegistry};
    use crate::scripting::result::DropbearNativeResult;

    pub fn is_model_handle(
        registry: &AssetRegistry,
        handle: u64,
    ) -> DropbearNativeResult<bool> {
        let handle = AssetHandle::new(handle);
        let result = registry.is_model(handle);
        Ok(result)
    }

    pub fn is_texture_handle(
        registry: &AssetRegistry,
        handle: u64,
    ) -> DropbearNativeResult<bool> {
        let handle = AssetHandle::new(handle);
        let result = registry.is_material(handle);
        Ok(result)
    }
}

pub mod jni {
    #![allow(non_snake_case)]
    use jni::sys::{jboolean, jlong};
    use jni::objects::JClass;
    use dropbear_engine::asset::AssetRegistry;
    use jni::JNIEnv;
    use crate::asset::shared::{is_model_handle, is_texture_handle};
    use crate::convert_ptr;

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_asset_AssetHandleNative_isModelHandle(
        _env: JNIEnv,
        _class: JClass,
        asset_registry_ptr: jlong,
        handle: jlong,
    ) -> jboolean {
        let asset = convert_ptr!(asset_registry_ptr => AssetRegistry);
        let result = is_model_handle(asset, handle as u64);
        match result {
            Ok(val) => val as jboolean,
            Err(e) => {
                crate::ffi_error_return!("[ERROR] {}", e)
            }
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_asset_AssetHandleNative_isTextureHandle(
        _env: JNIEnv,
        _class: JClass,
        asset_registry_ptr: jlong,
        handle: jlong,
    ) -> jboolean {
        let asset = convert_ptr!(asset_registry_ptr => AssetRegistry);
        let result = is_texture_handle(asset, handle as u64);
        match result {
            Ok(val) => val as jboolean,
            Err(e) => {
                crate::ffi_error_return!("[ERROR] {}", e)
            }
        }
    }
}

#[dropbear_macro::impl_c_api]
pub mod native {
    use dropbear_engine::asset::AssetRegistry;
    use crate::asset::shared::{is_model_handle, is_texture_handle};
    use crate::convert_ptr;
    use crate::ptr::AssetRegistryPtr;
    use crate::scripting::result::DropbearNativeResult;

    pub fn dropbear_is_model_handle(
        asset_registry_ptr: AssetRegistryPtr,
        handle: u64,
    ) -> DropbearNativeResult<bool> {
        let asset = convert_ptr!(asset_registry_ptr => AssetRegistry);
        is_model_handle(asset, handle)
    }

    pub fn dropbear_is_texture_handle(
        asset_registry_ptr: AssetRegistryPtr,
        handle: u64,
    ) -> DropbearNativeResult<bool> {
        let asset = convert_ptr!(asset_registry_ptr => AssetRegistry);
        is_texture_handle(asset, handle)
    }
}