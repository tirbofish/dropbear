//! Helper pointers and typedef definitions.

use crate::command::CommandBuffer;
use crate::input::InputState;
use crate::physics::PhysicsState;
use crate::scene::loading::SceneLoader;
use crossbeam_channel::Sender;
use dropbear_engine::asset::AssetRegistry;
use dropbear_engine::graphics::SharedGraphicsContext;
use hecs::World;
use parking_lot::{Mutex, RwLock};
use std::ffi::c_void;
use std::sync::Arc;

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
pub type CommandBufferUnwrapped = Sender<CommandBuffer>;

/// A non-mutable pointer to the shared graphics context.
///
/// Defined in `dropbear_common.h` as `GraphicsContext`.
pub type GraphicsContextPtr = *const SharedGraphicsContext;

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

/// A mutable pointer to the UI command buffer/state.
///
/// This is treated as an opaque pointer by scripting layers.
pub type UiBufferPtr = *mut c_void;
