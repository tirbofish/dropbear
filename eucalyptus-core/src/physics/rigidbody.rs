//! [rapier3d] RigidBodies

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

pub mod shared {
	
}

pub mod jni {
	
}

#[dropbear_macro::impl_c_api]
pub mod native {
	
}