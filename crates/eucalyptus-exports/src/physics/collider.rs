use glam::{DQuat, Vec3};
use jni::{jni_sig, jni_str, Env, JValue};
use jni::objects::JObject;
use jni::sys::jdouble;
use eucalyptus_core::physics::collider::ColliderShape;
use eucalyptus_core::physics::PhysicsState;
use eucalyptus_core::ptr::PhysicsStatePtr;
use eucalyptus_core::rapier3d::geometry::{SharedShape, TypedShape};
use eucalyptus_core::rapier3d::math::{Rotation, Vector};
use eucalyptus_core::scripting::native::DropbearNativeError;
use eucalyptus_core::scripting::result::DropbearNativeResult;
use crate::{FromJObject, ToJObject};
use crate::physics::NCollider;
use crate::math::NVector3;
use crate::physics::collider::shared::{get_collider, get_collider_mut};

mod group {
    use eucalyptus_core::physics::collider::ColliderGroup;
    use eucalyptus_core::physics::PhysicsState;
    use eucalyptus_core::ptr::{PhysicsStatePtr, WorldPtr};
    use eucalyptus_core::scripting::native::DropbearNativeError;
    use eucalyptus_core::scripting::result::DropbearNativeResult;
    use crate::physics::{IndexNative, NCollider};

    #[dropbear_macro::export(
        kotlin(
            class = "com.dropbear.physics.ColliderGroupNative",
            func = "colliderGroupExistsForEntity"
        ),
        c
    )]
    fn exists_for_entity(
        #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
        #[dropbear_macro::entity] entity: hecs::Entity,
    ) -> DropbearNativeResult<bool> {
        Ok(world.get::<&ColliderGroup>(entity).is_ok())
    }

    #[dropbear_macro::export(
        kotlin(
            class = "com.dropbear.physics.ColliderGroupNative",
            func = "getColliderGroupColliders"
        ),
        c
    )]
    fn get_colliders(
        #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
        #[dropbear_macro::define(PhysicsStatePtr)] physics: &PhysicsState,
        #[dropbear_macro::entity] entity: hecs::Entity,
    ) -> DropbearNativeResult<Vec<NCollider>> {
        if world.get::<&ColliderGroup>(entity).is_ok() {
            let handles_opt = physics
                .entity_label_map
                .get(&entity)
                .and_then(|label| physics.colliders_entity_map.get(label));

            let mut colliders: Vec<NCollider> = Vec::new();

            if let Some(handles) = handles_opt {
                for (_, handle) in handles {
                    let (idx, generation) = handle.into_raw_parts();

                    let col = NCollider {
                        index: IndexNative {
                            index: idx,
                            generation,
                        },
                        entity_id: entity.to_bits().get(),
                        id: idx,
                    };
                    colliders.push(col);
                }
            }

            Ok(colliders)
        } else {
            Err(DropbearNativeError::MissingComponent)?
        }
    }
}

impl ToJObject for ColliderShape {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        match self {
            ColliderShape::Box { half_extents } => {
                let vec_cls = env
                    .load_class(jni_str!("com/dropbear/math/Vector3d"))
                    .map_err(|e| {
                        eprintln!("[JNI Error] Vector3d class not found: {:?}", e);
                        DropbearNativeError::JNIClassNotFound
                    })?;

                let vec_obj = env
                    .new_object(
                        &vec_cls,
                        jni_sig!("(DDD)V"),
                        &[
                            JValue::Double(half_extents.x as jdouble),
                            JValue::Double(half_extents.y as jdouble),
                            JValue::Double(half_extents.z as jdouble),
                        ],
                    )
                    .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

                let cls = env
                    .load_class(jni_str!("com/dropbear/physics/ColliderShape$Box"))
                    .map_err(|e| {
                        eprintln!("[JNI Error] ColliderShape$Box class not found: {:?}", e);
                        DropbearNativeError::JNIClassNotFound
                    })?;

                let obj = env
                    .new_object(
                        &cls,
                        jni_sig!("(Lcom/dropbear/math/Vector3d;)V"),
                        &[JValue::Object(&vec_obj)],
                    )
                    .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

                Ok(obj)
            }
            ColliderShape::Sphere { radius } => {
                let cls = env
                    .load_class(jni_str!("com/dropbear/physics/ColliderShape$Sphere"))
                    .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

                let obj = env
                    .new_object(&cls, jni_sig!("(F)V"), &[JValue::Float(*radius)])
                    .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

                Ok(obj)
            }
            ColliderShape::Capsule {
                half_height,
                radius,
            } => {
                let cls = env
                    .load_class(jni_str!("com/dropbear/physics/ColliderShape$Capsule"))
                    .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

                let obj = env
                    .new_object(
                        &cls,
                        jni_sig!("(FF)V"),
                        &[JValue::Float(*half_height), JValue::Float(*radius)],
                    )
                    .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

                Ok(obj)
            }
            ColliderShape::Cylinder {
                half_height,
                radius,
            } => {
                let cls = env
                    .load_class(jni_str!("com/dropbear/physics/ColliderShape$Cylinder"))
                    .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

                // let ctor = env.get_method_id(&cls, "<init>", "(FF)V")
                //     .map_err(|_| DropbearNativeError::JNIMethodNotFound)?;

                let obj = env
                    .new_object(
                        &cls,
                        jni_sig!("(FF)V"),
                        &[JValue::Float(*half_height), JValue::Float(*radius)],
                    )
                    .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

                Ok(obj)
            }
            ColliderShape::Cone {
                half_height,
                radius,
            } => {
                let cls = env
                    .load_class(jni_str!("com/dropbear/physics/ColliderShape$Cone"))
                    .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

                let obj = env
                    .new_object(
                        &cls,
                        jni_sig!("(FF)V"),
                        &[JValue::Float(*half_height), JValue::Float(*radius)],
                    )
                    .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

                Ok(obj)
            }
        }
    }
}

impl FromJObject for ColliderShape {
    fn from_jobject(env: &mut Env, obj: &JObject) -> DropbearNativeResult<Self>
    where
        Self: Sized,
    {
        let is_instance = |env: &mut Env,
                           obj: &JObject,
                           class_name: &jni::strings::JNIStr|
                           -> bool { env.is_instance_of(obj, class_name).unwrap_or(false) };

        if is_instance(env, obj, jni_str!("com/dropbear/physics/ColliderShape$Box")) {
            let vec_obj_val = env
                .get_field(
                    obj,
                    jni_str!("halfExtents"),
                    jni_sig!("Lcom/dropbear/math/Vector3d;"),
                )
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?;
            let vec_obj = vec_obj_val
                .l()
                .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

            let x = env
                .get_field(&vec_obj, jni_str!("x"), jni_sig!("D"))
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
                .d()
                .unwrap_or(0.0);
            let y = env
                .get_field(&vec_obj, jni_str!("y"), jni_sig!("D"))
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
                .d()
                .unwrap_or(0.0);
            let z = env
                .get_field(&vec_obj, jni_str!("z"), jni_sig!("D"))
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
                .d()
                .unwrap_or(0.0);

            return Ok(ColliderShape::Box {
                half_extents: Vec3::from([x as f32, y as f32, z as f32]),
            });
        }

        if is_instance(
            env,
            obj,
            jni_str!("com/dropbear/physics/ColliderShape$Sphere"),
        ) {
            let radius = env
                .get_field(obj, jni_str!("radius"), jni_sig!("F"))
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
                .f()
                .unwrap_or(0.0);

            return Ok(ColliderShape::Sphere { radius });
        }

        if is_instance(
            env,
            obj,
            jni_str!("com/dropbear/physics/ColliderShape$Capsule"),
        ) {
            let hh = env
                .get_field(obj, jni_str!("halfHeight"), jni_sig!("F"))
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
                .f()
                .unwrap_or(0.0);
            let r = env
                .get_field(obj, jni_str!("radius"), jni_sig!("F"))
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
                .f()
                .unwrap_or(0.0);

            return Ok(ColliderShape::Capsule {
                half_height: hh,
                radius: r,
            });
        }

        if is_instance(
            env,
            obj,
            jni_str!("com/dropbear/physics/ColliderShape$Cylinder"),
        ) {
            let hh = env
                .get_field(obj, jni_str!("halfHeight"), jni_sig!("F"))
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
                .f()
                .unwrap_or(0.0);
            let r = env
                .get_field(obj, jni_str!("radius"), jni_sig!("F"))
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
                .f()
                .unwrap_or(0.0);

            return Ok(ColliderShape::Cylinder {
                half_height: hh,
                radius: r,
            });
        }

        if is_instance(
            env,
            obj,
            jni_str!("com/dropbear/physics/ColliderShape$Cone"),
        ) {
            let hh = env
                .get_field(obj, jni_str!("halfHeight"), jni_sig!("F"))
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
                .f()
                .unwrap_or(0.0);
            let r = env
                .get_field(obj, jni_str!("radius"), jni_sig!("F"))
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
                .f()
                .unwrap_or(0.0);

            return Ok(ColliderShape::Cone {
                half_height: hh,
                radius: r,
            });
        }

        Err(DropbearNativeError::GenericError)
    }
}

// -------------------------------- collider ------------------------------------

pub mod shared {
    use eucalyptus_core::physics::PhysicsState;
    use eucalyptus_core::types::NCollider;
    use eucalyptus_core::rapier3d::prelude::ColliderHandle;

    pub fn get_collider_mut<'a>(
        physics: &'a mut PhysicsState,
        ffi: &NCollider,
    ) -> Option<&'a mut eucalyptus_core::rapier3d::prelude::Collider> {
        let handle = ColliderHandle::from_raw_parts(ffi.index.index, ffi.index.generation);
        physics.colliders.get_mut(handle)
    }

    pub fn get_collider<'a>(
        physics: &'a PhysicsState,
        ffi: &NCollider,
    ) -> Option<&'a eucalyptus_core::rapier3d::prelude::Collider> {
        let handle = ColliderHandle::from_raw_parts(ffi.index.index, ffi.index.generation);
        physics.colliders.get(handle)
    }
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.ColliderNative",
        func = "getColliderShape"
    ),
    c
)]
fn get_collider_shape(
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &PhysicsState,
    collider: &NCollider,
) -> DropbearNativeResult<ColliderShape> {
    let collider =
        get_collider(physics, &collider).ok_or(DropbearNativeError::PhysicsObjectNotFound)?;

    let rapier_shape = collider.shape();
    let my_shape = match rapier_shape.as_typed_shape() {
        TypedShape::Cuboid(c) => {
            let he = c.half_extents;
            ColliderShape::Box {
                half_extents: glam::Vec3::from([he.x, he.y, he.z]),
            }
        }
        TypedShape::Ball(b) => ColliderShape::Sphere { radius: b.radius },
        TypedShape::Capsule(c) => {
            let height = c.segment.length();
            ColliderShape::Capsule {
                half_height: height * 0.5,
                radius: c.radius,
            }
        }
        TypedShape::Cylinder(c) => ColliderShape::Cylinder {
            half_height: c.half_height,
            radius: c.radius,
        },
        TypedShape::Cone(c) => ColliderShape::Cone {
            half_height: c.half_height,
            radius: c.radius,
        },
        _ => return Err(DropbearNativeError::InvalidArgument),
    };

    Ok(my_shape)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.ColliderNative",
        func = "setColliderShape"
    ),
    c
)]
fn set_collider_shape(
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &mut PhysicsState,
    collider: &NCollider,
    shape: &ColliderShape,
) -> DropbearNativeResult<()> {
    let collider =
        get_collider_mut(physics, &collider).ok_or(DropbearNativeError::PhysicsObjectNotFound)?;

    let new_shape = match shape {
        ColliderShape::Box { half_extents } => SharedShape::cuboid(
            half_extents.x as f32,
            half_extents.y as f32,
            half_extents.z as f32,
        ),
        ColliderShape::Sphere { radius } => SharedShape::ball(*radius),
        ColliderShape::Capsule {
            half_height,
            radius,
        } => SharedShape::capsule_y(*half_height, *radius),
        ColliderShape::Cylinder {
            half_height,
            radius,
        } => SharedShape::cylinder(*half_height, *radius),
        ColliderShape::Cone {
            half_height,
            radius,
        } => SharedShape::cone(*half_height, *radius),
    };

    collider.set_shape(new_shape);
    Ok(())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.ColliderNative",
        func = "getColliderDensity"
    ),
    c
)]
fn get_collider_density(
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &PhysicsState,
    collider: &NCollider,
) -> DropbearNativeResult<f64> {
    let collider =
        get_collider(physics, &collider).ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    Ok(collider.density() as f64)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.ColliderNative",
        func = "setColliderDensity"
    ),
    c
)]
fn set_collider_density(
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &mut PhysicsState,
    collider: &NCollider,
    density: f64,
) -> DropbearNativeResult<()> {
    let collider =
        get_collider_mut(physics, &collider).ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    collider.set_density(density as f32);
    Ok(())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.ColliderNative",
        func = "getColliderFriction"
    ),
    c
)]
fn get_collider_friction(
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &PhysicsState,
    collider: &NCollider,
) -> DropbearNativeResult<f64> {
    let collider =
        get_collider(physics, &collider).ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    Ok(collider.friction() as f64)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.ColliderNative",
        func = "setColliderFriction"
    ),
    c
)]
fn set_collider_friction(
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &mut PhysicsState,
    collider: &NCollider,
    friction: f64,
) -> DropbearNativeResult<()> {
    let collider =
        get_collider_mut(physics, &collider).ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    collider.set_friction(friction as f32);
    Ok(())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.ColliderNative",
        func = "getColliderRestitution"
    ),
    c
)]
fn get_collider_restitution(
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &PhysicsState,
    collider: &NCollider,
) -> DropbearNativeResult<f64> {
    let collider =
        get_collider(physics, &collider).ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    Ok(collider.restitution() as f64)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.ColliderNative",
        func = "setColliderRestitution"
    ),
    c
)]
fn set_collider_restitution(
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &mut PhysicsState,
    collider: &NCollider,
    restitution: f64,
) -> DropbearNativeResult<()> {
    let collider =
        get_collider_mut(physics, &collider).ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    collider.set_restitution(restitution as f32);
    Ok(())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.ColliderNative",
        func = "getColliderMass"
    ),
    c
)]
fn get_collider_mass(
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &PhysicsState,
    collider: &NCollider,
) -> DropbearNativeResult<f64> {
    let collider =
        get_collider(physics, &collider).ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    Ok(collider.mass() as f64)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.ColliderNative",
        func = "setColliderMass"
    ),
    c
)]
fn set_collider_mass(
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &mut PhysicsState,
    collider: &NCollider,
    mass: f64,
) -> DropbearNativeResult<()> {
    let collider =
        get_collider_mut(physics, &collider).ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    collider.set_mass(mass as f32);
    Ok(())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.ColliderNative",
        func = "getColliderIsSensor"
    ),
    c
)]
fn get_collider_is_sensor(
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &PhysicsState,
    collider: &NCollider,
) -> DropbearNativeResult<bool> {
    let collider =
        get_collider(physics, &collider).ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    Ok(collider.is_sensor())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.ColliderNative",
        func = "setColliderIsSensor"
    ),
    c
)]
fn set_collider_is_sensor(
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &mut PhysicsState,
    collider: &NCollider,
    is_sensor: bool,
) -> DropbearNativeResult<()> {
    let collider =
        get_collider_mut(physics, &collider).ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    collider.set_sensor(is_sensor);
    Ok(())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.ColliderNative",
        func = "getColliderTranslation"
    ),
    c
)]
fn get_collider_translation(
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &PhysicsState,
    collider: &NCollider,
) -> DropbearNativeResult<NVector3> {
    let collider =
        get_collider(physics, &collider).ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    let t: Vector = collider.translation();
    Ok(NVector3::new(t.x as f64, t.y as f64, t.z as f64))
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.ColliderNative",
        func = "setColliderTranslation"
    ),
    c
)]
fn set_collider_translation(
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &mut PhysicsState,
    collider: &NCollider,
    translation: &NVector3,
) -> DropbearNativeResult<()> {
    let collider =
        get_collider_mut(physics, &collider).ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    let t = Vector::new(
        translation.x as f32,
        translation.y as f32,
        translation.z as f32,
    );
    collider.set_translation(t);
    Ok(())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.ColliderNative",
        func = "getColliderRotation"
    ),
    c
)]
fn get_collider_rotation(
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &PhysicsState,
    collider: &NCollider,
) -> DropbearNativeResult<NVector3> {
    let collider =
        get_collider(physics, &collider).ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    let r: Rotation = collider.rotation();
    let q = DQuat::from_xyzw(r.x as f64, r.y as f64, r.z as f64, r.w as f64);
    let euler = q.to_euler(glam::EulerRot::XYZ);
    Ok(NVector3::new(euler.0, euler.1, euler.2))
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.ColliderNative",
        func = "setColliderRotation"
    ),
    c
)]
fn set_collider_rotation(
    #[dropbear_macro::define(PhysicsStatePtr)] physics: &mut PhysicsState,
    collider: &NCollider,
    rotation: &NVector3,
) -> DropbearNativeResult<()> {
    let collider =
        get_collider_mut(physics, &collider).ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    let q = DQuat::from_euler(glam::EulerRot::XYZ, rotation.x, rotation.y, rotation.z);
    let r = Rotation::from_array([q.w as f32, q.x as f32, q.y as f32, q.z as f32]);
    collider.set_rotation(r);
    Ok(())
}