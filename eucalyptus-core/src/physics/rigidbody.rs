use serde::{Deserialize, Serialize};
use dropbear_macro::SerializableComponent;
use dropbear_traits::SerializableComponent;
use crate::states::Label;

/// How this entity behaves in the physics simulation.
///
/// This intentionally mirrors Rapier's rigid-body types, but stays engine-owned and serializable.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum RigidBodyMode {
	/// A fully simulated body affected by forces and contacts.
	Dynamic,
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
pub struct AxisLock {
	pub x: bool,
	pub y: bool,
	pub z: bool,
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

	/// Whether this body is allowed to sleep.
	#[serde(default = "RigidBody::default_can_sleep")]
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
			can_sleep: Self::default_can_sleep(),
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

	const fn default_can_sleep() -> bool {
		true
	}
}

pub mod jni {
	use glam::DVec3;
	use jni::objects::{JObject, JString};
	use jni::JNIEnv;
	use rapier3d::prelude::RigidBodyHandle;
	use crate::physics::rigidbody::{AxisLock, RigidBody, RigidBodyMode};
	use crate::scripting::jni::utils::FromJObject;
	use crate::states::Label;

	impl FromJObject for AxisLock {
		fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> anyhow::Result<Self>
		where
			Self: Sized
		{
			let x = env.get_field(obj, "x", "Z")?.z()?;
			let y = env.get_field(obj, "y", "Z")?.z()?;
			let z = env.get_field(obj, "z", "Z")?.z()?;
			Ok(AxisLock { x, y, z })
		}
	}

	impl RigidBody {
		pub fn from_jni(env: &mut JNIEnv, obj: &JObject) -> anyhow::Result<(RigidBodyHandle, Self)> {
			let entity_jobj = env.get_field(obj, "entity", "Ljava/lang/String;")?.l()?;
			let entity_jstr: JString = entity_jobj.into();
			let entity_str: String = env.get_string(&entity_jstr)?.into();

			let entity = Label::from(entity_str);

			let gravity_scale: f32 = env.get_field(obj, "gravityScale", "D")?.d()? as f32;
			let can_sleep: bool = env.get_field(obj, "canSleep", "Z")?.z()?;
			let ccd_enabled: bool = env.get_field(obj, "ccdEnabled", "Z")?.z()?;
			let linear_damping: f32 = env.get_field(obj, "linearDamping", "D")?.d()? as f32;
			let angular_damping: f32 = env.get_field(obj, "angularDamping", "D")?.d()? as f32;

			let mode_obj = env.get_field(obj, "rigidBodyMode", "Lcom/dropbear/physics/RigidBodyMode;")?.l()?;
			let mode_ordinal = env.call_method(&mode_obj, "ordinal", "()I", &[])?.i()?;

			let mode = match mode_ordinal {
				0 => RigidBodyMode::Dynamic,
				1 => RigidBodyMode::Fixed,
				2 => RigidBodyMode::KinematicPosition,
				3 => RigidBodyMode::KinematicVelocity,
				_ => RigidBodyMode::Dynamic,
			};

			let index_obj = env.get_field(obj, "index", "Lcom/dropbear/physics/Index;")?.l()?;
			let idx_val = env.get_field(&index_obj, "index", "I")?.i()? as u32;
			let gen_val = env.get_field(&index_obj, "generation", "I")?.i()? as u32;

			let linvel_obj = env.get_field(obj, "linearVelocity", "Lcom/dropbear/math/Vector3D;")?.l()?;
			let linvel = DVec3::from_jobject(env, &linvel_obj)?.as_vec3().to_array();

			let angvel_obj = env.get_field(obj, "angularVelocity", "Lcom/dropbear/math/Vector3D;")?.l()?;
			let angvel = DVec3::from_jobject(env, &angvel_obj)?.as_vec3().to_array();

			let lock_trans_obj = env.get_field(obj, "lockTranslation", "Lcom/dropbear/physics/AxisLock;")?.l()?;
			let lock_translation = AxisLock::from_jobject(env, &lock_trans_obj)?;

			let lock_rot_obj = env.get_field(obj, "lockRotation", "Lcom/dropbear/physics/AxisLock;")?.l()?;
			let lock_rotation = AxisLock::from_jobject(env, &lock_rot_obj)?;

			let handle = RigidBodyHandle::from_raw_parts(idx_val, gen_val);

			Ok((handle, Self {
				entity,
				disable_physics: false,
				mode,
				gravity_scale,
				can_sleep,
				ccd_enabled,
				linvel,
				angvel,
				linear_damping,
				angular_damping,
				lock_translation,
				lock_rotation,
			}))
		}
	}
}