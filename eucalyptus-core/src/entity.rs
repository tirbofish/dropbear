pub mod jni {
    #![allow(non_snake_case)]
    use crate::hierarchy::{Children, Parent};
    use crate::states::Label;
    use crate::{convert_jlong_to_entity, convert_jstring, convert_ptr};
    use hecs::World;
    use jni::objects::{JClass, JString, JValue};
    use jni::sys::{jlong, jlongArray, jobject, jsize, jstring};
    use jni::JNIEnv;

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_EntityRefNative_getEntityLabel(
        mut env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
    ) -> jstring {
        let world = crate::convert_ptr!(world_ptr => World);
        let entity = crate::convert_jlong_to_entity!(entity_id);

        let Ok(label) = world.get::<&Label>(entity) else {
            let _ = env.throw_new("java/lang/InvalidArgumentException", "Unable to locate label entity");
            return std::ptr::null_mut();
        };

        let label = label.as_str();

        let Ok(string) = env.new_string(label) else {
            let _ = env.throw_new("java/lang/OutOfMemoryException", "Unable to create new string");
            return std::ptr::null_mut();
        };

        string.into_raw()
    }

    pub fn Java_com_dropbear_EntityRefNative_getChildren(
        mut env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
    ) -> jlongArray {
        let world = crate::convert_ptr!(world_ptr => World);
        let entity = crate::convert_jlong_to_entity!(entity_id);

        if let Ok(mut q) = world.query_one::<&Children>(entity)
            && let Some(children) = q.get()
        {
            let entity_bytes = children.children().iter().map(|c| c.to_bits().get() as i64).collect::<Vec<_>>();

            let Ok(jarray) = env.new_long_array(entity_bytes.len() as jsize) else {
                let _ = env.throw_new("java/lang/OutOfMemoryException", "Unable to create new long array");
                return std::ptr::null_mut();
            };

            if let Err(e) = env.set_long_array_region(&jarray, 0, entity_bytes.as_slice()) {
                let _ = env.throw_new("java/lang/RuntimeException", format!("Unable to populate long array: {}", e));
                return std::ptr::null_mut()
            }

            jarray.into_raw()
        } else {
            // could be that the entity just doesn't have any children.
            std::ptr::null_mut()
        }
    }

    pub fn Java_com_dropbear_EntityRefNative_getChildByLabel(
        mut env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
        label: JString,
    ) -> jobject {
        let world = crate::convert_ptr!(world_ptr => World);
        let entity = crate::convert_jlong_to_entity!(entity_id);
        let target = convert_jstring!(env, label);

        if let Ok(mut q) = world.query_one::<&Children>(entity)
            && let Some(children) = q.get()
        {
            for child in children.children() {
                if let Ok(label) = world.get::<&Label>(entity) {
                    if label.as_str() == target {
                        let found = child.clone();

                        return crate::return_boxed!(&mut env, Some(JValue::Long(found.to_bits().get() as i64)), "(J)Ljava/lang/Long", "java/lang/Long");
                    }
                } else {
                    // skip if error or no entity
                    continue;
                }
            }
            std::ptr::null_mut()
        } else {
            // could be that the entity just doesn't have any children.
            std::ptr::null_mut()
        }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_EntityRefNative_getParent(
        mut env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
    ) -> jobject {
        let world = convert_ptr!(world_ptr => World);
        let entity = convert_jlong_to_entity!(entity_id);

        if let Ok(mut q) = world.query_one::<&Parent>(entity) {
            if let Some(parent) = q.get() {
                crate::return_boxed!(&mut env, Some(JValue::Long(parent.parent().to_bits().get() as jlong)), "(J)Ljava/lang/Long", "java/lang/Long")
            } else {
                std::ptr::null_mut()
            }
        } else {
            crate::ffi_error_return!("No entity exists")
        }
    }
}

#[dropbear_macro::impl_c_api]
pub mod native {

}