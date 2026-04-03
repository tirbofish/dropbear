use eucalyptus_core::hierarchy::EntityTransformExt;
use eucalyptus_core::ptr::WorldPtr;
use eucalyptus_core::scripting::native::DropbearNativeError;
use eucalyptus_core::scripting::result::DropbearNativeResult;
use eucalyptus_core::transform::{OnRails, RailDrive, RailPoint};
use crate::math::{NQuaternion, NTransform, NVector3};
use crate::{FromJObject, ToJObject};
use dropbear_engine::entity::{EntityTransform, Transform};
use glam::{DQuat, DVec3, Vec3};
use jni::objects::{JObject, JValue};
use jni::{Env, jni_sig, jni_str};

impl FromJObject for Transform {
    fn from_jobject(env: &mut Env, obj: &JObject) -> DropbearNativeResult<Self> {
        let pos_val = env
            .get_field(obj, jni_str!("position"), jni_sig!(com.dropbear.math.Vector3d))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?;
        let pos_obj = pos_val.l().map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let rot_val = env
            .get_field(obj, jni_str!("rotation"), jni_sig!(com.dropbear.math.Quaterniond))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?;
        let rot_obj = rot_val.l().map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let scale_val = env
            .get_field(obj, jni_str!("scale"), jni_sig!(com.dropbear.math.Vector3d))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?;
        let scale_obj = scale_val.l().map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let position: DVec3 = NVector3::from_jobject(env, &pos_obj)?.into();
        let scale: DVec3 = NVector3::from_jobject(env, &scale_obj)?.into();

        let mut get_double = |field| -> DropbearNativeResult<f64> {
            env.get_field(&rot_obj, field, jni_sig!(double))
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
                .d()
                .map_err(|_| DropbearNativeError::JNIUnwrapFailed)
        };

        let rx = get_double(jni_str!("x"))?;
        let ry = get_double(jni_str!("y"))?;
        let rz = get_double(jni_str!("z"))?;
        let rw = get_double(jni_str!("w"))?;
        let rotation = DQuat::from_xyzw(rx, ry, rz, rw);

        Ok(Transform { position, rotation, scale })
    }
}

impl ToJObject for Transform {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let cls = env
            .load_class(jni_str!("com/dropbear/math/Transform"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let p = self.position;
        let r = self.rotation;
        let s = self.scale;

        let args = [
            JValue::Double(p.x),
            JValue::Double(p.y),
            JValue::Double(p.z),
            JValue::Double(r.x),
            JValue::Double(r.y),
            JValue::Double(r.z),
            JValue::Double(r.w),
            JValue::Double(s.x),
            JValue::Double(s.y),
            JValue::Double(s.z),
        ];

        env.new_object(
            cls,
            jni_sig!((double, double, double, double, double, double, double, double, double, double) -> void),
            &args,
        )
        .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
    }
}

impl FromJObject for EntityTransform {
    fn from_jobject(env: &mut Env, obj: &JObject) -> DropbearNativeResult<Self> {
        let local_val = env
            .get_field(obj, jni_str!("local"), jni_sig!(com.dropbear.math.Transform))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?;
        let local_obj = local_val.l().map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let world_val = env
            .get_field(obj, jni_str!("world"), jni_sig!(com.dropbear.math.Transform))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?;
        let world_obj = world_val.l().map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let local = Transform::from_jobject(env, &local_obj)?;
        let world = Transform::from_jobject(env, &world_obj)?;

        Ok(EntityTransform::new(local, world))
    }
}

impl ToJObject for EntityTransform {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let cls = env
            .load_class(jni_str!("com/dropbear/components/EntityTransform"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let local_obj = self.local().to_jobject(env)?;
        let world_obj = self.world().to_jobject(env)?;

        let args = [JValue::Object(&local_obj), JValue::Object(&world_obj)];

        env.new_object(
            cls,
            jni_sig!((com.dropbear.math.Transform, com.dropbear.math.Transform) -> void),
            &args,
        )
        .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
    }
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.EntityTransformNative",
        func = "entityTransformExistsForEntity"
    ),
    c
)]
fn exists_for_entity(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<bool> {
    Ok(world.get::<&EntityTransform>(entity).is_ok())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.EntityTransformNative",
        func = "getLocalTransform"
    ),
    c
)]
fn get_local_transform(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<NTransform> {
    if let Ok(et) = world.get::<&EntityTransform>(entity) {
        Ok((*et.local()).into())
    } else {
        Err(DropbearNativeError::MissingComponent)
    }
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.EntityTransformNative",
        func = "setLocalTransform"
    ),
    c
)]
fn set_local_transform(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    transform: &NTransform,
) -> DropbearNativeResult<()> {
    if let Ok(mut et) = world.get::<&mut EntityTransform>(entity) {
        *et.local_mut() = (*transform).into();
        Ok(())
    } else {
        Err(DropbearNativeError::MissingComponent)
    }
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.EntityTransformNative",
        func = "getWorldTransform"
    ),
    c
)]
fn get_world_transform(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<NTransform> {
    if let Ok(et) = world.get::<&EntityTransform>(entity) {
        Ok((*et.world()).into())
    } else {
        Err(DropbearNativeError::MissingComponent)
    }
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.EntityTransformNative",
        func = "setWorldTransform"
    ),
    c
)]
fn set_world_transform(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    transform: &NTransform,
) -> DropbearNativeResult<()> {
    if let Ok(mut et) = world.get::<&mut EntityTransform>(entity) {
        *et.world_mut() = (*transform).into();
        Ok(())
    } else {
        Err(DropbearNativeError::MissingComponent)
    }
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.EntityTransformNative",
        func = "propogateTransform"
    ),
    c
)]
fn propagate_transform(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<NTransform> {
    if let Ok(et) = world.get::<&mut EntityTransform>(entity) {
        let result = et.propagate(world, entity);
        Ok(result.into())
    } else {
        Err(DropbearNativeError::MissingComponent)
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "getEnabled"),
    c
)]
fn on_rails_get_enabled(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<bool> {
    let rails = world
        .get::<&OnRails>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    Ok(rails.enabled)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "setEnabled"),
    c
)]
fn on_rails_set_enabled(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    enabled: bool,
) -> DropbearNativeResult<()> {
    let mut rails = world
        .get::<&mut OnRails>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    rails.enabled = enabled;
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "existsForEntity"),
    c
)]
fn on_rails_exists_for_entity(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<bool> {
    Ok(world.get::<&OnRails>(entity).is_ok())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "getProgress"),
    c
)]
fn on_rails_get_progress(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<f32> {
    let rails = world
        .get::<&OnRails>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    Ok(rails.progress)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "setProgress"),
    c
)]
fn on_rails_set_progress(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    progress: f32,
) -> DropbearNativeResult<()> {
    let mut rails = world
        .get::<&mut OnRails>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    rails.progress = progress.clamp(0.0, 1.0);
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "getPathLen"),
    c
)]
fn on_rails_get_path_len(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<i32> {
    let rails = world
        .get::<&OnRails>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    Ok(rails.path.len() as i32)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "getPathPoint"),
    c
)]
fn on_rails_get_path_point(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    index: i32,
) -> DropbearNativeResult<NVector3> {
    let rails = world
        .get::<&OnRails>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    let point = rails
        .path
        .get(index as usize)
        .ok_or(DropbearNativeError::InvalidArgument)?;
    Ok(NVector3::from(&point.position))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "clearPath"),
    c
)]
fn on_rails_clear_path(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<()> {
    let mut rails = world
        .get::<&mut OnRails>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    rails.path.clear();
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "pushPathPoint"),
    c
)]
fn on_rails_push_path_point(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    point: &NVector3,
) -> DropbearNativeResult<()> {
    let mut rails = world
        .get::<&mut OnRails>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    rails.path.push(RailPoint { position: DVec3::from(point), rotation: None });
    Ok(())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.camera.OnRailsNative",
        func = "getPathPointHasRotation"
    ),
    c
)]
fn on_rails_get_path_point_has_rotation(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    index: i32,
) -> DropbearNativeResult<bool> {
    let rails = world
        .get::<&OnRails>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    let point = rails
        .path
        .get(index as usize)
        .ok_or(DropbearNativeError::InvalidArgument)?;
    Ok(point.rotation.is_some())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.camera.OnRailsNative",
        func = "getPathPointRotation"
    ),
    c
)]
fn on_rails_get_path_point_rotation(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    index: i32,
) -> DropbearNativeResult<NQuaternion> {
    let rails = world
        .get::<&OnRails>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    let point = rails
        .path
        .get(index as usize)
        .ok_or(DropbearNativeError::InvalidArgument)?;
    Ok(NQuaternion::from(point.rotation.unwrap_or(DQuat::IDENTITY)))
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.camera.OnRailsNative",
        func = "pushPathPointWithRotation"
    ),
    c
)]
fn on_rails_push_path_point_with_rotation(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    point: &NVector3,
    rotation: &NQuaternion,
) -> DropbearNativeResult<()> {
    let mut rails = world
        .get::<&mut OnRails>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    rails.path.push(RailPoint {
        position: DVec3::from(point),
        rotation: Some(DQuat::from(*rotation)),
    });
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "getDriveType"),
    c
)]
fn on_rails_get_drive_type(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<i32> {
    let rails = world
        .get::<&OnRails>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    let tag = match &rails.drive {
        RailDrive::Automatic { .. } => 0,
        RailDrive::FollowEntity { .. } => 1,
        RailDrive::AxisDriven { .. } => 2,
        RailDrive::Manual => 3,
    };
    Ok(tag)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.camera.OnRailsNative",
        func = "setDriveAutomatic"
    ),
    c
)]
fn on_rails_set_drive_automatic(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    speed: f32,
    looping: bool,
) -> DropbearNativeResult<()> {
    let mut rails = world
        .get::<&mut OnRails>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    rails.drive = RailDrive::Automatic { speed, looping };
    Ok(())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.camera.OnRailsNative",
        func = "setDriveFollowEntity"
    ),
    c
)]
fn on_rails_set_drive_follow_entity(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    target: u64,
    monotonic: bool,
) -> DropbearNativeResult<()> {
    let target_entity =
        hecs::Entity::from_bits(target).ok_or(DropbearNativeError::InvalidEntity)?;
    let mut rails = world
        .get::<&mut OnRails>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    rails.drive = RailDrive::FollowEntity { target: target_entity, monotonic };
    Ok(())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.camera.OnRailsNative",
        func = "setDriveAxisDriven"
    ),
    c
)]
fn on_rails_set_drive_axis_driven(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    target: u64,
    axis: &NVector3,
    range_min: f32,
    range_max: f32,
) -> DropbearNativeResult<()> {
    let target_entity =
        hecs::Entity::from_bits(target).ok_or(DropbearNativeError::InvalidEntity)?;
    let mut rails = world
        .get::<&mut OnRails>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    rails.drive = RailDrive::AxisDriven {
        target: target_entity,
        axis: Vec3::new(axis.x as f32, axis.y as f32, axis.z as f32),
        range: (range_min, range_max),
    };
    Ok(())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.camera.OnRailsNative",
        func = "setDriveManual"
    ),
    c
)]
fn on_rails_set_drive_manual(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<()> {
    let mut rails = world
        .get::<&mut OnRails>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    rails.drive = RailDrive::Manual;
    Ok(())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.camera.OnRailsNative",
        func = "getDriveAutomaticSpeed"
    ),
    c
)]
fn on_rails_get_drive_automatic_speed(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<f32> {
    let rails = world
        .get::<&OnRails>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    if let RailDrive::Automatic { speed, .. } = &rails.drive {
        Ok(*speed)
    } else {
        Err(DropbearNativeError::InvalidArgument)
    }
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.camera.OnRailsNative",
        func = "getDriveAutomaticLooping"
    ),
    c
)]
fn on_rails_get_drive_automatic_looping(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<bool> {
    let rails = world
        .get::<&OnRails>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    if let RailDrive::Automatic { looping, .. } = &rails.drive {
        Ok(*looping)
    } else {
        Err(DropbearNativeError::InvalidArgument)
    }
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.camera.OnRailsNative",
        func = "getDriveFollowEntityTarget"
    ),
    c
)]
fn on_rails_get_drive_follow_entity_target(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<u64> {
    let rails = world
        .get::<&OnRails>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    if let RailDrive::FollowEntity { target, .. } = &rails.drive {
        Ok(target.to_bits().get())
    } else {
        Err(DropbearNativeError::InvalidArgument)
    }
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.camera.OnRailsNative",
        func = "getDriveFollowEntityMonotonic"
    ),
    c
)]
fn on_rails_get_drive_follow_entity_monotonic(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<bool> {
    let rails = world
        .get::<&OnRails>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    if let RailDrive::FollowEntity { monotonic, .. } = &rails.drive {
        Ok(*monotonic)
    } else {
        Err(DropbearNativeError::InvalidArgument)
    }
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.camera.OnRailsNative",
        func = "getDriveAxisDrivenTarget"
    ),
    c
)]
fn on_rails_get_drive_axis_driven_target(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<u64> {
    let rails = world
        .get::<&OnRails>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    if let RailDrive::AxisDriven { target, .. } = &rails.drive {
        Ok(target.to_bits().get())
    } else {
        Err(DropbearNativeError::InvalidArgument)
    }
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.camera.OnRailsNative",
        func = "getDriveAxisDrivenAxis"
    ),
    c
)]
fn on_rails_get_drive_axis_driven_axis(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<NVector3> {
    let rails = world
        .get::<&OnRails>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    if let RailDrive::AxisDriven { axis, .. } = &rails.drive {
        Ok(NVector3::from(*axis))
    } else {
        Err(DropbearNativeError::InvalidArgument)
    }
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.camera.OnRailsNative",
        func = "getDriveAxisDrivenRangeMin"
    ),
    c
)]
fn on_rails_get_drive_axis_driven_range_min(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<f32> {
    let rails = world
        .get::<&OnRails>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    if let RailDrive::AxisDriven { range, .. } = &rails.drive {
        Ok(range.0)
    } else {
        Err(DropbearNativeError::InvalidArgument)
    }
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.camera.OnRailsNative",
        func = "getDriveAxisDrivenRangeMax"
    ),
    c
)]
fn on_rails_get_drive_axis_driven_range_max(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<f32> {
    let rails = world
        .get::<&OnRails>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    if let RailDrive::AxisDriven { range, .. } = &rails.drive {
        Ok(range.1)
    } else {
        Err(DropbearNativeError::InvalidArgument)
    }
}
