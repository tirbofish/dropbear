//! Helper pointers and typedef definitions.

use std::sync::Arc;
use crate::input::InputState;
use crate::command::CommandBuffer;
use crossbeam_channel::Sender;
use dropbear_engine::asset::AssetRegistry;
use hecs::World;
use parking_lot::{Mutex, RwLock};
use crate::physics::PhysicsState;
use crate::scene::loading::SceneLoader;
use crate::ui::{UiContext};

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
pub type AssetRegistryPtr = *const AssetRegistryUnwrapped;
pub type AssetRegistryUnwrapped = Arc<RwLock<AssetRegistry>>;

/// A mutable pointer to a [`parking_lot::Mutex<SceneLoader>`]
///
/// Defined in `dropbear_common.h` as `SceneLoader`
///
/// # Safety
/// Despite there being issues about Mutexes not being ABI safe, this is
/// provided to the scripting module as an OpaquePointer.
pub type SceneLoaderPtr = *const SceneLoaderUnwrapped;
pub type SceneLoaderUnwrapped = Mutex<SceneLoader>;

/// A mutable pointer to a [`PhysicsState`].
///
/// Defined in `dropbear_common.h` as `PhysicsEngine`
pub type PhysicsStatePtr = *mut PhysicsState;

/// A mutable pointer to a [`UiContext`], used for queueing UI components
/// in the scripting module. 
/// 
/// Defined in `dropbear_common.h` as `UiBufferPtr`. 
pub type UiBufferPtr = *const UiContext;