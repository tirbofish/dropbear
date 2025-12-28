use hecs::World;
use jni::JNIEnv;
use jni::objects::{JClass, JObject};
use jni::sys::{jdouble, jlong, jobjectArray};
use rapier3d::na::Vector3;
use rapier3d::prelude::RigidBodyType;
use crate::{convert_jlong_to_entity, convert_ptr};
use crate::physics::PhysicsState;
use crate::physics::rigidbody::{RigidBody, RigidBodyMode};
use crate::states::Label;

/**
 * Class:     `com_dropbear_ffi_components_RigidBodyNative`
 *
 * Method:    `applyImpulse`
 *
 * Signature: `(JLcom/dropbear/physics/Index;DDD)V`
 *
 * `JNIEXPORT void JNICALL Java_com_dropbear_ffi_components_RigidBodyNative_applyImpulse
 * (JNIEnv *, jclass, jlong, jobject, jdouble, jdouble, jdouble);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_RigidBodyNative_applyImpulse(
    mut env: JNIEnv,
    _class: JClass,
    physics_handle: jlong,
    index_obj: JObject,
    x: jdouble,
    y: jdouble,
    z: jdouble,
) {
    // TODO: Implementation
}

/**
 * Class:     `com_dropbear_ffi_components_RigidBodyNative`
 *
 * Method:    `applyTorqueImpulse`
 *
 * Signature: `(JLcom/dropbear/physics/Index;DDD)V`
 *
 * `JNIEXPORT void JNICALL Java_com_dropbear_ffi_components_RigidBodyNative_applyTorqueImpulse
 * (JNIEnv *, jclass, jlong, jobject, jdouble, jdouble, jdouble);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_RigidBodyNative_applyTorqueImpulse(
    mut env: JNIEnv,
    _class: JClass,
    physics_handle: jlong,
    index_obj: JObject,
    x: jdouble,
    y: jdouble,
    z: jdouble,
) {
    // TODO: Implementation
}

/**
 * Class:     `com_dropbear_ffi_components_RigidBodyNative`
 *
 * Method:    `setRigidBody`
 *
 * Signature: `(JJLcom/dropbear/physics/RigidBody;)V`
 *
 * `JNIEXPORT void JNICALL Java_com_dropbear_ffi_components_RigidBodyNative_setRigidBody
 * (JNIEnv *, jclass, jlong, jlong, jobject);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_RigidBodyNative_setRigidBody(
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    physics_handle: jlong,
    rigidbody_obj: JObject,
) {
    let world = convert_ptr!(mut world_handle => World);
    let physics = convert_ptr!(mut physics_handle => PhysicsState);

    RigidBody::from_jni(&mut env, &rigidbody_obj).map(|(handle, rb_data)| {
        if let Some(rapier_body) = physics.bodies.get_mut(handle) {
            let rapier_mode = match rb_data.mode {
                RigidBodyMode::Dynamic => RigidBodyType::Dynamic,
                RigidBodyMode::Fixed => RigidBodyType::Fixed,
                RigidBodyMode::KinematicPosition => RigidBodyType::KinematicPositionBased,
                RigidBodyMode::KinematicVelocity => RigidBodyType::KinematicVelocityBased,
            };
            rapier_body.set_body_type(rapier_mode, true);

            rapier_body.set_gravity_scale(rb_data.gravity_scale, true);
            rapier_body.set_linvel(Vector3::from(rb_data.linvel), true);
            rapier_body.set_angvel(Vector3::from(rb_data.angvel), true);
            rapier_body.set_linear_damping(rb_data.linear_damping);
            rapier_body.set_angular_damping(rb_data.angular_damping);

            if !rb_data.can_sleep {
                rapier_body.wake_up(true);
            }
            rapier_body.enable_ccd(rb_data.ccd_enabled);

            rapier_body.set_enabled_translations(
                !rb_data.lock_translation.x,
                !rb_data.lock_translation.y,
                !rb_data.lock_translation.z,
                true
            );
            rapier_body.set_enabled_rotations(
                !rb_data.lock_rotation.x,
                !rb_data.lock_rotation.y,
                !rb_data.lock_rotation.z,
                true
            );
        } else {
            eprintln!("Attempted to set RigidBody with invalid handle: {:?}", handle);
        }

        if let Some(ecs_entity) = rb_data.entity.locate_entity(&world) {
            if let Ok(mut component) = world.get::<&mut RigidBody>(ecs_entity) {
                *component = rb_data;
            }
        }
    }).ok();
}

/**
 * Class:     `com_dropbear_ffi_components_RigidBodyNative`
 *
 * Method:    `getChildColliders`
 *
 * Signature: `(JJLcom/dropbear/physics/Index;)[Lcom/dropbear/physics/Collider;`
 *
 * `JNIEXPORT jobjectArray JNICALL Java_com_dropbear_ffi_components_RigidBodyNative_getChildColliders
 * (JNIEnv *, jclass, jlong, jlong, jobject);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_RigidBodyNative_getChildColliders(
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    physics_handle: jlong,
    index_obj: JObject,
) -> jobjectArray {
    // TODO: Implementation
    std::ptr::null_mut()
}