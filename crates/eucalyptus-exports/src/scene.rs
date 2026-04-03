use eucalyptus_core::ptr::{CommandBufferPtr, CommandBufferUnwrapped, SceneLoaderPtr, SceneLoaderUnwrapped};
use eucalyptus_core::scene::scripting::shared;
use eucalyptus_core::scripting::result::DropbearNativeResult;
use eucalyptus_core::utils::Progress;
use crate::ToJObject;
use jni::objects::{JObject, JValue};
use jni::{Env, jni_sig, jni_str};
use eucalyptus_core::scripting::native::DropbearNativeError;

impl ToJObject for Progress {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .load_class(jni_str!("com/dropbear/utils/Progress"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let message_obj = env
            .new_string(&self.message)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        let args = [
            JValue::Double(self.current as f64),
            JValue::Double(self.total as f64),
            JValue::Object(&JObject::from(message_obj)),
        ];

        env.new_object(&class, jni_sig!((double, double, java.lang.String) -> void), &args)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.scene.SceneManagerNative", func = "loadSceneAsync"),
    c
)]
fn load_scene_async(
    #[dropbear_macro::define(CommandBufferPtr)] command_buffer: &CommandBufferUnwrapped,
    #[dropbear_macro::define(SceneLoaderPtr)] scene_loader: &SceneLoaderUnwrapped,
    scene_name: String,
) -> DropbearNativeResult<u64> {
    Ok(shared::load_scene_async(command_buffer, scene_loader, scene_name, None)?)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.scene.SceneManagerNative", func = "loadSceneAsyncWithLoading"),
    c
)]
fn load_scene_async_with_loading(
    #[dropbear_macro::define(CommandBufferPtr)] command_buffer: &CommandBufferUnwrapped,
    #[dropbear_macro::define(SceneLoaderPtr)] scene_loader: &SceneLoaderUnwrapped,
    scene_name: String,
    loading_scene: String,
) -> DropbearNativeResult<u64> {
    Ok(shared::load_scene_async(command_buffer, scene_loader, scene_name, Some(loading_scene))?)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.scene.SceneManagerNative", func = "switchToSceneImmediate"),
    c
)]
fn switch_to_scene_immediate(
    #[dropbear_macro::define(CommandBufferPtr)] command_buffer: &CommandBufferUnwrapped,
    scene_name: String,
) -> DropbearNativeResult<()> {
    Ok(shared::switch_to_scene_immediate(command_buffer, scene_name)?)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.scene.SceneLoadHandleNative",
        func = "getSceneLoadHandleSceneName"
    ),
    c
)]
fn get_scene_load_handle_scene_name(
    #[dropbear_macro::define(SceneLoaderPtr)] scene_loader: &SceneLoaderUnwrapped,
    scene_id: u64,
) -> DropbearNativeResult<String> {
    Ok(shared::get_scene_load_handle_scene_name(scene_loader, scene_id)?)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.scene.SceneLoadHandleNative", func = "switchToSceneAsync"),
    c
)]
fn switch_to_scene_async(
    #[dropbear_macro::define(CommandBufferPtr)] command_buffer: &CommandBufferUnwrapped,
    #[dropbear_macro::define(SceneLoaderPtr)] scene_loader: &SceneLoaderUnwrapped,
    scene_id: u64,
) -> DropbearNativeResult<()> {
    Ok(shared::switch_to_scene_async(command_buffer, scene_loader, scene_id)?)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.scene.SceneLoadHandleNative", func = "getSceneLoadProgress"),
    c
)]
fn get_scene_load_progress(
    #[dropbear_macro::define(SceneLoaderPtr)] scene_loader: &SceneLoaderUnwrapped,
    scene_id: u64,
) -> DropbearNativeResult<Progress> {
    Ok(shared::get_scene_load_progress(scene_loader, scene_id)?)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.scene.SceneLoadHandleNative", func = "getSceneLoadStatus"),
    c
)]
fn get_scene_load_status(
    #[dropbear_macro::define(SceneLoaderPtr)] scene_loader: &SceneLoaderUnwrapped,
    scene_id: u64,
) -> DropbearNativeResult<u32> {
    Ok(shared::get_scene_load_status(scene_loader, scene_id)?)
}
