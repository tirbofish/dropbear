use eucalyptus_core::scripting::result::DropbearNativeResult;
use eucalyptus_core::ptr::WorldPtr;
use hecs::{Entity, World};
use crate::component::SurfaceNets;


#[dropbear_macro::export(
    kotlin(class = "io.github.tirbofish.surfaceNets.SurfaceNetsNative", func = "surfaceNetsExistsForEntity"),
    c(name = "surface_nets_plugin_exists_for_entity")
)]
pub fn exists_for_entity(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
) -> DropbearNativeResult<bool> {
    Ok(world.get::<&SurfaceNets>(entity).is_ok())
}
