use crate::hierarchy::EntityTransformExt;
use crate::ptr::WorldPtr;
use crate::scripting::jni::utils::{FromJObject, ToJObject};
use crate::scripting::native::DropbearNativeError;
use crate::scripting::result::DropbearNativeResult;
use dropbear_engine::entity::{EntityTransform, Transform};
use glam::{DQuat, DVec3};
use ::jni::objects::{JObject, JValue};
use ::jni::JNIEnv;
use crate::types::{NTransform, NVector3};

impl FromJObject for Transform {
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

        Ok(Transform {
            position,
            rotation,
            scale,
        })
    }
}

impl ToJObject for Transform {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let cls = env.find_class("com/dropbear/math/Transform")
            .map_err(|e| {
                eprintln!("Could not find Transform class: {:?}", e);
                DropbearNativeError::JNIClassNotFound
            })?;

        let p = self.position;
        let r = self.rotation;
        let s = self.scale;


        let args = [
            JValue::Double(p.x), JValue::Double(p.y), JValue::Double(p.z),
            JValue::Double(r.x), JValue::Double(r.y), JValue::Double(r.z), JValue::Double(r.w),
            JValue::Double(s.x), JValue::Double(s.y), JValue::Double(s.z),
        ];

        let obj = env.new_object(cls, "(DDDDDDDDDD)V", &args)
            .map_err(|e| {
                eprintln!("Failed to create Transform object: {:?}", e);
                DropbearNativeError::JNIFailedToCreateObject
            })?;

        Ok(obj)
    }
}

impl FromJObject for EntityTransform {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self> {
        let local_val = env.get_field(obj, "local", "Lcom/dropbear/math/Transform;")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?;

        let local_obj = local_val.l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let world_val = env.get_field(obj, "world", "Lcom/dropbear/math/Transform;")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?;

        let world_obj = world_val.l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let local = Transform::from_jobject(env, &local_obj)?;
        let world = Transform::from_jobject(env, &world_obj)?;

        Ok(EntityTransform::new(local, world))
    }
}

impl ToJObject for EntityTransform {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let cls = env.find_class("com/dropbear/components/EntityTransform")
            .map_err(|e| {
                eprintln!("Could not find EntityTransform class: {:?}", e);
                DropbearNativeError::JNIClassNotFound
            })?;

        let local_obj = self.local().to_jobject(env)?;
        let world_obj = self.world().to_jobject(env)?;

        let args = [
            JValue::Object(&local_obj),
            JValue::Object(&world_obj)
        ];

        let obj = env.new_object(
            cls,
            "(Lcom/dropbear/math/Transform;Lcom/dropbear/math/Transform;)V",
            &args
        ).map_err(|e| {
            eprintln!("Failed to create EntityTransform object: {:?}", e);
            DropbearNativeError::JNIFailedToCreateObject
        })?;

        Ok(obj)
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.EntityTransformNative", func = "entityTransformExistsForEntity"),
    c
)]
fn exists_for_entity(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::entity]
    entity: hecs::Entity,
) -> DropbearNativeResult<bool> {
    Ok(world.get::<&EntityTransform>(entity).is_ok())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.EntityTransformNative", func = "getLocalTransform"),
    c
)]
fn get_local_transform(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::entity]
    entity: hecs::Entity,
) -> DropbearNativeResult<NTransform> {
    if let Ok(et) = world.get::<&EntityTransform>(entity) {
        Ok((*et.local()).into())
    } else {
        Err(DropbearNativeError::MissingComponent)
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.EntityTransformNative", func = "setLocalTransform"),
    c
)]
fn set_local_transform(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::entity]
    entity: hecs::Entity,
    transform: &NTransform
) -> DropbearNativeResult<()> {
    if let Ok(mut et) = world.get::<&mut EntityTransform>(entity) {
        *et.local_mut() = (*transform).into();

        Ok(())
    } else {
        Err(DropbearNativeError::MissingComponent)
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.EntityTransformNative", func = "getWorldTransform"),
    c
)]
fn get_world_transform(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::entity]
    entity: hecs::Entity,
) -> DropbearNativeResult<NTransform> {
    if let Ok(et) = world.get::<&EntityTransform>(entity) {
        Ok((*et.world()).into())
    } else {
        Err(DropbearNativeError::MissingComponent)
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.EntityTransformNative", func = "setWorldTransform"),
    c
)]
fn set_world_transform(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::entity]
    entity: hecs::Entity,
    transform: &NTransform
) -> DropbearNativeResult<()> {
    if let Ok(mut et) = world.get::<&mut EntityTransform>(entity) {
        *et.world_mut() = (*transform).into();

        Ok(())
    } else {
        Err(DropbearNativeError::MissingComponent)
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.EntityTransformNative", func = "propogateTransform"),
    c
)]
fn propogate_transform(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::entity]
    entity: hecs::Entity,
) -> DropbearNativeResult<NTransform> {
    if let Ok(mut et) = world.get::<&mut EntityTransform>(entity) {
        let result = et.propagate(world, entity);
        Ok(result.into())
    } else {
        Err(DropbearNativeError::MissingComponent)
    }
}