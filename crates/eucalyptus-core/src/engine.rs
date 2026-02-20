use crate::ptr::{CommandBufferPtr, CommandBufferUnwrapped, WorldPtr};
use crate::scripting::result::DropbearNativeResult;

pub mod shared {
    use crate::command::CommandBuffer;
    use crate::scripting::native::DropbearNativeError;
    use crate::scripting::result::DropbearNativeResult;
    use crate::states::Label;
    use hecs::{Entity, World};

    pub fn get_entity(world: &World, label: &str) -> DropbearNativeResult<u64> {
        for (id, entity_label) in world.query::<(Entity, &Label)>().iter() {
            if entity_label.as_str() == label {
                return Ok(id.to_bits().get());
            }
        }
        Err(DropbearNativeError::EntityNotFound)
    }

    pub fn quit(
        command_buffer: &crossbeam_channel::Sender<CommandBuffer>,
    ) -> DropbearNativeResult<()> {
        command_buffer
            .send(CommandBuffer::Quit)
            .map_err(|_| DropbearNativeError::SendError)
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.DropbearEngineNative", func = "getEntity",),
    c
)]
fn get_entity(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    label: String,
) -> DropbearNativeResult<u64> {
    shared::get_entity(&world, &label)
}

#[dropbear_macro::export(kotlin(class = "com.dropbear.DropbearEngineNative", func = "quit",), c)]
fn quit(
    #[dropbear_macro::define(CommandBufferPtr)] command_buffer: &CommandBufferUnwrapped,
) -> DropbearNativeResult<()> {
    shared::quit(command_buffer)
}
