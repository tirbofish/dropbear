//! Module that relates to the [Kinematic Character Controller](https://rapier.rs/docs/user_guides/rust/character_controller)
//! (or kcc for short) in the [rapier3d] physics engine.

use crate::traits::SerializableComponent;
use rapier3d::control::KinematicCharacterController;
use serde::{Deserialize, Serialize};
use dropbear_macro::SerializableComponent;
use crate::states::Label;

/// The kinematic character controller (kcc) component.
#[derive(Debug, Default, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct KCC {
    pub entity: Label,
    pub controller: KinematicCharacterController,
}

impl KCC {
    pub fn new(label: &Label) -> Self {
        KCC {
            entity: label.clone(),
            controller: KinematicCharacterController::default(),
        }
    }
}

pub mod shared {

}

pub mod jni {
    #![allow(non_snake_case)]

    use hecs::World;
    use jni::JNIEnv;
    use jni::objects::{JClass, JObject};
    use jni::sys::{jboolean, jdouble, jlong};
    use rapier3d::dynamics::RigidBodyType;
    use rapier3d::prelude::QueryFilter;
    use crate::{convert_jlong_to_entity, convert_ptr};
    use crate::physics::kcc::KCC;
    use crate::physics::PhysicsState;
    use crate::scripting::jni::utils::FromJObject;
    use crate::states::Label;
    use crate::types::Vector3;

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_physics_KinematicCharacterControllerNative_existsForEntity(
        _env: JNIEnv,
        _: JClass,
        world_handle: jlong,
        entity: jlong,
    ) -> jboolean {
        let world = convert_ptr!(world_handle => World);
        let entity = convert_jlong_to_entity!(entity);

        let result = world.get::<&KCC>(entity).is_ok();
        if result { 1 } else { 0 }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_physics_KinematicCharacterControllerNative_moveCharacter(
        mut env: JNIEnv,
        _: JClass,
        world_handle: jlong,
        physics_handle: jlong,
        entity: jlong,
        translation: JObject,
        delta_time: jdouble,
    ) {
        let world = convert_ptr!(world_handle => World);
        let physics_state = convert_ptr!(mut physics_handle => PhysicsState);
        let entity = convert_jlong_to_entity!(entity);

        let movement = match Vector3::from_jobject(&mut env, &translation) {
            Ok(v) => v,
            Err(e) => {
                let _ = env.throw_new("java/lang/RuntimeException", format!("Unable to convert JObject to Vector3: {}", e));
                return;
            }
        };

        if let Ok(mut q) = world.query_one::<(&Label, &KCC)>(entity)
            && let Some((label, kcc)) = q.get()
        {
            let Some(rigid_body_handle) = physics_state.bodies_entity_map.get(label) else {
                return;
            };

            let (body_type, body_pos) = {
                let Some(body) = physics_state.bodies.get(*rigid_body_handle) else {
                    return;
                };

                (body.body_type(), *body.position())
            };

            match body_type {
                RigidBodyType::KinematicPositionBased => {}
                _ => return,
            }

            let Some(collider_handles) = physics_state.colliders_entity_map.get(label) else {
                return;
            };
            let Some((_, collider_handle)) = collider_handles.first() else {
                return;
            };
            let Some(collider) = physics_state.colliders.get(*collider_handle) else {
                return;
            };

            let character_pos = if let Some(pos_wrt_parent) = collider.position_wrt_parent() {
                body_pos * (*pos_wrt_parent)
            } else {
                *collider.position()
            };

            let filter = QueryFilter::default().exclude_rigid_body(*rigid_body_handle);
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
                rapier3d::prelude::Vector::new(movement.x as f32, movement.y as f32, movement.z as f32),
                |collision| {
                    if let Some(collisions) = physics_state.collision_events_to_deal_with.get_mut(&entity) {
                        collisions.push(collision)
                    } else {
                        physics_state.collision_events_to_deal_with.insert(entity, vec![collision]);
                    }
                },
            );

            if let Some(body) = physics_state.bodies.get_mut(*rigid_body_handle) {
                let current_pos = body.translation();
                let new_pos = current_pos + movement.translation;
                body.set_next_kinematic_translation(new_pos);
            }
        }
    }
}