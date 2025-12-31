use crate::scripting::jni::utils::{FromJObject, ToJObject};
use crate::scripting::native::DropbearNativeError;
use crate::scripting::result::DropbearNativeResult;
use dropbear_engine::entity::{EntityTransform, Transform};
use glam::{DQuat, DVec3};
use ::jni::objects::{JObject, JValue};
use ::jni::JNIEnv;

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

        let position = DVec3::from_jobject(env, &pos_obj)?;
        let scale = DVec3::from_jobject(env, &scale_obj)?;

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

pub mod shared {
    use dropbear_engine::entity::EntityTransform;
    use hecs::{Entity, World};

    pub fn entity_transform_exists_for_entity(world: &World, entity: Entity) -> bool {
        world.get::<&EntityTransform>(entity).is_ok()
    }
}

pub mod jni {
    #![allow(non_snake_case)]
    use crate::convert_jlong_to_entity;
    use crate::hierarchy::EntityTransformExt;
    use crate::scripting::jni::utils::{FromJObject, ToJObject};
    use dropbear_engine::entity::EntityTransform;
    use hecs::World;
    use jni::objects::{JClass, JObject};
    use jni::sys::{jboolean, jlong, jobject};
    use jni::JNIEnv;

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_EntityTransformNative_entityTransformExistsForEntity(
        _env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
    ) -> jboolean {
        let world = crate::convert_ptr!(world_ptr => World);
        let entity = convert_jlong_to_entity!(entity_id);

        if world.get::<&EntityTransform>(entity).is_ok() { 1 } else { 0 }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_EntityTransformNative_getLocalTransform(
        mut env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
    ) -> jobject {
        let world = crate::convert_ptr!(world_ptr => World);
        let entity = convert_jlong_to_entity!(entity_id);

        if let Ok(et) = world.get::<&EntityTransform>(entity) {
            match et.local().to_jobject(&mut env) {
                Ok(obj) => obj.into_raw(),
                Err(_) => std::ptr::null_mut(),
            }
        } else {
            let _ = env.throw_new("java/lang/RuntimeException", "Entity missing EntityTransform");
            std::ptr::null_mut()
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_EntityTransformNative_setLocalTransform(
        mut env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
        transform_obj: JObject,
    ) {
        let world = crate::convert_ptr!(world_ptr => World);
        let entity = convert_jlong_to_entity!(entity_id);

        let new_transform = match dropbear_engine::entity::Transform::from_jobject(&mut env, &transform_obj) {
            Ok(t) => t,
            Err(e) => {
                let _ = env.throw_new("java/lang/IllegalArgumentException", format!("Invalid Transform: {:?}", e));
                return;
            }
        };

        if let Ok(mut et) = world.get::<&mut EntityTransform>(entity) {
            *et.local_mut() = new_transform;
        } else {
            let _ = env.throw_new("java/lang/RuntimeException", "Entity missing EntityTransform");
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_EntityTransformNative_getWorldTransform(
        mut env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
    ) -> jobject {
        let world = crate::convert_ptr!(world_ptr => World);
        let entity = convert_jlong_to_entity!(entity_id);

        if let Ok(et) = world.get::<&EntityTransform>(entity) {
            match et.world().to_jobject(&mut env) {
                Ok(obj) => obj.into_raw(),
                Err(_) => std::ptr::null_mut(),
            }
        } else {
            let _ = env.throw_new("java/lang/RuntimeException", "Entity missing EntityTransform");
            std::ptr::null_mut()
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_EntityTransformNative_setWorldTransform(
        mut env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
        transform_obj: JObject,
    ) {
        let world = crate::convert_ptr!(world_ptr => World);
        let entity = convert_jlong_to_entity!(entity_id);

        let new_transform = match dropbear_engine::entity::Transform::from_jobject(&mut env, &transform_obj) {
            Ok(t) => t,
            Err(_) => return,
        };

        if let Ok(mut et) = world.get::<&mut EntityTransform>(entity) {
            *et.world_mut() = new_transform;
        } else {
            let _ = env.throw_new("java/lang/RuntimeException", "Entity missing EntityTransform");
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_EntityTransformNative_propagateTransform(
        mut env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
    ) -> jobject {
        let world = crate::convert_ptr!(world_ptr => World);
        let entity = convert_jlong_to_entity!(entity_id);

        if let Ok(et) = world.get::<&EntityTransform>(entity) {
            let propagated = et.propagate(&world, entity);
            match propagated.to_jobject(&mut env) {
                Ok(obj) => obj.into_raw(),
                Err(_) => std::ptr::null_mut(),
            }
        } else {
            let _ = env.throw_new("java/lang/RuntimeException", "Entity missing EntityTransform");
            std::ptr::null_mut()
        }
    }
}

#[dropbear_macro::impl_c_api]
pub mod native {
    use crate::hierarchy::EntityTransformExt;
    use crate::types::{TransformNative as Transform};
    use dropbear_engine::entity::EntityTransform;
    use hecs::{Entity, World};

    use crate::convert_ptr;
    use crate::ptr::WorldPtr;
    use crate::scripting::native::DropbearNativeError;
    use crate::scripting::result::DropbearNativeResult;

    pub fn dropbear_entity_transform_exists_for_entity(
        world_ptr: WorldPtr,
        entity_id: u64,
    ) -> DropbearNativeResult<bool> {
        let world = convert_ptr!(world_ptr => World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;
        DropbearNativeResult::Ok(world.get::<&EntityTransform>(entity).is_ok())
    }

    pub fn dropbear_get_local_transform(
        world_ptr: WorldPtr,
        entity_id: u64,
    ) -> DropbearNativeResult<Transform> {
        let world = convert_ptr!(world_ptr => World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(et) = world.get::<&EntityTransform>(entity) {
            DropbearNativeResult::Ok(Transform::from(et.local().clone()))
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }


    pub fn dropbear_set_local_transform(
        world_ptr: WorldPtr,
        entity_id: u64,
        value: Transform,
    ) -> DropbearNativeResult<()> {
        let world = convert_ptr!(world_ptr => World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(mut et) = world.get::<&mut EntityTransform>(entity) {
            *et.local_mut() = dropbear_engine::entity::Transform::from(value);
            DropbearNativeResult::Ok(())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_get_world_transform(
        world_ptr: WorldPtr,
        entity_id: u64,
    ) -> DropbearNativeResult<Transform> {
        let world = convert_ptr!(world_ptr => World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(et) = world.get::<&EntityTransform>(entity) {
            DropbearNativeResult::Ok(Transform::from(et.world().clone()))
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_set_world_transform(
        world_ptr: WorldPtr,
        entity_id: u64,
        value: Transform,
    ) -> DropbearNativeResult<()> {
        let world = convert_ptr!(world_ptr => World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(mut et) = world.get::<&mut EntityTransform>(entity) {
            *et.world_mut() = dropbear_engine::entity::Transform::from(value);
            DropbearNativeResult::Ok(())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_propagate_transform(
        world_ptr: WorldPtr,
        entity_id: u64,
    ) -> DropbearNativeResult<Transform> {
        let world = convert_ptr!(world_ptr => World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(et) = world.get::<&EntityTransform>(entity) {
            let propagated = et.propagate(world, entity);
            DropbearNativeResult::Ok(Transform::from(propagated))
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }
}