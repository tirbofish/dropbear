//! [rapier3d] RigidBodies

use crate::scripting::jni::utils::{FromJObject, ToJObject};
use crate::scripting::native::DropbearNativeError;
use crate::scripting::result::DropbearNativeResult;
use crate::states::Label;
use dropbear_macro::SerializableComponent;
use dropbear_traits::SerializableComponent;
use ::jni::objects::{JObject, JValue};
use ::jni::JNIEnv;
use rapier3d::prelude::RigidBodyType;
use serde::{Deserialize, Serialize};

/// How this entity behaves in the physics simulation.
///
/// This intentionally mirrors Rapier's rigid-body types, but stays engine-owned and serializable.
#[repr(C)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum RigidBodyMode {
	/// A fully simulated body affected by forces and contacts.
	Dynamic = 0,
	/// An immovable body.
	Fixed,
	/// A kinematic body controlled by setting its next position.
	KinematicPosition,
	/// A kinematic body controlled by setting its velocities.
	KinematicVelocity,
}

impl Default for RigidBodyMode {
	fn default() -> Self {
		Self::Dynamic
	}
}

impl From<RigidBodyType> for RigidBodyMode {
	fn from(value: RigidBodyType) -> Self {
		match value {
			RigidBodyType::Dynamic => RigidBodyMode::Dynamic,
			RigidBodyType::Fixed => Self::Fixed,
			RigidBodyType::KinematicPositionBased => Self::KinematicPosition,
			RigidBodyType::KinematicVelocityBased => Self::KinematicVelocity,
		}
	}
}

#[repr(C)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
pub struct AxisLock {
	pub x: bool,
	pub y: bool,
	pub z: bool,
}

impl FromJObject for AxisLock {
	fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self>
	where
		Self: Sized
	{
		let class = env.find_class("com/dropbear/physics/AxisLock")
			.map_err(|_| DropbearNativeError::JNIClassNotFound)?;

		if !env.is_instance_of(obj, &class)
			.map_err(|_| DropbearNativeError::JNIUnwrapFailed)?
		{
			return Err(DropbearNativeError::InvalidArgument);
		}

		let x = env.get_field(obj, "x", "Z")
			.map_err(|_| DropbearNativeError::JNIFailedToGetField)?
			.z()
			.map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

		let y = env.get_field(obj, "y", "Z")
			.map_err(|_| DropbearNativeError::JNIFailedToGetField)?
			.z()
			.map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

		let z = env.get_field(obj, "z", "Z")
			.map_err(|_| DropbearNativeError::JNIFailedToGetField)?
			.z()
			.map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

		Ok(AxisLock { x, y, z })
	}
}

impl ToJObject for AxisLock {
	fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
		let class = env.find_class("com/dropbear/physics/AxisLock")
			.map_err(|_| DropbearNativeError::JNIClassNotFound)?;

		let constructor_sig = "(ZZZ)V";

		let args = [
			JValue::Bool(self.x as u8),
			JValue::Bool(self.y as u8),
			JValue::Bool(self.z as u8),
		];

		let obj = env.new_object(&class, constructor_sig, &args)
			.map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

		Ok(obj)
	}
}

/// A serializable physics rigid-body component.
///
/// Notes:
/// - This component should NOT store Rapier handles (`RigidBodyHandle`, `ColliderHandle`, ...).
///   Those are runtime-only and belong in a physics-world resource/system.
/// - The body's initial pose should typically come from your `EntityTransform`/`Transform`.
/// - Colliders/material (shape, friction, restitution, sensor, etc.) should usually be a separate
///   component (e.g. `Collider`).
#[derive(Debug, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct RigidBody {
	/// The entity this component is attached to.
	#[serde(default)]
	pub entity: Label,

	#[serde(default)]
	pub disable_physics: bool,

	/// Body type/mode.
	#[serde(default)]
	pub mode: RigidBodyMode,

	/// Scaling factor applied to gravity for this body.
	#[serde(default = "RigidBody::default_gravity_scale")]
	pub gravity_scale: f32,
	
	/// If the rigidbody is currently sleeping or not. 
	pub sleeping: bool,

	/// Whether this body is allowed to sleep.
	#[serde(default = "RigidBody::default_sleep")]
	pub can_sleep: bool,

	/// Whether continuous collision detection is enabled.
	#[serde(default)]
	pub ccd_enabled: bool,

	/// Initial linear velocity (m/s).
	#[serde(default)]
	pub linvel: [f32; 3],

	/// Initial angular velocity (rad/s).
	#[serde(default)]
	pub angvel: [f32; 3],

	/// Linear damping coefficient.
	#[serde(default)]
	pub linear_damping: f32,

	/// Angular damping coefficient.
	#[serde(default)]
	pub angular_damping: f32,

	/// Locks translation along specific axes.
	#[serde(default)]
	pub lock_translation: AxisLock,

	/// Locks rotation around specific axes.
	#[serde(default)]
	pub lock_rotation: AxisLock,
}

impl Default for RigidBody {
	fn default() -> Self {
		Self {
			entity: Label::default(),
			disable_physics: false,
			mode: RigidBodyMode::default(),
			gravity_scale: Self::default_gravity_scale(),
			sleeping: false,
			can_sleep: Self::default_sleep(),
			ccd_enabled: false,
			linvel: [0.0, 0.0, 0.0],
			angvel: [0.0, 0.0, 0.0],
			linear_damping: 0.0,
			angular_damping: 0.0,
			lock_translation: AxisLock::default(),
			lock_rotation: AxisLock::default(),
		}
	}
}

impl RigidBody {
	const fn default_gravity_scale() -> f32 {
		1.0
	}

	const fn default_sleep() -> bool {
		true
	}
}

pub mod shared {
	use crate::physics::rigidbody::{AxisLock, RigidBody, RigidBodyMode};
	use crate::physics::PhysicsState;
	use crate::scripting::native::DropbearNativeError;
	use crate::scripting::result::DropbearNativeResult;
	use crate::states::Label;
	use crate::types::{IndexNative, RigidBodyContext, Vector3};
	use hecs::{Entity, World};
	use rapier3d::dynamics::{RigidBodyHandle, RigidBodyType};
	use rapier3d::prelude::vector;
	use rapier3d::prelude::{nalgebra, ColliderHandle, LockedAxes};

	pub fn rigid_body_exists_for_entity(
		world: &World,
		physics: &PhysicsState,
		entity: Entity
	) -> Option<IndexNative> {
		if let Ok(mut q) = world.query_one::<(&Label, &RigidBody)>(entity)
			&& let Some((label, _)) = q.get()
		{
			if let Some(handle) = physics.bodies_entity_map.get(label) {
				Some(IndexNative::from(handle.0))
			} else {
				None
			}
		} else {
			None
		}
	}

	pub fn get_rigidbody_type(physics: &PhysicsState, rb_context: RigidBodyContext) -> DropbearNativeResult<RigidBodyType> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get(handle) {
			Ok(rb.body_type())
		} else {
			Err(DropbearNativeError::PhysicsObjectNotFound)
		}
	}

	pub fn set_rigidbody_type(physics: &mut PhysicsState, world: &World, rb_context: RigidBodyContext, mode: i64) -> DropbearNativeResult<()> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get_mut(handle) {
			let mode = match mode {
				0 => RigidBodyType::Dynamic,
				1 => RigidBodyType::Fixed,
				2 => RigidBodyType::KinematicPositionBased,
				3 => RigidBodyType::KinematicVelocityBased,
				_ => { return Err(DropbearNativeError::InvalidArgument); }
			};
			rb.set_body_type(mode, true);

			let entity = Entity::from_bits(rb_context.entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

			if let Ok(mut q) = world.query_one::<&mut RigidBody>(entity) && let Some(rb) = q.get() {
				let rb_mode = RigidBodyMode::from(mode);
				rb.mode = rb_mode;
			}

			Ok(())
		} else {
			Err(DropbearNativeError::NoSuchHandle)
		}
	}

	pub fn get_rigidbody_gravity_scale(physics: &PhysicsState, rb_context: RigidBodyContext) -> DropbearNativeResult<f64> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get(handle) {
			Ok(rb.gravity_scale().into())
		} else {
			Err(DropbearNativeError::PhysicsObjectNotFound)
		}
	}
	
	pub fn set_rigidbody_gravity_scale(physics: &mut PhysicsState, world: &World, rb_context: RigidBodyContext, new_scale: f64) -> DropbearNativeResult<()> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get_mut(handle) {
			rb.set_gravity_scale(new_scale as f32, true);
			
			let entity = Entity::from_bits(rb_context.entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

			if let Ok(mut q) = world.query_one::<&mut RigidBody>(entity) && let Some(rb) = q.get() {
				rb.gravity_scale = new_scale as f32;				
			}

			Ok(())
		} else {
			Err(DropbearNativeError::NoSuchHandle)
		}
	}

	pub fn get_rigidbody_linear_damping(physics: &PhysicsState, rb_context: RigidBodyContext) -> DropbearNativeResult<f64> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get(handle) {
			Ok(rb.linear_damping().into())
		} else {
			Err(DropbearNativeError::PhysicsObjectNotFound)
		}
	}

	pub fn set_rigidbody_linear_damping(physics: &mut PhysicsState, world: &World, rb_context: RigidBodyContext, new: f64) -> DropbearNativeResult<()> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get_mut(handle) {
			rb.set_linear_damping(new as f32);

			let entity = Entity::from_bits(rb_context.entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

			if let Ok(mut q) = world.query_one::<&mut RigidBody>(entity) && let Some(rb) = q.get() {
				rb.linear_damping = new as f32;
			}

			Ok(())
		} else {
			Err(DropbearNativeError::NoSuchHandle)
		}
	}

	pub fn get_rigidbody_angular_damping(physics: &PhysicsState, rb_context: RigidBodyContext) -> DropbearNativeResult<f64> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get(handle) {
			Ok(rb.angular_damping().into())
		} else {
			Err(DropbearNativeError::PhysicsObjectNotFound)
		}
	}

	pub fn set_rigidbody_angular_damping(physics: &mut PhysicsState, world: &World, rb_context: RigidBodyContext, new: f64) -> DropbearNativeResult<()> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get_mut(handle) {
			rb.set_angular_damping(new as f32);

			let entity = Entity::from_bits(rb_context.entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

			if let Ok(mut q) = world.query_one::<&mut RigidBody>(entity) && let Some(rb) = q.get() {
				rb.angular_damping = new as f32;
			}

			Ok(())
		} else {
			Err(DropbearNativeError::NoSuchHandle)
		}
	}

	pub fn get_rigidbody_sleep(physics: &PhysicsState, rb_context: RigidBodyContext) -> DropbearNativeResult<bool> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get(handle) {
			Ok(rb.is_sleeping().into())
		} else {
			Err(DropbearNativeError::PhysicsObjectNotFound)
		}
	}

	pub fn set_rigidbody_sleep(physics: &mut PhysicsState, world: &World, rb_context: RigidBodyContext, new: bool) -> DropbearNativeResult<()> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get_mut(handle) {
			if new {
				rb.sleep();
			} else {
				rb.wake_up(true);
			}

			let entity = Entity::from_bits(rb_context.entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

			if let Ok(mut q) = world.query_one::<&mut RigidBody>(entity) && let Some(rb) = q.get() {
				rb.sleeping = new;
			}

			Ok(())
		} else {
			Err(DropbearNativeError::NoSuchHandle)
		}
	}

	pub fn get_rigidbody_ccd(physics: &PhysicsState, rb_context: RigidBodyContext) -> DropbearNativeResult<bool> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get(handle) {
			Ok(rb.is_ccd_enabled().into())
		} else {
			Err(DropbearNativeError::PhysicsObjectNotFound)
		}
	}

	pub fn set_rigidbody_ccd(physics: &mut PhysicsState, world: &World, rb_context: RigidBodyContext, new: bool) -> DropbearNativeResult<()> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get_mut(handle) {
			rb.enable_ccd(new);

			let entity = Entity::from_bits(rb_context.entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

			if let Ok(mut q) = world.query_one::<&mut RigidBody>(entity) && let Some(rb) = q.get() {
				rb.ccd_enabled = new;
			}

			Ok(())
		} else {
			Err(DropbearNativeError::NoSuchHandle)
		}
	}

	pub fn get_rigidbody_linvel(physics: &PhysicsState, rb_context: RigidBodyContext) -> DropbearNativeResult<Vector3> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get(handle) {
			let linvel = rb.linvel().clone();
			Ok(Vector3::new(linvel.x as f64, linvel.y as f64, linvel.z as f64))
		} else {
			Err(DropbearNativeError::PhysicsObjectNotFound)
		}
	}

	pub fn set_rigidbody_linvel(physics: &mut PhysicsState, world: &World, rb_context: RigidBodyContext, new: Vector3) -> DropbearNativeResult<()> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get_mut(handle) {
			rb.set_linvel(vector![new.x as f32, new.y as f32, new.z as f32], true);

			let entity = Entity::from_bits(rb_context.entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

			if let Ok(mut q) = world.query_one::<&mut RigidBody>(entity) && let Some(rb) = q.get() {
				rb.linvel = [new.x as f32, new.y as f32, new.z as f32];
			}

			Ok(())
		} else {
			Err(DropbearNativeError::NoSuchHandle)
		}
	}

	pub fn get_rigidbody_angvel(physics: &PhysicsState, rb_context: RigidBodyContext) -> DropbearNativeResult<Vector3> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get(handle) {
			let angvel = rb.angvel().clone();
			Ok(Vector3::new(angvel.x as f64, angvel.y as f64, angvel.z as f64))
		} else {
			Err(DropbearNativeError::PhysicsObjectNotFound)
		}
	}

	pub fn set_rigidbody_angvel(physics: &mut PhysicsState, world: &World, rb_context: RigidBodyContext, new: Vector3) -> DropbearNativeResult<()> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get_mut(handle) {
			rb.set_angvel(vector![new.x as f32, new.y as f32, new.z as f32], true);

			let entity = Entity::from_bits(rb_context.entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

			if let Ok(mut q) = world.query_one::<&mut RigidBody>(entity) && let Some(rb) = q.get() {
				rb.angvel = [new.x as f32, new.y as f32, new.z as f32];
			}

			Ok(())
		} else {
			Err(DropbearNativeError::NoSuchHandle)
		}
	}

	pub fn get_rigidbody_lock_translation(physics: &PhysicsState, rb_context: RigidBodyContext) -> DropbearNativeResult<AxisLock> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get(handle) {
			let lock = rb.locked_axes();
			let translation_lock = AxisLock {
				x: lock.contains(LockedAxes::TRANSLATION_LOCKED_X),
				y: lock.contains(LockedAxes::TRANSLATION_LOCKED_Y),
				z: lock.contains(LockedAxes::TRANSLATION_LOCKED_Z),
			};
			Ok(translation_lock)
		} else {
			Err(DropbearNativeError::PhysicsObjectNotFound)
		}
	}

	pub fn set_rigidbody_lock_translation(physics: &mut PhysicsState, world: &World, rb_context: RigidBodyContext, new: AxisLock) -> DropbearNativeResult<()> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get_mut(handle) {
			let mut bits = rb.locked_axes().clone();

			bits.remove(LockedAxes::TRANSLATION_LOCKED_X);
			bits.remove(LockedAxes::TRANSLATION_LOCKED_Y);
			bits.remove(LockedAxes::TRANSLATION_LOCKED_Z);
			
			if new.x {
				bits = bits | LockedAxes::TRANSLATION_LOCKED_X;
			}
			if new.y {
				bits = bits | LockedAxes::TRANSLATION_LOCKED_Y;
			}
			if new.z {
				bits = bits | LockedAxes::TRANSLATION_LOCKED_Z;
			}
			
			rb.set_locked_axes(bits, true);
			
			let entity = Entity::from_bits(rb_context.entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

			if let Ok(mut q) = world.query_one::<&mut RigidBody>(entity) && let Some(rb) = q.get() {
				rb.lock_translation = new;
			}

			Ok(())
		} else {
			Err(DropbearNativeError::NoSuchHandle)
		}
	}

	pub fn get_rigidbody_lock_rotation(physics: &PhysicsState, rb_context: RigidBodyContext) -> DropbearNativeResult<AxisLock> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get(handle) {
			let lock = rb.locked_axes();
			let rot_lock = AxisLock {
				x: lock.contains(LockedAxes::ROTATION_LOCKED_X),
				y: lock.contains(LockedAxes::ROTATION_LOCKED_Y),
				z: lock.contains(LockedAxes::ROTATION_LOCKED_Z),
			};
			Ok(rot_lock)
		} else {
			Err(DropbearNativeError::PhysicsObjectNotFound)
		}
	}

	pub fn set_rigidbody_lock_rotation(physics: &mut PhysicsState, world: &World, rb_context: RigidBodyContext, new: AxisLock) -> DropbearNativeResult<()> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get_mut(handle) {
			let mut bits = rb.locked_axes().clone();
			
			bits.remove(LockedAxes::ROTATION_LOCKED_X);
			bits.remove(LockedAxes::ROTATION_LOCKED_Y);
			bits.remove(LockedAxes::ROTATION_LOCKED_Z);

			if new.x {
				bits = bits | LockedAxes::ROTATION_LOCKED_X;
			}
			if new.y {
				bits = bits | LockedAxes::ROTATION_LOCKED_Y;
			}
			if new.z {
				bits = bits | LockedAxes::ROTATION_LOCKED_Z;
			}

			rb.set_locked_axes(bits, true);

			let entity = Entity::from_bits(rb_context.entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

			if let Ok(mut q) = world.query_one::<&mut RigidBody>(entity) && let Some(rb) = q.get() {
				rb.lock_translation = new;
			}

			Ok(())
		} else {
			Err(DropbearNativeError::NoSuchHandle)
		}
	}

	pub fn get_rigidbody_children(physics: &PhysicsState, rb_context: RigidBodyContext) -> DropbearNativeResult<Vec<ColliderHandle>> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get(handle) {
			let children = rb.colliders().to_vec();
			Ok(children)
		} else {
			Err(DropbearNativeError::PhysicsObjectNotFound)
		}
	}

	pub fn apply_impulse(physics: &mut PhysicsState, world: &World, rb_context: RigidBodyContext, new: Vector3) -> DropbearNativeResult<()> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get_mut(handle) {
			rb.apply_impulse(vector![new.x as f32, new.y as f32, new.z as f32], true);

			Ok(())
		} else {
			Err(DropbearNativeError::NoSuchHandle)
		}
	}

	pub fn apply_torque_impulse(physics: &mut PhysicsState, world: &World, rb_context: RigidBodyContext, new: Vector3) -> DropbearNativeResult<()> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get_mut(handle) {
			rb.apply_torque_impulse(vector![new.x as f32, new.y as f32, new.z as f32], true);

			Ok(())
		} else {
			Err(DropbearNativeError::NoSuchHandle)
		}
	}
}

pub mod jni {
	#![allow(non_snake_case)]
	use crate::physics::rigidbody::AxisLock;
	use crate::physics::PhysicsState;
	use crate::scripting::jni::utils::{FromJObject, ToJObject};
	use crate::types::{IndexNative, RigidBodyContext, Vector3};
	use crate::{convert_jlong_to_entity, convert_ptr};
	use hecs::World;
	use jni::objects::{JClass, JObject};
	use jni::sys::{jboolean, jdouble, jint, jlong, jobject, jobjectArray, jsize};
	use jni::JNIEnv;
	use rapier3d::dynamics::RigidBodyType;

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_RigidBodyNative_rigidBodyExistsForEntity(
		mut env: JNIEnv,
		_: JClass,
		world_ptr: jlong,
		physics_ptr: jlong,
		entity_id: jlong,
	) -> jobject {
		let world = convert_ptr!(world_ptr => World);
		let physics = convert_ptr!(physics_ptr => PhysicsState);
		let entity = convert_jlong_to_entity!(entity_id);

		match super::shared::rigid_body_exists_for_entity(world, physics, entity) {
			Some(v) => {
				match v.to_jobject(&mut env) {
					Ok(val) => val.into_raw(),
					Err(e) => {
						eprintln!("Failed to create new Index jobject: {}", e);
						let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to create new Index jobject: {}", e));
						std::ptr::null_mut()
					}
				}
			}
			None => std::ptr::null_mut()
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_RigidBodyNative_getRigidBodyMode(
		mut env: JNIEnv,
		_: JClass,
		_world_ptr: jlong,
		physics_ptr: jlong,
		rigidbody: JObject,
	) -> jint {
		let physics = convert_ptr!(physics_ptr => PhysicsState);
		let Ok(rb_context) = RigidBodyContext::from_jobject(&mut env, &rigidbody) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Unable to convert com/dropbear/physics/RigidBody into a Rust rigidbody");
			return -1 as jint;
		};

		match super::shared::get_rigidbody_type(physics, rb_context) {
			Ok(v) => {
				match v {
					RigidBodyType::Dynamic => 0 as jint,
					RigidBodyType::Fixed => 1 as jint,
					RigidBodyType::KinematicPositionBased => 2 as jint,
					RigidBodyType::KinematicVelocityBased => 3 as jint,
				}
			}
			Err(e) => {
				eprintln!("Failed to get RigidBody type: {}", e);
				let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to get RigidBody type: {}", e));
				-1 as jint
			}
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_RigidBodyNative_setRigidBodyMode(
		mut env: JNIEnv,
		_: JClass,
		world_ptr: jlong,
		physics_ptr: jlong,
		rigidbody: JObject,
		mode: jint,
	) {
		let world = convert_ptr!(world_ptr => World);
		let mut physics = convert_ptr!(mut physics_ptr => PhysicsState);
		let Ok(rb_context) = RigidBodyContext::from_jobject(&mut env, &rigidbody) else {
			let _ = env.throw_new("java/lang/RuntimeException",
								  "Unable to convert com/dropbear/physics/RigidBody to rust eucalyptus_core::physics::RigidBodyContext");
			return;
		};

		if let Err(e) = super::shared::set_rigidbody_type(&mut physics, world, rb_context, mode as i64) {
			eprintln!("Failed to set RigidBody Type: {}", e);
			let _ = env.throw_new("java/lang/IllegalArgumentException", format!("Failed to set RigidBody type: {}", e));
			return;
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_RigidBodyNative_getRigidBodyGravityScale(
		mut env: JNIEnv,
		_: JClass,
		_world_ptr: jlong,
		physics_ptr: jlong,
		rigidbody: JObject,
	) -> jdouble {
		let physics = convert_ptr!(physics_ptr => PhysicsState);
		let Ok(rb_context) = RigidBodyContext::from_jobject(&mut env, &rigidbody) else {
			let _ = env.throw_new("java/lang/RuntimeException",
								  "Unable to convert com/dropbear/physics/RigidBody to rust eucalyptus_core::physics::RigidBodyContext");
			return -1 as jdouble;
		};

		match super::shared::get_rigidbody_gravity_scale(physics, rb_context) {
			Ok(v) => v as jdouble,
			Err(e) => {
				eprintln!("Failed to get RigidBody component: {}", e);
				let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to get RigidBody component: {}", e));
				-1 as jdouble
			}
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_RigidBodyNative_setRigidBodyGravityScale(
		mut env: JNIEnv,
		_: JClass,
		world_ptr: jlong,
		physics_ptr: jlong,
		rigidbody: JObject,
		gravity_scale: jdouble,
	) {
		let world = convert_ptr!(world_ptr => World);
		let mut physics = convert_ptr!(mut physics_ptr => PhysicsState);
		let Ok(rb_context) = RigidBodyContext::from_jobject(&mut env, &rigidbody) else {
			let _ = env.throw_new("java/lang/RuntimeException",
								  "Unable to convert com/dropbear/physics/RigidBody to rust eucalyptus_core::physics::RigidBodyContext");
			return;
		};

		if let Err(e) = super::shared::set_rigidbody_gravity_scale(&mut physics, world, rb_context, gravity_scale as f64) {
			eprintln!("Failed to set RigidBody gravity scale: {}", e);
			let _ = env.throw_new("java/lang/IllegalArgumentException", format!("Failed to set RigidBody gravity scale: {}", e));
			return;
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_RigidBodyNative_getRigidBodyLinearDamping(
		mut env: JNIEnv,
		_: JClass,
		_world_ptr: jlong,
		physics_ptr: jlong,
		rigidbody: JObject,
	) -> jdouble {
		let physics = convert_ptr!(physics_ptr => PhysicsState);
		let Ok(rb_context) = RigidBodyContext::from_jobject(&mut env, &rigidbody) else {
			let _ = env.throw_new("java/lang/RuntimeException",
								  "Unable to convert com/dropbear/physics/RigidBody to rust eucalyptus_core::physics::RigidBodyContext");
			return -1 as jdouble;
		};

		match super::shared::get_rigidbody_linear_damping(physics, rb_context) {
			Ok(v) => v as jdouble,
			Err(e) => {
				eprintln!("Failed to get RigidBody linear damping: {}", e);
				let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to get RigidBody linear damping: {}", e));
				-1 as jdouble
			}
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_RigidBodyNative_setRigidBodyLinearDamping(
		mut env: JNIEnv,
		_: JClass,
		world_ptr: jlong,
		physics_ptr: jlong,
		rigidbody: JObject,
		linear_damping: jdouble,
	) {
		let world = convert_ptr!(world_ptr => World);
		let mut physics = convert_ptr!(mut physics_ptr => PhysicsState);
		let Ok(rb_context) = RigidBodyContext::from_jobject(&mut env, &rigidbody) else {
			let _ = env.throw_new("java/lang/RuntimeException",
								  "Unable to convert com/dropbear/physics/RigidBody to rust eucalyptus_core::physics::RigidBodyContext");
			return;
		};

		if let Err(e) = super::shared::set_rigidbody_linear_damping(&mut physics, world, rb_context, linear_damping as f64) {
			eprintln!("Failed to set RigidBody linear damping: {}", e);
			let _ = env.throw_new("java/lang/IllegalArgumentException", format!("Failed to set RigidBody linear damping: {}", e));
			return;
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_RigidBodyNative_getRigidBodyAngularDamping(
		mut env: JNIEnv,
		_: JClass,
		_world_ptr: jlong,
		physics_ptr: jlong,
		rigidbody: JObject,
	) -> jdouble {
		let physics = convert_ptr!(physics_ptr => PhysicsState);
		let Ok(rb_context) = RigidBodyContext::from_jobject(&mut env, &rigidbody) else {
			let _ = env.throw_new("java/lang/RuntimeException",
								  "Unable to convert com/dropbear/physics/RigidBody to rust eucalyptus_core::physics::RigidBodyContext");
			return -1 as jdouble;
		};

		match super::shared::get_rigidbody_angular_damping(physics, rb_context) {
			Ok(v) => v as jdouble,
			Err(e) => {
				eprintln!("Failed to get RigidBody angular damping: {}", e);
				let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to get RigidBody angular damping: {}", e));
				-1 as jdouble
			}
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_RigidBodyNative_setRigidBodyAngularDamping(
		mut env: JNIEnv,
		_: JClass,
		world_ptr: jlong,
		physics_ptr: jlong,
		rigidbody: JObject,
		angular_damping: jdouble,
	) {
		let world = convert_ptr!(world_ptr => World);
		let mut physics = convert_ptr!(mut physics_ptr => PhysicsState);
		let Ok(rb_context) = RigidBodyContext::from_jobject(&mut env, &rigidbody) else {
			let _ = env.throw_new("java/lang/RuntimeException",
								  "Unable to convert com/dropbear/physics/RigidBody to rust eucalyptus_core::physics::RigidBodyContext");
			return;
		};

		if let Err(e) = super::shared::set_rigidbody_angular_damping(&mut physics, world, rb_context, angular_damping as f64) {
			eprintln!("Failed to set RigidBody angular damping: {}", e);
			let _ = env.throw_new("java/lang/IllegalArgumentException", format!("Failed to set RigidBody angular damping: {}", e));
			return;
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_RigidBodyNative_getRigidBodySleep(
		mut env: JNIEnv,
		_: JClass,
		_world_ptr: jlong,
		physics_ptr: jlong,
		rigidbody: JObject,
	) -> jboolean {
		let physics = convert_ptr!(physics_ptr => PhysicsState);
		let Ok(rb_context) = RigidBodyContext::from_jobject(&mut env, &rigidbody) else {
			let _ = env.throw_new("java/lang/RuntimeException",
								  "Unable to convert com/dropbear/physics/RigidBody to rust eucalyptus_core::physics::RigidBodyContext");
			return 0 as jboolean;
		};

		match super::shared::get_rigidbody_sleep(physics, rb_context) {
			Ok(v) => v as jboolean,
			Err(e) => {
				eprintln!("Failed to get RigidBody sleep state: {}", e);
				let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to get RigidBody sleep state: {}", e));
				0 as jboolean
			}
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_RigidBodyNative_setRigidBodySleep(
		mut env: JNIEnv,
		_: JClass,
		world_ptr: jlong,
		physics_ptr: jlong,
		rigidbody: JObject,
		sleep: jboolean,
	) {
		let world = convert_ptr!(world_ptr => World);
		let mut physics = convert_ptr!(mut physics_ptr => PhysicsState);
		let Ok(rb_context) = RigidBodyContext::from_jobject(&mut env, &rigidbody) else {
			let _ = env.throw_new("java/lang/RuntimeException",
								  "Unable to convert com/dropbear/physics/RigidBody to rust eucalyptus_core::physics::RigidBodyContext");
			return;
		};

		if let Err(e) = super::shared::set_rigidbody_sleep(&mut physics, world, rb_context, sleep != 0) {
			eprintln!("Failed to set RigidBody sleep state: {}", e);
			let _ = env.throw_new("java/lang/IllegalArgumentException", format!("Failed to set RigidBody sleep state: {}", e));
			return;
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_RigidBodyNative_getRigidBodyCcdEnabled(
		mut env: JNIEnv,
		_: JClass,
		_world_ptr: jlong,
		physics_ptr: jlong,
		rigidbody: JObject,
	) -> jboolean {
		let physics = convert_ptr!(physics_ptr => PhysicsState);
		let Ok(rb_context) = RigidBodyContext::from_jobject(&mut env, &rigidbody) else {
			let _ = env.throw_new("java/lang/RuntimeException",
								  "Unable to convert com/dropbear/physics/RigidBody to rust eucalyptus_core::physics::RigidBodyContext");
			return 0 as jboolean;
		};

		match super::shared::get_rigidbody_ccd(physics, rb_context) {
			Ok(v) => v as jboolean,
			Err(e) => {
				eprintln!("Failed to get RigidBody CCD state: {}", e);
				let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to get RigidBody CCD state: {}", e));
				0 as jboolean
			}
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_RigidBodyNative_setRigidBodyCcdEnabled(
		mut env: JNIEnv,
		_: JClass,
		world_ptr: jlong,
		physics_ptr: jlong,
		rigidbody: JObject,
		ccd_enabled: jboolean,
	) {
		let world = convert_ptr!(world_ptr => World);
		let mut physics = convert_ptr!(mut physics_ptr => PhysicsState);
		let Ok(rb_context) = RigidBodyContext::from_jobject(&mut env, &rigidbody) else {
			let _ = env.throw_new("java/lang/RuntimeException",
								  "Unable to convert com/dropbear/physics/RigidBody to rust eucalyptus_core::physics::RigidBodyContext");
			return;
		};

		if let Err(e) = super::shared::set_rigidbody_ccd(&mut physics, world, rb_context, ccd_enabled != 0) {
			eprintln!("Failed to set RigidBody CCD state: {}", e);
			let _ = env.throw_new("java/lang/IllegalArgumentException", format!("Failed to set RigidBody CCD state: {}", e));
			return;
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_RigidBodyNative_getRigidBodyLinearVelocity(
		mut env: JNIEnv,
		_: JClass,
		_world_ptr: jlong,
		physics_ptr: jlong,
		rigidbody: JObject,
	) -> jobject {
		let physics = convert_ptr!(physics_ptr => PhysicsState);
		let Ok(rb_context) = RigidBodyContext::from_jobject(&mut env, &rigidbody) else {
			let _ = env.throw_new("java/lang/RuntimeException",
								  "Unable to convert com/dropbear/physics/RigidBody to rust eucalyptus_core::physics::RigidBodyContext");
			return std::ptr::null_mut();
		};

		match super::shared::get_rigidbody_linvel(physics, rb_context) {
			Ok(v) => {
				match v.to_jobject(&mut env) {
					Ok(val) => val.into_raw(),
					Err(e) => {
						eprintln!("Failed to create Vector3d jobject: {}", e);
						let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to create Vector3d jobject: {}", e));
						std::ptr::null_mut()
					}
				}
			}
			Err(e) => {
				eprintln!("Failed to get RigidBody linear velocity: {}", e);
				let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to get RigidBody linear velocity: {}", e));
				std::ptr::null_mut()
			}
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_RigidBodyNative_setRigidBodyLinearVelocity(
		mut env: JNIEnv,
		_: JClass,
		world_ptr: jlong,
		physics_ptr: jlong,
		rigidbody: JObject,
		linear_velocity: JObject,
	) {
		let world = convert_ptr!(world_ptr => World);
		let mut physics = convert_ptr!(mut physics_ptr => PhysicsState);
		let Ok(rb_context) = RigidBodyContext::from_jobject(&mut env, &rigidbody) else {
			let _ = env.throw_new("java/lang/RuntimeException",
								  "Unable to convert com/dropbear/physics/RigidBody to rust eucalyptus_core::physics::RigidBodyContext");
			return;
		};

		let Ok(velocity) = Vector3::from_jobject(&mut env, &linear_velocity) else {
			let _ = env.throw_new("java/lang/RuntimeException",
								  "Unable to convert com/dropbear/math/Vector3d to rust Vector3");
			return;
		};

		if let Err(e) = super::shared::set_rigidbody_linvel(&mut physics, world, rb_context, velocity) {
			eprintln!("Failed to set RigidBody linear velocity: {}", e);
			let _ = env.throw_new("java/lang/IllegalArgumentException", format!("Failed to set RigidBody linear velocity: {}", e));
			return;
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_RigidBodyNative_getRigidBodyAngularVelocity(
		mut env: JNIEnv,
		_: JClass,
		_world_ptr: jlong,
		physics_ptr: jlong,
		rigidbody: JObject,
	) -> jobject {
		let physics = convert_ptr!(physics_ptr => PhysicsState);
		let Ok(rb_context) = RigidBodyContext::from_jobject(&mut env, &rigidbody) else {
			let _ = env.throw_new("java/lang/RuntimeException",
								  "Unable to convert com/dropbear/physics/RigidBody to rust eucalyptus_core::physics::RigidBodyContext");
			return std::ptr::null_mut();
		};

		match super::shared::get_rigidbody_angvel(physics, rb_context) {
			Ok(v) => {
				match v.to_jobject(&mut env) {
					Ok(val) => val.into_raw(),
					Err(e) => {
						eprintln!("Failed to create Vector3d jobject: {}", e);
						let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to create Vector3d jobject: {}", e));
						std::ptr::null_mut()
					}
				}
			}
			Err(e) => {
				eprintln!("Failed to get RigidBody angular velocity: {}", e);
				let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to get RigidBody angular velocity: {}", e));
				std::ptr::null_mut()
			}
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_RigidBodyNative_setRigidBodyAngularVelocity(
		mut env: JNIEnv,
		_: JClass,
		world_ptr: jlong,
		physics_ptr: jlong,
		rigidbody: JObject,
		angular_velocity: JObject,
	) {
		let world = convert_ptr!(world_ptr => World);
		let mut physics = convert_ptr!(mut physics_ptr => PhysicsState);
		let Ok(rb_context) = RigidBodyContext::from_jobject(&mut env, &rigidbody) else {
			let _ = env.throw_new("java/lang/RuntimeException",
								  "Unable to convert com/dropbear/physics/RigidBody to rust eucalyptus_core::physics::RigidBodyContext");
			return;
		};

		let Ok(velocity) = Vector3::from_jobject(&mut env, &angular_velocity) else {
			let _ = env.throw_new("java/lang/RuntimeException",
								  "Unable to convert com/dropbear/math/Vector3d to rust Vector3");
			return;
		};

		if let Err(e) = super::shared::set_rigidbody_angvel(&mut physics, world, rb_context, velocity) {
			eprintln!("Failed to set RigidBody angular velocity: {}", e);
			let _ = env.throw_new("java/lang/IllegalArgumentException", format!("Failed to set RigidBody angular velocity: {}", e));
			return;
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_RigidBodyNative_getRigidBodyLockTranslation(
		mut env: JNIEnv,
		_: JClass,
		_world_ptr: jlong,
		physics_ptr: jlong,
		rigidbody: JObject,
	) -> jobject {
		let physics = convert_ptr!(physics_ptr => PhysicsState);
		let Ok(rb_context) = RigidBodyContext::from_jobject(&mut env, &rigidbody) else {
			let _ = env.throw_new("java/lang/RuntimeException",
								  "Unable to convert com/dropbear/physics/RigidBody to rust eucalyptus_core::physics::RigidBodyContext");
			return std::ptr::null_mut();
		};

		match super::shared::get_rigidbody_lock_translation(physics, rb_context) {
			Ok(v) => {
				match v.to_jobject(&mut env) {
					Ok(val) => val.into_raw(),
					Err(e) => {
						eprintln!("Failed to create AxisLock jobject: {}", e);
						let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to create AxisLock jobject: {}", e));
						std::ptr::null_mut()
					}
				}
			}
			Err(e) => {
				eprintln!("Failed to get RigidBody translation lock: {}", e);
				let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to get RigidBody translation lock: {}", e));
				std::ptr::null_mut()
			}
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_RigidBodyNative_setRigidBodyLockTranslation(
		mut env: JNIEnv,
		_: JClass,
		world_ptr: jlong,
		physics_ptr: jlong,
		rigidbody: JObject,
		lock: JObject,
	) {
		let world = convert_ptr!(world_ptr => World);
		let mut physics = convert_ptr!(mut physics_ptr => PhysicsState);
		let Ok(rb_context) = RigidBodyContext::from_jobject(&mut env, &rigidbody) else {
			let _ = env.throw_new("java/lang/RuntimeException",
								  "Unable to convert com/dropbear/physics/RigidBody to rust eucalyptus_core::physics::RigidBodyContext");
			return;
		};

		let Ok(axis_lock) = AxisLock::from_jobject(&mut env, &lock) else {
			let _ = env.throw_new("java/lang/RuntimeException",
								  "Unable to convert com/dropbear/physics/AxisLock to rust AxisLock");
			return;
		};

		if let Err(e) = super::shared::set_rigidbody_lock_translation(&mut physics, world, rb_context, axis_lock) {
			eprintln!("Failed to set RigidBody translation lock: {}", e);
			let _ = env.throw_new("java/lang/IllegalArgumentException", format!("Failed to set RigidBody translation lock: {}", e));
			return;
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_RigidBodyNative_getRigidBodyLockRotation(
		mut env: JNIEnv,
		_: JClass,
		_world_ptr: jlong,
		physics_ptr: jlong,
		rigidbody: JObject,
	) -> jobject {
		let physics = convert_ptr!(physics_ptr => PhysicsState);
		let Ok(rb_context) = RigidBodyContext::from_jobject(&mut env, &rigidbody) else {
			let _ = env.throw_new("java/lang/RuntimeException",
								  "Unable to convert com/dropbear/physics/RigidBody to rust eucalyptus_core::physics::RigidBodyContext");
			return std::ptr::null_mut();
		};

		match super::shared::get_rigidbody_lock_rotation(physics, rb_context) {
			Ok(v) => {
				match v.to_jobject(&mut env) {
					Ok(val) => val.into_raw(),
					Err(e) => {
						eprintln!("Failed to create AxisLock jobject: {}", e);
						let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to create AxisLock jobject: {}", e));
						std::ptr::null_mut()
					}
				}
			}
			Err(e) => {
				eprintln!("Failed to get RigidBody rotation lock: {}", e);
				let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to get RigidBody rotation lock: {}", e));
				std::ptr::null_mut()
			}
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_RigidBodyNative_setRigidBodyLockRotation(
		mut env: JNIEnv,
		_: JClass,
		world_ptr: jlong,
		physics_ptr: jlong,
		rigidbody: JObject,
		lock: JObject,
	) {
		let world = convert_ptr!(world_ptr => World);
		let mut physics = convert_ptr!(mut physics_ptr => PhysicsState);
		let Ok(rb_context) = RigidBodyContext::from_jobject(&mut env, &rigidbody) else {
			let _ = env.throw_new("java/lang/RuntimeException",
								  "Unable to convert com/dropbear/physics/RigidBody to rust eucalyptus_core::physics::RigidBodyContext");
			return;
		};

		let Ok(axis_lock) = AxisLock::from_jobject(&mut env, &lock) else {
			let _ = env.throw_new("java/lang/RuntimeException",
								  "Unable to convert com/dropbear/physics/AxisLock to rust AxisLock");
			return;
		};

		if let Err(e) = super::shared::set_rigidbody_lock_rotation(&mut physics, world, rb_context, axis_lock) {
			eprintln!("Failed to set RigidBody rotation lock: {}", e);
			let _ = env.throw_new("java/lang/IllegalArgumentException", format!("Failed to set RigidBody rotation lock: {}", e));
			return;
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_RigidBodyNative_getRigidBodyChildren(
		mut env: JNIEnv,
		_: JClass,
		_world_ptr: jlong,
		physics_ptr: jlong,
		rigidbody: JObject,
	) -> jobjectArray {
		let physics = convert_ptr!(physics_ptr => PhysicsState);
		let Ok(rb_context) = RigidBodyContext::from_jobject(&mut env, &rigidbody) else {
			let _ = env.throw_new("java/lang/RuntimeException",
								  "Unable to convert com/dropbear/physics/RigidBody to rust eucalyptus_core::physics::RigidBodyContext");
			return std::ptr::null_mut();
		};

		match super::shared::get_rigidbody_children(physics, rb_context) {
			Ok(children) => {
				let mut handles: Vec<JObject> = Vec::new();
				for c in children {
					match IndexNative::from(c.0).to_jobject(&mut env) {
						Ok(val) => { handles.push(val); }
						Err(_) => { continue; }
					}
				}

				let array = match env.new_object_array(handles.len() as i32, "com/dropbear/physics/Collider", JObject::null()) {
					Ok(array) => array,
					Err(e) => {
						eprintln!("Failed to create jlong array: {}", e);
						let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to create jlong array: {}", e));
						return std::ptr::null_mut();
					}
				};

				for (i, h) in handles.iter().enumerate() {
					if let Err(e) = env.set_object_array_element(&array, i as jsize, h) {
						eprintln!("Failed to set jlong array region: {}", e);
						let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to set jobject array region at index {}: {}", i, e));
						return std::ptr::null_mut();
					}
				}

				array.into_raw()
			}
			Err(e) => {
				eprintln!("Failed to get RigidBody children: {}", e);
				let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to get RigidBody children: {}", e));
				std::ptr::null_mut()
			}
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_RigidBodyNative_applyImpulse(
		mut env: JNIEnv,
		_: JClass,
		world_ptr: jlong,
		physics_ptr: jlong,
		rigidbody: JObject,
		x: jdouble,
		y: jdouble,
		z: jdouble,
	) {
		let world = convert_ptr!(world_ptr => World);
		let mut physics = convert_ptr!(mut physics_ptr => PhysicsState);
		let Ok(rb_context) = RigidBodyContext::from_jobject(&mut env, &rigidbody) else {
			let _ = env.throw_new("java/lang/RuntimeException",
								  "Unable to convert com/dropbear/physics/RigidBody to rust eucalyptus_core::physics::RigidBodyContext");
			return;
		};

		let impulse = Vector3::new(x as f64, y as f64, z as f64);
		if let Err(e) = super::shared::apply_impulse(&mut physics, world, rb_context, impulse) {
			eprintln!("Failed to apply impulse: {}", e);
			let _ = env.throw_new("java/lang/IllegalArgumentException", format!("Failed to apply impulse: {}", e));
			return;
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_RigidBodyNative_applyTorqueImpulse(
		mut env: JNIEnv,
		_: JClass,
		world_ptr: jlong,
		physics_ptr: jlong,
		rigidbody: JObject,
		x: jdouble,
		y: jdouble,
		z: jdouble,
	) {
		let world = convert_ptr!(world_ptr => World);
		let mut physics = convert_ptr!(mut physics_ptr => PhysicsState);
		let Ok(rb_context) = RigidBodyContext::from_jobject(&mut env, &rigidbody) else {
			let _ = env.throw_new("java/lang/RuntimeException",
								  "Unable to convert com/dropbear/physics/RigidBody to rust eucalyptus_core::physics::RigidBodyContext");
			return;
		};

		let torque = Vector3::new(x as f64, y as f64, z as f64);
		if let Err(e) = super::shared::apply_torque_impulse(&mut physics, world, rb_context, torque) {
			eprintln!("Failed to apply torque impulse: {}", e);
			let _ = env.throw_new("java/lang/IllegalArgumentException", format!("Failed to apply torque impulse: {}", e));
			return;
		}
	}
}

#[dropbear_macro::impl_c_api]
pub mod native {
	
}