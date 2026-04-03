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

        if collider_component.is_sensor {
            builder = builder.active_collision_types(ActiveCollisionTypes::all());
        }

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
