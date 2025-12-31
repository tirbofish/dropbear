//! Components in the eucalyptus-editor and redback-runtime that relate to rapier3d based physics.

use crate::physics::collider::NEXT_ID;
use crate::physics::rigidbody::RigidBodyMode;
use crate::states::Label;
use dropbear_engine::entity::Transform;
use hecs::Entity;
use rapier3d::na::{Quaternion, UnitQuaternion, Vector3};
use rapier3d::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::AddAssign;

pub mod rigidbody;
pub mod collider;

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
        }
    }

    pub fn step(&mut self, entity_label_map: HashMap<Entity, Label>, pipeline: &mut PhysicsPipeline, physics_hooks: (), event_handler: ()) {
        self.entity_label_map = entity_label_map;
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

    pub fn register_rigidbody(&mut self, rigid_body: &rigidbody::RigidBody, transform: Transform) {
        let mode = match rigid_body.mode {
            RigidBodyMode::Dynamic => RigidBodyType::Dynamic,
            RigidBodyMode::Fixed => RigidBodyType::Fixed,
            RigidBodyMode::KinematicPosition => RigidBodyType::KinematicPositionBased,
            RigidBodyMode::KinematicVelocity => RigidBodyType::KinematicVelocityBased,
        };

        let pos = transform.position.as_vec3().to_array();
        let rot = transform.rotation.as_quat().to_array();

        let body = RigidBodyBuilder::new(mode)
            .translation(vector![pos[0], pos[1], pos[2]])
            .rotation(UnitQuaternion::from_quaternion(Quaternion::new(
                rot[3] as f32, rot[0] as f32, rot[1] as f32, rot[2] as f32
            )).scaled_axis())
            .gravity_scale(rigid_body.gravity_scale)
            .sleeping(rigid_body.sleeping)
            .can_sleep(rigid_body.can_sleep)
            .ccd_enabled(rigid_body.ccd_enabled)
            .linvel(Vector3::from_column_slice(&rigid_body.linvel))
            .angvel(Vector3::from_column_slice(&rigid_body.angvel))
            .linear_damping(rigid_body.linear_damping)
            .angular_damping(rigid_body.angular_damping)
            .enabled_translations(!rigid_body.lock_translation.x, !rigid_body.lock_translation.y, !rigid_body.lock_translation.z)
            .enabled_rotations(!rigid_body.lock_rotation.x, !rigid_body.lock_rotation.y, !rigid_body.lock_rotation.z)
            .build();

        let body_handle = self.bodies.insert(body);
        self.bodies_entity_map.insert(rigid_body.entity.clone(), body_handle);
        
        if let Some(collider_handles) = self.colliders_entity_map.get(&rigid_body.entity) {
            let handles_to_attach = collider_handles.clone();

            for (id, handle) in handles_to_attach {
                self.colliders.set_parent(handle, Some(body_handle), &mut self.bodies);
            }
        }
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

        let handle = if let Some(&rigid_body_handle) = self.bodies_entity_map.get(&collider_component.entity) {
            self.colliders.insert_with_parent(
                builder.build(),
                rigid_body_handle,
                &mut self.bodies
            )
        } else {
            self.colliders.insert(builder.build())
        };

        let mut next_id = *NEXT_ID.get_mut();
        next_id.add_assign(1);

        self.colliders_entity_map
            .entry(collider_component.entity.clone())
            .or_insert_with(Vec::new)
            .push((next_id as u32, handle));

        handle
    }

    /// Remove all colliders associated with an entity
    pub fn remove_colliders(&mut self, entity: &Label) {
        if let Some(handles) = self.colliders_entity_map.remove(entity) {
            for (_id, handle) in handles {
                self.colliders.remove(
                    handle,
                    &mut self.islands,
                    &mut self.bodies,
                    false
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
                false
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
    use crate::types::Vector3;

    pub fn get_gravity(physics: &PhysicsState) -> Vector3 {
        Vector3::from(physics.gravity)
    }

    pub fn set_gravity(physics: &mut PhysicsState, new: Vector3) {
        physics.gravity = new.to_float_array();
    }
}

pub mod jni {
    #![allow(non_snake_case)]

    use crate::physics::nalgebra;
    use crate::physics::PhysicsState;
    use crate::scripting::jni::utils::{FromJObject, ToJObject};
    use crate::types::{IndexNative, RayHit, Vector3};
    use dropbear_engine::wgpu::hal::ShouldBeNonZeroExt;
    use hecs::Entity;
    use jni::objects::{JClass, JObject};
    use jni::sys::{jboolean, jdouble, jlong, jobject};
    use jni::JNIEnv;
    use rapier3d::parry::query::DefaultQueryDispatcher;
    use rapier3d::pipeline::QueryFilter;
    use rapier3d::prelude::{point, vector, Ray};

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_physics_PhysicsNative_getGravity(
        mut env: JNIEnv,
        _: JClass,
        physics_handle: jlong,
    ) -> jobject {
        let physics = crate::convert_ptr!(physics_handle => PhysicsState);

        match super::shared::get_gravity(&physics).to_jobject(&mut env) {
            Ok(v) => v.into_raw(),
            Err(e) => {
                let _ = env.throw_new("java/lang/RuntimeException", format!("Unable to create new Vector3d object for gravity: {}", e));
                std::ptr::null_mut()
            }
        }

    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_physics_PhysicsNative_setGravity(
        mut env: JNIEnv,
        _: JClass,
        physics_handle: jlong,
        new_gravity: JObject,
    ) {
        let mut physics = crate::convert_ptr!(mut physics_handle => PhysicsState);
        let vec3 = match Vector3::from_jobject(&mut env, &new_gravity) {
            Ok(v) => v,
            Err(e) => {
                let _ = env.throw_new("java/lang/RuntimeException", format!("Unable to create new Vector3d object for gravity: {}", e));
                return;
            }
        };

        super::shared::set_gravity(&mut physics, vec3);
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_physics_PhysicsNative_raycast(
        mut env: JNIEnv,
        _: JClass,
        physics_handle: jlong,
        origin: JObject,
        direction: JObject,
        time_of_impact: jdouble,
        solid: jboolean,
    ) -> jobject {
        let physics = crate::convert_ptr!(mut physics_handle => PhysicsState);

        let qp = physics.broad_phase.as_query_pipeline(&DefaultQueryDispatcher, &physics.bodies, &physics.colliders, QueryFilter::new());

        let origin = match Vector3::from_jobject(&mut env, &origin) {
            Ok(v) => v,
            Err(e) => {
                let _ = env.throw_new("java/lang/RuntimeException", format!("Unable to create a new rust Vector3 object: {}", e));
                return std::ptr::null_mut();
            }
        };

        let dir = match Vector3::from_jobject(&mut env, &direction) {
            Ok(v) => v,
            Err(e) => {
                let _ = env.throw_new("java/lang/RuntimeException", format!("Unable to create a new rust Vector3 object: {}", e));
                return std::ptr::null_mut();
            }
        };

        let ray = Ray::new(
            point![origin.x as f32, origin.y as f32, origin.z as f32],
            vector![dir.x as f32, dir.y as f32, dir.z as f32],
        );

        if let Some((hit, distance)) = qp.cast_ray(&ray, time_of_impact as f32, solid != 0) {
            let raw = hit.0;

            let mut found = None;

            for (l, colliders) in physics.colliders_entity_map.iter() {
                for (id, c) in colliders {
                    if c.0 == hit.0 {
                        found = Some((l, c.0));
                    }
                }
            }

            if let Some((label, index)) = found {
                let entity = physics.entity_label_map.iter().find(|(e, l)| *l == label);
                if let Some((e, _)) = entity {
                    let rayhit = RayHit {
                        collider: crate::types::ColliderFFI {
                            index: IndexNative::from(raw),
                            entity_id: e.to_bits().get(),
                            id: raw.into_raw_parts().0,
                        },
                        distance: distance as f64,
                    };

                    match rayhit.to_jobject(&mut env) {
                        Ok(v) => v.into_raw(),
                        Err(e) => {
                            let _ = env.throw_new("java/lang/RuntimeException", format!("Unable to create a new rust RayHit object: {}", e));
                            return std::ptr::null_mut();
                        }
                    }
                } else {
                    std::ptr::null_mut()
                }
            } else {
                eprintln!("Unknown collider, still returning value without entity_id");

                let rayhit = RayHit {
                    collider: crate::types::ColliderFFI {
                        index: IndexNative::from(raw),
                        entity_id: Entity::DANGLING.to_bits().get(),
                        id: raw.into_raw_parts().0,
                    },
                    distance: distance as f64,
                };

                match rayhit.to_jobject(&mut env) {
                    Ok(v) => v.into_raw(),
                    Err(e) => {
                        let _ = env.throw_new("java/lang/RuntimeException", format!("Unable to create a new rust RayHit object: {}", e));
                        return std::ptr::null_mut();
                    }
                }
            }
        } else {
            std::ptr::null_mut()
        }
    }
}

pub mod native {

}