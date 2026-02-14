use ::jni::JNIEnv;
use ::jni::objects::{JObject, JValue};
use crate::scripting::jni::utils::ToJObject;
use crate::scripting::native::DropbearNativeError;
use crate::scripting::result::DropbearNativeResult;
use crate::types::{NCollider, IndexNative, NVector3};
use dropbear_engine::entity::Transform as DbTransform;
use hecs::{Entity, World};
use rapier3d::control::CharacterCollision;
use rapier3d::parry::query::ShapeCastStatus;

use crate::physics::collider::ColliderGroup;
use crate::physics::kcc::KCC;

fn get_collision_from_world(world: &World, entity: Entity, collision_handle: IndexNative) -> DropbearNativeResult<CharacterCollision> {
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

impl ToJObject for ShapeCastStatus {
	fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
		let class = env
			.find_class("com/dropbear/physics/ShapeCastStatus")
			.map_err(|_| DropbearNativeError::JNIClassNotFound)?;

		let name = match self {
			ShapeCastStatus::OutOfIterations => "OutOfIterations",
			ShapeCastStatus::Converged => "Converged",
			ShapeCastStatus::Failed => "Failed",
			ShapeCastStatus::PenetratingOrWithinTargetDist => "PenetratingOrWithinTargetDist",
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

	pub fn get_collider(world: &World, entity: Entity, collision_handle: IndexNative) -> DropbearNativeResult<NCollider> {
		let collision = super::get_collision_from_world(world, entity, collision_handle)?;
		collider_ffi_from_handle(world, collision.handle)
			.ok_or(DropbearNativeError::PhysicsObjectNotFound)
	}

	pub fn get_character_position(world: &World, entity: Entity, collision_handle: IndexNative) -> DropbearNativeResult<DbTransform> {
		let collision = super::get_collision_from_world(world, entity, collision_handle)?;

		let iso = collision.character_pos;
		let t = iso.translation;
		let rot = iso.rotation;
		let q: Quaternion<f32> = Quaternion::from(rot);

		Ok(DbTransform {
			position: DVec3::new(t.x as f64, t.y as f64, t.z as f64),
			rotation: DQuat::from_xyzw(q.i as f64, q.j as f64, q.k as f64, q.w as f64),
			scale: DVec3::ONE,
		})
	}

	pub fn get_translation_applied(world: &World, entity: Entity, collision_handle: IndexNative) -> DropbearNativeResult<NVector3> {
		let collision = super::get_collision_from_world(world, entity, collision_handle)?;
		let v = collision.translation_applied;
		Ok(NVector3 { x: v.x as f64, y: v.y as f64, z: v.z as f64 })
	}

	pub fn get_translation_remaining(world: &World, entity: Entity, collision_handle: IndexNative) -> DropbearNativeResult<NVector3> {
		let collision = super::get_collision_from_world(world, entity, collision_handle)?;
		let v = collision.translation_remaining;
		Ok(NVector3 { x: v.x as f64, y: v.y as f64, z: v.z as f64 })
	}

	pub fn get_time_of_impact(world: &World, entity: Entity, collision_handle: IndexNative) -> DropbearNativeResult<f64> {
		let collision = super::get_collision_from_world(world, entity, collision_handle)?;
		Ok(collision.hit.time_of_impact as f64)
	}

	pub fn get_witness1(world: &World, entity: Entity, collision_handle: IndexNative) -> DropbearNativeResult<NVector3> {
		let collision = super::get_collision_from_world(world, entity, collision_handle)?;
		let p = collision.hit.witness1;
		Ok(NVector3 { x: p.x as f64, y: p.y as f64, z: p.z as f64 })
	}

	pub fn get_witness2(world: &World, entity: Entity, collision_handle: IndexNative) -> DropbearNativeResult<NVector3> {
		let collision = super::get_collision_from_world(world, entity, collision_handle)?;
		let p = collision.hit.witness2;
		Ok(NVector3 { x: p.x as f64, y: p.y as f64, z: p.z as f64 })
	}

	pub fn get_normal1(world: &World, entity: Entity, collision_handle: IndexNative) -> DropbearNativeResult<NVector3> {
		let collision = super::get_collision_from_world(world, entity, collision_handle)?;
		let n = collision.hit.normal1;
		Ok(NVector3 { x: n.x as f64, y: n.y as f64, z: n.z as f64 })
	}

	pub fn get_normal2(world: &World, entity: Entity, collision_handle: IndexNative) -> DropbearNativeResult<NVector3> {
		let collision = super::get_collision_from_world(world, entity, collision_handle)?;
		let n = collision.hit.normal2;
		Ok(NVector3 { x: n.x as f64, y: n.y as f64, z: n.z as f64 })
	}

	pub fn get_status(world: &World, entity: Entity, collision_handle: IndexNative) -> DropbearNativeResult<ShapeCastStatus> {
		let collision = super::get_collision_from_world(world, entity, collision_handle)?;
		Ok(collision.hit.status)
	}
}

pub mod jni {
	#![allow(non_snake_case)]

	use hecs::World;
	use jni::objects::JClass;
	use jni::sys::{jdouble, jlong, jobject};
	use jni::JNIEnv;
	use crate::convert_jlong_to_entity;
	use crate::convert_ptr;
	use crate::scripting::jni::utils::{FromJObject, ToJObject};
	use crate::types::IndexNative;
	use jni::objects::JObject;

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_CharacterCollisionNative_getCollider(
		mut env: JNIEnv,
		_: JClass,
		world_handle: jlong,
		entity: jlong,
		collision_handle: JObject,
	) -> jobject {
		let world = convert_ptr!(world_handle => World);
		let entity = convert_jlong_to_entity!(entity);

		let Ok(index_native) = IndexNative::from_jobject(&mut env, &collision_handle) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Failed to read Index for collision handle");
			return std::ptr::null_mut();
		};

		match super::shared::get_collider(&world, entity, index_native) {
			Ok(ffi) => match ffi.to_jobject(&mut env) {
				Ok(obj) => obj.into_raw(),
				Err(e) => {
					let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to create Collider object: {e}"));
					std::ptr::null_mut()
				}
			},
			Err(e) => {
				let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to resolve collider: {e:?}"));
				std::ptr::null_mut()
			}
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_CharacterCollisionNative_getCharacterPosition(
		mut env: JNIEnv,
		_: JClass,
		world_handle: jlong,
		entity: jlong,
		collision_handle: JObject,
	) -> jobject {
		let world = convert_ptr!(world_handle => World);
		let entity = convert_jlong_to_entity!(entity);

		let Ok(index_native) = IndexNative::from_jobject(&mut env, &collision_handle) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Failed to read Index for collision handle");
			return std::ptr::null_mut();
		};

		match super::shared::get_character_position(&world, entity, index_native) {
			Ok(transform) => match transform.to_jobject(&mut env) {
				Ok(obj) => obj.into_raw(),
				Err(e) => {
					let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to create Transform object: {e}"));
					std::ptr::null_mut()
				}
			},
			Err(e) => {
				let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to resolve character position: {e:?}"));
				std::ptr::null_mut()
			}
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_CharacterCollisionNative_getTranslationApplied(
		mut env: JNIEnv,
		_: JClass,
		world_handle: jlong,
		entity: jlong,
		collision_handle: JObject,
	) -> jobject {
		let world = convert_ptr!(world_handle => World);
		let entity = convert_jlong_to_entity!(entity);

		let Ok(index_native) = IndexNative::from_jobject(&mut env, &collision_handle) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Failed to read Index for collision handle");
			return std::ptr::null_mut();
		};

		match super::shared::get_translation_applied(&world, entity, index_native) {
			Ok(v) => match v.to_jobject(&mut env) {
				Ok(obj) => obj.into_raw(),
				Err(e) => {
					let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to create Vector3d: {e}"));
					std::ptr::null_mut()
				}
			},
			Err(e) => {
				let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to resolve translation applied: {e:?}"));
				std::ptr::null_mut()
			}
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_CharacterCollisionNative_getTranslationRemaining(
		mut env: JNIEnv,
		_: JClass,
		world_handle: jlong,
		entity: jlong,
		collision_handle: JObject,
	) -> jobject {
		let world = convert_ptr!(world_handle => World);
		let entity = convert_jlong_to_entity!(entity);

		let Ok(index_native) = IndexNative::from_jobject(&mut env, &collision_handle) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Failed to read Index for collision handle");
			return std::ptr::null_mut();
		};

		match super::shared::get_translation_remaining(&world, entity, index_native) {
			Ok(v) => match v.to_jobject(&mut env) {
				Ok(obj) => obj.into_raw(),
				Err(e) => {
					let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to create Vector3d: {e}"));
					std::ptr::null_mut()
				}
			},
			Err(e) => {
				let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to resolve translation remaining: {e:?}"));
				std::ptr::null_mut()
			}
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_CharacterCollisionNative_getTimeOfImpact(
		mut env: JNIEnv,
		_: JClass,
		world_handle: jlong,
		entity: jlong,
		collision_handle: JObject,
	) -> jdouble {
		let world = convert_ptr!(world_handle => World);
		let entity = convert_jlong_to_entity!(entity);

		let Ok(index_native) = IndexNative::from_jobject(&mut env, &collision_handle) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Failed to read Index for collision handle");
			return -1.0 as jdouble;
		};

		match super::shared::get_time_of_impact(&world, entity, index_native) {
			Ok(v) => v as jdouble,
			Err(e) => {
				let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to resolve time of impact: {e:?}"));
				-1.0 as jdouble
			}
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_CharacterCollisionNative_getWitness1(
		mut env: JNIEnv,
		_: JClass,
		world_handle: jlong,
		entity: jlong,
		collision_handle: JObject,
	) -> jobject {
		let world = convert_ptr!(world_handle => World);
		let entity = convert_jlong_to_entity!(entity);

		let Ok(index_native) = IndexNative::from_jobject(&mut env, &collision_handle) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Failed to read Index for collision handle");
			return std::ptr::null_mut();
		};

		match super::shared::get_witness1(&world, entity, index_native) {
			Ok(v) => match v.to_jobject(&mut env) {
				Ok(obj) => obj.into_raw(),
				Err(e) => {
					let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to create Vector3d: {e}"));
					std::ptr::null_mut()
				}
			},
			Err(e) => {
				let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to resolve witness1: {e:?}"));
				std::ptr::null_mut()
			}
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_CharacterCollisionNative_getWitness2(
		mut env: JNIEnv,
		_: JClass,
		world_handle: jlong,
		entity: jlong,
		collision_handle: JObject,
	) -> jobject {
		let world = convert_ptr!(world_handle => World);
		let entity = convert_jlong_to_entity!(entity);

		let Ok(index_native) = IndexNative::from_jobject(&mut env, &collision_handle) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Failed to read Index for collision handle");
			return std::ptr::null_mut();
		};

		match super::shared::get_witness2(&world, entity, index_native) {
			Ok(v) => match v.to_jobject(&mut env) {
				Ok(obj) => obj.into_raw(),
				Err(e) => {
					let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to create Vector3d: {e}"));
					std::ptr::null_mut()
				}
			},
			Err(e) => {
				let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to resolve witness2: {e:?}"));
				std::ptr::null_mut()
			}
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_CharacterCollisionNative_getNormal1(
		mut env: JNIEnv,
		_: JClass,
		world_handle: jlong,
		entity: jlong,
		collision_handle: JObject,
	) -> jobject {
		let world = convert_ptr!(world_handle => World);
		let entity = convert_jlong_to_entity!(entity);

		let Ok(index_native) = IndexNative::from_jobject(&mut env, &collision_handle) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Failed to read Index for collision handle");
			return std::ptr::null_mut();
		};

		match super::shared::get_normal1(&world, entity, index_native) {
			Ok(v) => match v.to_jobject(&mut env) {
				Ok(obj) => obj.into_raw(),
				Err(e) => {
					let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to create Vector3d: {e}"));
					std::ptr::null_mut()
				}
			},
			Err(e) => {
				let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to resolve normal1: {e:?}"));
				std::ptr::null_mut()
			}
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_CharacterCollisionNative_getNormal2(
		mut env: JNIEnv,
		_: JClass,
		world_handle: jlong,
		entity: jlong,
		collision_handle: JObject,
	) -> jobject {
		let world = convert_ptr!(world_handle => World);
		let entity = convert_jlong_to_entity!(entity);

		let Ok(index_native) = IndexNative::from_jobject(&mut env, &collision_handle) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Failed to read Index for collision handle");
			return std::ptr::null_mut();
		};

		match super::shared::get_normal2(&world, entity, index_native) {
			Ok(v) => match v.to_jobject(&mut env) {
				Ok(obj) => obj.into_raw(),
				Err(e) => {
					let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to create Vector3d: {e}"));
					std::ptr::null_mut()
				}
			},
			Err(e) => {
				let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to resolve normal2: {e:?}"));
				std::ptr::null_mut()
			}
		}
	}

	#[unsafe(no_mangle)]
	pub extern "system" fn Java_com_dropbear_physics_CharacterCollisionNative_getStatus(
		mut env: JNIEnv,
		_: JClass,
		world_handle: jlong,
		entity: jlong,
		collision_handle: JObject,
	) -> jobject {
		let world = convert_ptr!(world_handle => World);
		let entity = convert_jlong_to_entity!(entity);

		let Ok(index_native) = IndexNative::from_jobject(&mut env, &collision_handle) else {
			let _ = env.throw_new("java/lang/RuntimeException", "Failed to read Index for collision handle");
			return std::ptr::null_mut();
		};

		match super::shared::get_status(&world, entity, index_native) {
			Ok(status) => match status.to_jobject(&mut env) {
				Ok(obj) => obj.into_raw(),
				Err(e) => {
					let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to create ShapeCastStatus: {e}"));
					std::ptr::null_mut()
				}
			},
			Err(e) => {
				let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to resolve status: {e:?}"));
				std::ptr::null_mut()
			}
		}
	}
}