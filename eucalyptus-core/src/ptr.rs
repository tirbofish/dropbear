//! Helper pointers and typedef definitions.
use crate::input::InputState;
use crate::command::CommandBuffer;
use crossbeam_channel::Sender;
use dropbear_engine::asset::AssetRegistry;
use hecs::World;
use parking_lot::Mutex;
use crate::scene::loading::SceneLoader;

/// A mutable pointer to a [`World`].
///
/// Defined in `dropbear_common.h` as `World`
pub type WorldPtr = *mut World;

/// A mutable pointer to an [`InputState`].
///
/// Defined in `dropbear_common.h` as `InputState`
pub type InputStatePtr = *mut InputState;

/// A non-mutable pointer to a [`crossbeam_channel::Sender`] that sends
/// [`CommandBuffer`] signals.
///
/// Defined in `dropbear_common.h` as `CommandBuffer`
pub type CommandBufferPtr = *const Sender<CommandBuffer>;

/// A non-mutable pointer to the [`AssetRegistry`].
///
/// Defined in `dropbear_common.h` as `AssetRegistry`
pub type AssetRegistryPtr = *const AssetRegistry;

/// A mutable pointer to a [`parking_lot::Mutex<SceneLoader>`]
///
/// Defined in `dropbear_common.h` as `SceneLoader`
///
/// # Safety
/// Despite there being issues about Mutexes not being ABI safe, this is
/// provided to the scripting module as an OpaquePointer.
pub type SceneLoaderPtr = *const Mutex<SceneLoader>;