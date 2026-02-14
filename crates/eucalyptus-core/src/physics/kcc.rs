//! Module that relates to the [Kinematic Character Controller](https://rapier.rs/docs/user_guides/rust/character_controller)
//! (or kcc for short) in the [rapier3d] physics engine.

pub mod character_collision;

use crate::traits::SerializableComponent;
use rapier3d::control::{CharacterCollision, KinematicCharacterController};
use serde::{Deserialize, Serialize};
use dropbear_macro::SerializableComponent;
use crate::states::Label;

/// The kinematic character controller (kcc) component.
#[derive(Debug, Default, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct KCC {
    pub entity: Label,
    pub controller: KinematicCharacterController,
    #[serde(skip)]
    pub collisions: Vec<CharacterCollision>,
}

impl KCC {
    pub fn new(label: &Label) -> Self {
        KCC {
            entity: label.clone(),
            controller: KinematicCharacterController::default(),
            collisions: vec![],
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
    use rapier3d::math::Rotation;
    use rapier3d::prelude::QueryFilter;
    use crate::{convert_jlong_to_entity, convert_ptr};
    use crate::physics::kcc::KCC;
    use crate::physics::PhysicsState;
    use crate::scripting::jni::utils::FromJObject;
    use crate::states::Label;
    use crate::types::Vector3;
    use jni::objects::JValue;
    use jni::sys::{jint, jobjectArray};
    use crate::types::IndexNative;
    use crate::scripting::jni::utils::ToJObject;

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_physics_KinematicCharacterControllerNative_existsForEntity(
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
    pub extern "system" fn Java_com_dropbear_physics_KinematicCharacterControllerNative_moveCharacter(
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

        if let Ok((label, kcc)) = world.query_one::<(&Label, &KCC)>(entity).get()
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

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_physics_KinematicCharacterControllerNative_setRotation(
        mut env: JNIEnv,
        _: JClass,
        world_handle: jlong,
        physics_handle: jlong,
        entity: jlong,
        rotation: JObject,
    ) {
        let world = convert_ptr!(world_handle => World);
        let physics_state = convert_ptr!(mut physics_handle => PhysicsState);
        let entity = convert_jlong_to_entity!(entity);

        let class = match env.find_class("com/dropbear/math/Quaterniond") {
            Ok(cls) => cls,
            Err(_) => {
                let _ = env.throw_new("java/lang/RuntimeException", "Unable to find Quaterniond class");
                return;
            }
        };

        if let Ok(false) = env.is_instance_of(&rotation, &class) {
            let _ = env.throw_new("java/lang/IllegalArgumentException", "rotation must be Quaterniond");
            return;
        }

        let mut get_double = |field: &str| -> Option<f64> {
            env.get_field(&rotation, field, "D").ok()?.d().ok()
        };

        let Some(rx) = get_double("x") else { return; };
        let Some(ry) = get_double("y") else { return; };
        let Some(rz) = get_double("z") else { return; };
        let Some(rw) = get_double("w") else { return; };

        let len = (rx * rx + ry * ry + rz * rz + rw * rw).sqrt();
        let (x, y, z, w) = if len > 0.0 {
            (rx / len, ry / len, rz / len, rw / len)
        } else {
            (0.0, 0.0, 0.0, 1.0)
        };

        if let Ok((label, _kcc)) = world.query_one::<(&Label, &KCC)>(entity).get() {
            let Some(rigid_body_handle) = physics_state.bodies_entity_map.get(label) else {
                return;
            };

            let body_type = {
                let Some(body) = physics_state.bodies.get(*rigid_body_handle) else {
                    return;
                };
                body.body_type()
            };

            match body_type {
                RigidBodyType::KinematicPositionBased => {}
                _ => return,
            }

            if let Some(body) = physics_state.bodies.get_mut(*rigid_body_handle) {
                let rot = Rotation::from_xyzw(x as f32, y as f32, z as f32, w as f32);
                body.set_next_kinematic_rotation(rot);
            }
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_physics_KinematicCharacterControllerNative_getHitNative(
        mut env: JNIEnv,
        _: JClass,
        world_handle: jlong,
        entity: jlong,
    ) -> jobjectArray {
        let world = convert_ptr!(world_handle => World);
        let entity = convert_jlong_to_entity!(entity);

        let Ok(kcc) = world.get::<&KCC>(entity) else {
            return std::ptr::null_mut();
        };

        let collision_cls = match env.find_class("com/dropbear/physics/CharacterCollision") {
            Ok(cls) => cls,
            Err(e) => {
                eprintln!("[JNI Error] Could not find CharacterCollision class: {:?}", e);
                return std::ptr::null_mut();
            }
        };

        let entity_cls = match env.find_class("com/dropbear/EntityId") {
            Ok(cls) => cls,
            Err(e) => {
                eprintln!("[JNI Error] Could not find EntityId class: {:?}", e);
                return std::ptr::null_mut();
            }
        };

        let out = match env.new_object_array(kcc.collisions.len() as jint, &collision_cls, JObject::null()) {
            Ok(arr) => arr,
            Err(e) => {
                eprintln!("[JNI Error] Failed to allocate CharacterCollision array: {:?}", e);
                let _ = env.throw_new("java/lang/OutOfMemoryError", "Could not allocate CharacterCollision array");
                return std::ptr::null_mut();
            }
        };

        let entity_id_obj = match env.new_object(&entity_cls, "(J)V", &[JValue::Long(entity.to_bits().get() as i64)]) {
            Ok(obj) => obj,
            Err(e) => {
                eprintln!("[JNI Error] Failed to create EntityId object: {:?}", e);
                return std::ptr::null_mut();
            }
        };

        for (i, _) in kcc.collisions.iter().enumerate() {
            let collision = &kcc.collisions[i];
            let (idx, generation) = collision.handle.into_raw_parts();
            let index_obj = match (IndexNative {
                index: idx,
                generation,
            })
            .to_jobject(&mut env)
            {
                Ok(obj) => obj,
                Err(e) => {
                    eprintln!("[JNI Error] Failed to create Index object: {e}");
                    return std::ptr::null_mut();
                }
            };

            let collision_obj = match env.new_object(
                &collision_cls,
                "(Lcom/dropbear/EntityId;Lcom/dropbear/physics/Index;)V",
                &[JValue::Object(&entity_id_obj), JValue::Object(&index_obj)],
            ) {
                Ok(obj) => obj,
                Err(e) => {
                    eprintln!("[JNI Error] Failed to create CharacterCollision object: {:?}", e);
                    return std::ptr::null_mut();
                }
            };

            if let Err(e) = env.set_object_array_element(&out, i as jint, collision_obj) {
                eprintln!("[JNI Error] Failed to set array element: {:?}", e);
                return std::ptr::null_mut();
            }
        }

        out.into_raw()
    }
}