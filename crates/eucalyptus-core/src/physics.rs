//! Components in the eucalyptus-editor and redback-runtime that relate to rapier3d based physics.

use crate::physics::rigidbody::RigidBodyMode;
use crate::ptr::PhysicsStatePtr;
use crate::scripting::result::DropbearNativeResult;
use crate::states::Label;
use crate::types::{IndexNative, NCollider, NShapeCastHit, NVector3, RayHit};
use dropbear_engine::entity::Transform;
use hecs::Entity;
use rapier3d::control::CharacterCollision;
use rapier3d::na::{Quaternion, UnitQuaternion};
use rapier3d::parry::query::{DefaultQueryDispatcher, ShapeCastOptions};
use rapier3d::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod collider;
pub mod kcc;
pub mod rigidbody;

/// A serializable [rapier3d] state that shows all the different actions and types related
/// to physics rendering.
#[derive(Serialize, Deserialize, Clone)]
pub struct PhysicsState {
    #[serde(default)]
    pub islands: IslandManager,
    #[serde(default)]
    pub broad_phase: DefaultBroadPhase,
    #[serde(default)]
    pub narrow_phase: NarrowPhase,
    #[serde(default)]
    pub bodies: RigidBodySet,
    #[serde(default)]
    pub colliders: ColliderSet,
    #[serde(default)]
    pub impulse_joints: ImpulseJointSet,
    #[serde(default)]
    pub multibody_joints: MultibodyJointSet,
    #[serde(default)]
    pub ccd_solver: CCDSolver,
    #[serde(default)]
    pub integration_parameters: IntegrationParameters,

    pub gravity: [f32; 3],

    #[serde(default)]
    pub bodies_entity_map: HashMap<Label, RigidBodyHandle>,
    #[serde(default)]
    pub colliders_entity_map: HashMap<Label, Vec<(u32, ColliderHandle)>>,
    #[serde(default)]
    pub entity_label_map: HashMap<Entity, Label>,

    #[serde(skip)]
    pub collision_events_to_deal_with: HashMap<Entity, Vec<CharacterCollision>>,
}

impl PhysicsState {
    pub fn new() -> Self {
        Self {
            islands: Default::default(),
            broad_phase: Default::default(),
            narrow_phase: Default::default(),
            bodies: Default::default(),
            colliders: Default::default(),
            impulse_joints: Default::default(),
            multibody_joints: Default::default(),
            ccd_solver: Default::default(),
            integration_parameters: Default::default(),
            gravity: [0.0, -9.81, 0.0],
            bodies_entity_map: Default::default(),
            colliders_entity_map: Default::default(),
            entity_label_map: Default::default(),
            collision_events_to_deal_with: Default::default(),
        }
    }

    pub fn step(
        &mut self,
        entity_label_map: HashMap<Entity, Label>,
        pipeline: &mut PhysicsPipeline,
        physics_hooks: &dyn PhysicsHooks,
        event_handler: &dyn EventHandler,
    ) {
        self.entity_label_map = entity_label_map;
        pipeline.step(
            Vector::new(self.gravity[0], self.gravity[1], self.gravity[2]), // a panic is deserved for those who don't specify a 3rd type in a vector array
            &self.integration_parameters,
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            physics_hooks,
            event_handler,
        );
    }

    pub fn register_rigidbody(&mut self, rigid_body: &rigidbody::RigidBody, transform: Transform) {
        let mode = match rigid_body.mode {
            RigidBodyMode::Dynamic => RigidBodyType::Dynamic,
            RigidBodyMode::Fixed => RigidBodyType::Fixed,
            RigidBodyMode::KinematicPosition => RigidBodyType::KinematicPositionBased,
            RigidBodyMode::KinematicVelocity => RigidBodyType::KinematicVelocityBased,
        };

        let pos = transform.position.as_vec3().to_array();
        let rot = transform.rotation.as_quat().to_array();

        let mut bits = LockedAxes::empty();

        let translation_lock = rigid_body.lock_translation;
        if translation_lock.x {
            bits = bits | LockedAxes::TRANSLATION_LOCKED_X;
        }
        if translation_lock.y {
            bits = bits | LockedAxes::TRANSLATION_LOCKED_Y;
        }
        if translation_lock.z {
            bits = bits | LockedAxes::TRANSLATION_LOCKED_Z;
        }

        let rotation_lock = rigid_body.lock_rotation;
        if rotation_lock.x {
            bits = bits | LockedAxes::ROTATION_LOCKED_X;
        }
        if rotation_lock.y {
            bits = bits | LockedAxes::ROTATION_LOCKED_Y;
        }
        if rotation_lock.z {
            bits = bits | LockedAxes::ROTATION_LOCKED_Z;
        }

        let body = RigidBodyBuilder::new(mode)
            .translation(Vector::from_array(pos))
            .rotation(
                UnitQuaternion::from_quaternion(Quaternion::new(rot[3], rot[0], rot[1], rot[2]))
                    .scaled_axis()
                    .into(),
            )
            .gravity_scale(rigid_body.gravity_scale)
            .sleeping(rigid_body.sleeping)
            .can_sleep(rigid_body.can_sleep)
            .ccd_enabled(rigid_body.ccd_enabled)
            .linvel(Vector::from_array(rigid_body.linvel))
            .angvel(Vector::from_array(rigid_body.angvel))
            .linear_damping(rigid_body.linear_damping)
            .angular_damping(rigid_body.angular_damping)
            .locked_axes(bits)
            .build();

        let body_handle = self.bodies.insert(body);
        self.bodies_entity_map
            .insert(rigid_body.entity.clone(), body_handle);

        if let Some(collider_handles) = self.colliders_entity_map.get(&rigid_body.entity) {
            let handles_to_attach = collider_handles.clone();

            for (_, handle) in handles_to_attach {
                self.colliders
                    .set_parent(handle, Some(body_handle), &mut self.bodies);
            }
        }
    }

    pub fn register_collider(&mut self, collider_component: &collider::Collider) -> ColliderHandle {
        use collider::ColliderShape;

        let mut builder = match &collider_component.shape {
            ColliderShape::Box { half_extents } => ColliderBuilder::cuboid(
                half_extents.x as f32,
                half_extents.y as f32,
                half_extents.z as f32,
            ),
            ColliderShape::Sphere { radius } => ColliderBuilder::ball(*radius),
            ColliderShape::Capsule {
                half_height,
                radius,
            } => ColliderBuilder::capsule_y(*half_height, *radius),
            ColliderShape::Cylinder {
                half_height,
                radius,
            } => ColliderBuilder::cylinder(*half_height, *radius),
            ColliderShape::Cone {
                half_height,
                radius,
            } => ColliderBuilder::cone(*half_height, *radius),
        };

        builder = builder
            .density(collider_component.density)
            .friction(collider_component.friction)
            .restitution(collider_component.restitution)
            .sensor(collider_component.is_sensor);

        let mut active_events = ActiveEvents::COLLISION_EVENTS;
        if !collider_component.is_sensor {
            active_events |= ActiveEvents::CONTACT_FORCE_EVENTS;
        }
        builder = builder.active_events(active_events);

        builder = builder.translation(Vector::from_array(collider_component.translation));

        let rotation = UnitQuaternion::from_euler_angles(
            collider_component.rotation[0],
            collider_component.rotation[1],
            collider_component.rotation[2],
        );
        builder = builder.rotation(rotation.scaled_axis().into());

        let handle = if let Some(&rigid_body_handle) =
            self.bodies_entity_map.get(&collider_component.entity)
        {
            self.colliders
                .insert_with_parent(builder.build(), rigid_body_handle, &mut self.bodies)
        } else {
            self.colliders.insert(builder.build())
        };

        self.colliders_entity_map
            .entry(collider_component.entity.clone())
            .or_insert_with(Vec::new)
            .push((handle.into_raw_parts().0, handle));

        handle
    }

    /// Remove all colliders associated with an entity
    pub fn remove_colliders(&mut self, entity: &Label) {
        if let Some(handles) = self.colliders_entity_map.remove(entity) {
            for (_id, handle) in handles {
                self.colliders
                    .remove(handle, &mut self.islands, &mut self.bodies, false);
            }
        }
    }

    /// Remove a rigid body and all its attached colliders
    pub fn remove_rigidbody(&mut self, entity: &Label) {
        if let Some(handle) = self.bodies_entity_map.remove(entity) {
            self.bodies.remove(
                handle,
                &mut self.islands,
                &mut self.colliders,
                &mut self.impulse_joints,
                &mut self.multibody_joints,
                false,
            );
        }
        self.colliders_entity_map.remove(entity);
    }
}

impl Default for PhysicsState {
    fn default() -> Self {
        Self::new()
    }
}

pub mod shared {
    use crate::physics::PhysicsState;
    use crate::types::NCollider;
    use crate::types::NVector3;
    use hecs::Entity;
    use rapier3d::prelude::ColliderHandle;

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
                    collider: crate::types::NCollider {
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
                collider: crate::types::NCollider {
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
    shape: &collider::ColliderShape,
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

    let cast_shape = {
        match shape {
            crate::physics::collider::ColliderShape::Box { half_extents } => {
                rapier3d::geometry::SharedShape::cuboid(
                    half_extents.x as f32,
                    half_extents.y as f32,
                    half_extents.z as f32,
                )
            }
            crate::physics::collider::ColliderShape::Sphere { radius } => {
                rapier3d::geometry::SharedShape::ball(*radius)
            }
            crate::physics::collider::ColliderShape::Capsule {
                half_height,
                radius,
            } => rapier3d::geometry::SharedShape::capsule_y(*half_height, *radius),
            crate::physics::collider::ColliderShape::Cylinder {
                half_height,
                radius,
            } => rapier3d::geometry::SharedShape::cylinder(*half_height, *radius),
            crate::physics::collider::ColliderShape::Cone {
                half_height,
                radius,
            } => rapier3d::geometry::SharedShape::cone(*half_height, *radius),
        }
    };
    let iso: Pose3 =
        nalgebra::Isometry3::translation(origin.x as f32, origin.y as f32, origin.z as f32).into();
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

    let collider = collider_ffi_from_handle(&physics, hit_handle);

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

fn collider_ffi_from_handle(
    physics: &PhysicsState,
    handle: rapier3d::prelude::ColliderHandle,
) -> NCollider {
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
        index: IndexNative {
            index: idx,
            generation,
        },
        entity_id,
        id: idx,
    }
}
