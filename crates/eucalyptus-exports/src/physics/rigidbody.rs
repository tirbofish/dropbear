use eucalyptus_core::physics::PhysicsState;
use eucalyptus_core::physics::rigidbody::AxisLock;
use eucalyptus_core::physics::rigidbody::shared;
use eucalyptus_core::ptr::{PhysicsStatePtr, WorldPtr};
use eucalyptus_core::rapier3d::prelude::RigidBodyType;
use eucalyptus_core::scripting::native::DropbearNativeError;
use eucalyptus_core::scripting::result::DropbearNativeResult;
use eucalyptus_core::types::{IndexNative, NCollider, NVector3, RigidBodyContext};
use jni::objects::{JObject, JValue};
use jni::{Env, jni_sig, jni_str};
use crate::{FromJObject, ToJObject};

// ---------------------------------------------------- AxisLock JNI impls --

impl FromJObject for AxisLock {
    fn from_jobject(env: &mut Env, obj: &JObject) -> DropbearNativeResult<Self>
    where
        Self: Sized,
    {
        let class = env
            .load_class(jni_str!("com/dropbear/physics/AxisLock"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        if !env
            .is_instance_of(obj, &class)
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?
        {
            return Err(DropbearNativeError::InvalidArgument);
        }

        let x = env
            .get_field(obj, jni_str!("x"), jni_sig!(boolean))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .z()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let y = env
            .get_field(obj, jni_str!("y"), jni_sig!(boolean))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .z()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let z = env
            .get_field(obj, jni_str!("z"), jni_sig!(boolean))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .z()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        Ok(AxisLock { x, y, z })
    }
}

impl ToJObject for AxisLock {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .load_class(jni_str!("com/dropbear/physics/AxisLock"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let args = [
            JValue::Bool(self.x),
            JValue::Bool(self.y),
            JValue::Bool(self.z),
        ];

        let obj = env
            .new_object(&class, jni_sig!((boolean, boolean, boolean) -> void), &args)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        Ok(obj)
    }
}

// ------------------------------------------------ RigidBodyNative exports --

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.RigidBodyNative",
        func = "rigidBodyExistsForEntity"
    ),
    c
)]
fn exists_for_entity(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &PhysicsState,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<Option<IndexNative>> {
    Ok(shared::rigid_body_exists_for_entity(world, physics, entity))
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.RigidBodyNative",
        func = "getRigidBodyMode"
    ),
    c
)]
fn get_rigidbody_mode(
    #[dropbear_macro::define(WorldPtr)] _world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &PhysicsState,
    rigidbody: &RigidBodyContext,
) -> DropbearNativeResult<i32> {
    let body_type = shared::get_rigidbody_type(physics, rigidbody)?;
    Ok(match body_type {
        RigidBodyType::Dynamic => 0,
        RigidBodyType::Fixed => 1,
        RigidBodyType::KinematicPositionBased => 2,
        RigidBodyType::KinematicVelocityBased => 3,
    })
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.RigidBodyNative",
        func = "setRigidBodyMode"
    ),
    c
)]
fn set_rigidbody_mode(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &mut PhysicsState,
    rigidbody: &RigidBodyContext,
    mode: i32,
) -> DropbearNativeResult<()> {
    shared::set_rigidbody_type(physics, world, rigidbody, mode as i64)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.RigidBodyNative",
        func = "getRigidBodyGravityScale"
    ),
    c
)]
fn get_rigidbody_gravity_scale(
    #[dropbear_macro::define(WorldPtr)] _world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &PhysicsState,
    rigidbody: &RigidBodyContext,
) -> DropbearNativeResult<f64> {
    shared::get_rigidbody_gravity_scale(physics, rigidbody)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.RigidBodyNative",
        func = "setRigidBodyGravityScale"
    ),
    c
)]
fn set_rigidbody_gravity_scale(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &mut PhysicsState,
    rigidbody: &RigidBodyContext,
    gravity_scale: f64,
) -> DropbearNativeResult<()> {
    shared::set_rigidbody_gravity_scale(physics, world, rigidbody, gravity_scale)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.RigidBodyNative",
        func = "getRigidBodyLinearDamping"
    ),
    c
)]
fn get_rigidbody_linear_damping(
    #[dropbear_macro::define(WorldPtr)] _world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &PhysicsState,
    rigidbody: &RigidBodyContext,
) -> DropbearNativeResult<f64> {
    shared::get_rigidbody_linear_damping(physics, rigidbody)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.RigidBodyNative",
        func = "setRigidBodyLinearDamping"
    ),
    c
)]
fn set_rigidbody_linear_damping(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &mut PhysicsState,
    rigidbody: &RigidBodyContext,
    linear_damping: f64,
) -> DropbearNativeResult<()> {
    shared::set_rigidbody_linear_damping(physics, world, rigidbody, linear_damping)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.RigidBodyNative",
        func = "getRigidBodyAngularDamping"
    ),
    c
)]
fn get_rigidbody_angular_damping(
    #[dropbear_macro::define(WorldPtr)] _world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &PhysicsState,
    rigidbody: &RigidBodyContext,
) -> DropbearNativeResult<f64> {
    shared::get_rigidbody_angular_damping(physics, rigidbody)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.RigidBodyNative",
        func = "setRigidBodyAngularDamping"
    ),
    c
)]
fn set_rigidbody_angular_damping(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &mut PhysicsState,
    rigidbody: &RigidBodyContext,
    angular_damping: f64,
) -> DropbearNativeResult<()> {
    shared::set_rigidbody_angular_damping(physics, world, rigidbody, angular_damping)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.RigidBodyNative",
        func = "getRigidBodySleep"
    ),
    c
)]
fn get_rigidbody_sleep(
    #[dropbear_macro::define(WorldPtr)] _world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &PhysicsState,
    rigidbody: &RigidBodyContext,
) -> DropbearNativeResult<bool> {
    shared::get_rigidbody_sleep(physics, rigidbody)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.RigidBodyNative",
        func = "setRigidBodySleep"
    ),
    c
)]
fn set_rigidbody_sleep(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &mut PhysicsState,
    rigidbody: &RigidBodyContext,
    sleep: bool,
) -> DropbearNativeResult<()> {
    shared::set_rigidbody_sleep(physics, world, rigidbody, sleep)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.RigidBodyNative",
        func = "getRigidBodyCcdEnabled"
    ),
    c
)]
fn get_rigidbody_ccd_enabled(
    #[dropbear_macro::define(WorldPtr)] _world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &PhysicsState,
    rigidbody: &RigidBodyContext,
) -> DropbearNativeResult<bool> {
    shared::get_rigidbody_ccd(physics, rigidbody)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.RigidBodyNative",
        func = "setRigidBodyCcdEnabled"
    ),
    c
)]
fn set_rigidbody_ccd_enabled(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &mut PhysicsState,
    rigidbody: &RigidBodyContext,
    ccd_enabled: bool,
) -> DropbearNativeResult<()> {
    shared::set_rigidbody_ccd(physics, world, rigidbody, ccd_enabled)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.RigidBodyNative",
        func = "getRigidBodyLinearVelocity"
    ),
    c
)]
fn get_rigidbody_linear_velocity(
    #[dropbear_macro::define(WorldPtr)] _world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &PhysicsState,
    rigidbody: &RigidBodyContext,
) -> DropbearNativeResult<NVector3> {
    shared::get_rigidbody_linvel(physics, rigidbody)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.RigidBodyNative",
        func = "setRigidBodyLinearVelocity"
    ),
    c
)]
fn set_rigidbody_linear_velocity(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &mut PhysicsState,
    rigidbody: &RigidBodyContext,
    linear_velocity: &NVector3,
) -> DropbearNativeResult<()> {
    shared::set_rigidbody_linvel(physics, world, rigidbody, *linear_velocity)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.RigidBodyNative",
        func = "getRigidBodyAngularVelocity"
    ),
    c
)]
fn get_rigidbody_angular_velocity(
    #[dropbear_macro::define(WorldPtr)] _world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &PhysicsState,
    rigidbody: &RigidBodyContext,
) -> DropbearNativeResult<NVector3> {
    shared::get_rigidbody_angvel(physics, rigidbody)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.RigidBodyNative",
        func = "setRigidBodyAngularVelocity"
    ),
    c
)]
fn set_rigidbody_angular_velocity(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &mut PhysicsState,
    rigidbody: &RigidBodyContext,
    angular_velocity: &NVector3,
) -> DropbearNativeResult<()> {
    shared::set_rigidbody_angvel(physics, world, rigidbody, *angular_velocity)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.RigidBodyNative",
        func = "getRigidBodyLockTranslation"
    ),
    c
)]
fn get_rigidbody_lock_translation(
    #[dropbear_macro::define(WorldPtr)] _world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &PhysicsState,
    rigidbody: &RigidBodyContext,
) -> DropbearNativeResult<AxisLock> {
    shared::get_rigidbody_lock_translation(physics, rigidbody)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.RigidBodyNative",
        func = "setRigidBodyLockTranslation"
    ),
    c
)]
fn set_rigidbody_lock_translation(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &mut PhysicsState,
    rigidbody: &RigidBodyContext,
    lock_translation: &AxisLock,
) -> DropbearNativeResult<()> {
    shared::set_rigidbody_lock_translation(physics, world, rigidbody, *lock_translation)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.RigidBodyNative",
        func = "getRigidBodyLockRotation"
    ),
    c
)]
fn get_rigidbody_lock_rotation(
    #[dropbear_macro::define(WorldPtr)] _world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &PhysicsState,
    rigidbody: &RigidBodyContext,
) -> DropbearNativeResult<AxisLock> {
    shared::get_rigidbody_lock_rotation(physics, rigidbody)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.RigidBodyNative",
        func = "setRigidBodyLockRotation"
    ),
    c
)]
fn set_rigidbody_lock_rotation(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &mut PhysicsState,
    rigidbody: &RigidBodyContext,
    lock_rotation: &AxisLock,
) -> DropbearNativeResult<()> {
    shared::set_rigidbody_lock_rotation(physics, world, rigidbody, *lock_rotation)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.RigidBodyNative",
        func = "getRigidBodyChildren"
    ),
    c
)]
fn get_rigidbody_children(
    #[dropbear_macro::define(WorldPtr)] _world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &PhysicsState,
    rigidbody: &RigidBodyContext,
) -> DropbearNativeResult<Vec<NCollider>> {
    let children = shared::get_rigidbody_children(physics, rigidbody)?;
    let colliders = children
        .into_iter()
        .map(|handle| {
            let (idx, generation) = handle.into_raw_parts();
            NCollider {
                index: IndexNative {
                    index: idx,
                    generation,
                },
                entity_id: rigidbody.entity_id,
                id: idx,
            }
        })
        .collect();

    Ok(colliders)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.RigidBodyNative", func = "applyImpulse"),
    c
)]
fn apply_impulse(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &mut PhysicsState,
    rigidbody: &RigidBodyContext,
    x: f64,
    y: f64,
    z: f64,
) -> DropbearNativeResult<()> {
    let impulse = NVector3::new(x, y, z);
    shared::apply_impulse(physics, world, rigidbody, impulse)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.RigidBodyNative",
        func = "applyTorqueImpulse"
    ),
    c
)]
fn apply_torque_impulse(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &mut PhysicsState,
    rigidbody: &RigidBodyContext,
    x: f64,
    y: f64,
    z: f64,
) -> DropbearNativeResult<()> {
    let torque = NVector3::new(x, y, z);
    shared::apply_torque_impulse(physics, world, rigidbody, torque)
}
