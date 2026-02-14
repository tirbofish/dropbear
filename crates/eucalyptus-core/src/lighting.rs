use crate::scripting::jni::utils::{FromJObject, ToJObject};
use crate::scripting::native::DropbearNativeError;
use crate::scripting::result::DropbearNativeResult;
use glam::DVec3;
use ::jni::objects::{JObject, JValue};
use ::jni::JNIEnv;

#[derive(Clone, Copy, Debug)]
struct JvmColour {
	r: u8,
	g: u8,
	b: u8,
	a: u8,
}

impl JvmColour {
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

impl FromJObject for JvmColour {
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

impl ToJObject for JvmColour {
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

#[derive(Clone, Copy, Debug)]
struct JvmRange {
	start: f32,
	end: f32,
}

impl FromJObject for JvmRange {
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

impl ToJObject for JvmRange {
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

#[derive(Clone, Copy, Debug)]
struct JvmAttenuation {
	constant: f32,
	linear: f32,
	quadratic: f32,
}

impl FromJObject for JvmAttenuation {
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

impl ToJObject for JvmAttenuation {
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
	use dropbear_engine::lighting::LightComponent;
	use hecs::{Entity, World};

	pub fn light_exists_for_entity(world: &World, entity: Entity) -> bool {
		world.get::<&LightComponent>(entity).is_ok()
	}
}

pub mod jni {
	#![allow(non_snake_case)]

	use super::{JvmAttenuation, JvmColour, JvmRange};
	use crate::scripting::jni::utils::{FromJObject, ToJObject};
	use crate::types::NVector3;
	use crate::{convert_jlong_to_entity, convert_ptr};
	use dropbear_engine::entity::{EntityTransform, Transform};
	use dropbear_engine::lighting::{LightComponent, LightType};
	use glam::{DQuat, DVec3};
	use hecs::World;
	use jni::objects::{JClass, JObject};
	use jni::sys::{jboolean, jdouble, jint, jlong, jobject};
	use jni::JNIEnv;

	fn get_transform(world: &World, entity: hecs::Entity) -> Option<Transform> {
		if let Ok(et) = world.get::<&EntityTransform>(entity) {
			Some(et.sync())
		} else if let Ok(t) = world.get::<&Transform>(entity) {
			Some(*t)
		} else {
			None
		}
	}

	fn set_transform_position(world: &mut World, entity: hecs::Entity, position: DVec3) -> bool {
		if let Ok(mut et) = world.get::<&mut EntityTransform>(entity) {
			et.local_mut().position = position;
			true
		} else if let Ok(mut t) = world.get::<&mut Transform>(entity) {
			t.position = position;
			true
		} else {
			false
		}
	}

	fn set_transform_rotation(world: &mut World, entity: hecs::Entity, rotation: DQuat) -> bool {
		if let Ok(mut et) = world.get::<&mut EntityTransform>(entity) {
			et.local_mut().rotation = rotation;
			true
		} else if let Ok(mut t) = world.get::<&mut Transform>(entity) {
			t.rotation = rotation;
			true
		} else {
			false
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_lighting_LightNative_lightExistsForEntity(
		_env: JNIEnv,
		_class: JClass,
		world_ptr: jlong,
		entity_id: jlong,
	) -> jboolean {
		let world = convert_ptr!(world_ptr => World);
		let entity = convert_jlong_to_entity!(entity_id);
		if world.get::<&LightComponent>(entity).is_ok() {
			1
		} else {
			0
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_lighting_LightNative_getPosition(
		mut env: JNIEnv,
		_class: JClass,
		world_ptr: jlong,
		entity_id: jlong,
	) -> jobject {
		let world = convert_ptr!(world_ptr => World);
		let entity = convert_jlong_to_entity!(entity_id);

		let Some(t) = get_transform(world, entity) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Entity missing Transform/EntityTransform");
			return std::ptr::null_mut();
		};

		match NVector3::from(t.position).to_jobject(&mut env) {
			Ok(obj) => obj.into_raw(),
			Err(_) => std::ptr::null_mut(),
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_lighting_LightNative_setPosition(
		mut env: JNIEnv,
		_class: JClass,
		world_ptr: jlong,
		entity_id: jlong,
		position: JObject,
	) {
		let world = convert_ptr!(mut world_ptr => World);
		let entity = convert_jlong_to_entity!(entity_id);

		let position: DVec3 = match NVector3::from_jobject(&mut env, &position) {
			Ok(v) => v.into(),
			Err(e) => {
				let _ = env.throw_new(
					"java/lang/IllegalArgumentException",
					format!("Invalid Vector3d: {:?}", e),
				);
				return;
			}
		};

		if !set_transform_position(world, entity, position) {
			let _ = env.throw_new("java/lang/RuntimeException", "Entity missing Transform/EntityTransform");
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_lighting_LightNative_getDirection(
		mut env: JNIEnv,
		_class: JClass,
		world_ptr: jlong,
		entity_id: jlong,
	) -> jobject {
		let world = convert_ptr!(world_ptr => World);
		let entity = convert_jlong_to_entity!(entity_id);

		let Some(t) = get_transform(world, entity) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Entity missing Transform/EntityTransform");
			return std::ptr::null_mut();
		};

		let forward = DVec3::new(0.0, 0.0, -1.0);
		let dir = (t.rotation * forward).normalize_or_zero();

		match NVector3::from(dir).to_jobject(&mut env) {
			Ok(obj) => obj.into_raw(),
			Err(_) => std::ptr::null_mut(),
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_lighting_LightNative_setDirection(
		mut env: JNIEnv,
		_class: JClass,
		world_ptr: jlong,
		entity_id: jlong,
		direction: JObject,
	) {
		let world = convert_ptr!(mut world_ptr => World);
		let entity = convert_jlong_to_entity!(entity_id);

		let dir: DVec3 = match NVector3::from_jobject(&mut env, &direction) {
			Ok(v) => DVec3::from(v),
			Err(e) => {
				let _ = env.throw_new(
					"java/lang/IllegalArgumentException",
					format!("Invalid Vector3d: {:?}", e),
				);
				return;
			}
		};

		let desired = dir.normalize_or_zero();
		if desired.length_squared() < 1e-12 {
			let _ = env.throw_new("java/lang/IllegalArgumentException", "Direction must be non-zero");
			return;
		}

		let forward = DVec3::new(0.0, 0.0, -1.0);
		let rotation = DQuat::from_rotation_arc(forward, desired);

		if !set_transform_rotation(world, entity, rotation) {
			let _ = env.throw_new("java/lang/RuntimeException", "Entity missing Transform/EntityTransform");
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_lighting_LightNative_getColour(
		mut env: JNIEnv,
		_class: JClass,
		world_ptr: jlong,
		entity_id: jlong,
	) -> jobject {
		let world = convert_ptr!(world_ptr => World);
		let entity = convert_jlong_to_entity!(entity_id);

		let Ok(light) = world.get::<&LightComponent>(entity) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Entity missing LightComponent");
			return std::ptr::null_mut();
		};

		let colour = JvmColour::from_linear_rgb(light.colour);
		match colour.to_jobject(&mut env) {
			Ok(obj) => obj.into_raw(),
			Err(_) => std::ptr::null_mut(),
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_lighting_LightNative_setColour(
		mut env: JNIEnv,
		_class: JClass,
		world_ptr: jlong,
		entity_id: jlong,
		colour: JObject,
	) {
		let world = convert_ptr!(mut world_ptr => World);
		let entity = convert_jlong_to_entity!(entity_id);

		let Ok(mut light) = world.get::<&mut LightComponent>(entity) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Entity missing LightComponent");
			return;
		};

		let colour = match JvmColour::from_jobject(&mut env, &colour) {
			Ok(c) => c,
			Err(e) => {
				let _ = env.throw_new(
					"java/lang/IllegalArgumentException",
					format!("Invalid Colour: {:?}", e),
				);
				return;
			}
		};

		light.colour = colour.to_linear_rgb();
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_lighting_LightNative_getLightType(
		mut env: JNIEnv,
		_class: JClass,
		world_ptr: jlong,
		entity_id: jlong,
	) -> jint {
		let world = convert_ptr!(world_ptr => World);
		let entity = convert_jlong_to_entity!(entity_id);

		let Ok(light) = world.get::<&LightComponent>(entity) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Entity missing LightComponent");
			return -1;
		};

		light.light_type as i32
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_lighting_LightNative_setLightType(
		mut env: JNIEnv,
		_class: JClass,
		world_ptr: jlong,
		entity_id: jlong,
		light_type: jint,
	) {
		let world = convert_ptr!(mut world_ptr => World);
		let entity = convert_jlong_to_entity!(entity_id);

		let Ok(mut light) = world.get::<&mut LightComponent>(entity) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Entity missing LightComponent");
			return;
		};

		light.light_type = match light_type {
			0 => LightType::Directional,
			1 => LightType::Point,
			2 => LightType::Spot,
			_ => {
				let _ = env.throw_new("java/lang/IllegalArgumentException", "Invalid lightType ordinal");
				return;
			}
		};
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_lighting_LightNative_getIntensity(
		mut env: JNIEnv,
		_class: JClass,
		world_ptr: jlong,
		entity_id: jlong,
	) -> jdouble {
		let world = convert_ptr!(world_ptr => World);
		let entity = convert_jlong_to_entity!(entity_id);

		let Ok(light) = world.get::<&LightComponent>(entity) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Entity missing LightComponent");
			return f64::NAN;
		};

		light.intensity as f64
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_lighting_LightNative_setIntensity(
		mut env: JNIEnv,
		_class: JClass,
		world_ptr: jlong,
		entity_id: jlong,
		intensity: jdouble,
	) {
		let world = convert_ptr!(mut world_ptr => World);
		let entity = convert_jlong_to_entity!(entity_id);

		let Ok(mut light) = world.get::<&mut LightComponent>(entity) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Entity missing LightComponent");
			return;
		};

		light.intensity = intensity as f32;
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_lighting_LightNative_getAttenuation(
		mut env: JNIEnv,
		_class: JClass,
		world_ptr: jlong,
		entity_id: jlong,
	) -> jobject {
		let world = convert_ptr!(world_ptr => World);
		let entity = convert_jlong_to_entity!(entity_id);

		let Ok(light) = world.get::<&LightComponent>(entity) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Entity missing LightComponent");
			return std::ptr::null_mut();
		};

		let att = JvmAttenuation {
			constant: light.attenuation.constant,
			linear: light.attenuation.linear,
			quadratic: light.attenuation.quadratic,
		};

		match att.to_jobject(&mut env) {
			Ok(obj) => obj.into_raw(),
			Err(_) => std::ptr::null_mut(),
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_lighting_LightNative_setAttenuation(
		mut env: JNIEnv,
		_class: JClass,
		world_ptr: jlong,
		entity_id: jlong,
		attenuation: JObject,
	) {
		let world = convert_ptr!(mut world_ptr => World);
		let entity = convert_jlong_to_entity!(entity_id);

		let Ok(mut light) = world.get::<&mut LightComponent>(entity) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Entity missing LightComponent");
			return;
		};

		let att = match JvmAttenuation::from_jobject(&mut env, &attenuation) {
			Ok(a) => a,
			Err(e) => {
				let _ = env.throw_new(
					"java/lang/IllegalArgumentException",
					format!("Invalid Attenuation: {:?}", e),
				);
				return;
			}
		};

		// Kotlin exposes only coefficients; preserve existing `range`.
		light.attenuation.constant = att.constant;
		light.attenuation.linear = att.linear;
		light.attenuation.quadratic = att.quadratic;
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_lighting_LightNative_getEnabled(
		mut env: JNIEnv,
		_class: JClass,
		world_ptr: jlong,
		entity_id: jlong,
	) -> jboolean {
		let world = convert_ptr!(world_ptr => World);
		let entity = convert_jlong_to_entity!(entity_id);

		let Ok(light) = world.get::<&LightComponent>(entity) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Entity missing LightComponent");
			return 0;
		};

		if light.enabled { 1 } else { 0 }
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_lighting_LightNative_setEnabled(
		mut env: JNIEnv,
		_class: JClass,
		world_ptr: jlong,
		entity_id: jlong,
		enabled: jboolean,
	) {
		let world = convert_ptr!(mut world_ptr => World);
		let entity = convert_jlong_to_entity!(entity_id);

		let Ok(mut light) = world.get::<&mut LightComponent>(entity) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Entity missing LightComponent");
			return;
		};

		light.enabled = enabled != 0;
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_lighting_LightNative_getCutoffAngle(
		mut env: JNIEnv,
		_class: JClass,
		world_ptr: jlong,
		entity_id: jlong,
	) -> jdouble {
		let world = convert_ptr!(world_ptr => World);
		let entity = convert_jlong_to_entity!(entity_id);

		let Ok(light) = world.get::<&LightComponent>(entity) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Entity missing LightComponent");
			return f64::NAN;
		};

		light.cutoff_angle as f64
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_lighting_LightNative_setCutoffAngle(
		mut env: JNIEnv,
		_class: JClass,
		world_ptr: jlong,
		entity_id: jlong,
		cutoff_angle: jdouble,
	) {
		let world = convert_ptr!(mut world_ptr => World);
		let entity = convert_jlong_to_entity!(entity_id);

		let Ok(mut light) = world.get::<&mut LightComponent>(entity) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Entity missing LightComponent");
			return;
		};

		light.cutoff_angle = cutoff_angle as f32;
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_lighting_LightNative_getOuterCutoffAngle(
		mut env: JNIEnv,
		_class: JClass,
		world_ptr: jlong,
		entity_id: jlong,
	) -> jdouble {
		let world = convert_ptr!(world_ptr => World);
		let entity = convert_jlong_to_entity!(entity_id);

		let Ok(light) = world.get::<&LightComponent>(entity) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Entity missing LightComponent");
			return f64::NAN;
		};

		light.outer_cutoff_angle as f64
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_lighting_LightNative_setOuterCutoffAngle(
		mut env: JNIEnv,
		_class: JClass,
		world_ptr: jlong,
		entity_id: jlong,
		outer_cutoff_angle: jdouble,
	) {
		let world = convert_ptr!(mut world_ptr => World);
		let entity = convert_jlong_to_entity!(entity_id);

		let Ok(mut light) = world.get::<&mut LightComponent>(entity) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Entity missing LightComponent");
			return;
		};

		light.outer_cutoff_angle = outer_cutoff_angle as f32;
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_lighting_LightNative_getCastsShadows(
		mut env: JNIEnv,
		_class: JClass,
		world_ptr: jlong,
		entity_id: jlong,
	) -> jboolean {
		let world = convert_ptr!(world_ptr => World);
		let entity = convert_jlong_to_entity!(entity_id);

		let Ok(light) = world.get::<&LightComponent>(entity) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Entity missing LightComponent");
			return 0;
		};

		if light.cast_shadows { 1 } else { 0 }
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_lighting_LightNative_setCastsShadows(
		mut env: JNIEnv,
		_class: JClass,
		world_ptr: jlong,
		entity_id: jlong,
		casts_shadows: jboolean,
	) {
		let world = convert_ptr!(mut world_ptr => World);
		let entity = convert_jlong_to_entity!(entity_id);

		let Ok(mut light) = world.get::<&mut LightComponent>(entity) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Entity missing LightComponent");
			return;
		};

		light.cast_shadows = casts_shadows != 0;
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_lighting_LightNative_getDepth(
		mut env: JNIEnv,
		_class: JClass,
		world_ptr: jlong,
		entity_id: jlong,
	) -> jobject {
		let world = convert_ptr!(world_ptr => World);
		let entity = convert_jlong_to_entity!(entity_id);

		let Ok(light) = world.get::<&LightComponent>(entity) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Entity missing LightComponent");
			return std::ptr::null_mut();
		};

		let range = JvmRange {
			start: light.depth.start,
			end: light.depth.end,
		};

		match range.to_jobject(&mut env) {
			Ok(obj) => obj.into_raw(),
			Err(_) => std::ptr::null_mut(),
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_lighting_LightNative_setDepth(
		mut env: JNIEnv,
		_class: JClass,
		world_ptr: jlong,
		entity_id: jlong,
		depth: JObject,
	) {
		let world = convert_ptr!(mut world_ptr => World);
		let entity = convert_jlong_to_entity!(entity_id);

		let Ok(mut light) = world.get::<&mut LightComponent>(entity) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Entity missing LightComponent");
			return;
		};

		let range = match JvmRange::from_jobject(&mut env, &depth) {
			Ok(r) => r,
			Err(e) => {
				let _ = env.throw_new(
					"java/lang/IllegalArgumentException",
					format!("Invalid Range: {:?}", e),
				);
				return;
			}
		};

		if !(range.start.is_finite() && range.end.is_finite()) {
			let _ = env.throw_new("java/lang/IllegalArgumentException", "Depth range must be finite");
			return;
		}
		if range.end <= range.start {
			let _ = env.throw_new("java/lang/IllegalArgumentException", "Depth range must satisfy start < end");
			return;
		}

		light.depth = range.start..range.end;
	}
}