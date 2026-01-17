pub mod shared {
    use dropbear_engine::asset::{AssetHandle, AssetRegistry};
    use crate::scripting::native::DropbearNativeError;
    use crate::scripting::result::DropbearNativeResult;

    pub fn get_texture_name(
        asset_registry: &AssetRegistry,
        handle: u64,
    ) -> DropbearNativeResult<String> {
        let texture = asset_registry
            .get_material(AssetHandle::new(handle))
            .ok_or_else(|| DropbearNativeError::NoSuchHandle)?;

        Ok(texture.name.clone())
    }
}

pub mod jni {
    #![allow(non_snake_case)]
    use jni::JNIEnv;
    use jni::sys::{jlong, jstring};
    use dropbear_engine::asset::AssetRegistry;
    use jni::objects::JClass;
    use crate::asset::texture::shared::get_texture_name;
    use crate::scripting::native::DropbearNativeError;

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_asset_TextureHandleNative_getTextureName(
        env: JNIEnv,
        _class: JClass,
        asset_registry_ptr: jlong,
        handle: jlong,
    ) -> jstring {
        let asset_registry = crate::convert_ptr!(asset_registry_ptr => AssetRegistry);
        let result = get_texture_name(asset_registry, handle as u64);
        match result {
            Ok(name) => {
                let output = env.new_string(name).map_err(|_| DropbearNativeError::JNIFailedToCreateObject);
                match output {
                    Ok(jstr) => jstr.into_raw(),
                    Err(e) => {
                        crate::ffi_error_return!("[ERROR] Failed to create Java string: {}", e)
                    }
                }
            }
            Err(e) => {
                crate::ffi_error_return!("[ERROR] {}", e)
            }
        }
    }
}

#[dropbear_macro::impl_c_api]
pub mod native {
    use std::ffi::CString;
    use crate::ptr::AssetRegistryPtr;
    use crate::scripting::native::DropbearNativeError;
    use crate::scripting::result::DropbearNativeResult;
    use std::ffi::c_char;
    use dropbear_engine::asset::AssetRegistry;
    use crate::asset::texture::shared::get_texture_name;

    pub fn dropbear_get_texture_name(
        asset_registry_ptr: AssetRegistryPtr,
        handle: u64,
    ) -> DropbearNativeResult<*mut c_char> {
        let asset_registry = crate::convert_ptr!(asset_registry_ptr => AssetRegistry);

        let result = get_texture_name(asset_registry, handle)
            .map(|name| {
                match CString::new(name) {
                    Ok(c_str) => {
                        Ok(c_str.into_raw())
                    },
                    Err(_) => {
                        Err(DropbearNativeError::CStringError)
                    }
                }
            })?;
        
        result
    }
}