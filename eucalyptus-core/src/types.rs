//! FFI and C types of other library types, as used in the scripting module.
use glam::{DQuat, DVec3};
use hecs::Entity;
use jni::JNIEnv;
use jni::objects::{JObject, JValue};
use jni::sys::jdouble;
use rapier3d::data::Index;
use rapier3d::prelude::ColliderHandle;
use dropbear_engine::entity::Transform;
use crate::physics::PhysicsState;
use crate::scripting::jni::utils::{FromJObject, ToJObject};
use crate::scripting::native::DropbearNativeError;
use crate::scripting::result::DropbearNativeResult;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    pub fn new(x: f64, y: f64, z: f64) -> Vector3 {
        Vector3 {
            x, y, z
        }
    }

    pub fn to_array(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
    pub fn to_float_array(&self) -> [f32; 3] {
        [self.x as f32, self.y as f32, self.z as f32]
    }
}

impl From<glam::DVec3> for Vector3 {
    fn from(v: glam::DVec3) -> Self {
        Self { x: v.x, y: v.y, z: v.z }
    }
}

impl From<[f64; 3]> for Vector3 {
    fn from(value: [f64; 3]) -> Self {
        Vector3 {
            x: value[0],
            y: value[1],
            z: value[2],
        }
    }
}

impl From<[f32; 3]> for Vector3 {
    fn from(value: [f32; 3]) -> Self {
        Vector3 {
            x: value[0] as f64,
            y: value[1] as f64,
            z: value[2] as f64,
        }
    }
}

impl From<Vector3> for glam::DVec3 {
    fn from(v: Vector3) -> Self {
        Self::new(v.x, v.y, v.z)
    }
}

impl FromJObject for Vector3 {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self>
    where
        Self: Sized
    {
        let class = env.find_class("com/dropbear/math/Vector3d")
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        if !env.is_instance_of(obj, &class)
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?
        {
            return Err(DropbearNativeError::InvalidArgument);
        }

        let x = env.get_field(obj, "x", "D")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .d()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let y = env.get_field(obj, "y", "D")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .d()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let z = env.get_field(obj, "z", "D")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .d()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        Ok(Vector3::new(x, y, z))
    }
}

impl ToJObject for Vector3 {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env.find_class("com/dropbear/math/Vector3d")
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let constructor_sig = "(DDD)V";

        let args = [
            jni::objects::JValue::Double(self.x),
            jni::objects::JValue::Double(self.y),
            jni::objects::JValue::Double(self.z),
        ];

        let obj = env.new_object(&class, constructor_sig, &args)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        Ok(obj)
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Quaternion {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl From<DQuat> for Quaternion {
    fn from(value: DQuat) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
            w: value.w,
        }
    }
}

impl From<Quaternion> for glam::DQuat {
    fn from(value: Quaternion) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
            w: value.w,
        }
    }
}

impl From<[f64; 4]> for Quaternion {
    fn from(value: [f64; 4]) -> Self {
        Self {
            x: value[0],
            y: value[1],
            z: value[2],
            w: value[3],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TransformNative {
    position: Vector3,
    rotation: Quaternion,
    scale: Vector3,
}

impl From<Transform> for TransformNative {
    fn from(value: Transform) -> Self {
        Self {
            position: Vector3::from(value.position),
            rotation: Quaternion::from(value.rotation),
            scale: Vector3::from(value.scale),
        }
    }
}

impl From<TransformNative> for Transform {
    fn from(value: TransformNative) -> Self {
        Self {
            position: DVec3::from(value.position),
            rotation: DQuat::from(value.rotation),
            scale: DVec3::from(value.scale),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vector2 {
    pub(crate) x: f64,
    pub(crate) y: f64,
}

impl Vector2 {
    pub fn to_array(&self) -> [f64; 2] {
        [self.x, self.y]
    }
}

impl From<glam::DVec2> for Vector2 {
    fn from(v: glam::DVec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl From<[f64; 2]> for Vector2 {
    fn from(value: [f64; 2]) -> Self {
        Vector2 {
            x: value[0],
            y: value[1],
        }
    }
}

impl From<Vector2> for glam::DVec2 {
    fn from(v: Vector2) -> Self {
        Self::new(v.x, v.y)
    }
}

impl From<(f64, f64)> for Vector2 {
    fn from(value: (f64, f64)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ColliderFFI {
    pub index: IndexNative,
    pub entity_id: u64,
    pub id: u32,
}

impl ToJObject for ColliderFFI {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let collider_cls = env.find_class("com/dropbear/physics/Collider")
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let index_cls = env.find_class("com/dropbear/physics/Index")
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let entity_cls = env.find_class("com/dropbear/EntityId")
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let entity_obj = env.new_object(
            &entity_cls,
            "(J)V",
            &[JValue::Long(self.entity_id as i64)]
        ).map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        let index_obj = env.new_object(
            &index_cls,
            "(II)V",
            &[
                JValue::Int(self.index.index as i32),
                JValue::Int(self.index.generation as i32)
            ]
        ).map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        let collider_obj = env.new_object(
            collider_cls,
            "(Lcom/dropbear/physics/Index;Lcom/dropbear/EntityId;I)V",
            &[
                JValue::Object(&index_obj),
                JValue::Object(&entity_obj),
                JValue::Int(self.id as i32)
            ]
        ).map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        Ok(collider_obj)
    }
}

impl FromJObject for ColliderFFI {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self>
    where
        Self: Sized
    {
        let index_obj = env.get_field(obj, "index", "Lcom/dropbear/physics/Index;")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let entity_obj = env.get_field(obj, "entity", "Lcom/dropbear/EntityId;")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let id_val = env.get_field(obj, "id", "I")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .i()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let entity_raw = env.get_field(&entity_obj, "raw", "J")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .j()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let idx_val = env.get_field(&index_obj, "index", "I")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .i()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let gen_val = env.get_field(&index_obj, "generation", "I")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .i()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        Ok(ColliderFFI {
            index: IndexNative {
                index: idx_val as u32,
                generation: gen_val as u32,
            },
            entity_id: entity_raw as u64,
            id: id_val as u32,
        })
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct IndexNative {
    pub(crate) index: u32,
    pub(crate) generation: u32,
}

impl From<Index> for IndexNative {
    fn from(value: Index) -> Self {
        let raw = value.into_raw_parts();
        Self {
            index: raw.0,
            generation: raw.1,
        }
    }
}

impl From<IndexNative> for Index {
    fn from(value: IndexNative) -> Self {
        Self::from_raw_parts(value.index, value.generation)
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum ColliderShapeType {
    Box = 0,
    Sphere = 1,
    Capsule = 2,
    Cylinder = 3,
    Cone = 4,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ColliderShapeFFI {
    pub shape_type: ColliderShapeType,
    pub radius: f32,
    pub half_height: f32,
    pub half_extents_x: f32,
    pub half_extents_y: f32,
    pub half_extents_z: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RigidBodyContext {
    pub index: IndexNative,
    pub entity_id: u64,
}

impl FromJObject for RigidBodyContext {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self>
    where
        Self: Sized
    {
        let index_obj = env.get_field(obj, "index", "Lcom/dropbear/physics/Index;")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let idx_val = env.get_field(&index_obj, "index", "I")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .i()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let gen_val = env.get_field(&index_obj, "generation", "I")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .i()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let entity_obj = env.get_field(obj, "entity", "Lcom/dropbear/EntityId;")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let entity_raw = env.get_field(&entity_obj, "raw", "J")
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
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let index_cls = env.find_class("com/dropbear/physics/Index")
            .map_err(|e| {
                eprintln!("[JNI Error] Class 'com/dropbear/physics/Index' not found: {:?}", e);
                DropbearNativeError::JNIClassNotFound
            })?;

        let entity_cls = env.find_class("com/dropbear/EntityId")
            .map_err(|e| {
                eprintln!("[JNI Error] Class 'com/dropbear/EntityId' not found: {:?}", e);
                DropbearNativeError::JNIClassNotFound
            })?;

        let rb_cls = env.find_class("com/dropbear/physics/RigidBody")
            .map_err(|e| {
                eprintln!("[JNI Error] Class 'com/dropbear/physics/RigidBody' not found: {:?}", e);
                DropbearNativeError::JNIClassNotFound
            })?;

        let index_obj = env.new_object(
            &index_cls,
            "(II)V",
            &[
                JValue::Int(self.index.index as i32),
                JValue::Int(self.index.generation as i32)
            ]
        ).map_err(|e| {
            eprintln!("[JNI Error] Failed to create Index object: {:?}", e);
            DropbearNativeError::JNIFailedToCreateObject
        })?;

        let entity_obj = env.new_object(
            &entity_cls,
            "(J)V",
            &[JValue::Long(self.entity_id as i64)]
        ).map_err(|e| {
            eprintln!("[JNI Error] Failed to create EntityId object: {:?}", e);
            DropbearNativeError::JNIFailedToCreateObject
        })?;

        let rb_obj = env.new_object(
            rb_cls,
            "(Lcom/dropbear/physics/Index;Lcom/dropbear/EntityId;)V",
            &[
                JValue::Object(&index_obj),
                JValue::Object(&entity_obj)
            ]
        ).map_err(|e| {
            eprintln!("[JNI Error] Failed to create RigidBody object: {:?}", e);
            DropbearNativeError::JNIFailedToCreateObject
        })?;

        Ok(rb_obj)
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RayHit {
    pub collider: ColliderFFI,
    pub distance: f64,
}

impl ToJObject for RayHit {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let collider = self.collider.to_jobject(env)?;
        let distance = self.distance as jdouble;

        let class = env.find_class("com/dropbear/physics/RayHit").map_err(|e| {
            eprintln!("[JNI Error] Failed to create RayHit object: {:?}", e);
            DropbearNativeError::JNIClassNotFound
        })?;

        let object = env.new_object(
            class,
            "(Lcom/dropbear/physics/Collider;D)V",
            &[
                JValue::Object(&collider),
                JValue::Double(distance),
            ]
        ).map_err(|e| {
            eprintln!("[JNI Error] Failed to create RayHit object: {:?}", e);
            DropbearNativeError::JNIFailedToCreateObject
        })?;

        Ok(object)
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
/// Class: `com.dropbear.physics.CollisionEventType`
pub enum CollisionEventType {
    Started,
    Stopped
}

impl ToJObject for CollisionEventType {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env.find_class("com/dropbear/physics/CollisionEventType")
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let name = match self {
            CollisionEventType::Started => "Started",
            CollisionEventType::Stopped => "Stopped",
        };
        let name_jstring = env.new_string(name)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        let value = env
            .call_static_method(
                class,
                "valueOf",
                "(Ljava/lang/String;)Lcom/dropbear/physics/CollisionEventType;",
                &[JValue::Object(&name_jstring)],
            )
            .map_err(|_| DropbearNativeError::JNIMethodNotFound)?
            .l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        Ok(value)
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CollisionEvent {
    pub(crate) event_type: CollisionEventType,
    pub(crate) collider1: ColliderFFI,
    pub(crate) collider2: ColliderFFI,
    pub(crate) flags: u64,
}

impl CollisionEvent {
    pub fn collider1_entity_id(&self) -> u64 {
        self.collider1.entity_id
    }

    pub fn collider2_entity_id(&self) -> u64 {
        self.collider2.entity_id
    }
}

impl CollisionEvent {
    pub fn from_rapier3d(
        physics: &PhysicsState,
        value: rapier3d::geometry::CollisionEvent,
    ) -> Option<CollisionEvent> {
        match value {
            rapier3d::prelude::CollisionEvent::Started(col1, col2, flags) => {
                let collider1_info = physics.colliders_entity_map.iter().find_map(|(l, s)| {
                    for (_, h) in s {
                        if col1 == *h {
                            return Some(l.clone());
                        }
                    }
                    None
                }).and_then(|label| {
                    physics.entity_label_map.iter().find_map(|(e, l)| {
                        if l == &label {
                            Some(*e)
                        } else {
                            None
                        }
                    })
                })?;

                let collider2_info = physics.colliders_entity_map.iter().find_map(|(l, s)| {
                    for (_, h) in s {
                        if col2 == *h {
                            return Some(l.clone());
                        }
                    }
                    None
                }).and_then(|label| {
                    physics.entity_label_map.iter().find_map(|(e, l)| {
                        if l == &label {
                            Some(*e)
                        } else {
                            None
                        }
                    })
                })?;
                
                Some(Self {
                    event_type: CollisionEventType::Started,
                    collider1: ColliderFFI {
                        index: IndexNative::from(col1.0),
                        entity_id: collider1_info.to_bits().get(),
                        id: col1.into_raw_parts().0,
                    },
                    collider2: ColliderFFI {
                        index: IndexNative::from(col2.0),
                        entity_id: collider2_info.to_bits().get(),
                        id: col2.into_raw_parts().0,
                    },
                    flags: flags.bits() as u64,
                })
            }
            rapier3d::prelude::CollisionEvent::Stopped(col1, col2, flags) => {
                let collider1_info = physics.colliders_entity_map.iter().find_map(|(l, s)| {
                    for (_, h) in s {
                        if col1 == *h {
                            return Some(l.clone());
                        }
                    }
                    None
                }).and_then(|label| {
                    physics.entity_label_map.iter().find_map(|(e, l)| {
                        if l == &label {
                            Some(*e)
                        } else {
                            None
                        }
                    })
                })?;

                let collider2_info = physics.colliders_entity_map.iter().find_map(|(l, s)| {
                    for (_, h) in s {
                        if col2 == *h {
                            return Some(l.clone());
                        }
                    }
                    None
                }).and_then(|label| {
                    physics.entity_label_map.iter().find_map(|(e, l)| {
                        if l == &label {
                            Some(*e)
                        } else {
                            None
                        }
                    })
                })?;

                Some(Self {
                    event_type: CollisionEventType::Stopped,
                    collider1: ColliderFFI {
                        index: IndexNative::from(col1.0),
                        entity_id: collider1_info.to_bits().get(),
                        id: col1.into_raw_parts().0,
                    },
                    collider2: ColliderFFI {
                        index: IndexNative::from(col2.0),
                        entity_id: collider2_info.to_bits().get(),
                        id: col2.into_raw_parts().0,
                    },
                    flags: flags.bits() as u64,
                })
            }
        }
    }
}

impl ToJObject for CollisionEvent {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env.find_class("com/dropbear/physics/CollisionEvent")
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let event_type = self.event_type.to_jobject(env)?;
        let collider1 = self.collider1.to_jobject(env)?;
        let collider2 = self.collider2.to_jobject(env)?;

        let flags = self.flags as i32;
        let obj = env
            .new_object(
                class,
                "(Lcom/dropbear/physics/CollisionEventType;Lcom/dropbear/physics/Collider;Lcom/dropbear/physics/Collider;I)V",
                &[
                    JValue::Object(&event_type),
                    JValue::Object(&collider1),
                    JValue::Object(&collider2),
                    JValue::Int(flags),
                ],
            )
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        Ok(obj)
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ContactForceEvent {
    pub(crate) collider1: ColliderFFI,
    pub(crate) collider2: ColliderFFI,
    pub(crate) total_force: Vector3,
    pub(crate) total_force_magnitude: f64,
    pub(crate) max_force_direction: Vector3,
    pub(crate) max_force_magnitude: f64,
}

impl ContactForceEvent {
    pub fn collider1_entity_id(&self) -> u64 {
        self.collider1.entity_id
    }

    pub fn collider2_entity_id(&self) -> u64 {
        self.collider2.entity_id
    }
}

impl ContactForceEvent {
    pub fn from_rapier3d(
        physics: &PhysicsState,
        event: rapier3d::prelude::ContactForceEvent,
    ) -> Option<ContactForceEvent> {
        let find_entity = |collider_handle: ColliderHandle| -> Option<Entity> {
            Some(physics.colliders_entity_map.iter().find_map(|(l, s)| {
                for (_, h) in s {
                    if collider_handle == *h {
                        return Some(l.clone());
                    }
                }
                None
            }).and_then(|label| {
                physics.entity_label_map.iter().find_map(|(e, l)| {
                    if l == &label {
                        Some(*e)
                    } else {
                        None
                    }
                })
            })?)
        };
        
        Some(Self {
            collider1: ColliderFFI {
                index: IndexNative::from(event.collider1.0),
                entity_id: find_entity(event.collider1)?.to_bits().get(),
                id: event.collider1.into_raw_parts().0,
            },
            collider2: ColliderFFI {
                index: IndexNative::from(event.collider2.0),
                entity_id: find_entity(event.collider2)?.to_bits().get(),
                id: event.collider2.into_raw_parts().0,
            },
            total_force: Vector3::new(
                event.total_force.x.into(), 
                event.total_force.y.into(), 
                event.total_force.z.into()
            ),
            total_force_magnitude: event.total_force_magnitude as f64,
            max_force_direction: Vector3::new(
                event.max_force_direction.x.into(), 
                event.max_force_direction.y.into(), 
                event.max_force_direction.z.into()
            ),
            max_force_magnitude: event.max_force_magnitude as f64,
        })
    }
}

impl ToJObject for ContactForceEvent {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env.find_class("com/dropbear/physics/ContactForceEvent")
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let collider1 = self.collider1.to_jobject(env)?;
        let collider2 = self.collider2.to_jobject(env)?;
        let total_force = self.total_force.to_jobject(env)?;
        let max_force_direction = self.max_force_direction.to_jobject(env)?;

        let obj = env
            .new_object(
                class,
                "(Lcom/dropbear/physics/Collider;Lcom/dropbear/physics/Collider;Lcom/dropbear/math/Vector3d;DLcom/dropbear/math/Vector3d;D)V",
                &[
                    JValue::Object(&collider1),
                    JValue::Object(&collider2),
                    JValue::Object(&total_force),
                    JValue::Double(self.total_force_magnitude),
                    JValue::Object(&max_force_direction),
                    JValue::Double(self.max_force_magnitude),
                ],
            )
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        Ok(obj)
    }
}
