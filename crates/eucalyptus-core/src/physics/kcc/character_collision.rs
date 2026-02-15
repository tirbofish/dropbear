use ::jni::JNIEnv;
use ::jni::objects::{JObject, JValue};
use crate::scripting::jni::utils::ToJObject;
use crate::scripting::native::DropbearNativeError;
use crate::scripting::result::DropbearNativeResult;
use crate::types::{IndexNative, NCollider, NShapeCastStatus, NTransform, NVector3};
use dropbear_engine::entity::Transform;
use hecs::{Entity, World};
use rapier3d::control::CharacterCollision;

use crate::physics::collider::ColliderGroup;
use crate::physics::kcc::KCC;
use crate::ptr::WorldPtr;

fn get_collision_from_world(world: &World, entity: Entity, collision_handle: &IndexNative) -> DropbearNativeResult<CharacterCollision> {
	let kcc = world
		.get::<&KCC>(entity)
		.map_err(|_| DropbearNativeError::NoSuchComponent)?;

	kcc.collisions
		.iter()
		.copied()
		.find(|c| {
			let (idx, generation) = c.handle.into_raw_parts();
			idx == collision_handle.index && generation == collision_handle.generation
		})
		.ok_or(DropbearNativeError::NoSuchHandle)
}

fn collider_ffi_from_handle(world: &World, handle: rapier3d::prelude::ColliderHandle) -> Option<NCollider> {
	let (idx, generation) = handle.into_raw_parts();

	for (entity, group) in world.query::<(Entity, &ColliderGroup)>().iter() {
		if group.colliders.iter().any(|c| c.id == idx) {
			return Some(NCollider {
				index: IndexNative { index: idx, generation },
				entity_id: entity.to_bits().get(),
				id: idx,
			});
		}
	}

	None
}

impl ToJObject for NShapeCastStatus {
	fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
		let class = env
			.find_class("com/dropbear/physics/ShapeCastStatus")
			.map_err(|_| DropbearNativeError::JNIClassNotFound)?;

		let name = match self {
			NShapeCastStatus::OutOfIterations => "OutOfIterations",
			NShapeCastStatus::Converged => "Converged",
			NShapeCastStatus::Failed => "Failed",
			NShapeCastStatus::PenetratingOrWithinTargetDist => "PenetratingOrWithinTargetDist",
		};

		let name_jstring = env
			.new_string(name)
			.map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

		let value = env
			.call_static_method(
				class,
				"valueOf",
				"(Ljava/lang/String;)Lcom/dropbear/physics/ShapeCastStatus;",
				&[JValue::Object(&name_jstring)],
			)
			.map_err(|_| DropbearNativeError::JNIMethodNotFound)?
			.l()
			.map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

		Ok(value)
	}
}

pub mod shared {
	use super::*;
	use glam::{DQuat, DVec3};
	use rapier3d::na::Quaternion;
	use crate::types::NVector3;

	pub fn get_collider(world: &World, entity: Entity, collision_handle: &IndexNative) -> DropbearNativeResult<NCollider> {
		let collision = get_collision_from_world(world, entity, collision_handle)?;
		collider_ffi_from_handle(world, collision.handle)
			.ok_or(DropbearNativeError::PhysicsObjectNotFound)
	}

	pub fn get_character_position(world: &World, entity: Entity, collision_handle: &IndexNative) -> DropbearNativeResult<NTransform> {
		let collision = get_collision_from_world(world, entity, collision_handle)?;

		let iso = collision.character_pos;
		let t = iso.translation;
		let rot = iso.rotation;
		let q: Quaternion<f32> = Quaternion::from(rot);

		Ok(NTransform {
			position: DVec3::new(t.x as f64, t.y as f64, t.z as f64).into(),
			rotation: DQuat::from_xyzw(q.i as f64, q.j as f64, q.k as f64, q.w as f64).into(),
			scale: DVec3::ONE.into(),
		})
	}

	pub fn get_translation_applied(world: &World, entity: Entity, collision_handle: &IndexNative) -> DropbearNativeResult<NVector3> {
		let collision = get_collision_from_world(world, entity, collision_handle)?;
		let v = collision.translation_applied;
		Ok(NVector3 { x: v.x as f64, y: v.y as f64, z: v.z as f64 })
	}

	pub fn get_translation_remaining(world: &World, entity: Entity, collision_handle: &IndexNative) -> DropbearNativeResult<NVector3> {
		let collision = get_collision_from_world(world, entity, collision_handle)?;
		let v = collision.translation_remaining;
		Ok(NVector3 { x: v.x as f64, y: v.y as f64, z: v.z as f64 })
	}

	pub fn get_time_of_impact(world: &World, entity: Entity, collision_handle: &IndexNative) -> DropbearNativeResult<f64> {
		let collision = get_collision_from_world(world, entity, collision_handle)?;
		Ok(collision.hit.time_of_impact as f64)
	}

	pub fn get_witness1(world: &World, entity: Entity, collision_handle: &IndexNative) -> DropbearNativeResult<NVector3> {
		let collision = get_collision_from_world(world, entity, collision_handle)?;
		let p = collision.hit.witness1;
		Ok(NVector3 { x: p.x as f64, y: p.y as f64, z: p.z as f64 })
	}

	pub fn get_witness2(world: &World, entity: Entity, collision_handle: &IndexNative) -> DropbearNativeResult<NVector3> {
		let collision = get_collision_from_world(world, entity, collision_handle)?;
		let p = collision.hit.witness2;
		Ok(NVector3 { x: p.x as f64, y: p.y as f64, z: p.z as f64 })
	}

	pub fn get_normal1(world: &World, entity: Entity, collision_handle: &IndexNative) -> DropbearNativeResult<NVector3> {
		let collision = get_collision_from_world(world, entity, collision_handle)?;
		let n = collision.hit.normal1;
		Ok(NVector3 { x: n.x as f64, y: n.y as f64, z: n.z as f64 })
	}

	pub fn get_normal2(world: &World, entity: Entity, collision_handle: &IndexNative) -> DropbearNativeResult<NVector3> {
		let collision = get_collision_from_world(world, entity, collision_handle)?;
		let n = collision.hit.normal2;
		Ok(NVector3 { x: n.x as f64, y: n.y as f64, z: n.z as f64 })
	}

	pub fn get_status(world: &World, entity: Entity, collision_handle: &IndexNative) -> DropbearNativeResult<NShapeCastStatus> {
		let collision = get_collision_from_world(world, entity, collision_handle)?;
		Ok(collision.hit.status.into())
	}
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.CharacterCollisionNative", func = "getCollider"),
	c
)]
fn get_character_collision_collider(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::entity]
    entity: hecs::Entity,
    collision_handle: &IndexNative,
) -> DropbearNativeResult<crate::types::NCollider> {
    shared::get_collider(world, entity, collision_handle)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.CharacterCollisionNative", func = "getCharacterPosition"),
	c
)]
fn get_character_collision_position(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::entity]
    entity: hecs::Entity,
    collision_handle: &IndexNative,
) -> DropbearNativeResult<NTransform> {
    shared::get_character_position(world, entity, collision_handle)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.CharacterCollisionNative", func = "getTranslationApplied"),
	c
)]
fn get_character_collision_translation_applied(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::entity]
    entity: hecs::Entity,
    collision_handle: &IndexNative,
) -> DropbearNativeResult<NVector3> {
    shared::get_translation_applied(world, entity, collision_handle)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.CharacterCollisionNative", func = "getTranslationRemaining"),
	c
)]
fn get_character_collision_translation_remaining(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::entity]
    entity: hecs::Entity,
    collision_handle: &IndexNative,
) -> DropbearNativeResult<NVector3> {
    shared::get_translation_remaining(world, entity, collision_handle)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.CharacterCollisionNative", func = "getTimeOfImpact"),
	c
)]
fn get_character_collision_time_of_impact(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::entity]
    entity: hecs::Entity,
    collision_handle: &IndexNative,
) -> DropbearNativeResult<f64> {
    shared::get_time_of_impact(world, entity, collision_handle)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.CharacterCollisionNative", func = "getWitness1"),
	c
)]
fn get_character_collision_witness1(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::entity]
    entity: hecs::Entity,
    collision_handle: &IndexNative,
) -> DropbearNativeResult<NVector3> {
    shared::get_witness1(world, entity, collision_handle)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.CharacterCollisionNative", func = "getWitness2"),
	c
)]
fn get_character_collision_witness2(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::entity]
    entity: hecs::Entity,
    collision_handle: &IndexNative,
) -> DropbearNativeResult<NVector3> {
    shared::get_witness2(world, entity, collision_handle)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.CharacterCollisionNative", func = "getNormal1"),
	c
)]
fn get_character_collision_normal1(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::entity]
    entity: hecs::Entity,
    collision_handle: &IndexNative,
) -> DropbearNativeResult<NVector3> {
    shared::get_normal1(world, entity, collision_handle)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.CharacterCollisionNative", func = "getNormal2"),
	c
)]
fn get_character_collision_normal2(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::entity]
    entity: hecs::Entity,
    collision_handle: &IndexNative,
) -> DropbearNativeResult<NVector3> {
    shared::get_normal2(world, entity, collision_handle)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.CharacterCollisionNative", func = "getStatus"),
	c
)]
fn get_character_collision_status(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropbear_macro::entity]
    entity: hecs::Entity,
    collision_handle: &IndexNative,
) -> DropbearNativeResult<NShapeCastStatus> {
    shared::get_status(world, entity, collision_handle)
}