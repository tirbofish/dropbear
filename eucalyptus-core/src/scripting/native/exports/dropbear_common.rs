use crate::ptr::{AssetRegistryPtr, CommandBufferPtr, InputStatePtr, PhysicsStatePtr, SceneLoaderPtr, WorldPtr};

/// The return code for a function.
///
/// Follows the same code as [`DropbearNativeError`]
pub type DropbearNativeReturn = i32;

/// The handle/id of an object, as a long.
///
/// Kotlin/Native requires this to be an `int64_t` or a Long.
pub type Handle = i64;

/// A helper type that defines a value that can either be a 0 or 1.
pub type Bool = i32;

/// Describes all the different pointers that can be passed into a scripting
/// module.
#[repr(C)]
pub struct DropbearContext {
    pub world: WorldPtr,
    pub input: InputStatePtr,
    pub graphics: CommandBufferPtr,
    pub assets: AssetRegistryPtr,
    pub scene_loader: SceneLoaderPtr,
    pub physics_state: PhysicsStatePtr,
}