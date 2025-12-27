//! Components in the eucalyptus-editor and redback-runtime that relate to rapier3d based physics.

use std::collections::HashMap;
use glam::{Vec3};
use rapier3d::na::Vector3;
use rapier3d::prelude::*;
use serde::{Deserialize, Serialize};
use crate::physics::rigidbody::RigidBodyMode;
use crate::states::Label;

pub mod rigidbody;

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
    pub gravity: Vec3,

    pub bodies_entity_map: HashMap<Label, RigidBodyHandle>
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
            gravity: Vec3::new(0.0, -9.81, 0.0),
            bodies_entity_map: Default::default(),
        }
    }

    pub fn step(&mut self, pipeline: &mut PhysicsPipeline, physics_hooks: (), event_handler: ()) {
        pipeline.step(
            &vector![self.gravity.x, self.gravity.y, self.gravity.z],
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
}

impl Default for PhysicsState {
    fn default() -> Self {
        Self::new()
    }
}