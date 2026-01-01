pub mod shared {
    use crate::command::CommandBuffer;
    use crate::scripting::native::DropbearNativeError;
    use crate::scripting::result::DropbearNativeResult;
    use crate::states::Label;
    use dropbear_engine::asset::AssetRegistry;
    use dropbear_engine::utils::ResourceReference;
    use hecs::World;
    use std::ffi::CStr;
    use std::os::raw::c_char;

    pub fn get_entity(world: &World, label: &str) -> DropbearNativeResult<u64> {
        for (id, entity_label) in world.query::<&Label>().iter() {
            if entity_label.as_str() == label {
                return Ok(id.to_bits().get());
            }
        }
        Err(DropbearNativeError::EntityNotFound)
    }

    pub fn get_asset(registry: &AssetRegistry, uri: &str) -> DropbearNativeResult<u64> {
        let reference = ResourceReference::from_euca_uri(uri)
            .map_err(|_| DropbearNativeError::InvalidURI)?;

        match registry.get_handle_from_reference(&reference) {
            Some(handle) => Ok(handle.raw()),
            None => Err(DropbearNativeError::AssetNotFound),
        }
    }

    pub fn quit(command_buffer: &crossbeam_channel::Sender<CommandBuffer>) -> DropbearNativeResult<()> {
        command_buffer.send(CommandBuffer::Quit)
            .map_err(|_| DropbearNativeError::SendError)
    }

    pub unsafe fn read_str(ptr: *const c_char) -> DropbearNativeResult<String> {
        if ptr.is_null() { return Err(DropbearNativeError::NullPointer); }
        unsafe { CStr::from_ptr(ptr) }.to_str()
            .map(|s| s.to_string())
            .map_err(|_| DropbearNativeError::InvalidUTF8)
    }
}

pub mod jni {
    #![allow(non_snake_case)]
    use hecs::World;
    use jni::objects::{JClass, JString};
    use jni::sys::{jlong, jobject};
    use jni::JNIEnv;
    use dropbear_engine::asset::AssetRegistry;
    use crate::command::CommandBuffer;
    use crate::return_boxed;

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_DropbearEngineNative_getEntity(
        mut env: JNIEnv,
        _class: JClass,
        world_handle: jlong,
        label: JString,
    ) -> jobject {
        let world = crate::convert_ptr!(world_handle => World);
        let label_str = crate::convert_jstring!(env, label);

        let value_opt = super::shared::get_entity(&world, &label_str)
            .ok()
            .map(|id| id as i64);

        return_boxed!(
            &mut env,
            value_opt,
            "(J)Ljava/lang/Long;",
            "java/lang/Long"
        )
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_DropbearEngineNative_getAsset(
        mut env: JNIEnv,
        _class: JClass,
        asset_handle: jlong,
        label: JString,
    ) -> jobject {
        let asset = crate::convert_ptr!(asset_handle => AssetRegistry);
        let label_str = crate::convert_jstring!(env, label);

        let value_opt = super::shared::get_asset(&asset, &label_str)
            .ok()
            .map(|id| id as i64);

        return_boxed!(
            &mut env,
            value_opt,
            "(J)Ljava/lang/Long;",
            "java/lang/Long"
        )
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_DropbearEngineNative_quit(
        mut env: JNIEnv,
        _class: JClass,
        command_buffer_ptr: jlong,
    ) {
        let sender = crate::convert_ptr!(command_buffer_ptr => crossbeam_channel::Sender<CommandBuffer>);

        if let Err(e) = super::shared::quit(sender) {
            let _ = env.throw_new(
                "java/lang/RuntimeException",
                format!("Failed to send quit command: {:?}", e)
            );
        }
    }
}

#[dropbear_macro::impl_c_api]
pub mod native {
    use crate::command::CommandBuffer;
    use crate::convert_ptr;
    use crate::engine::shared::read_str;
    use crate::ptr::{AssetRegistryPtr, CommandBufferPtr, WorldPtr};
    use crate::scripting::result::DropbearNativeResult;
    use dropbear_engine::asset::AssetRegistry;
    use hecs::World;
    use std::os::raw::c_char;

    pub fn dropbear_get_entity(
        world_ptr: WorldPtr,
        label: *const c_char,
    ) -> DropbearNativeResult<u64> {
        let world = convert_ptr!(world_ptr => World);
        let label_str = unsafe { read_str(label)? };

        super::shared::get_entity(world, &label_str)
    }

    pub fn dropbear_get_asset(
        asset_ptr: AssetRegistryPtr,
        uri: *const c_char,
    ) -> DropbearNativeResult<u64> {
        let asset_registry = convert_ptr!(asset_ptr => AssetRegistry);
        let uri_str = unsafe { read_str(uri)? };
        super::shared::get_asset(&asset_registry, &uri_str)
    }

    pub fn dropbear_quit(
        command_ptr: CommandBufferPtr,
    ) -> DropbearNativeResult<()> {
        let sender = convert_ptr!(command_ptr => crossbeam_channel::Sender<CommandBuffer>);
        super::shared::quit(sender)
    }
}