use eucalyptus_core::physics::kcc::{CharacterMovementResult, KCC};
use eucalyptus_core::physics::PhysicsState;
use eucalyptus_core::ptr::WorldPtr;
use eucalyptus_core::third_party::rapier3d;
use eucalyptus_core::third_party::rapier3d::dynamics::RigidBodyType;
use eucalyptus_core::third_party::rapier3d::math::Rotation;
use eucalyptus_core::third_party::rapier3d::pipeline::QueryFilter;
use eucalyptus_core::scripting::native::DropbearNativeError;
use eucalyptus_core::scripting::result::DropbearNativeResult;
use eucalyptus_core::states::Label;
use crate::math::{NTransform, NVector3, NQuaternion};
use crate::physics::{IndexNative, NCollider, NShapeCastStatus, CharacterCollisionArray};

pub mod shared {
    use glam::{DQuat, DVec3};
    use hecs::{Entity, World};
    use eucalyptus_core::physics::collider::ColliderGroup;
    use eucalyptus_core::physics::kcc::KCC;
    use eucalyptus_core::third_party::rapier3d;
    use eucalyptus_core::third_party::rapier3d::control::CharacterCollision;
    use eucalyptus_core::third_party::rapier3d::na::Quaternion;
    use eucalyptus_core::scripting::native::DropbearNativeError;
    use crate::math::{NTransform, NVector3};
    use crate::physics::{IndexNative, NCollider, NShapeCastStatus};
    use super::*;

    fn get_collision_from_world(
        world: &World,
        entity: Entity,
        collision_handle: &IndexNative,
    ) -> DropbearNativeResult<CharacterCollision> {
        let kcc = world
            .get::<&KCC>(entity)
            .map_err(|_| DropbearNativeError::NoSuchComponent)?;

        kcc.collisions
            .iter()
            .copied()
            .find(|c| {
                let (idx, generation) = c.handle.into_raw_parts();
                idx == collision_handle.index && generation == collision_handle.generation
            })
            .ok_or(DropbearNativeError::NoSuchHandle)
    }

    fn collider_ffi_from_handle(
        world: &World,
        handle: rapier3d::prelude::ColliderHandle,
    ) -> Option<NCollider> {
        let (idx, generation) = handle.into_raw_parts();

        for (entity, group) in world.query::<(Entity, &ColliderGroup)>().iter() {
            if group.colliders.iter().any(|c| c.id == idx) {
                return Some(NCollider {
                    index: IndexNative {
                        index: idx,
                        generation,
                    },
                    entity_id: entity.to_bits().get(),
                    id: idx,
                });
            }
        }

        None
    }

    pub fn get_collider(
        world: &World,
        entity: Entity,
        collision_handle: &IndexNative,
    ) -> DropbearNativeResult<NCollider> {
        let collision = get_collision_from_world(world, entity, collision_handle)?;
        collider_ffi_from_handle(world, collision.handle)
            .ok_or(DropbearNativeError::PhysicsObjectNotFound)
    }

    pub fn get_character_position(
        world: &World,
        entity: Entity,
        collision_handle: &IndexNative,
    ) -> DropbearNativeResult<NTransform> {
        let collision = get_collision_from_world(world, entity, collision_handle)?;

        let iso = collision.character_pos;
        let t = iso.translation;
        let rot = iso.rotation;
        let q: Quaternion<f32> = Quaternion::from(rot);

        Ok(NTransform {
            position: DVec3::new(t.x as f64, t.y as f64, t.z as f64).into(),
            rotation: DQuat::from_xyzw(q.i as f64, q.j as f64, q.k as f64, q.w as f64).into(),
            scale: DVec3::ONE.into(),
        })
    }

    pub fn get_translation_applied(
        world: &World,
        entity: Entity,
        collision_handle: &IndexNative,
    ) -> DropbearNativeResult<NVector3> {
        let collision = get_collision_from_world(world, entity, collision_handle)?;
        let v = collision.translation_applied;
        Ok(NVector3 {
            x: v.x as f64,
            y: v.y as f64,
            z: v.z as f64,
        })
    }

    pub fn get_translation_remaining(
        world: &World,
        entity: Entity,
        collision_handle: &IndexNative,
    ) -> DropbearNativeResult<NVector3> {
        let collision = get_collision_from_world(world, entity, collision_handle)?;
        let v = collision.translation_remaining;
        Ok(NVector3 {
            x: v.x as f64,
            y: v.y as f64,
            z: v.z as f64,
        })
    }

    pub fn get_time_of_impact(
        world: &World,
        entity: Entity,
        collision_handle: &IndexNative,
    ) -> DropbearNativeResult<f64> {
        let collision = get_collision_from_world(world, entity, collision_handle)?;
        Ok(collision.hit.time_of_impact as f64)
    }

    pub fn get_witness1(
        world: &World,
        entity: Entity,
        collision_handle: &IndexNative,
    ) -> DropbearNativeResult<NVector3> {
        let collision = get_collision_from_world(world, entity, collision_handle)?;
        let p = collision.hit.witness1;
        Ok(NVector3 {
            x: p.x as f64,
            y: p.y as f64,
            z: p.z as f64,
        })
    }

    pub fn get_witness2(
        world: &World,
        entity: Entity,
        collision_handle: &IndexNative,
    ) -> DropbearNativeResult<NVector3> {
        let collision = get_collision_from_world(world, entity, collision_handle)?;
        let p = collision.hit.witness2;
        Ok(NVector3 {
            x: p.x as f64,
            y: p.y as f64,
            z: p.z as f64,
        })
    }

    pub fn get_normal1(
        world: &World,
        entity: Entity,
        collision_handle: &IndexNative,
    ) -> DropbearNativeResult<NVector3> {
        let collision = get_collision_from_world(world, entity, collision_handle)?;
        let n = collision.hit.normal1;
        Ok(NVector3 {
            x: n.x as f64,
            y: n.y as f64,
            z: n.z as f64,
        })
    }

    pub fn get_normal2(
        world: &World,
        entity: Entity,
        collision_handle: &IndexNative,
    ) -> DropbearNativeResult<NVector3> {
        let collision = get_collision_from_world(world, entity, collision_handle)?;
        let n = collision.hit.normal2;
        Ok(NVector3 {
            x: n.x as f64,
            y: n.y as f64,
            z: n.z as f64,
        })
    }

    pub fn get_status(
        world: &World,
        entity: Entity,
        collision_handle: &IndexNative,
    ) -> DropbearNativeResult<NShapeCastStatus> {
        let collision = get_collision_from_world(world, entity, collision_handle)?;
        Ok(collision.hit.status.into())
    }
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.CharacterCollisionNative",
        func = "getCollider"
    ),
    c
)]
fn get_character_collision_collider(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    collision_handle: &IndexNative,
) -> DropbearNativeResult<NCollider> {
    shared::get_collider(world, entity, collision_handle)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.CharacterCollisionNative",
        func = "getCharacterPosition"
    ),
    c
)]
fn get_character_collision_position(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    collision_handle: &IndexNative,
) -> DropbearNativeResult<NTransform> {
    shared::get_character_position(world, entity, collision_handle)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.CharacterCollisionNative",
        func = "getTranslationApplied"
    ),
    c
)]
fn get_character_collision_translation_applied(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    collision_handle: &IndexNative,
) -> DropbearNativeResult<NVector3> {
    shared::get_translation_applied(world, entity, collision_handle)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.CharacterCollisionNative",
        func = "getTranslationRemaining"
    ),
    c
)]
fn get_character_collision_translation_remaining(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    collision_handle: &IndexNative,
) -> DropbearNativeResult<NVector3> {
    shared::get_translation_remaining(world, entity, collision_handle)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.CharacterCollisionNative",
        func = "getTimeOfImpact"
    ),
    c
)]
fn get_character_collision_time_of_impact(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    collision_handle: &IndexNative,
) -> DropbearNativeResult<f64> {
    shared::get_time_of_impact(world, entity, collision_handle)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.CharacterCollisionNative",
        func = "getWitness1"
    ),
    c
)]
fn get_character_collision_witness1(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    collision_handle: &IndexNative,
) -> DropbearNativeResult<NVector3> {
    shared::get_witness1(world, entity, collision_handle)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.CharacterCollisionNative",
        func = "getWitness2"
    ),
    c
)]
fn get_character_collision_witness2(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    collision_handle: &IndexNative,
) -> DropbearNativeResult<NVector3> {
    shared::get_witness2(world, entity, collision_handle)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.CharacterCollisionNative",
        func = "getNormal1"
    ),
    c
)]
fn get_character_collision_normal1(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    collision_handle: &IndexNative,
) -> DropbearNativeResult<NVector3> {
    shared::get_normal1(world, entity, collision_handle)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.CharacterCollisionNative",
        func = "getNormal2"
    ),
    c
)]
fn get_character_collision_normal2(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    collision_handle: &IndexNative,
) -> DropbearNativeResult<NVector3> {
    shared::get_normal2(world, entity, collision_handle)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.CharacterCollisionNative",
        func = "getStatus"
    ),
    c
)]
fn get_character_collision_status(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    collision_handle: &IndexNative,
) -> DropbearNativeResult<NShapeCastStatus> {
    shared::get_status(world, entity, collision_handle)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.KinematicCharacterControllerNative",
        func = "existsForEntity"
    ),
    c
)]
fn kcc_exists_for_entity(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<bool> {
    Ok(world.get::<&KCC>(entity).is_ok())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.KinematicCharacterControllerNative",
        func = "moveCharacter"
    ),
    c
)]
fn move_character(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::define(crate::ptr::PhysicsStatePtr)] physics_state: &mut PhysicsState,
    #[dropbear_macro::entity] entity: hecs::Entity,
    translation: &NVector3,
    delta_time: f64,
) -> DropbearNativeResult<()> {
    if let Ok((label, kcc)) = world.query_one::<(&Label, &mut KCC)>(entity).get() {
        let rigid_body_handle = physics_state
            .bodies_entity_map
            .get(label)
            .ok_or(DropbearNativeError::NoSuchHandle)?;

        let (body_type, body_pos) = {
            let body = physics_state
                .bodies
                .get(*rigid_body_handle)
                .ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
            (body.body_type(), *body.position())
        };

        if body_type != RigidBodyType::KinematicPositionBased {
            return Ok(()); // soft error, just tell the user
        }

        let collider_handles = physics_state
            .colliders_entity_map
            .get(label)
            .ok_or(DropbearNativeError::NoSuchHandle)?;
        let (_, collider_handle) = collider_handles
            .first()
            .ok_or(DropbearNativeError::NoSuchHandle)?;
        let collider = physics_state
            .colliders
            .get(*collider_handle)
            .ok_or(DropbearNativeError::PhysicsObjectNotFound)?;

        let character_pos = if let Some(pos_wrt_parent) = collider.position_wrt_parent() {
            body_pos * (*pos_wrt_parent)
        } else {
            *collider.position()
        };

        let filter = QueryFilter::default()
            .exclude_rigid_body(*rigid_body_handle)
            .exclude_sensors();
        let query_pipeline = physics_state.broad_phase.as_query_pipeline(
            physics_state.narrow_phase.query_dispatcher(),
            &physics_state.bodies,
            &physics_state.colliders,
            filter,
        );

        let movement = kcc.controller.move_shape(
            delta_time as f32,
            &query_pipeline,
            collider.shape(),
            &character_pos,
            rapier3d::prelude::Vector::new(
                translation.x as f32,
                translation.y as f32,
                translation.z as f32,
            ),
            |collision| {
                if let Some(collisions) =
                    physics_state.collision_events_to_deal_with.get_mut(&entity)
                {
                    collisions.push(collision)
                } else {
                    physics_state
                        .collision_events_to_deal_with
                        .insert(entity, vec![collision]);
                }
            },
        );

        if let Some(body) = physics_state.bodies.get_mut(*rigid_body_handle) {
            let current_pos = body.translation();
            let new_pos = current_pos + movement.translation;
            body.set_next_kinematic_translation(new_pos);
        }

        kcc.movement = Some(CharacterMovementResult {
            translation: movement.translation.into(),
            grounded: movement.grounded,
            is_sliding_down_slope: movement.is_sliding_down_slope,
        });

        Ok(())
    } else {
        Err(DropbearNativeError::MissingComponent)
    }
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.KinematicCharacterControllerNative",
        func = "setRotation"
    ),
    c
)]
fn set_rotation(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::define(crate::ptr::PhysicsStatePtr)] physics_state: &mut PhysicsState,
    #[dropbear_macro::entity] entity: hecs::Entity,
    rotation: &NQuaternion,
) -> DropbearNativeResult<()> {
    if let Ok((label, _)) = world.query_one::<(&Label, &KCC)>(entity).get() {
        let rigid_body_handle = physics_state
            .bodies_entity_map
            .get(label)
            .ok_or(DropbearNativeError::NoSuchHandle)?;

        let body_type = {
            let body = physics_state
                .bodies
                .get(*rigid_body_handle)
                .ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
            body.body_type()
        };

        if body_type != RigidBodyType::KinematicPositionBased {
            return Err(DropbearNativeError::InvalidArgument);
        }

        let len = (rotation.x * rotation.x
            + rotation.y * rotation.y
            + rotation.z * rotation.z
            + rotation.w * rotation.w)
            .sqrt();
        let (x, y, z, w) = if len > 0.0 {
            (
                rotation.x / len,
                rotation.y / len,
                rotation.z / len,
                rotation.w / len,
            )
        } else {
            (0.0, 0.0, 0.0, 1.0)
        };

        if let Some(body) = physics_state.bodies.get_mut(*rigid_body_handle) {
            let rot = Rotation::from_xyzw(x as f32, y as f32, z as f32, w as f32);
            body.set_next_kinematic_rotation(rot);
        }

        Ok(())
    } else {
        Err(DropbearNativeError::MissingComponent)
    }
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.KinematicCharacterControllerNative",
        func = "getHit"
    ),
    c
)]
fn get_hit(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<CharacterCollisionArray> {
    let kcc = world
        .get::<&KCC>(entity)
        .map_err(|_| DropbearNativeError::NoSuchComponent)?;

    let mut collisions = Vec::with_capacity(kcc.collisions.len());
    for collision in &kcc.collisions {
        let (idx, generation) = collision.handle.into_raw_parts();
        collisions.push(IndexNative {
            index: idx,
            generation,
        });
    }

    Ok(CharacterCollisionArray {
        entity_id: entity.to_bits().get(),
        collisions,
    })
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.KinematicCharacterControllerNative",
        func = "getMovementResult"
    ),
    c
)]
fn get_movement_result(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<Option<CharacterMovementResult>> {
    world
        .get::<&KCC>(entity)
        .map(|kcc| kcc.movement.clone())
        .map(Ok)
        .unwrap_or(Ok(None))
}