#![allow(non_snake_case)]

use hecs::World;
use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::jlong;
use dropbear_engine::utils::ResourceReference;
use crate::ptr::AssetRegistryPtr;
use crate::states::Label;


/**
 * Class:     `com_dropbear_ffi_JNINative`
 *
 * Method:    `getEntity`
 *
 * Signature: `(JLjava/lang/String;)J`
 *
 * `JNIEXPORT jlong JNICALL Java_com_dropbear_ffi_JNINative_getEntity
 * (JNIEnv *, jclass, jlong, jstring);`
*/
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_JNINative_getEntity(
    mut env: JNIEnv,
    _obj: JClass,
    world_handle: jlong,
    label: JString,
) -> jlong {
    let label_jni_result = env.get_string(&label);
    let label_str = match label_jni_result {
        Ok(java_string) => match java_string.to_str() {
            Ok(rust_str) => rust_str.to_string(),
            Err(e) => {
                println!(
                    "[Java_com_dropbear_ffi_JNINative_getEntity] [ERROR] Failed to convert Java string to Rust string: {}",
                    e
                );
                return -1;
            }
        },
        Err(e) => {
            println!(
                "[Java_com_dropbear_ffi_JNINative_getEntity] [ERROR] Failed to get string from JNI: {}",
                e
            );
            return -1;
        }
    };

    let world = world_handle as *mut World;

    if world.is_null() {
        println!("[Java_com_dropbear_ffi_JNINative_getEntity] [ERROR] World pointer is null");
        return -1;
    }

    let world = unsafe { &mut *world };

    for (id, entity_label) in world.query::<&Label>().iter() {
        if entity_label.as_str() == label_str {
            return id.to_bits().get() as jlong;
        }
    }
    0
}

/**
 * Class:     `com_dropbear_ffi_JNINative`
 *
 * Method:    `getAsset`
 *
 * Signature: `(JLjava/lang/String;)J`
 *
 * `JNIEXPORT jlong JNICALL Java_com_dropbear_ffi_JNINative_getAsset
 * (JNIEnv *, jclass, jlong, jstring);`
*/
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_JNINative_getAsset(
    mut env: JNIEnv,
    _class: JClass,
    asset_handle: jlong,
    euca_uri: JString,
) -> jlong {
    let asset = asset_handle as AssetRegistryPtr;
    if asset.is_null() {
        println!(
            "[Java_com_dropbear_ffi_JNINative_getAsset] [ERROR] Asset registry pointer is null"
        );
        return -1;
    }

    let asset = unsafe { &*asset };

    let jni_result = env.get_string(&euca_uri);
    let str = match jni_result {
        Ok(java_string) => match java_string.to_str() {
            Ok(rust_str) => rust_str.to_string(),
            Err(e) => {
                println!(
                    "[Java_com_dropbear_ffi_JNINative_getAsset] [ERROR] Failed to convert Java string to Rust string: {}",
                    e
                );
                return -1;
            }
        },
        Err(e) => {
            println!(
                "[Java_com_dropbear_ffi_JNINative_getAsset] [ERROR] Failed to get string from JNI: {}",
                e
            );
            return -1;
        }
    };
    if let Ok(res) = ResourceReference::from_euca_uri(str)
        && let Some(asset_handle) = asset.get_handle_from_reference(&res)
    {
        return asset_handle.raw() as jlong;
    };
    0 as jlong
}