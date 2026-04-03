use eucalyptus_core::ptr::WorldPtr;
use eucalyptus_core::scripting::native::DropbearNativeError;
use eucalyptus_core::scripting::result::DropbearNativeResult;
use eucalyptus_core::types::{NColour, NVector3};
use crate::{FromJObject, ToJObject};
use dropbear_engine::entity::{EntityTransform, Transform};
use dropbear_engine::lighting::{Light, LightType};
use glam::{DQuat, DVec3};
use hecs::{Entity, World};
use jni::objects::{JObject, JValue};
use jni::{Env, jni_sig, jni_str};

const LIGHT_FORWARD_AXIS: DVec3 = DVec3::new(0.0, -1.0, 0.0);

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NRange {
    pub start: f32,
    pub end: f32,
}

impl FromJObject for NRange {
    fn from_jobject(env: &mut Env, obj: &JObject) -> DropbearNativeResult<Self> {
        let class = env
            .load_class(jni_str!("com/dropbear/utils/Range"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        if !env
            .is_instance_of(obj, &class)
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?
        {
            return Err(DropbearNativeError::InvalidArgument);
        }

        let start = env
            .get_field(obj, jni_str!("start"), jni_sig!(double))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .d()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)? as f32;

        let end = env
            .get_field(obj, jni_str!("end"), jni_sig!(double))
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .d()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)? as f32;

        Ok(Self { start, end })
    }
}

impl ToJObject for NRange {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .load_class(jni_str!("com/dropbear/utils/Range"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let args = [
            JValue::Double(self.start as f64),
            JValue::Double(self.end as f64),
        ];

        env.new_object(&class, jni_sig!((double, double) -> void), &args)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NAttenuation {
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
}

impl FromJObject for NAttenuation {
    fn from_jobject(env: &mut Env, obj: &JObject) -> DropbearNativeResult<Self> {
        let class = env
            .load_class(jni_str!("com/dropbear/lighting/Attenuation"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        if !env
            .is_instance_of(obj, &class)
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?
        {
            return Err(DropbearNativeError::InvalidArgument);
        }

        let constant = env
            .call_method(obj, jni_str!("getConstant"), jni_sig!(() -> float), &[])
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .f()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let linear = env
            .call_method(obj, jni_str!("getLinear"), jni_sig!(() -> float), &[])
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .f()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let quadratic = env
            .call_method(obj, jni_str!("getQuadratic"), jni_sig!(() -> float), &[])
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .f()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        Ok(Self { constant, linear, quadratic })
    }
}

impl ToJObject for NAttenuation {
    fn to_jobject<'a>(&self, env: &mut Env<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .load_class(jni_str!("com/dropbear/lighting/Attenuation"))
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let args = [
            JValue::Float(self.constant),
            JValue::Float(self.linear),
            JValue::Float(self.quadratic),
        ];

        env.new_object(&class, jni_sig!((float, float, float) -> void), &args)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
    }
}

fn get_transform(world: &World, entity: Entity) -> DropbearNativeResult<Transform> {
    if let Ok(et) = world.get::<&EntityTransform>(entity) {
        Ok(et.sync())
    } else if let Ok(t) = world.get::<&Transform>(entity) {
        Ok(*t)
    } else {
        Err(DropbearNativeError::MissingComponent)
    }
}

fn set_transform_position(
    world: &mut World,
    entity: Entity,
    position: DVec3,
) -> DropbearNativeResult<()> {
    if let Ok(mut et) = world.get::<&mut EntityTransform>(entity) {
        et.local_mut().position = position;
        Ok(())
    } else if let Ok(mut t) = world.get::<&mut Transform>(entity) {
        t.position = position;
        Ok(())
    } else {
        Err(DropbearNativeError::MissingComponent)
    }
}

fn set_transform_rotation(
    world: &mut World,
    entity: Entity,
    rotation: DQuat,
) -> DropbearNativeResult<()> {
    if let Ok(mut et) = world.get::<&mut EntityTransform>(entity) {
        et.local_mut().rotation = rotation;
        Ok(())
    } else if let Ok(mut t) = world.get::<&mut Transform>(entity) {
        t.rotation = rotation;
        Ok(())
    } else {
        Err(DropbearNativeError::MissingComponent)
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "lightExistsForEntity"),
    c
)]
fn light_exists_for_entity(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
) -> DropbearNativeResult<bool> {
    Ok(world.get::<&dropbear_engine::lighting::LightComponent>(entity).is_ok())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "getPosition"),
    c
)]
fn get_position(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
) -> DropbearNativeResult<NVector3> {
    let transform = get_transform(world, entity)?;
    Ok(NVector3::from(transform.position))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "setPosition"),
    c
)]
fn set_position(
    #[dropbear_macro::define(WorldPtr)] world: &mut World,
    #[dropbear_macro::entity] entity: Entity,
    position: &NVector3,
) -> DropbearNativeResult<()> {
    set_transform_position(world, entity, (*position).into())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "getDirection"),
    c
)]
fn get_direction(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
) -> DropbearNativeResult<NVector3> {
    let transform = get_transform(world, entity)?;
    let forward = LIGHT_FORWARD_AXIS;
    let dir = (transform.rotation * forward).normalize_or_zero();
    Ok(NVector3::from(dir))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "setDirection"),
    c
)]
fn set_direction(
    #[dropbear_macro::define(WorldPtr)] world: &mut World,
    #[dropbear_macro::entity] entity: Entity,
    direction: &NVector3,
) -> DropbearNativeResult<()> {
    let dir: DVec3 = (*direction).into();
    let desired = dir.normalize_or_zero();
    if desired.length_squared() < 1e-12 {
        return Err(DropbearNativeError::InvalidArgument);
    }
    let forward = LIGHT_FORWARD_AXIS;
    let rotation = DQuat::from_rotation_arc(forward, desired);
    set_transform_rotation(world, entity, rotation)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "getColour"),
    c
)]
fn get_colour(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
) -> DropbearNativeResult<NColour> {
    let light = world
        .get::<&Light>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    Ok(NColour::from_linear_rgb(light.component.colour))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "setColour"),
    c
)]
fn set_colour(
    #[dropbear_macro::define(WorldPtr)] world: &mut World,
    #[dropbear_macro::entity] entity: Entity,
    colour: &NColour,
) -> DropbearNativeResult<()> {
    let mut light = world
        .get::<&mut Light>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    light.component.colour = colour.to_linear_rgb();
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "getLightType"),
    c
)]
fn get_light_type(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
) -> DropbearNativeResult<i32> {
    let light = world
        .get::<&Light>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    Ok(light.component.light_type as i32)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "setLightType"),
    c
)]
fn set_light_type(
    #[dropbear_macro::define(WorldPtr)] world: &mut World,
    #[dropbear_macro::entity] entity: Entity,
    light_type: i32,
) -> DropbearNativeResult<()> {
    let mut light = world
        .get::<&mut Light>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    light.component.light_type = match light_type {
        0 => LightType::Directional,
        1 => LightType::Point,
        2 => LightType::Spot,
        _ => return Err(DropbearNativeError::InvalidArgument),
    };
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "getIntensity"),
    c
)]
fn get_intensity(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
) -> DropbearNativeResult<f64> {
    let light = world
        .get::<&Light>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    Ok(light.component.intensity as f64)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "setIntensity"),
    c
)]
fn set_intensity(
    #[dropbear_macro::define(WorldPtr)] world: &mut World,
    #[dropbear_macro::entity] entity: Entity,
    intensity: f64,
) -> DropbearNativeResult<()> {
    let mut light = world
        .get::<&mut Light>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    light.component.intensity = intensity as f32;
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "getAttenuation"),
    c
)]
fn get_attenuation(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
) -> DropbearNativeResult<NAttenuation> {
    let light = world
        .get::<&Light>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    Ok(NAttenuation {
        constant: light.component.attenuation.constant,
        linear: light.component.attenuation.linear,
        quadratic: light.component.attenuation.quadratic,
    })
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "setAttenuation"),
    c
)]
fn set_attenuation(
    #[dropbear_macro::define(WorldPtr)] world: &mut World,
    #[dropbear_macro::entity] entity: Entity,
    attenuation: &NAttenuation,
) -> DropbearNativeResult<()> {
    let mut light = world
        .get::<&mut Light>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    light.component.attenuation.constant = attenuation.constant;
    light.component.attenuation.linear = attenuation.linear;
    light.component.attenuation.quadratic = attenuation.quadratic;
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "getEnabled"),
    c
)]
fn get_enabled(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
) -> DropbearNativeResult<bool> {
    let light = world
        .get::<&Light>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    Ok(light.component.enabled)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "setEnabled"),
    c
)]
fn set_enabled(
    #[dropbear_macro::define(WorldPtr)] world: &mut World,
    #[dropbear_macro::entity] entity: Entity,
    enabled: bool,
) -> DropbearNativeResult<()> {
    let mut light = world
        .get::<&mut Light>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    light.component.enabled = enabled;
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "getCutoffAngle"),
    c
)]
fn get_cutoff_angle(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
) -> DropbearNativeResult<f64> {
    let light = world
        .get::<&Light>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    Ok(light.component.cutoff_angle as f64)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "setCutoffAngle"),
    c
)]
fn set_cutoff_angle(
    #[dropbear_macro::define(WorldPtr)] world: &mut World,
    #[dropbear_macro::entity] entity: Entity,
    cutoff_angle: f64,
) -> DropbearNativeResult<()> {
    let mut light = world
        .get::<&mut Light>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    light.component.cutoff_angle = cutoff_angle as f32;
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "getOuterCutoffAngle"),
    c
)]
fn get_outer_cutoff_angle(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
) -> DropbearNativeResult<f64> {
    let light = world
        .get::<&Light>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    Ok(light.component.outer_cutoff_angle as f64)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "setOuterCutoffAngle"),
    c
)]
fn set_outer_cutoff_angle(
    #[dropbear_macro::define(WorldPtr)] world: &mut World,
    #[dropbear_macro::entity] entity: Entity,
    outer_cutoff_angle: f64,
) -> DropbearNativeResult<()> {
    let mut light = world
        .get::<&mut Light>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    light.component.outer_cutoff_angle = outer_cutoff_angle as f32;
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "getCastsShadows"),
    c
)]
fn get_casts_shadows(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
) -> DropbearNativeResult<bool> {
    let light = world
        .get::<&Light>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    Ok(light.component.cast_shadows)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "setCastsShadows"),
    c
)]
fn set_casts_shadows(
    #[dropbear_macro::define(WorldPtr)] world: &mut World,
    #[dropbear_macro::entity] entity: Entity,
    casts_shadows: bool,
) -> DropbearNativeResult<()> {
    let mut light = world
        .get::<&mut Light>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    light.component.cast_shadows = casts_shadows;
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "getDepth"),
    c
)]
fn get_depth(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
) -> DropbearNativeResult<NRange> {
    let light = world
        .get::<&Light>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    Ok(NRange {
        start: light.component.depth.start,
        end: light.component.depth.end,
    })
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "setDepth"),
    c
)]
fn set_depth(
    #[dropbear_macro::define(WorldPtr)] world: &mut World,
    #[dropbear_macro::entity] entity: Entity,
    depth: &NRange,
) -> DropbearNativeResult<()> {
    if !(depth.start.is_finite() && depth.end.is_finite()) {
        return Err(DropbearNativeError::InvalidArgument);
    }
    if depth.end <= depth.start {
        return Err(DropbearNativeError::InvalidArgument);
    }
    let mut light = world
        .get::<&mut Light>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    light.component.depth = depth.start..depth.end;
    Ok(())
}
