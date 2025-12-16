use hecs::World;
use jni::JNIEnv;
use jni::sys::{jclass, jlong, jstring};
use crate::{convert_jlong_to_entity, ffi_error_return};
use crate::states::Label;

/**
 * Class:     `com_dropbear_ffi_components_LabelNative`
 *
 * Method:    `getEntityLabel`
 *
 * Signature: `(JJ)Ljava/lang/String;`
 *
 * `JNIEXPORT jstring JNICALL Java_com_dropbear_ffi_components_LabelNative_getEntityLabel
 * (JNIEnv *, jclass, jlong, jlong);`
*/
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_LabelNative_getEntityLabel(
    env: JNIEnv,
    _class: jclass,
    world_handle: jlong,
    entity_id: jlong,
) -> jstring {
    let world = world_handle as *mut World;

    if world.is_null() {
        return ffi_error_return!("[Java_com_dropbear_ffi_JNINative_getTransform] [ERROR] World pointer is null");
    }

    let world = unsafe { &mut *world };

    let entity = convert_jlong_to_entity!(entity_id);

    if let Ok(mut q) = world.query_one::<&Label>(entity) && let Some(label) = q.get() {
        let label_str = label.as_str();
        let Ok(str) = env.new_string(label_str) else {
            return ffi_error_return!("[Java_com_dropbear_ffi_JNINative_getTransform] [ERROR] Unable to create new string from label");
        };
        return str.into_raw();
    }

    ffi_error_return!("[Java_com_dropbear_ffi_JNINative_getTransform] [ERROR] Unable to locate Label for player, likely engine bug")
}