use crate::convert_ptr;
use crate::physics::collider::{Collider, ColliderGroup};
use crate::physics::PhysicsState;
use hecs::World;
use jni::objects::{JClass, JObject};
use jni::sys::jlong;
use jni::JNIEnv;
use rapier3d::na::{UnitQuaternion, Vector3};
use rapier3d::prelude::AngVector;

/**
 * Class:     `com_dropbear_ffi_components_ColliderNative`
 *
 * Method:    `setCollider`
 *
 * Signature: `(JLcom/dropbear/physics/Collider;)V`
 *
 * `JNIEXPORT void JNICALL Java_com_dropbear_ffi_components_ColliderNative_setCollider
 * (JNIEnv *, jclass, jlong, jlong, jobject);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_ColliderNative_setCollider(
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    physics_handle: jlong,
    collider_obj: JObject,
) {
    let world = convert_ptr!(mut world_handle => World);
    let physics = convert_ptr!(mut physics_handle => PhysicsState);

    Collider::from_jni_object(&mut env, &collider_obj).and_then(|(handle, collider)| {
        if let Some(rapier_collider) = physics.colliders.get_mut(handle) {
            rapier_collider.set_friction(collider.friction);
            rapier_collider.set_restitution(collider.restitution);
            rapier_collider.set_density(collider.density);
            rapier_collider.set_sensor(collider.is_sensor);

            let translation = Vector3::from_column_slice(&collider.translation);
            let rotation = UnitQuaternion::from_euler_angles(
                collider.rotation[0],
                collider.rotation[1],
                collider.rotation[2]
            );

            rapier_collider.set_translation_wrt_parent(translation);
            let rot = rotation.euler_angles();
            rapier_collider.set_rotation_wrt_parent(AngVector::new(rot.0, rot.1, rot.2));

            let temp_collider = collider.to_rapier();
            rapier_collider.set_shape(temp_collider.shared_shape().clone());
        }

        if let Some(ecs_entity) = collider.entity.locate_entity(&world) {
            if let Ok(mut group) = world.get::<&mut ColliderGroup>(ecs_entity) {
                let mut success = false;
                for c in group.colliders.iter_mut() {
                    if c.id == collider.id {
                        *c = collider.clone();
                        success = true;
                        break;
                    }
                }
                if success {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Failed to find Collider [{}] in ColliderGroup", collider.id))
                }
            } else {
                Err(anyhow::anyhow!("Failed to get ColliderGroup from ECS"))
            }
        } else {
            Err(anyhow::anyhow!("Failed to locate entity in ECS"))
        }
    }).ok();
}