//! FFI and C types of other library types, as used in the scripting module.
use glam::{DQuat, DVec3, Vec3};
use hecs::Entity;
use jni::JNIEnv;
use jni::objects::{JObject, JValue};
use jni::sys::jdouble;
use rapier3d::data::Index;
use rapier3d::parry::query::{ShapeCastOptions, ShapeCastStatus};
use rapier3d::prelude::ColliderHandle;
use dropbear_engine::entity::Transform;
use crate::physics::PhysicsState;
use crate::scripting::jni::utils::{FromJObject, ToJObject};
use crate::scripting::native::DropbearNativeError;
use crate::scripting::result::DropbearNativeResult;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NVector2 {
    pub x: f64,
    pub y: f64,
}

impl NVector2 {
    pub fn to_array(&self) -> [f64; 2] {
        [self.x, self.y]
    }
}

impl From<glam::DVec2> for NVector2 {
    fn from(v: glam::DVec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl From<[f64; 2]> for NVector2 {
    fn from(value: [f64; 2]) -> Self {
        NVector2 {
            x: value[0],
            y: value[1],
        }
    }
}

impl From<[f32; 2]> for NVector2 {
    fn from(value: [f32; 2]) -> Self {
        NVector2 {
            x: value[0] as f64,
            y: value[1] as f64,
        }
    }
}

impl From<NVector2> for glam::DVec2 {
    fn from(v: NVector2) -> Self {
        Self::new(v.x, v.y)
    }
}

impl From<(f64, f64)> for NVector2 {
    fn from(value: (f64, f64)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NVector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl NVector3 {
    pub fn new(x: f64, y: f64, z: f64) -> NVector3 {
        NVector3 {
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

impl From<glam::DVec3> for NVector3 {
    fn from(v: glam::DVec3) -> Self {
        Self { x: v.x, y: v.y, z: v.z }
    }
}

impl From<&glam::DVec3> for NVector3 {
    fn from(v: &glam::DVec3) -> Self {
        Self { x: v.x, y: v.y, z: v.z }
    }
}

impl From<&NVector3> for glam::DVec3 {
    fn from(v: &NVector3) -> Self {
        Self { x: v.x, y: v.y, z: v.z }
    }
}

impl From<[f64; 3]> for NVector3 {
    fn from(value: [f64; 3]) -> Self {
        NVector3 {
            x: value[0],
            y: value[1],
            z: value[2],
        }
    }
}

impl From<[f32; 3]> for NVector3 {
    fn from(value: [f32; 3]) -> Self {
        NVector3 {
            x: value[0] as f64,
            y: value[1] as f64,
            z: value[2] as f64,
        }
    }
}

impl From<glam::Vec3> for NVector3 {
    fn from(value: Vec3) -> Self {
        Self {
            x: value.x as f64,
            y: value.y as f64,
            z: value.z as f64,
        }
    }
}

impl From<NVector3> for glam::DVec3 {
    fn from(v: NVector3) -> Self {
        Self::new(v.x, v.y, v.z)
    }
}

impl FromJObject for NVector3 {
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

        Ok(NVector3::new(x, y, z))
    }
}

impl ToJObject for NVector3 {
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
#[derive(Clone, Copy, Debug)]
pub struct NVector4 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl NVector4 {
    pub fn new(x: f64, y: f64, z: f64, w: f64) -> NVector4 {
        NVector4 {
            x, y, z, w
        }
    }

    pub fn to_array(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
    pub fn to_float_array(&self) -> [f32; 3] {
        [self.x as f32, self.y as f32, self.z as f32]
    }
}

impl From<glam::DVec4> for NVector4 {
    fn from(v: glam::DVec4) -> Self {
        Self { x: v.x, y: v.y, z: v.z, w: v.w }
    }
}

impl From<[f64; 4]> for NVector4 {
    fn from(value: [f64; 4]) -> Self {
        NVector4 {
            x: value[0],
            y: value[1],
            z: value[2],
            w: value[3],
        }
    }
}

impl From<[f32; 4]> for NVector4 {
    fn from(value: [f32; 4]) -> Self {
        NVector4 {
            x: value[0] as f64,
            y: value[1] as f64,
            z: value[2] as f64,
            w: value[3] as f64,
        }
    }
}

impl From<NVector4> for glam::DVec4 {
    fn from(v: NVector4) -> Self {
        Self::new(v.x, v.y, v.z, v.w)
    }
}

impl FromJObject for NVector4 {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self>
    where
        Self: Sized
    {
        let class = env.find_class("com/dropbear/math/Vector4d")
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

        let w = env.get_field(obj, "w", "D")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .d()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        Ok(NVector4::new(x, y, z, w))
    }
}

impl ToJObject for NVector4 {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env.find_class("com/dropbear/math/Vector3d")
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let constructor_sig = "(DDD)V";

        let args = [
            jni::objects::JValue::Double(self.x),
            jni::objects::JValue::Double(self.y),
            jni::objects::JValue::Double(self.z),
            jni::objects::JValue::Double(self.w),
        ];

        let obj = env.new_object(&class, constructor_sig, &args)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        Ok(obj)
    }
}

impl From<NQuaternion> for NVector4 {
    fn from(value: NQuaternion) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
            w: value.w,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NQuaternion {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl From<DQuat> for NQuaternion {
    fn from(value: DQuat) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
            w: value.w,
        }
    }
}

impl From<NQuaternion> for glam::DQuat {
    fn from(value: NQuaternion) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
            w: value.w,
        }
    }
}

impl From<[f64; 4]> for NQuaternion {
    fn from(value: [f64; 4]) -> Self {
        Self {
            x: value[0],
            y: value[1],
            z: value[2],
            w: value[3],
        }
    }
}

impl From<glam::Quat> for NQuaternion {
    fn from(value: glam::Quat) -> Self {
        Self {
            x: value.x as f64,
            y: value.y as f64,
            z: value.z as f64,
            w: value.w as f64,

        }
    }
}

impl From<NVector4> for NQuaternion {
    fn from(value: NVector4) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
            w: value.w,
        }
    }
}

impl ToJObject for NQuaternion {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env.find_class("com/dropbear/math/Quaterniond")
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let args = [
            JValue::Double(self.x),
            JValue::Double(self.y),
            JValue::Double(self.z),
            JValue::Double(self.w),
        ];

        env.new_object(&class, "(DDDD)V", &args)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
    }
}

impl FromJObject for NQuaternion {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self>
    where
        Self: Sized
    {
        let class = env.find_class("com/dropbear/math/Quaterniond")
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

        let w = env.get_field(obj, "w", "D")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .d()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        Ok(NQuaternion { x, y, z, w })
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NTransform {
    pub position: NVector3,
    pub rotation: NQuaternion,
    pub scale: NVector3,
}

impl From<Transform> for NTransform {
    fn from(value: Transform) -> Self {
        Self {
            position: NVector3::from(value.position),
            rotation: NQuaternion::from(value.rotation),
            scale: NVector3::from(value.scale),
        }
    }
}

impl From<NTransform> for Transform {
    fn from(value: NTransform) -> Self {
        Self {
            position: DVec3::from(value.position),
            rotation: DQuat::from(value.rotation),
            scale: DVec3::from(value.scale),
        }
    }
}

impl FromJObject for NTransform {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self> {
        let pos_val = env.get_field(obj, "position", "Lcom/dropbear/math/Vector3d;")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?;

        let pos_obj = pos_val.l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let rot_val = env.get_field(obj, "rotation", "Lcom/dropbear/math/Quaterniond;")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?;

        let rot_obj = rot_val.l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let scale_val = env.get_field(obj, "scale", "Lcom/dropbear/math/Vector3d;")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?;

        let scale_obj = scale_val.l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let position: DVec3 = NVector3::from_jobject(env, &pos_obj)?.into();
        let scale: DVec3 = NVector3::from_jobject(env, &scale_obj)?.into();

        let mut get_double = |field: &str| -> DropbearNativeResult<f64> {
            env.get_field(&rot_obj, field, "D")
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
                .d()
                .map_err(|_| DropbearNativeError::JNIUnwrapFailed)
        };

        let rx = get_double("x")?;
        let ry = get_double("y")?;
        let rz = get_double("z")?;
        let rw = get_double("w")?;

        let rotation = DQuat::from_xyzw(rx, ry, rz, rw);

        Ok(NTransform {
            position: position.into(),
            rotation: rotation.into(),
            scale: scale.into(),
        })
    }
}

impl ToJObject for NTransform {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .find_class("com/dropbear/math/Transform")
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let args = [
            JValue::Double(self.position.x),
            JValue::Double(self.position.y),
            JValue::Double(self.position.z),
            JValue::Double(self.rotation.x),
            JValue::Double(self.rotation.y),
            JValue::Double(self.rotation.z),
            JValue::Double(self.rotation.w),
            JValue::Double(self.scale.x),
            JValue::Double(self.scale.y),
            JValue::Double(self.scale.z),
        ];

        env.new_object(&class, "(DDDDDDDDDD)V", &args)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NCollider {
    pub index: IndexNative,
    pub entity_id: u64,
    pub id: u32,
}

impl ToJObject for NCollider {
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

impl FromJObject for NCollider {
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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct IndexNative {
    pub index: u32,
    pub generation: u32,
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

impl ToJObject for IndexNative {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let cls = env.find_class("com/dropbear/physics/Index")
            .map_err(|e| {
                eprintln!("[JNI Error] Could not find Index class: {:?}", e);
                DropbearNativeError::GenericError
            })?;

        let obj = env.new_object(
            cls,
            "(II)V",
            &[
                JValue::Int(self.index as i32),
                JValue::Int(self.generation as i32)
            ]
        ).map_err(|e| {
            eprintln!("[JNI Error] Failed to create Index object: {:?}", e);
            DropbearNativeError::GenericError
        })?;

        Ok(obj)
    }
}

impl ToJObject for Option<IndexNative> {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        match self {
            Some(value) => value.to_jobject(env),
            None => Ok(JObject::null()),
        }
    }
}

impl FromJObject for IndexNative {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self>
    where
        Self: Sized
    {
        let idx_val = env.get_field(obj, "index", "I")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .i()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let gen_val = env.get_field(obj, "generation", "I")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .i()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        Ok(IndexNative {
            index: idx_val as u32,
            generation: gen_val as u32,
        })
    }
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
    pub collider: NCollider,
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
#[dropbear_macro::repr_c_enum]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NShapeCastStatus {
    OutOfIterations,
    Converged,
    Failed,
    PenetratingOrWithinTargetDist,
}

impl Into<ShapeCastStatus> for NShapeCastStatus {
    fn into(self) -> ShapeCastStatus {
        match self {
            NShapeCastStatus::OutOfIterations => ShapeCastStatus::OutOfIterations,
            NShapeCastStatus::Converged => ShapeCastStatus::Converged,
            NShapeCastStatus::Failed => ShapeCastStatus::Failed,
            NShapeCastStatus::PenetratingOrWithinTargetDist => ShapeCastStatus::PenetratingOrWithinTargetDist,
        }
    }
}

impl Into<NShapeCastStatus> for ShapeCastStatus {
    fn into(self) -> NShapeCastStatus {
        match self {
            ShapeCastStatus::OutOfIterations => NShapeCastStatus::OutOfIterations,
            ShapeCastStatus::Converged => NShapeCastStatus::Converged,
            ShapeCastStatus::Failed => NShapeCastStatus::Failed,
            ShapeCastStatus::PenetratingOrWithinTargetDist => NShapeCastStatus::PenetratingOrWithinTargetDist,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NShapeCastHit {
    pub collider: NCollider,
    pub distance: f64,
    pub witness1: NVector3,
    pub witness2: NVector3,
    pub normal1: NVector3,
    pub normal2: NVector3,
    pub status: NShapeCastStatus,
}

impl ToJObject for NShapeCastHit {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        use jni::sys::jdouble;

        let collider = self.collider.to_jobject(env)?;
        let witness1 = self.witness1.to_jobject(env)?;
        let witness2 = self.witness2.to_jobject(env)?;
        let normal1 = self.normal1.to_jobject(env)?;
        let normal2 = self.normal2.to_jobject(env)?;
        let status = self.status.to_jobject(env)?;

        let distance = self.distance as jdouble;

        let class = env.find_class("com/dropbear/physics/ShapeCastHit").map_err(|e| {
            eprintln!("[JNI Error] Failed to find ShapeCastHit class: {:?}", e);
            DropbearNativeError::JNIClassNotFound
        })?;

        let object = env
            .new_object(
                class,
                "(Lcom/dropbear/physics/Collider;DLcom/dropbear/math/Vector3d;Lcom/dropbear/math/Vector3d;Lcom/dropbear/math/Vector3d;Lcom/dropbear/math/Vector3d;Lcom/dropbear/physics/ShapeCastStatus;)V",
                &[
                    JValue::Object(&collider),
                    JValue::Double(distance),
                    JValue::Object(&witness1),
                    JValue::Object(&witness2),
                    JValue::Object(&normal1),
                    JValue::Object(&normal2),
                    JValue::Object(&status),
                ],
            )
            .map_err(|e| {
                eprintln!("[JNI Error] Failed to create ShapeCastHit object: {:?}", e);
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
    pub(crate) collider1: NCollider,
    pub(crate) collider2: NCollider,
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
                    collider1: NCollider {
                        index: IndexNative::from(col1.0),
                        entity_id: collider1_info.to_bits().get(),
                        id: col1.into_raw_parts().0,
                    },
                    collider2: NCollider {
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
                    collider1: NCollider {
                        index: IndexNative::from(col1.0),
                        entity_id: collider1_info.to_bits().get(),
                        id: col1.into_raw_parts().0,
                    },
                    collider2: NCollider {
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
    pub(crate) collider1: NCollider,
    pub(crate) collider2: NCollider,
    pub(crate) total_force: NVector3,
    pub(crate) total_force_magnitude: f64,
    pub(crate) max_force_direction: NVector3,
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
            collider1: NCollider {
                index: IndexNative::from(event.collider1.0),
                entity_id: find_entity(event.collider1)?.to_bits().get(),
                id: event.collider1.into_raw_parts().0,
            },
            collider2: NCollider {
                index: IndexNative::from(event.collider2.0),
                entity_id: find_entity(event.collider2)?.to_bits().get(),
                id: event.collider2.into_raw_parts().0,
            },
            total_force: NVector3::new(
                event.total_force.x.into(), 
                event.total_force.y.into(), 
                event.total_force.z.into()
            ),
            total_force_magnitude: event.total_force_magnitude as f64,
            max_force_direction: NVector3::new(
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
