//! [rapier3d] RigidBodies

use crate::scripting::jni::utils::{FromJObject, ToJObject};
use crate::scripting::native::DropbearNativeError;
use crate::scripting::result::DropbearNativeResult;
use crate::states::Label;
use std::any::Any;
use std::sync::Arc;
use ::jni::objects::{JObject, JValue};
use ::jni::JNIEnv;
use egui::{CollapsingHeader, ComboBox, DragValue, Ui};
use hecs::{Entity, World};
use rapier3d::prelude::RigidBodyType;
use serde::{Deserialize, Serialize};
use dropbear_engine::graphics::SharedGraphicsContext;
use crate::component::{Component, ComponentDescriptor, InspectableComponent, SerializedComponent};
use crate::types::{IndexNative, NCollider, RigidBodyContext, NVector3};
use crate::physics::PhysicsState;
use crate::ptr::{PhysicsStatePtr, WorldPtr};

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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

	#[serde(default)]
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

#[typetag::serde]
impl SerializedComponent for RigidBody {}

impl Component for RigidBody {
	type SerializedForm = Self;
	type RequiredComponentTypes = (Self, );

	fn descriptor() -> ComponentDescriptor {
		ComponentDescriptor {
			fqtn: "eucalyptus_core::physics::rigidbody::RigidBody".to_string(),
			type_name: "RigidBody".to_string(),
			category: Some("Physics".to_string()),
			description: Some("An object that can move, rotate or collide with another object".to_string()),
		}
	}

	fn init<'a>(
		ser: &'a Self::SerializedForm,
		_graphics: Arc<SharedGraphicsContext>,
	) -> crate::component::ComponentInitFuture<'a, Self> {
		Box::pin(async move { Ok((ser.clone(), )) })
	}

	fn update_component(&mut self, _world: &World, _physics: &mut PhysicsState, _entity: Entity, _dt: f32, _graphics: Arc<SharedGraphicsContext>) {}

	fn save(&self, _world: &World, _entity: Entity) -> Box<dyn SerializedComponent> {
		Box::new(self.clone())
	}
}

impl InspectableComponent for RigidBody {
	fn inspect(&mut self, ui: &mut Ui, _graphics: Arc<SharedGraphicsContext>) {
		CollapsingHeader::new("RigidBody").default_open(true).show(ui, |ui| {
			ui.vertical(|ui| {
				let mut selected = self.mode;
				ComboBox::from_id_salt("rb")
					.selected_text(format!("{:?}", self.mode))
					.show_ui(ui, |ui| {
						ui.selectable_value(&mut selected, RigidBodyMode::Dynamic, "Dynamic");
						ui.selectable_value(&mut selected, RigidBodyMode::Fixed, "Fixed");
						ui.selectable_value(&mut selected, RigidBodyMode::KinematicPosition, "Kinematic Position");
						ui.selectable_value(&mut selected, RigidBodyMode::KinematicVelocity, "Kinematic Velocity");
					});

				if selected != self.mode {
					self.mode = selected;
				}

				ui.add_space(8.0);

				ui.horizontal(|ui| {
					ui.label("Gravity Scale:");
					ui.add(DragValue::new(&mut self.gravity_scale)
						.speed(0.1)
						.range(0.0..=10.0));
				});

				ui.checkbox(&mut self.can_sleep, "Can Sleep");
				ui.checkbox(&mut self.sleeping, "Initially sleeping?");
				ui.checkbox(&mut self.ccd_enabled, "CCD Enabled");

				ui.add_space(8.0);

				ui.label("Linear Velocity:");
				ui.horizontal(|ui| {
					ui.label("X:");
					ui.add(DragValue::new(&mut self.linvel[0]).speed(0.1));
					ui.label("Y:");
					ui.add(DragValue::new(&mut self.linvel[1]).speed(0.1));
					ui.label("Z:");
					ui.add(DragValue::new(&mut self.linvel[2]).speed(0.1));
				});

				ui.label("Angular Velocity:");
				ui.horizontal(|ui| {
					ui.label("X:");
					ui.add(DragValue::new(&mut self.angvel[0]).speed(0.1));
					ui.label("Y:");
					ui.add(DragValue::new(&mut self.angvel[1]).speed(0.1));
					ui.label("Z:");
					ui.add(DragValue::new(&mut self.angvel[2]).speed(0.1));
				});

				ui.add_space(8.0);

				ui.horizontal(|ui| {
					ui.label("Linear Damping:");
					ui.add(DragValue::new(&mut self.linear_damping)
						.speed(0.01)
						.range(0.0..=10.0));
				});

				ui.horizontal(|ui| {
					ui.label("Angular Damping:");
					ui.add(DragValue::new(&mut self.angular_damping)
						.speed(0.01)
						.range(0.0..=10.0));
				});

				ui.add_space(8.0);

				ui.label("Lock Translation:");
				ui.horizontal(|ui| {
					ui.checkbox(&mut self.lock_translation.x, "X");
					ui.checkbox(&mut self.lock_translation.y, "Y");
					ui.checkbox(&mut self.lock_translation.z, "Z");
				});

				ui.label("Lock Rotation:");
				ui.horizontal(|ui| {
					ui.checkbox(&mut self.lock_rotation.x, "X");
					ui.checkbox(&mut self.lock_rotation.y, "Y");
					ui.checkbox(&mut self.lock_rotation.z, "Z");
				});
			});
		});
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
	use crate::types::{IndexNative, RigidBodyContext, NVector3};
	use hecs::{Entity, World};
	use rapier3d::dynamics::{RigidBodyHandle, RigidBodyType};
	use rapier3d::prelude::Vector;
	use rapier3d::prelude::{ColliderHandle, LockedAxes};

	pub fn rigid_body_exists_for_entity(
		world: &World,
		physics: &PhysicsState,
		entity: Entity
	) -> Option<IndexNative> {
		if let Ok((label, _)) = world.query_one::<(&Label, &RigidBody)>(entity).get()
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

	pub fn get_rigidbody_type(physics: &PhysicsState, rb_context: &RigidBodyContext) -> DropbearNativeResult<RigidBodyType> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get(handle) {
			Ok(rb.body_type())
		} else {
			Err(DropbearNativeError::PhysicsObjectNotFound)
		}
	}

	pub fn set_rigidbody_type(physics: &mut PhysicsState, world: &World, rb_context: &RigidBodyContext, mode: i64) -> DropbearNativeResult<()> {
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

			if let Ok(rb) = world.query_one::<&mut RigidBody>(entity).get() {
				let rb_mode = RigidBodyMode::from(mode);
				rb.mode = rb_mode;
			}

			Ok(())
		} else {
			Err(DropbearNativeError::NoSuchHandle)
		}
	}

	pub fn get_rigidbody_gravity_scale(physics: &PhysicsState, rb_context: &RigidBodyContext) -> DropbearNativeResult<f64> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get(handle) {
			Ok(rb.gravity_scale().into())
		} else {
			Err(DropbearNativeError::PhysicsObjectNotFound)
		}
	}
	
	pub fn set_rigidbody_gravity_scale(physics: &mut PhysicsState, world: &World, rb_context: &RigidBodyContext, new_scale: f64) -> DropbearNativeResult<()> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get_mut(handle) {
			rb.set_gravity_scale(new_scale as f32, true);
			
			let entity = Entity::from_bits(rb_context.entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

			if let Ok(rb) = world.query_one::<&mut RigidBody>(entity).get() {
				rb.gravity_scale = new_scale as f32;				
			}

			Ok(())
		} else {
			Err(DropbearNativeError::NoSuchHandle)
		}
	}

	pub fn get_rigidbody_linear_damping(physics: &PhysicsState, rb_context: &RigidBodyContext) -> DropbearNativeResult<f64> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get(handle) {
			Ok(rb.linear_damping().into())
		} else {
			Err(DropbearNativeError::PhysicsObjectNotFound)
		}
	}

	pub fn set_rigidbody_linear_damping(physics: &mut PhysicsState, world: &World, rb_context: &RigidBodyContext, new: f64) -> DropbearNativeResult<()> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get_mut(handle) {
			rb.set_linear_damping(new as f32);

			let entity = Entity::from_bits(rb_context.entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

			if let Ok(rb) = world.query_one::<&mut RigidBody>(entity).get() {
				rb.linear_damping = new as f32;
			}

			Ok(())
		} else {
			Err(DropbearNativeError::NoSuchHandle)
		}
	}

	pub fn get_rigidbody_angular_damping(physics: &PhysicsState, rb_context: &RigidBodyContext) -> DropbearNativeResult<f64> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get(handle) {
			Ok(rb.angular_damping().into())
		} else {
			Err(DropbearNativeError::PhysicsObjectNotFound)
		}
	}

	pub fn set_rigidbody_angular_damping(physics: &mut PhysicsState, world: &World, rb_context: &RigidBodyContext, new: f64) -> DropbearNativeResult<()> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get_mut(handle) {
			rb.set_angular_damping(new as f32);

			let entity = Entity::from_bits(rb_context.entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

			if let Ok(rb) = world.query_one::<&mut RigidBody>(entity).get() {
				rb.angular_damping = new as f32;
			}

			Ok(())
		} else {
			Err(DropbearNativeError::NoSuchHandle)
		}
	}

	pub fn get_rigidbody_sleep(physics: &PhysicsState, rb_context: &RigidBodyContext) -> DropbearNativeResult<bool> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get(handle) {
			Ok(rb.is_sleeping().into())
		} else {
			Err(DropbearNativeError::PhysicsObjectNotFound)
		}
	}

	pub fn set_rigidbody_sleep(physics: &mut PhysicsState, world: &World, rb_context: &RigidBodyContext, new: bool) -> DropbearNativeResult<()> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get_mut(handle) {
			if new {
				rb.sleep();
			} else {
				rb.wake_up(true);
			}

			let entity = Entity::from_bits(rb_context.entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

			if let Ok(rb) = world.query_one::<&mut RigidBody>(entity).get() {
				rb.sleeping = new;
			}

			Ok(())
		} else {
			Err(DropbearNativeError::NoSuchHandle)
		}
	}

	pub fn get_rigidbody_ccd(physics: &PhysicsState, rb_context: &RigidBodyContext) -> DropbearNativeResult<bool> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get(handle) {
			Ok(rb.is_ccd_enabled().into())
		} else {
			Err(DropbearNativeError::PhysicsObjectNotFound)
		}
	}

	pub fn set_rigidbody_ccd(physics: &mut PhysicsState, world: &World, rb_context: &RigidBodyContext, new: bool) -> DropbearNativeResult<()> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get_mut(handle) {
			rb.enable_ccd(new);

			let entity = Entity::from_bits(rb_context.entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

			if let Ok(rb) = world.query_one::<&mut RigidBody>(entity).get() {
				rb.ccd_enabled = new;
			}

			Ok(())
		} else {
			Err(DropbearNativeError::NoSuchHandle)
		}
	}

	pub fn get_rigidbody_linvel(physics: &PhysicsState, rb_context: &RigidBodyContext) -> DropbearNativeResult<NVector3> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get(handle) {
			let linvel = rb.linvel().clone();
			Ok(NVector3::new(linvel.x as f64, linvel.y as f64, linvel.z as f64))
		} else {
			Err(DropbearNativeError::PhysicsObjectNotFound)
		}
	}

	pub fn set_rigidbody_linvel(physics: &mut PhysicsState, world: &World, rb_context: &RigidBodyContext, new: NVector3) -> DropbearNativeResult<()> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get_mut(handle) {
			rb.set_linvel(Vector::new(new.x as f32, new.y as f32, new.z as f32), true);

			let entity = Entity::from_bits(rb_context.entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

			if let Ok(rb) = world.query_one::<&mut RigidBody>(entity).get() {
				rb.linvel = [new.x as f32, new.y as f32, new.z as f32];
			}

			Ok(())
		} else {
			Err(DropbearNativeError::NoSuchHandle)
		}
	}

	pub fn get_rigidbody_angvel(physics: &PhysicsState, rb_context: &RigidBodyContext) -> DropbearNativeResult<NVector3> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get(handle) {
			let angvel = rb.angvel().clone();
			Ok(NVector3::new(angvel.x as f64, angvel.y as f64, angvel.z as f64))
		} else {
			Err(DropbearNativeError::PhysicsObjectNotFound)
		}
	}

	pub fn set_rigidbody_angvel(physics: &mut PhysicsState, world: &World, rb_context: &RigidBodyContext, new: NVector3) -> DropbearNativeResult<()> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get_mut(handle) {
			rb.set_angvel(Vector::new(new.x as f32, new.y as f32, new.z as f32), true);

			let entity = Entity::from_bits(rb_context.entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

			if let Ok(rb) = world.query_one::<&mut RigidBody>(entity).get() {
				rb.angvel = [new.x as f32, new.y as f32, new.z as f32];
			}

			Ok(())
		} else {
			Err(DropbearNativeError::NoSuchHandle)
		}
	}

	pub fn get_rigidbody_lock_translation(physics: &PhysicsState, rb_context: &RigidBodyContext) -> DropbearNativeResult<AxisLock> {
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

	pub fn set_rigidbody_lock_translation(physics: &mut PhysicsState, world: &World, rb_context: &RigidBodyContext, new: AxisLock) -> DropbearNativeResult<()> {
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

			if let Ok(rb) = world.query_one::<&mut RigidBody>(entity).get() {
				rb.lock_translation = new;
			}

			Ok(())
		} else {
			Err(DropbearNativeError::NoSuchHandle)
		}
	}

	pub fn get_rigidbody_lock_rotation(physics: &PhysicsState, rb_context: &RigidBodyContext) -> DropbearNativeResult<AxisLock> {
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

	pub fn set_rigidbody_lock_rotation(physics: &mut PhysicsState, world: &World, rb_context: &RigidBodyContext, new: AxisLock) -> DropbearNativeResult<()> {
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

			if let Ok(rb) = world.query_one::<&mut RigidBody>(entity).get() {
				rb.lock_translation = new;
			}

			Ok(())
		} else {
			Err(DropbearNativeError::NoSuchHandle)
		}
	}

	pub fn get_rigidbody_children(physics: &PhysicsState, rb_context: &RigidBodyContext) -> DropbearNativeResult<Vec<ColliderHandle>> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get(handle) {
			let children = rb.colliders().to_vec();
			Ok(children)
		} else {
			Err(DropbearNativeError::PhysicsObjectNotFound)
		}
	}

	pub fn apply_impulse(physics: &mut PhysicsState, _world: &World, rb_context: &RigidBodyContext, new: NVector3) -> DropbearNativeResult<()> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get_mut(handle) {
			rb.apply_impulse(Vector::new(new.x as f32, new.y as f32, new.z as f32), true);

			Ok(())
		} else {
			Err(DropbearNativeError::NoSuchHandle)
		}
	}

	pub fn apply_torque_impulse(physics: &mut PhysicsState, _world: &World, rb_context: &RigidBodyContext, new: NVector3) -> DropbearNativeResult<()> {
		let handle = RigidBodyHandle::from_raw_parts(rb_context.index.index, rb_context.index.generation);
		if let Some(rb) = physics.bodies.get_mut(handle) {
			rb.apply_torque_impulse(Vector::new(new.x as f32, new.y as f32, new.z as f32), true);

			Ok(())
		} else {
			Err(DropbearNativeError::NoSuchHandle)
		}
	}
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.RigidBodyNative", func = "rigidBodyExistsForEntity"),
	c
)]
fn exists_for_entity(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &PhysicsState,
    #[dropbear_macro::entity]
    entity: hecs::Entity,
) -> DropbearNativeResult<Option<IndexNative>> {
    Ok(shared::rigid_body_exists_for_entity(world, physics, entity))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.RigidBodyNative", func = "getRigidBodyMode"),
	c
)]
fn get_rigidbody_mode(
	#[dropbear_macro::define(WorldPtr)]
	_world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &PhysicsState,
    rigidbody: &RigidBodyContext,
) -> DropbearNativeResult<i32> {
    let body_type = shared::get_rigidbody_type(physics, rigidbody)?;
    Ok(match body_type {
        RigidBodyType::Dynamic => 0,
        RigidBodyType::Fixed => 1,
        RigidBodyType::KinematicPositionBased => 2,
        RigidBodyType::KinematicVelocityBased => 3,
    })
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.RigidBodyNative", func = "setRigidBodyMode"),
	c
)]
fn set_rigidbody_mode(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &mut PhysicsState,
    rigidbody: &RigidBodyContext,
    mode: i32,
) -> DropbearNativeResult<()> {
    shared::set_rigidbody_type(physics, world, rigidbody, mode as i64)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.RigidBodyNative", func = "getRigidBodyGravityScale"),
	c
)]
fn get_rigidbody_gravity_scale(
	#[dropbear_macro::define(WorldPtr)]
	_world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &PhysicsState,
    rigidbody: &RigidBodyContext,
) -> DropbearNativeResult<f64> {
    shared::get_rigidbody_gravity_scale(physics, rigidbody)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.RigidBodyNative", func = "setRigidBodyGravityScale"),
	c
)]
fn set_rigidbody_gravity_scale(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &mut PhysicsState,
    rigidbody: &RigidBodyContext,
    gravity_scale: f64,
) -> DropbearNativeResult<()> {
    shared::set_rigidbody_gravity_scale(physics, world, rigidbody, gravity_scale)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.RigidBodyNative", func = "getRigidBodyLinearDamping"),
	c
)]
fn get_rigidbody_linear_damping(
	#[dropbear_macro::define(WorldPtr)]
	_world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &PhysicsState,
    rigidbody: &RigidBodyContext,
) -> DropbearNativeResult<f64> {
    shared::get_rigidbody_linear_damping(physics, rigidbody)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.RigidBodyNative", func = "setRigidBodyLinearDamping"),
	c
)]
fn set_rigidbody_linear_damping(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &mut PhysicsState,
    rigidbody: &RigidBodyContext,
    linear_damping: f64,
) -> DropbearNativeResult<()> {
    shared::set_rigidbody_linear_damping(physics, world, rigidbody, linear_damping)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.RigidBodyNative", func = "getRigidBodyAngularDamping"),
	c
)]
fn get_rigidbody_angular_damping(
	#[dropbear_macro::define(WorldPtr)]
	_world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &PhysicsState,
    rigidbody: &RigidBodyContext,
) -> DropbearNativeResult<f64> {
    shared::get_rigidbody_angular_damping(physics, rigidbody)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.RigidBodyNative", func = "setRigidBodyAngularDamping"),
	c
)]
fn set_rigidbody_angular_damping(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &mut PhysicsState,
    rigidbody: &RigidBodyContext,
    angular_damping: f64,
) -> DropbearNativeResult<()> {
    shared::set_rigidbody_angular_damping(physics, world, rigidbody, angular_damping)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.RigidBodyNative", func = "getRigidBodySleep"),
	c
)]
fn get_rigidbody_sleep(
	#[dropbear_macro::define(WorldPtr)]
	_world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &PhysicsState,
    rigidbody: &RigidBodyContext,
) -> DropbearNativeResult<bool> {
    shared::get_rigidbody_sleep(physics, rigidbody)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.RigidBodyNative", func = "setRigidBodySleep"),
	c
)]
fn set_rigidbody_sleep(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &mut PhysicsState,
    rigidbody: &RigidBodyContext,
    sleep: bool,
) -> DropbearNativeResult<()> {
    shared::set_rigidbody_sleep(physics, world, rigidbody, sleep)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.RigidBodyNative", func = "getRigidBodyCcdEnabled"),
	c
)]
fn get_rigidbody_ccd_enabled(
	#[dropbear_macro::define(WorldPtr)]
	_world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &PhysicsState,
    rigidbody: &RigidBodyContext,
) -> DropbearNativeResult<bool> {
    shared::get_rigidbody_ccd(physics, rigidbody)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.RigidBodyNative", func = "setRigidBodyCcdEnabled"),
	c
)]
fn set_rigidbody_ccd_enabled(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &mut PhysicsState,
    rigidbody: &RigidBodyContext,
    ccd_enabled: bool,
) -> DropbearNativeResult<()> {
    shared::set_rigidbody_ccd(physics, world, rigidbody, ccd_enabled)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.RigidBodyNative", func = "getRigidBodyLinearVelocity"),
	c
)]
fn get_rigidbody_linear_velocity(
	#[dropbear_macro::define(WorldPtr)]
	_world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &PhysicsState,
    rigidbody: &RigidBodyContext,
) -> DropbearNativeResult<NVector3> {
    shared::get_rigidbody_linvel(physics, rigidbody)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.RigidBodyNative", func = "setRigidBodyLinearVelocity"),
	c
)]
fn set_rigidbody_linear_velocity(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &mut PhysicsState,
    rigidbody: &RigidBodyContext,
    linear_velocity: &NVector3,
) -> DropbearNativeResult<()> {
    shared::set_rigidbody_linvel(physics, world, rigidbody, *linear_velocity)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.RigidBodyNative", func = "getRigidBodyAngularVelocity"),
	c
)]
fn get_rigidbody_angular_velocity(
	#[dropbear_macro::define(WorldPtr)]
	_world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &PhysicsState,
    rigidbody: &RigidBodyContext,
) -> DropbearNativeResult<NVector3> {
    shared::get_rigidbody_angvel(physics, rigidbody)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.RigidBodyNative", func = "setRigidBodyAngularVelocity"),
	c
)]
fn set_rigidbody_angular_velocity(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &mut PhysicsState,
    rigidbody: &RigidBodyContext,
    angular_velocity: &NVector3,
) -> DropbearNativeResult<()> {
    shared::set_rigidbody_angvel(physics, world, rigidbody, *angular_velocity)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.RigidBodyNative", func = "getRigidBodyLockTranslation"),
	c
)]
fn get_rigidbody_lock_translation(
	#[dropbear_macro::define(WorldPtr)]
	_world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &PhysicsState,
    rigidbody: &RigidBodyContext,
) -> DropbearNativeResult<AxisLock> {
    shared::get_rigidbody_lock_translation(physics, rigidbody)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.RigidBodyNative", func = "setRigidBodyLockTranslation"),
	c
)]
fn set_rigidbody_lock_translation(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &mut PhysicsState,
    rigidbody: &RigidBodyContext,
    lock_translation: &AxisLock,
) -> DropbearNativeResult<()> {
    shared::set_rigidbody_lock_translation(physics, world, rigidbody, *lock_translation)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.RigidBodyNative", func = "getRigidBodyLockRotation"),
	c
)]
fn get_rigidbody_lock_rotation(
	#[dropbear_macro::define(WorldPtr)]
	_world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &PhysicsState,
    rigidbody: &RigidBodyContext,
) -> DropbearNativeResult<AxisLock> {
    shared::get_rigidbody_lock_rotation(physics, rigidbody)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.RigidBodyNative", func = "setRigidBodyLockRotation"),
	c
)]
fn set_rigidbody_lock_rotation(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &mut PhysicsState,
    rigidbody: &RigidBodyContext,
    lock_rotation: &AxisLock,
) -> DropbearNativeResult<()> {
    shared::set_rigidbody_lock_rotation(physics, world, rigidbody, *lock_rotation)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.RigidBodyNative", func = "getRigidBodyChildren"),
	c
)]
fn get_rigidbody_children(
	#[dropbear_macro::define(WorldPtr)]
	_world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &PhysicsState,
    rigidbody: &RigidBodyContext,
) -> DropbearNativeResult<Vec<NCollider>> {
    let children = shared::get_rigidbody_children(physics, rigidbody)?;
    let colliders = children
        .into_iter()
        .map(|handle| {
            let (idx, generation) = handle.into_raw_parts();
            NCollider {
                index: IndexNative { index: idx, generation },
                entity_id: rigidbody.entity_id,
                id: idx,
            }
        })
        .collect();

    Ok(colliders)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.RigidBodyNative", func = "applyImpulse"),
	c
)]
fn apply_impulse(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &mut PhysicsState,
    rigidbody: &RigidBodyContext,
    x: f64,
    y: f64,
    z: f64,
) -> DropbearNativeResult<()> {
    let impulse = NVector3::new(x, y, z);
    shared::apply_impulse(physics, world, rigidbody, impulse)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.RigidBodyNative", func = "applyTorqueImpulse"),
	c
)]
fn apply_torque_impulse(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &mut PhysicsState,
    rigidbody: &RigidBodyContext,
    x: f64,
    y: f64,
    z: f64,
) -> DropbearNativeResult<()> {
    let torque = NVector3::new(x, y, z);
    shared::apply_torque_impulse(physics, world, rigidbody, torque)
}