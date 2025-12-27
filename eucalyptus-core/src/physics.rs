//! Components in the eucalyptus-editor and redback-runtime that relate to rapier3d based physics.

use std::collections::HashMap;
use rapier3d::na::{UnitQuaternion, Vector3};
use rapier3d::prelude::*;
use serde::{Deserialize, Serialize};
use crate::physics::rigidbody::RigidBodyMode;
use crate::states::Label;

pub mod rigidbody;
pub mod collider;

/// A serializable [rapier3d] state that shows all the different actions and types related
/// to physics rendering.
#[derive(Serialize, Deserialize, Clone)]
pub struct PhysicsState {
    pub islands: IslandManager,
    pub broad_phase: DefaultBroadPhase,
    pub narrow_phase: NarrowPhase,
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub impulse_joints: ImpulseJointSet,
    pub multibody_joints: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub integration_parameters: IntegrationParameters,
    pub gravity: [f32; 3],

    pub bodies_entity_map: HashMap<Label, RigidBodyHandle>,
    pub colliders_entity_map: HashMap<Label, Vec<ColliderHandle>>
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
        }
    }

    pub fn step(&mut self, pipeline: &mut PhysicsPipeline, physics_hooks: (), event_handler: ()) {
        pipeline.step(
            &vector![self.gravity[0], self.gravity[1], self.gravity[2]], // a panic is deserved for those who don't specify a 3rd type in a vector array
            &self.integration_parameters,
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            &physics_hooks,
            &event_handler,
        );
    }

    pub fn register_rigidbody(&mut self, rigid_body: &rigidbody::RigidBody) {
        let mode = match rigid_body.mode {
            RigidBodyMode::Dynamic => RigidBodyType::Dynamic,
            RigidBodyMode::Fixed => RigidBodyType::Fixed,
            RigidBodyMode::KinematicPosition => RigidBodyType::KinematicPositionBased,
            RigidBodyMode::KinematicVelocity => RigidBodyType::KinematicVelocityBased,
        };

        let body = RigidBodyBuilder::new(mode)
            .gravity_scale(rigid_body.gravity_scale)
            .can_sleep(rigid_body.can_sleep)
            .ccd_enabled(rigid_body.ccd_enabled)
            .linvel(Vector3::from_column_slice(&rigid_body.linvel))
            .angvel(Vector3::from_column_slice(&rigid_body.angvel))
            .linear_damping(rigid_body.linear_damping)
            .angular_damping(rigid_body.angular_damping)
            .enabled_translations(rigid_body.lock_translation.x, rigid_body.lock_translation.y, rigid_body.lock_translation.z)
            .enabled_rotations(rigid_body.lock_rotation.x, rigid_body.lock_rotation.y, rigid_body.lock_rotation.z)
            .build();

        let handle = self.bodies.insert(body);

        self.bodies_entity_map.insert(rigid_body.entity.clone(), handle);
    }

    pub fn register_collider(&mut self, collider_component: &collider::Collider) -> ColliderHandle {
        use collider::ColliderShape;

        let mut builder = match &collider_component.shape {
            ColliderShape::Box { half_extents } => {
                ColliderBuilder::cuboid(half_extents[0], half_extents[1], half_extents[2])
            }
            ColliderShape::Sphere { radius } => {
                ColliderBuilder::ball(*radius)
            }
            ColliderShape::Capsule { half_height, radius } => {
                ColliderBuilder::capsule_y(*half_height, *radius)
            }
            ColliderShape::Cylinder { half_height, radius } => {
                ColliderBuilder::cylinder(*half_height, *radius)
            }
            ColliderShape::Cone { half_height, radius } => {
                ColliderBuilder::cone(*half_height, *radius)
            }
        };

        builder = builder
            .density(collider_component.density)
            .friction(collider_component.friction)
            .restitution(collider_component.restitution)
            .sensor(collider_component.is_sensor);

        builder = builder.translation(Vector3::from_column_slice(&collider_component.translation));

        let rotation = UnitQuaternion::from_euler_angles(
            collider_component.rotation[0],
            collider_component.rotation[1],
            collider_component.rotation[2]
        );
        builder = builder.rotation(rotation.scaled_axis());

        // check if entity has rigid body
        let handle = if let Some(&rigid_body_handle) = self.bodies_entity_map.get(&collider_component.entity) {
            // attach
            self.colliders.insert_with_parent(
                builder.build(),
                rigid_body_handle,
                &mut self.bodies
            )
        } else {
            // create a static collider if it doesn't exist
            self.colliders.insert(builder.build())
        };

        self.colliders_entity_map
            .entry(collider_component.entity.clone())
            .or_insert_with(Vec::new)
            .push(handle);

        handle
    }

    /// Remove all colliders associated with an entity
    pub fn remove_colliders(&mut self, entity: &Label) {
        if let Some(handles) = self.colliders_entity_map.remove(entity) {
            for handle in handles {
                self.colliders.remove(
                    handle,
                    &mut self.islands,
                    &mut self.bodies,
                    false // wake_up
                );
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
                false // wake_up
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