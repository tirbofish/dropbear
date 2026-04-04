use glam::Vec3;
use eucalyptus_core::ptr::PhysicsStatePtr;
use eucalyptus_core::scripting::result::DropbearNativeResult;
use eucalyptus_core::types::{IndexNative, NCollider, NShapeCastHit, NVector3, RayHit};
use eucalyptus_core::third_party::rapier3d::parry::query::{DefaultQueryDispatcher, ShapeCastOptions};
use hecs::Entity;
use eucalyptus_core::physics::collider::ColliderShape;
use eucalyptus_core::physics::PhysicsState;
use eucalyptus_core::third_party::rapier3d::prelude::{nalgebra, point, vector, ColliderHandle, Pose3, QueryFilter, Ray, SharedShape};

pub mod shared {
    use hecs::Entity;
    use eucalyptus_core::physics::PhysicsState;
    use eucalyptus_core::third_party::rapier3d::prelude::ColliderHandle;
    use eucalyptus_core::types::{NCollider, NVector3};

    pub fn get_gravity(physics: &PhysicsState) -> NVector3 {
        NVector3::from(physics.gravity)
    }

    pub fn set_gravity(physics: &mut PhysicsState, new: NVector3) {
        physics.gravity = new.to_float_array();
    }

    fn collider_handle_from_ffi(collider: &NCollider) -> ColliderHandle {
        ColliderHandle::from_raw_parts(collider.index.index, collider.index.generation)
    }

    pub fn overlapping(
        physics: &PhysicsState,
        collider1: &NCollider,
        collider2: &NCollider,
    ) -> bool {
        let h1 = collider_handle_from_ffi(collider1);
        let h2 = collider_handle_from_ffi(collider2);

        if physics.colliders.get(h1).is_none() || physics.colliders.get(h2).is_none() {
            return false;
        }

        physics
            .narrow_phase
            .intersection_pair(h1, h2)
            .unwrap_or(false)
    }

    pub fn triggering(
        physics: &PhysicsState,
        collider1: &NCollider,
        collider2: &NCollider,
    ) -> bool {
        let h1 = collider_handle_from_ffi(collider1);
        let h2 = collider_handle_from_ffi(collider2);

        let is_sensor_1 = physics
            .colliders
            .get(h1)
            .map(|c| c.is_sensor())
            .unwrap_or(false);
        let is_sensor_2 = physics
            .colliders
            .get(h2)
            .map(|c| c.is_sensor())
            .unwrap_or(false);

        (is_sensor_1 || is_sensor_2) && overlapping(physics, collider1, collider2)
    }

    pub fn touching(physics: &PhysicsState, entity1: Entity, entity2: Entity) -> bool {
        let Some(label1) = physics.entity_label_map.get(&entity1) else {
            return false;
        };
        let Some(label2) = physics.entity_label_map.get(&entity2) else {
            return false;
        };

        let Some(handles1) = physics.colliders_entity_map.get(label1) else {
            return false;
        };
        let Some(handles2) = physics.colliders_entity_map.get(label2) else {
            return false;
        };

        for (_, h1) in handles1 {
            for (_, h2) in handles2 {
                if let Some(pair) = physics.narrow_phase.contact_pair(*h1, *h2) {
                    if pair.has_any_active_contact() {
                        return true;
                    }
                }
            }
        }

        false
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.PhysicsNative", func = "getGravity"),
    c
)]
fn get_gravity(
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &PhysicsState,
) -> DropbearNativeResult<NVector3> {
    Ok(shared::get_gravity(physics))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.PhysicsNative", func = "setGravity"),
    c
)]
fn set_gravity(
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &mut PhysicsState,
    gravity: &NVector3,
) -> DropbearNativeResult<()> {
    Ok(shared::set_gravity(physics, *gravity))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.PhysicsNative", func = "raycast"),
    c
)]
fn raycast(
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &mut PhysicsState,
    origin: &NVector3,
    dir: &NVector3,
    time_of_impact: f64,
    solid: bool,
) -> DropbearNativeResult<Option<RayHit>> {
    let qp = physics.broad_phase.as_query_pipeline(
        &DefaultQueryDispatcher,
        &physics.bodies,
        &physics.colliders,
        QueryFilter::new(),
    );

    let ray = Ray::new(
        point![origin.x as f32, origin.y as f32, origin.z as f32].into(),
        vector![dir.x as f32, dir.y as f32, dir.z as f32].into(),
    );

    if let Some((hit, distance)) = qp.cast_ray(&ray, time_of_impact as f32, solid) {
        let raw = hit.0;

        let mut found = None;
        for (l, colliders) in physics.colliders_entity_map.iter() {
            for (_, c) in colliders {
                if c.0 == hit.0 {
                    found = Some((l, c.0));
                }
            }
        }

        if let Some((label, _)) = found {
            let entity = physics.entity_label_map.iter().find(|(_, l)| *l == label);
            if let Some((e, _)) = entity {
                let rayhit = RayHit {
                    collider: NCollider {
                        index: IndexNative::from(raw),
                        entity_id: e.to_bits().get(),
                        id: raw.into_raw_parts().0,
                    },
                    distance: distance as f64,
                };
                Ok(Some(rayhit))
            } else {
                Ok(None)
            }
        } else {
            eprintln!("Unknown collider, still returning value without entity_id");
            let rayhit = RayHit {
                collider: NCollider {
                    index: IndexNative::from(raw),
                    entity_id: Entity::DANGLING.to_bits().get(),
                    id: raw.into_raw_parts().0,
                },
                distance: distance as f64,
            };
            Ok(Some(rayhit))
        }
    } else {
        Ok(None)
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.PhysicsNative", func = "shapeCast"),
    c
)]
fn shape_cast(
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &mut PhysicsState,
    origin: &NVector3,
    direction: &NVector3,
    shape: &ColliderShape,
    time_of_impact: f64,
    solid: bool,
) -> DropbearNativeResult<Option<NShapeCastHit>> {
    let qp = physics.broad_phase.as_query_pipeline(
        &DefaultQueryDispatcher,
        &physics.bodies,
        &physics.colliders,
        QueryFilter::new(),
    );

    let dir_len =
        ((direction.x * direction.x) + (direction.y * direction.y) + (direction.z * direction.z))
            .sqrt();
    if dir_len <= f64::EPSILON {
        return Ok(None);
    }

    let dir_unit = NVector3 {
        x: direction.x / dir_len,
        y: direction.y / dir_len,
        z: direction.z / dir_len,
    };

    let cast_shape = match shape {
        ColliderShape::Box { half_extents } => SharedShape::cuboid(
            half_extents.x as f32,
            half_extents.y as f32,
            half_extents.z as f32,
        ),
        ColliderShape::Sphere { radius } => SharedShape::ball(*radius),
        ColliderShape::Capsule { half_height, radius } => {
            SharedShape::capsule_y(*half_height, *radius)
        }
        ColliderShape::Cylinder { half_height, radius } => {
            SharedShape::cylinder(*half_height, *radius)
        }
        ColliderShape::Cone { half_height, radius } => {
            SharedShape::cone(*half_height, *radius)
        }
    };

    let iso: Pose3 =
        nalgebra::Isometry3::translation(origin.x as f32, origin.y as f32, origin.z as f32)
            .into();
    let vel: Vec3 = vector![dir_unit.x as f32, dir_unit.y as f32, dir_unit.z as f32].into();

    let options = ShapeCastOptions {
        max_time_of_impact: time_of_impact as f32,
        target_distance: 0.0,
        stop_at_penetration: solid,
        compute_impact_geometry_on_penetration: true,
    };

    let Some((hit_handle, toi)) = qp.cast_shape(&iso, vel, cast_shape.as_ref(), options) else {
        return Ok(None);
    };

    let collider = collider_ffi_from_handle(physics, hit_handle);

    let hit = NShapeCastHit {
        collider,
        distance: toi.time_of_impact as f64,
        witness1: NVector3::from([toi.witness1.x, toi.witness1.y, toi.witness1.z]),
        witness2: NVector3::from([toi.witness2.x, toi.witness2.y, toi.witness2.z]),
        normal1: NVector3::from([toi.normal1.x, toi.normal1.y, toi.normal1.z]),
        normal2: NVector3::from([toi.normal2.x, toi.normal2.y, toi.normal2.z]),
        status: toi.status.into(),
    };

    Ok(Some(hit))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.PhysicsNative", func = "isOverlapping"),
    c
)]
fn is_overlapping(
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &PhysicsState,
    collider1: &NCollider,
    collider2: &NCollider,
) -> DropbearNativeResult<bool> {
    Ok(shared::overlapping(physics, collider1, collider2))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.PhysicsNative", func = "isTriggering"),
    c
)]
fn is_triggering(
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &PhysicsState,
    collider1: &NCollider,
    collider2: &NCollider,
) -> DropbearNativeResult<bool> {
    Ok(shared::triggering(physics, collider1, collider2))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.PhysicsNative", func = "isTouching"),
    c
)]
fn is_touching(
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &PhysicsState,
    #[dropbear_macro::entity] entity1: Entity,
    #[dropbear_macro::entity] entity2: Entity,
) -> DropbearNativeResult<bool> {
    Ok(shared::touching(physics, entity1, entity2))
}

fn collider_ffi_from_handle(physics: &PhysicsState, handle: ColliderHandle) -> NCollider {
    let (idx, generation) = handle.into_raw_parts();

    let mut found_label = None;
    for (label, colliders) in physics.colliders_entity_map.iter() {
        for (_, c) in colliders {
            if c.0 == handle.0 {
                found_label = Some(label);
                break;
            }
        }
        if found_label.is_some() {
            break;
        }
    }

    let entity_id = if let Some(label) = found_label {
        physics
            .entity_label_map
            .iter()
            .find(|(_, l)| *l == label)
            .map(|(e, _)| e.to_bits().get())
            .unwrap_or(Entity::DANGLING.to_bits().get())
    } else {
        Entity::DANGLING.to_bits().get()
    };

    NCollider {
        index: IndexNative { index: idx, generation },
        entity_id,
        id: idx,
    }
}
