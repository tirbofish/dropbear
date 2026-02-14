//! Scripting module for collider groups. 

use crate::scripting::jni::utils::{FromJObject, ToJObject};
use crate::scripting::native::DropbearNativeError;
use crate::scripting::result::DropbearNativeResult;
use crate::types::IndexNative;
use ::jni::objects::{JObject, JValue};
use ::jni::JNIEnv;

impl ToJObject for IndexNative {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let cls = env.find_class("com/dropbear/physics/Index")
            .map_err(|e| {
                eprintln!("[JNI Error] Could not find Index class: {:?}", e);
                DropbearNativeError::GenericError
            })?;

        let obj = env.new_object(
            cls,
            "(II)V",
            &[
                JValue::Int(self.index as i32),
                JValue::Int(self.generation as i32)
            ]
        ).map_err(|e| {
            eprintln!("[JNI Error] Failed to create Index object: {:?}", e);
            DropbearNativeError::GenericError
        })?;

        Ok(obj)
    }
}

impl FromJObject for IndexNative {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self>
    where
        Self: Sized
    {
        let idx_val = env.get_field(obj, "index", "I")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .i()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let gen_val = env.get_field(obj, "generation", "I")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .i()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        Ok(IndexNative {
            index: idx_val as u32,
            generation: gen_val as u32,
        })
    }
}

pub mod shared {
    use crate::physics::collider::ColliderGroup;
    use hecs::{Entity, World};

    pub fn collider_group_exists_for_entity(world: &World, entity: Entity) -> bool {
        world.get::<&ColliderGroup>(entity).is_ok()
    }
}

pub mod jni {
    #![allow(non_snake_case)]
    use crate::physics::collider::ColliderGroup;
    use crate::types::{NCollider, IndexNative};
    use hecs::World;
    use jni::objects::{JClass, JObject};
    use jni::sys::{jboolean, jlong, jobjectArray};
    use jni::JNIEnv;

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_physics_ColliderGroupNative_colliderGroupExistsForEntity(
        _env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
    ) -> jboolean {
        let world = crate::convert_ptr!(world_ptr => World);
        let entity = crate::convert_jlong_to_entity!(entity_id);

        if super::shared::collider_group_exists_for_entity(&world, entity) {
            true.into()
        } else {
            false.into()
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_physics_ColliderGroupNative_getColliderGroupColliders(
        mut env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        physics_ptr: jlong,
        entity_id: jlong,
    ) -> jobjectArray {
        let world = crate::convert_ptr!(world_ptr => hecs::World);
        let physics = crate::convert_ptr!(physics_ptr => crate::physics::PhysicsState);
        let entity = crate::convert_jlong_to_entity!(entity_id);

        // We check if the ColliderGroup component exists first
        if world.get::<&ColliderGroup>(entity).is_ok() {
            let handles_opt = physics.entity_label_map
                .get(&entity)
                .and_then(|label| physics.colliders_entity_map.get(label));

            let mut colliders: Vec<NCollider> = Vec::new();

            if let Some(handles) = handles_opt {
                for (_, handle) in handles {
                    let (idx, generation) = handle.into_raw_parts();

                    let col = NCollider {
                        index: IndexNative {
                            index: idx,
                            generation,
                        },
                        entity_id: entity_id as u64,
                        id: idx,
                    };
                    colliders.push(col);
                }
            }

            let collider_cls = match env.find_class("com/dropbear/physics/Collider") {
                Ok(cls) => cls,
                Err(e) => {
                    eprintln!("[JNI Error] Could not find Collider class: {:?}", e);
                    return std::ptr::null_mut();
                }
            };

            let output_array = match env.new_object_array(colliders.len() as i32, &collider_cls, JObject::null()) {
                Ok(arr) => arr,
                Err(e) => {
                    eprintln!("[JNI Error] Failed to allocate Collider array: {:?}", e);
                    let _ = env.throw_new("java/lang/OutOfMemoryError", "Could not allocate collider array");
                    return std::ptr::null_mut();
                }
            };

            use crate::scripting::jni::utils::ToJObject;

            for (i, ffi) in colliders.iter().enumerate() {
                let java_obj = match ffi.to_jobject(&mut env) {
                    Ok(obj) => obj,
                    Err(_) => return std::ptr::null_mut(),
                };

                if let Err(e) = env.set_object_array_element(&output_array, i as i32, java_obj) {
                    eprintln!("[JNI Error] Failed to set array element: {:?}", e);
                    return std::ptr::null_mut();
                }
            }

            output_array.into_raw()

        } else {
            let _ = env.throw_new("java/lang/RuntimeException", "Entity does not have a ColliderGroup component");
            std::ptr::null_mut()
        }
    }
}

#[dropbear_macro::impl_c_api]
pub mod native {
    use crate::convert_ptr;
    use crate::physics::collider::ColliderGroup;
    use crate::physics::PhysicsState;
    use crate::ptr::{PhysicsStatePtr, WorldPtr};
    use crate::scripting::native::DropbearNativeError;
    use crate::scripting::result::DropbearNativeResult;
    use crate::types::{NCollider, IndexNative};
    use hecs::Entity;

    pub fn dropbear_collider_group_exists_for_entity(
        world_ptr: WorldPtr,
        entity_id: u64,
    ) -> DropbearNativeResult<bool> {
        let world = convert_ptr!(world_ptr => hecs::World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        DropbearNativeResult::Ok(super::shared::collider_group_exists_for_entity(world, entity))
    }

    pub fn dropbear_get_collider_group_colliders(
        world_ptr: WorldPtr,
        physics_ptr: PhysicsStatePtr,
        entity_id: u64,
        out_count: *mut usize,
    ) -> DropbearNativeResult<*mut NCollider> {
        let world = convert_ptr!(world_ptr => hecs::World);
        let physics = convert_ptr!(physics_ptr => PhysicsState);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if out_count.is_null() {
            return DropbearNativeResult::Err(DropbearNativeError::NullPointer);
        }

        if world.get::<&ColliderGroup>(entity).is_err() {
            unsafe { *out_count = 0; }
            return DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent);
        }

        let handles_opt = physics.entity_label_map
            .get(&entity)
            .and_then(|label| physics.colliders_entity_map.get(label));

        let mut colliders: Vec<NCollider> = Vec::new();

        if let Some(handles) = handles_opt {
            for (_, handle) in handles {
                let (idx, generation) = handle.into_raw_parts();

                let col = NCollider {
                    index: IndexNative {
                        index: idx,
                        generation,
                    },
                    entity_id,
                    id: idx,
                };
                colliders.push(col);
            }
        }

        unsafe { *out_count = colliders.len(); }

        colliders.shrink_to_fit();
        let ptr = colliders.as_mut_ptr();
        std::mem::forget(colliders);

        DropbearNativeResult::Ok(ptr)
    }

    pub fn dropbear_free_collider_array(
        ptr: *mut NCollider,
        count: usize,
    ) -> DropbearNativeResult<()> {
        if ptr.is_null() {
            return DropbearNativeResult::Ok(());
        }

        unsafe {
            let _ = Vec::from_raw_parts(ptr, count, count);
        }

        DropbearNativeResult::Ok(())
    }
}