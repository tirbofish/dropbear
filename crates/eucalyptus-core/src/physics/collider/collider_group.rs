//! Scripting module for collider groups.

use crate::physics::PhysicsState;
use crate::physics::collider::ColliderGroup;
use crate::ptr::WorldPtr;
use crate::scripting::native::DropbearNativeError;
use crate::scripting::result::DropbearNativeResult;
use crate::types::{IndexNative, NCollider};

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.ColliderGroupNative",
        func = "colliderGroupExistsForEntity"
    ),
    c
)]
fn exists_for_entity(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<bool> {
    Ok(world.get::<&ColliderGroup>(entity).is_ok())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.ColliderGroupNative",
        func = "getColliderGroupColliders"
    ),
    c
)]
fn get_colliders(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::define(crate::ptr::PhysicsStatePtr)] physics: &PhysicsState,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<Vec<NCollider>> {
    if world.get::<&ColliderGroup>(entity).is_ok() {
        let handles_opt = physics
            .entity_label_map
            .get(&entity)
            .and_then(|label| physics.colliders_entity_map.get(label));

        let mut colliders: Vec<NCollider> = Vec::new();

        if let Some(handles) = handles_opt {
            for (_, handle) in handles {
                let (idx, generation) = handle.into_raw_parts();

                let col = NCollider {
                    index: IndexNative {
                        index: idx,
                        generation,
                    },
                    entity_id: entity.to_bits().get(),
                    id: idx,
                };
                colliders.push(col);
            }
        }

        Ok(colliders)
    } else {
        Err(DropbearNativeError::MissingComponent)?
    }
}
