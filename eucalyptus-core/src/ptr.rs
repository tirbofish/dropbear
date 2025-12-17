use crate::input::InputState;
use crate::window::CommandBuffer;
use crossbeam_channel::Sender;
use dropbear_engine::asset::AssetRegistry;
use hecs::World;

pub type WorldPtr = *mut World;
pub type InputStatePtr = *mut InputState;
pub type CommandBufferPtr = *const Sender<CommandBuffer>;
pub type AssetRegistryPtr = *const AssetRegistry;
