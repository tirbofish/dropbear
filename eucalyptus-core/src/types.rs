//! FFI and C types of other library types, as used in the scripting module.
use glam::{DQuat, DVec3};
use jni::JNIEnv;
use jni::objects::{JObject, JValue};
use jni::sys::jdouble;
use rapier3d::data::Index;
use dropbear_engine::entity::Transform;
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