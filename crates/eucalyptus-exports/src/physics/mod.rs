pub use eucalyptus_core::types::{
    CollisionEvent, CollisionEventType, ContactForceEvent, IndexNative, NCollider, NShapeCastHit,
    NShapeCastStatus, RayHit, RigidBodyContext,
};

pub mod physics;

use jni::objects::JObject;
use jni::sys::jdouble;
use jni::{jni_sig, jni_str, Env, JValue};
use eucalyptus_core::physics::kcc::CharacterMovementResult;
use eucalyptus_core::scripting::native::DropbearNativeError;
use eucalyptus_core::scripting::result::DropbearNativeResult;
use crate::{FromJObject, ToJObject};
use crate::math::NVector3;

pub mod collider;
pub mod kcc;
pub mod rigidbody;

// ----------------------------------------------------------- CharacterCollision

/// Returned by `getHit` — an entity's KCC collision handles packed for JNI transfer.
pub struct CharacterCollisionArray {
    pub entity_id: u64,
    pub collisions: Vec<IndexNative>,
}

// ------------------------------------------------------- NCollider JNI impls --

impl FromJObject for NCollider {
    fn from_jobject(env: &mut Env, obj: &JObject) -> DropbearNativeResult<Self> {
        let index_obj = env
            .get_field(obj, jni_str!("index"), jni_sig!(com.dropbear.physics.Index))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let entity_obj = env
            .get_field(obj, jni_str!("entity"), jni_sig!(com.dropbear.EntityId))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let id_val = env
            .get_field(obj, jni_str!("id"), jni_sig!(int))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .i()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let entity_raw = env
            .get_field(&entity_obj, jni_str!("raw"), jni_sig!(long))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .j()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let idx_val = env
            .get_field(&index_obj, jni_str!("index"), jni_sig!(int))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .i()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let gen_val = env
            .get_field(&index_obj, jni_str!("generation"), jni_sig!(int))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .i()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        Ok(NCollider {
            index: IndexNative {
                index: idx_val as u32,
                generation: gen_val as u32,
            },
            entity_id: entity_raw as u64,
            id: id_val as u32,
        })
    }
}

impl ToJObject for NCollider {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let collider_cls = env
            .load_class(jni_str!("com/dropbear/physics/Collider"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let index_cls = env
            .load_class(jni_str!("com/dropbear/physics/Index"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let entity_cls = env
            .load_class(jni_str!("com/dropbear/EntityId"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let entity_obj = env
            .new_object(
                &entity_cls,
                jni_sig!((long) -> void),
                &[JValue::Long(self.entity_id as i64)],
            )
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        let index_obj = env
            .new_object(
                &index_cls,
                jni_sig!((int, int) -> void),
                &[
                    JValue::Int(self.index.index as i32),
                    JValue::Int(self.index.generation as i32),
                ],
            )
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        let collider_obj = env
            .new_object(
                collider_cls,
                jni_sig!((com.dropbear.physics.Index, com.dropbear.EntityId, int) -> void),
                &[
                    JValue::Object(&index_obj),
                    JValue::Object(&entity_obj),
                    JValue::Int(self.id as i32),
                ],
            )
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        Ok(collider_obj)
    }
}

// ----------------------------------------------- CollisionEventType JNI impl -

impl ToJObject for CollisionEventType {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .load_class(jni_str!("com/dropbear/physics/CollisionEventType"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let name = match self {
            CollisionEventType::Started => "Started",
            CollisionEventType::Stopped => "Stopped",
        };
        let name_jstring = env
            .new_string(name)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        env.call_static_method(
            class,
            jni_str!("valueOf"),
            jni_sig!((java.lang.String) -> com.dropbear.physics.CollisionEventType),
            &[JValue::from(&name_jstring)],
        )
        .map_err(|_| DropbearNativeError::JNIMethodNotFound)?
        .l()
        .map_err(|_| DropbearNativeError::JNIUnwrapFailed)
    }
}

// -------------------------------------------------- CollisionEvent JNI impl --

impl ToJObject for CollisionEvent {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .load_class(jni_str!("com/dropbear/physics/CollisionEvent"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let event_type = self.event_type.to_jobject(env)?;
        let collider1 = self.collider1.to_jobject(env)?;
        let collider2 = self.collider2.to_jobject(env)?;

        let flags = self.flags as i32;
        env.new_object(
            class,
            jni_sig!("(Lcom/dropbear/physics/CollisionEventType;Lcom/dropbear/physics/Collider;Lcom/dropbear/physics/Collider;I)V"),
            &[
                JValue::Object(&event_type),
                JValue::Object(&collider1),
                JValue::Object(&collider2),
                JValue::Int(flags),
            ],
        )
        .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
    }
}

// ----------------------------------------------- ContactForceEvent JNI impl --

impl ToJObject for ContactForceEvent {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .load_class(jni_str!("com/dropbear/physics/ContactForceEvent"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let collider1 = self.collider1.to_jobject(env)?;
        let collider2 = self.collider2.to_jobject(env)?;
        let total_force = self.total_force.to_jobject(env)?;
        let max_force_direction = self.max_force_direction.to_jobject(env)?;

        env.new_object(
            class,
            jni_sig!("(Lcom/dropbear/physics/Collider;Lcom/dropbear/physics/Collider;Lcom/dropbear/math/Vector3d;DLcom/dropbear/math/Vector3d;D)V"),
            &[
                JValue::Object(&collider1),
                JValue::Object(&collider2),
                JValue::Object(&total_force),
                JValue::Double(self.total_force_magnitude),
                JValue::Object(&max_force_direction),
                JValue::Double(self.max_force_magnitude),
            ],
        )
        .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
    }
}

// ------------------------------------------------ NShapeCastHit JNI impl -----

impl ToJObject for NShapeCastHit {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let collider = self.collider.to_jobject(env)?;
        let witness1 = self.witness1.to_jobject(env)?;
        let witness2 = self.witness2.to_jobject(env)?;
        let normal1 = self.normal1.to_jobject(env)?;
        let normal2 = self.normal2.to_jobject(env)?;
        let status = self.status.to_jobject(env)?;

        let class = env
            .load_class(jni_str!("com/dropbear/physics/ShapeCastHit"))
            .map_err(|_| {
                eprintln!("[JNI Error] Failed to find ShapeCastHit class");
                DropbearNativeError::JNIClassNotFound
            })?;

        env.new_object(
            class,
            jni_sig!("(Lcom/dropbear/physics/Collider;DLcom/dropbear/math/Vector3d;Lcom/dropbear/math/Vector3d;Lcom/dropbear/math/Vector3d;Lcom/dropbear/math/Vector3d;Lcom/dropbear/physics/ShapeCastStatus;)V"),
            &[
                JValue::Object(&collider),
                JValue::Double(self.distance as jdouble),
                JValue::Object(&witness1),
                JValue::Object(&witness2),
                JValue::Object(&normal1),
                JValue::Object(&normal2),
                JValue::Object(&status),
            ],
        )
        .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
    }
}

// ------------------------------------------------ NShapeCastStatus JNI impl --

impl ToJObject for NShapeCastStatus {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .load_class(jni_str!("com/dropbear/physics/ShapeCastStatus"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let name = match self {
            NShapeCastStatus::OutOfIterations => "OutOfIterations",
            NShapeCastStatus::Converged => "Converged",
            NShapeCastStatus::Failed => "Failed",
            NShapeCastStatus::PenetratingOrWithinTargetDist => "PenetratingOrWithinTargetDist",
        };

        let name_jstring = env
            .new_string(name)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        env.call_static_method(
            class,
            jni_str!("valueOf"),
            jni_sig!((java.lang.String) -> com.dropbear.physics.ShapeCastStatus),
            &[JValue::from(&name_jstring)],
        )
        .map_err(|_| DropbearNativeError::JNIMethodNotFound)?
        .l()
        .map_err(|_| DropbearNativeError::JNIUnwrapFailed)
    }
}

// ------------------------------------------------ IndexNative JNI impls -------

impl FromJObject for IndexNative {
    fn from_jobject(env: &mut Env, obj: &JObject) -> DropbearNativeResult<Self> {
        let idx_val = env
            .get_field(obj, jni_str!("index"), jni_sig!(int))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .i()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let gen_val = env
            .get_field(obj, jni_str!("generation"), jni_sig!(int))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .i()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        Ok(IndexNative {
            index: idx_val as u32,
            generation: gen_val as u32,
        })
    }
}

impl ToJObject for IndexNative {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let cls = env
            .load_class(jni_str!("com/dropbear/physics/Index"))
            .map_err(|_| DropbearNativeError::GenericError)?;

        env.new_object(
            cls,
            jni_sig!((int, int) -> void),
            &[
                JValue::Int(self.index as i32),
                JValue::Int(self.generation as i32),
            ],
        )
        .map_err(|_| DropbearNativeError::GenericError)
    }
}

impl ToJObject for Option<IndexNative> {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        match self {
            Some(v) => v.to_jobject(env),
            None => Ok(JObject::null()),
        }
    }
}

// ------------------------------------------------ RigidBodyContext JNI impls -

impl FromJObject for RigidBodyContext {
    fn from_jobject(env: &mut Env, obj: &JObject) -> DropbearNativeResult<Self> {
        let index_obj = env
            .get_field(obj, jni_str!("index"), jni_sig!(com.dropbear.physics.Index))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let idx_val = env
            .get_field(&index_obj, jni_str!("index"), jni_sig!(int))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .i()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let gen_val = env
            .get_field(&index_obj, jni_str!("generation"), jni_sig!(int))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .i()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let entity_obj = env
            .get_field(obj, jni_str!("entity"), jni_sig!(com.dropbear.EntityId))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let entity_raw = env
            .get_field(&entity_obj, jni_str!("raw"), jni_sig!(long))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .j()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        Ok(RigidBodyContext {
            index: IndexNative {
                index: idx_val as u32,
                generation: gen_val as u32,
            },
            entity_id: entity_raw as u64,
        })
    }
}

impl ToJObject for RigidBodyContext {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let index_cls = env
            .load_class(jni_str!("com/dropbear/physics/Index"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let entity_cls = env
            .load_class(jni_str!("com/dropbear/EntityId"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let rb_cls = env
            .load_class(jni_str!("com/dropbear/physics/RigidBody"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let index_obj = env
            .new_object(
                &index_cls,
                jni_sig!((int, int) -> void),
                &[
                    JValue::Int(self.index.index as i32),
                    JValue::Int(self.index.generation as i32),
                ],
            )
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        let entity_obj = env
            .new_object(
                &entity_cls,
                jni_sig!((long) -> void),
                &[JValue::Long(self.entity_id as i64)],
            )
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        env.new_object(
            rb_cls,
            jni_sig!((com.dropbear.physics.Index, com.dropbear.EntityId) -> void),
            &[JValue::Object(&index_obj), JValue::Object(&entity_obj)],
        )
        .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
    }
}

// ---------------------------------------------------- RayHit JNI impl ---------

impl ToJObject for RayHit {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let collider = self.collider.to_jobject(env)?;

        let class = env
            .load_class(jni_str!("com/dropbear/physics/RayHit"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        env.new_object(
            class,
            jni_sig!((com.dropbear.physics.Collider, double) -> void),
            &[
                JValue::Object(&collider),
                JValue::Double(self.distance as jdouble),
            ],
        )
        .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
    }
}

// ------------------------------------- CharacterMovementResult JNI impl -------

impl ToJObject for CharacterMovementResult {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .load_class(jni_str!("com/dropbear/physics/CharacterMovementResult"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let translation_obj = NVector3::from(self.translation).to_jobject(env)?;

        env.new_object(
            &class,
            jni_sig!((com.dropbear.math.Vector3d, boolean, boolean) -> void),
            &[
                JValue::Object(&translation_obj),
                JValue::Bool(self.grounded),
                JValue::Bool(self.is_sliding_down_slope),
            ],
        )
        .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
    }
}

// ------------------------------------- CharacterCollisionArray JNI impl -------

impl ToJObject for CharacterCollisionArray {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let collision_cls = env
            .load_class(jni_str!("com/dropbear/physics/CharacterCollision"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let entity_cls = env
            .load_class(jni_str!("com/dropbear/EntityId"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let entity_obj = env
            .new_object(
                &entity_cls,
                jni_sig!((long) -> void),
                &[JValue::Long(self.entity_id as i64)],
            )
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        let out = env
            .new_object_array(
                self.collisions.len() as i32,
                &collision_cls,
                JObject::null(),
            )
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        for (i, handle) in self.collisions.iter().enumerate() {
            let index_obj = handle.to_jobject(env)?;
            let collision_obj = env
                .new_object(
                    &collision_cls,
                    jni_sig!((com.dropbear.EntityId, com.dropbear.physics.Index) -> void),
                    &[JValue::Object(&entity_obj), JValue::Object(&index_obj)],
                )
                .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

            out.set_element(env, i, collision_obj)
                .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;
        }

        Ok(JObject::from(out))
    }
}
