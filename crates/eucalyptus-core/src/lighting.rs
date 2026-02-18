use std::sync::Arc;
use egui::{CollapsingHeader, ComboBox, DragValue, Ui};
use crate::ptr::WorldPtr;
use crate::scripting::jni::utils::{FromJObject, ToJObject};
use crate::scripting::native::DropbearNativeError;
use crate::scripting::result::DropbearNativeResult;
use crate::types::NVector3;
use dropbear_engine::entity::{EntityTransform, Transform};
use dropbear_engine::lighting::{Light, LightType};
use glam::{DQuat, DVec3, Vec3};
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
    type RequiredComponentTypes = (Self, Transform);

    fn descriptor() -> ComponentDescriptor {
        ComponentDescriptor {
            fqtn: "dropbear_engine::lighting::Light".to_string(),
            type_name: "Light".to_string(),
            category: Some("Lighting".to_string()),
            description: Some("An object that emits light".to_string()),
        }
    }

    fn init<'a>(
        ser: &'a Self::SerializedForm,
        graphics: Arc<SharedGraphicsContext>,
    ) -> ComponentInitFuture<'a, Self> {
        Box::pin(async move {
            let light_component = ser.light_component.clone();
            let light = Light::new(
                graphics.clone(),
                light_component.clone(),
                Some(ser.label.as_str())
            ).await;
            let transform = light_component.to_transform();

            Ok((light, transform))
        })
    }

    fn update_component(&mut self, world: &World, _physics: &mut crate::physics::PhysicsState, entity: Entity, _dt: f32, graphics: Arc<SharedGraphicsContext>) {
        let synced = &mut self.component;
        if let Ok(entity_transform) = world.query_one::<&EntityTransform>(entity).get() {
            let transform = entity_transform.sync();
            synced.position = transform.position;
            synced.direction = (transform.rotation * DVec3::new(0.0, 0.0, -1.0)).normalize_or_zero();
        } else if let Ok(transform) = world.query_one::<&Transform>(entity).get() {
            synced.position = transform.position;
            synced.direction = (transform.rotation * DVec3::new(0.0, 0.0, -1.0)).normalize_or_zero();
        }

        self.update(&graphics);
    }

    fn save(&self, _: &World, entity: Entity) -> Box<dyn SerializedComponent> {
        Box::new(SerializedLight {
            label: self.label.clone(),
            light_component: self.component.clone(),
            entity_id: Some(entity),
        })
    }
}

impl InspectableComponent for Light {
    fn inspect(&mut self, ui: &mut Ui, _graphics: Arc<SharedGraphicsContext>) {
        CollapsingHeader::new("Light").default_open(true).show(ui, |ui| {
            ui.add_space(6.0);
            ui.label("Uniform");

            ui.label("Light Type");
            ComboBox::from_id_salt("Light Type").show_ui(ui, |ui| {
                ui.selectable_value(&mut self.component.light_type, LightType::Directional, "Directional");
                ui.selectable_value(&mut self.component.light_type, LightType::Point, "Point");
                ui.selectable_value(&mut self.component.light_type, LightType::Spot, "Spot");
            });

            let mut display_pos = |yueye: &mut Ui| {
                yueye.horizontal(|yueye| {
                    yueye.label("Position");
                    yueye.add(DragValue::new(&mut self.component.position.x).speed(0.01));
                    yueye.add(DragValue::new(&mut self.component.position.y).speed(0.01));
                    yueye.add(DragValue::new(&mut self.component.position.z).speed(0.01));
                });
            };

            let mut display_dir = |yueye: &mut Ui| {
                yueye.horizontal(|yueye| {
                    yueye.label("Direction");
                    yueye.add(DragValue::new(&mut self.component.direction.x).speed(0.01));
                    yueye.add(DragValue::new(&mut self.component.direction.y).speed(0.01));
                    yueye.add(DragValue::new(&mut self.component.direction.z).speed(0.01));
                });
            };

            match self.component.light_type {
                LightType::Directional => {
                    display_dir(ui);
                },
                LightType::Point => {
                    display_pos(ui);
                },
                LightType::Spot => {
                    display_pos(ui);
                    display_dir(ui);
                },
            }

            let mut colour_rgb = [
                self.component.colour[0] as f32,
                self.component.colour[1] as f32,
                self.component.colour[2] as f32,
            ];
            egui::color_picker::color_edit_button_rgb(ui, &mut colour_rgb);
            self.component.colour = Vec3::from_array(colour_rgb).as_dvec3();

            ui.horizontal(|ui| {
                ui.label("Intensity");
                ui.add(DragValue::new(&mut self.component.intensity).speed(0.05));
            });

            if matches!(self.component.light_type, LightType::Point | LightType::Spot) {
                ui.horizontal(|ui| {
                    ui.label("Attenuation");
                    ui.add(DragValue::new(&mut self.component.attenuation.constant).speed(0.01));
                    ui.add(DragValue::new(&mut self.component.attenuation.linear).speed(0.01));
                    ui.add(DragValue::new(&mut self.component.attenuation.quadratic).speed(0.01));
                });
            }

            ui.horizontal(|ui| {
                ui.checkbox(&mut self.component.enabled, "Enabled");
                ui.checkbox(&mut self.component.visible, "Visible");
            });

            if matches!(self.component.light_type, LightType::Spot) {
                ui.horizontal(|ui| {
                    ui.label("Cutoff");
                    ui.add(DragValue::new(&mut self.component.cutoff_angle).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label("Outer Cutoff");
                    ui.add(DragValue::new(&mut self.component.outer_cutoff_angle).speed(0.1));
                });

                if self.component.outer_cutoff_angle <= self.component.cutoff_angle {
                    self.component.outer_cutoff_angle = self.component.cutoff_angle + 1.0;
                }
            }

            ui.separator();
            ui.label("Shadows");
            ui.checkbox(&mut self.component.cast_shadows, "Cast Shadows");
            ui.horizontal(|ui| {
                ui.label("Depth");
                ui.add(DragValue::new(&mut self.component.depth.start).speed(0.1));
                ui.label("..");
                ui.add(DragValue::new(&mut self.component.depth.end).speed(0.1));
            });

            if self.component.depth.end < self.component.depth.start {
                self.component.depth.end = self.component.depth.start;
            }

            
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
        .get::<&Light>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    Ok(NColour::from_linear_rgb(light.component.colour))
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
        .get::<&mut Light>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    light.component.colour = (*colour).to_linear_rgb();
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
        .get::<&Light>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    Ok(light.component.light_type as i32)
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
    #[dropbear_macro::define(WorldPtr)]
    world: &World,
    #[dropbear_macro::entity]
    entity: Entity,
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
    #[dropbear_macro::define(WorldPtr)]
    world: &mut World,
    #[dropbear_macro::entity]
    entity: Entity,
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
    #[dropbear_macro::define(WorldPtr)]
    world: &World,
    #[dropbear_macro::entity]
    entity: Entity,
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
    #[dropbear_macro::define(WorldPtr)]
    world: &mut World,
    #[dropbear_macro::entity]
    entity: Entity,
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
    #[dropbear_macro::define(WorldPtr)]
    world: &World,
    #[dropbear_macro::entity]
    entity: Entity,
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
    #[dropbear_macro::define(WorldPtr)]
    world: &mut World,
    #[dropbear_macro::entity]
    entity: Entity,
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
    #[dropbear_macro::define(WorldPtr)]
    world: &World,
    #[dropbear_macro::entity]
    entity: Entity,
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
    #[dropbear_macro::define(WorldPtr)]
    world: &mut World,
    #[dropbear_macro::entity]
    entity: Entity,
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
    #[dropbear_macro::define(WorldPtr)]
    world: &World,
    #[dropbear_macro::entity]
    entity: Entity,
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
    #[dropbear_macro::define(WorldPtr)]
    world: &mut World,
    #[dropbear_macro::entity]
    entity: Entity,
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
    #[dropbear_macro::define(WorldPtr)]
    world: &World,
    #[dropbear_macro::entity]
    entity: Entity,
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
    #[dropbear_macro::define(WorldPtr)]
    world: &mut World,
    #[dropbear_macro::entity]
    entity: Entity,
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
    #[dropbear_macro::define(WorldPtr)]
    world: &World,
    #[dropbear_macro::entity]
    entity: Entity,
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
        .get::<&mut Light>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    light.component.depth = depth.start..depth.end;
    Ok(())
}