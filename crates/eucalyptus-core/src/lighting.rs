use std::sync::Arc;
use egui::{CollapsingHeader, Ui};
use crate::ptr::WorldPtr;
use crate::scripting::jni::utils::{FromJObject, ToJObject};
use crate::scripting::native::DropbearNativeError;
use crate::scripting::result::DropbearNativeResult;
use crate::types::NVector3;
use dropbear_engine::entity::{EntityTransform, Transform};
use dropbear_engine::lighting::{Light, LightComponent, LightType};
use glam::{DQuat, DVec3};
use hecs::{Entity, World};
use jni::objects::{JObject, JValue};
use jni::JNIEnv;
use dropbear_engine::graphics::SharedGraphicsContext;
use crate::component::{Component, ComponentDescriptor, ComponentInitFuture, InspectableComponent, SerializedComponent};
use crate::states::SerializedLight;

#[typetag::serde]
impl SerializedComponent for SerializedLight {}

impl Component for Light {
    type SerializedForm = SerializedLight;
    type RequiredComponentTypes = (Self, LightComponent);

    fn descriptor() -> ComponentDescriptor {
        ComponentDescriptor {
            fqtn: "dropbear_engine::lighting::Light".to_string(),
            type_name: "Light".to_string(),
            category: Some("Lighting".to_string()),
            description: Some("An object that emits light".to_string()),
        }
    }

    async fn first_time(graphics: Arc<SharedGraphicsContext>) -> anyhow::Result<Self::RequiredComponentTypes>
    where
        Self: Sized
    {
        let comp = LightComponent::default();
        let light = Light::new(graphics.clone(), comp.clone(), None).await;
        Ok((light, comp))
    }

    fn init<'a>(
        ser: &'a Self::SerializedForm,
        graphics: Arc<SharedGraphicsContext>,
    ) -> ComponentInitFuture<'a, Self> {
        Box::pin(async move {
            let light = Light::new(
                graphics.clone(),
                ser.light_component.clone(),
                Some(ser.label.as_str())
            ).await;

            Ok((light, ser.light_component.clone()))
        })
    }

    fn update_component(&mut self, world: &World, entity: Entity, _dt: f32, graphics: Arc<SharedGraphicsContext>) {
        if let Ok(comp) = world.query_one::<&LightComponent>(entity).get() {
            self.update(&graphics, comp);
        }
    }

    fn save(&self, world: &World, entity: Entity) -> Box<dyn SerializedComponent> {
        if let Ok(comp) = world.query_one::<&LightComponent>(entity).get() {
            Box::new(SerializedLight {
                label: self.label.clone(),
                light_component: comp.clone(),
                enabled: comp.enabled,
                entity_id: Some(entity),
            })
        } else {
            Box::new(SerializedLight::default())
        }
    }
}

impl InspectableComponent for Light {
    fn inspect(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Light").default_open(true).show(ui, |ui| {
            ui.label("Not implemented yet"); 
        });
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct NColour {
	r: u8,
	g: u8,
	b: u8,
	a: u8,
}

impl NColour {
	fn to_linear_rgb(self) -> DVec3 {
		DVec3::new(
			self.r as f64 / 255.0,
			self.g as f64 / 255.0,
			self.b as f64 / 255.0,
		)
	}

	fn from_linear_rgb(rgb: DVec3) -> Self {
		fn clamp_to_u8(x: f64) -> u8 {
			let v = (x * 255.0).round();
			v.clamp(0.0, 255.0) as u8
		}

		Self {
			r: clamp_to_u8(rgb.x),
			g: clamp_to_u8(rgb.y),
			b: clamp_to_u8(rgb.z),
			a: 255,
		}
	}
}

impl FromJObject for NColour {
	fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self> {
		let class = env
			.find_class("com/dropbear/utils/Colour")
			.map_err(|_| DropbearNativeError::JNIClassNotFound)?;

		if !env
			.is_instance_of(obj, &class)
			.map_err(|_| DropbearNativeError::JNIUnwrapFailed)?
		{
			return Err(DropbearNativeError::InvalidArgument);
		}

		let mut get_byte = |field: &str| -> DropbearNativeResult<u8> {
			let v = env
				.get_field(obj, field, "B")
				.map_err(|_| DropbearNativeError::JNIFailedToGetField)?
				.b()
				.map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;
			Ok(v as u8)
		};

		Ok(Self {
			r: get_byte("r")?,
			g: get_byte("g")?,
			b: get_byte("b")?,
			a: get_byte("a")?,
		})
	}
}

impl ToJObject for NColour {
	fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
		let class = env
			.find_class("com/dropbear/utils/Colour")
			.map_err(|_| DropbearNativeError::JNIClassNotFound)?;

		let args = [
			JValue::Byte(self.r as i8),
			JValue::Byte(self.g as i8),
			JValue::Byte(self.b as i8),
			JValue::Byte(self.a as i8),
		];

		env.new_object(&class, "(BBBB)V", &args)
			.map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
	}
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct NRange {
	start: f32,
	end: f32,
}

impl FromJObject for NRange {
	fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self> {
		let class = env
			.find_class("com/dropbear/utils/Range")
			.map_err(|_| DropbearNativeError::JNIClassNotFound)?;

		if !env
			.is_instance_of(obj, &class)
			.map_err(|_| DropbearNativeError::JNIUnwrapFailed)?
		{
			return Err(DropbearNativeError::InvalidArgument);
		}

		let start = env
			.get_field(obj, "start", "D")
			.map_err(|_| DropbearNativeError::JNIFailedToGetField)?
			.d()
			.map_err(|_| DropbearNativeError::JNIUnwrapFailed)? as f32;

		let end = env
			.get_field(obj, "end", "D")
			.map_err(|_| DropbearNativeError::JNIFailedToGetField)?
			.d()
			.map_err(|_| DropbearNativeError::JNIUnwrapFailed)? as f32;

		Ok(Self { start, end })
	}
}

impl ToJObject for NRange {
	fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
		let class = env
			.find_class("com/dropbear/utils/Range")
			.map_err(|_| DropbearNativeError::JNIClassNotFound)?;

		let args = [
			JValue::Double(self.start as f64),
			JValue::Double(self.end as f64),
		];

		env.new_object(&class, "(DD)V", &args)
			.map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
	}
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct NAttenuation {
	constant: f32,
	linear: f32,
	quadratic: f32,
}

impl FromJObject for NAttenuation {
	fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self> {
		let class = env
			.find_class("com/dropbear/lighting/Attenuation")
			.map_err(|_| DropbearNativeError::JNIClassNotFound)?;

		if !env
			.is_instance_of(obj, &class)
			.map_err(|_| DropbearNativeError::JNIUnwrapFailed)?
		{
			return Err(DropbearNativeError::InvalidArgument);
		}

		let constant = env
			.call_method(obj, "getConstant", "()F", &[])
			.map_err(|_| DropbearNativeError::JNIFailedToGetField)?
			.f()
			.map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

		let linear = env
			.call_method(obj, "getLinear", "()F", &[])
			.map_err(|_| DropbearNativeError::JNIFailedToGetField)?
			.f()
			.map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

		let quadratic = env
			.call_method(obj, "getQuadratic", "()F", &[])
			.map_err(|_| DropbearNativeError::JNIFailedToGetField)?
			.f()
			.map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

		Ok(Self {
			constant,
			linear,
			quadratic,
		})
	}
}

impl ToJObject for NAttenuation {
	fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
		let class = env
			.find_class("com/dropbear/lighting/Attenuation")
			.map_err(|_| DropbearNativeError::JNIClassNotFound)?;

		let args = [
			JValue::Float(self.constant),
			JValue::Float(self.linear),
			JValue::Float(self.quadratic),
		];

		env.new_object(&class, "(FFF)V", &args)
			.map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
	}
}

pub mod shared {
	use hecs::{Entity, World};

	pub fn light_exists_for_entity(world: &World, entity: Entity) -> bool {
		world.get::<&dropbear_engine::lighting::LightComponent>(entity).is_ok()
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
    #[dropbear_macro::define(WorldPtr)]
    world: &World,
    #[dropbear_macro::entity]
    entity: Entity,
) -> DropbearNativeResult<bool> {
    Ok(shared::light_exists_for_entity(world, entity))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "getPosition"),
    c
)]
fn get_position(
    #[dropbear_macro::define(WorldPtr)]
    world: &World,
    #[dropbear_macro::entity]
    entity: Entity,
) -> DropbearNativeResult<NVector3> {
    let transform = get_transform(world, entity)?;
    Ok(NVector3::from(transform.position))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "setPosition"),
    c
)]
fn set_position(
    #[dropbear_macro::define(WorldPtr)]
    world: &mut World,
    #[dropbear_macro::entity]
    entity: Entity,
    position: &NVector3,
) -> DropbearNativeResult<()> {
    set_transform_position(world, entity, (*position).into())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "getDirection"),
    c
)]
fn get_direction(
    #[dropbear_macro::define(WorldPtr)]
    world: &World,
    #[dropbear_macro::entity]
    entity: Entity,
) -> DropbearNativeResult<NVector3> {
    let transform = get_transform(world, entity)?;
    let forward = DVec3::new(0.0, 0.0, -1.0);
    let dir = (transform.rotation * forward).normalize_or_zero();
    Ok(NVector3::from(dir))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "setDirection"),
    c
)]
fn set_direction(
    #[dropbear_macro::define(WorldPtr)]
    world: &mut World,
    #[dropbear_macro::entity]
    entity: Entity,
    direction: &NVector3,
) -> DropbearNativeResult<()> {
    let dir: DVec3 = (*direction).into();
    let desired = dir.normalize_or_zero();
    if desired.length_squared() < 1e-12 {
        return Err(DropbearNativeError::InvalidArgument);
    }

    let forward = DVec3::new(0.0, 0.0, -1.0);
    let rotation = DQuat::from_rotation_arc(forward, desired);
    set_transform_rotation(world, entity, rotation)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "getColour"),
    c
)]
fn get_colour(
    #[dropbear_macro::define(WorldPtr)]
    world: &World,
    #[dropbear_macro::entity]
    entity: Entity,
) -> DropbearNativeResult<NColour> {
    let light = world
        .get::<&LightComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    Ok(NColour::from_linear_rgb(light.colour))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "setColour"),
    c
)]
fn set_colour(
    #[dropbear_macro::define(WorldPtr)]
    world: &mut World,
    #[dropbear_macro::entity]
    entity: Entity,
    colour: &NColour,
) -> DropbearNativeResult<()> {
    let mut light = world
        .get::<&mut LightComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    light.colour = (*colour).to_linear_rgb();
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "getLightType"),
    c
)]
fn get_light_type(
    #[dropbear_macro::define(WorldPtr)]
    world: &World,
    #[dropbear_macro::entity]
    entity: Entity,
) -> DropbearNativeResult<i32> {
    let light = world
        .get::<&LightComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    Ok(light.light_type as i32)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "setLightType"),
    c
)]
fn set_light_type(
    #[dropbear_macro::define(WorldPtr)]
    world: &mut World,
    #[dropbear_macro::entity]
    entity: Entity,
    light_type: i32,
) -> DropbearNativeResult<()> {
    let mut light = world
        .get::<&mut LightComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;

    light.light_type = match light_type {
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
    #[dropbear_macro::define(WorldPtr)]
    world: &World,
    #[dropbear_macro::entity]
    entity: Entity,
) -> DropbearNativeResult<f64> {
    let light = world
        .get::<&LightComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    Ok(light.intensity as f64)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "setIntensity"),
    c
)]
fn set_intensity(
    #[dropbear_macro::define(WorldPtr)]
    world: &mut World,
    #[dropbear_macro::entity]
    entity: Entity,
    intensity: f64,
) -> DropbearNativeResult<()> {
    let mut light = world
        .get::<&mut LightComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    light.intensity = intensity as f32;
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "getAttenuation"),
    c
)]
fn get_attenuation(
    #[dropbear_macro::define(WorldPtr)]
    world: &World,
    #[dropbear_macro::entity]
    entity: Entity,
) -> DropbearNativeResult<NAttenuation> {
    let light = world
        .get::<&LightComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;

    Ok(NAttenuation {
        constant: light.attenuation.constant,
        linear: light.attenuation.linear,
        quadratic: light.attenuation.quadratic,
    })
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "setAttenuation"),
    c
)]
fn set_attenuation(
    #[dropbear_macro::define(WorldPtr)]
    world: &mut World,
    #[dropbear_macro::entity]
    entity: Entity,
    attenuation: &NAttenuation,
) -> DropbearNativeResult<()> {
    let mut light = world
        .get::<&mut LightComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;

    light.attenuation.constant = attenuation.constant;
    light.attenuation.linear = attenuation.linear;
    light.attenuation.quadratic = attenuation.quadratic;
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "getEnabled"),
    c
)]
fn get_enabled(
    #[dropbear_macro::define(WorldPtr)]
    world: &World,
    #[dropbear_macro::entity]
    entity: Entity,
) -> DropbearNativeResult<bool> {
    let light = world
        .get::<&LightComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    Ok(light.enabled)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "setEnabled"),
    c
)]
fn set_enabled(
    #[dropbear_macro::define(WorldPtr)]
    world: &mut World,
    #[dropbear_macro::entity]
    entity: Entity,
    enabled: bool,
) -> DropbearNativeResult<()> {
    let mut light = world
        .get::<&mut LightComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    light.enabled = enabled;
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "getCutoffAngle"),
    c
)]
fn get_cutoff_angle(
    #[dropbear_macro::define(WorldPtr)]
    world: &World,
    #[dropbear_macro::entity]
    entity: Entity,
) -> DropbearNativeResult<f64> {
    let light = world
        .get::<&LightComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    Ok(light.cutoff_angle as f64)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "setCutoffAngle"),
    c
)]
fn set_cutoff_angle(
    #[dropbear_macro::define(WorldPtr)]
    world: &mut World,
    #[dropbear_macro::entity]
    entity: Entity,
    cutoff_angle: f64,
) -> DropbearNativeResult<()> {
    let mut light = world
        .get::<&mut LightComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    light.cutoff_angle = cutoff_angle as f32;
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "getOuterCutoffAngle"),
    c
)]
fn get_outer_cutoff_angle(
    #[dropbear_macro::define(WorldPtr)]
    world: &World,
    #[dropbear_macro::entity]
    entity: Entity,
) -> DropbearNativeResult<f64> {
    let light = world
        .get::<&LightComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    Ok(light.outer_cutoff_angle as f64)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "setOuterCutoffAngle"),
    c
)]
fn set_outer_cutoff_angle(
    #[dropbear_macro::define(WorldPtr)]
    world: &mut World,
    #[dropbear_macro::entity]
    entity: Entity,
    outer_cutoff_angle: f64,
) -> DropbearNativeResult<()> {
    let mut light = world
        .get::<&mut LightComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    light.outer_cutoff_angle = outer_cutoff_angle as f32;
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "getCastsShadows"),
    c
)]
fn get_casts_shadows(
    #[dropbear_macro::define(WorldPtr)]
    world: &World,
    #[dropbear_macro::entity]
    entity: Entity,
) -> DropbearNativeResult<bool> {
    let light = world
        .get::<&LightComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    Ok(light.cast_shadows)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "setCastsShadows"),
    c
)]
fn set_casts_shadows(
    #[dropbear_macro::define(WorldPtr)]
    world: &mut World,
    #[dropbear_macro::entity]
    entity: Entity,
    casts_shadows: bool,
) -> DropbearNativeResult<()> {
    let mut light = world
        .get::<&mut LightComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    light.cast_shadows = casts_shadows;
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "getDepth"),
    c
)]
fn get_depth(
    #[dropbear_macro::define(WorldPtr)]
    world: &World,
    #[dropbear_macro::entity]
    entity: Entity,
) -> DropbearNativeResult<NRange> {
    let light = world
        .get::<&LightComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;

    Ok(NRange {
        start: light.depth.start,
        end: light.depth.end,
    })
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.lighting.LightNative", func = "setDepth"),
    c
)]
fn set_depth(
    #[dropbear_macro::define(WorldPtr)]
    world: &mut World,
    #[dropbear_macro::entity]
    entity: Entity,
    depth: &NRange,
) -> DropbearNativeResult<()> {
    if !(depth.start.is_finite() && depth.end.is_finite()) {
        return Err(DropbearNativeError::InvalidArgument);
    }
    if depth.end <= depth.start {
        return Err(DropbearNativeError::InvalidArgument);
    }

    let mut light = world
        .get::<&mut LightComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    light.depth = depth.start..depth.end;
    Ok(())
}