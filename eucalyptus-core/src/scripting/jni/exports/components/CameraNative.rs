#![allow(non_snake_case)]

use hecs::World;
use jni::JNIEnv;
use jni::objects::{JClass, JObject, JString, JValue};
use jni::sys::{jlong, jobject};
use dropbear_engine::camera::Camera;
use crate::camera::{CameraComponent, CameraType};
use crate::convert_jlong_to_entity;
use crate::scripting::jni::utils::{create_vector3, extract_vector3};

/**
 * Class:     `com_dropbear_ffi_components_CameraNative`
 *
 * Method:    `getCamera`
 *
 * Signature: `(JLjava/lang/String;)Lcom/dropbear/Camera;`
 *
 * `JNIEXPORT jobject JNICALL Java_com_dropbear_ffi_components_CameraNative_getCamera
 * (JNIEnv *, jclass, jlong, jstring);`
*/
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_CameraNative_getCamera(
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    camera_name: JString,
) -> jobject {
    let world = world_handle as *mut World;
    if world.is_null() {
        eprintln!("[Java_com_dropbear_ffi_components_CameraNative_getCamera] [ERROR] World pointer is null");
        return std::ptr::null_mut();
    }

    let world = unsafe { &*world };

    let label = env.get_string(&camera_name);
    let label: String = if let Ok(str) = label {
        str.to_string_lossy().to_string()
    } else {
        eprintln!("[Java_com_dropbear_ffi_components_CameraNative_getCamera] [ERROR] Failed to get camera name");
        return std::ptr::null_mut();
    };

    if let Some((id, (cam, comp))) = world
        .query::<(&Camera, &CameraComponent)>()
        .iter()
        .find(|(_, (cam, _))| cam.label == label)
    {
        if matches!(comp.camera_type, CameraType::Debug) {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CameraNative_getCamera] [WARN] Querying a CameraType::Debug is illegal, returning null"
            );
            return std::ptr::null_mut();
        }

        let entity_id = if let Ok(v) = env.find_class("com/dropbear/EntityId") {
            v
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CameraNative_getCamera] [ERROR] Unable to find EntityId class"
            );
            return std::ptr::null_mut();
        };
        let entity_id = if let Ok(v) = env.new_object(
            entity_id,
            "(J)V",
            &[JValue::Long(id.to_bits().get() as i64)],
        ) {
            v
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CameraNative_getCamera] [ERROR] Unable to create new entity_id object"
            );
            return std::ptr::null_mut();
        };

        let label = if let Ok(v) = env.new_string(cam.label.as_str()) {
            v
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CameraNative_getCamera] [ERROR] Unable to create new string for label"
            );
            return std::ptr::null_mut();
        };

        let eye = if let Ok(v) = create_vector3(&mut env, cam.eye.x, cam.eye.y, cam.eye.z) {
            v
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CameraNative_getCamera] [ERROR] Unable to create vector3 for eye"
            );
            return std::ptr::null_mut();
        };

        let target = if let Ok(v) =
            create_vector3(&mut env, cam.target.x, cam.target.y, cam.target.z)
        {
            v
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CameraNative_getCamera] [ERROR] Unable to create vector3 for target"
            );
            return std::ptr::null_mut();
        };

        let up = if let Ok(v) = create_vector3(&mut env, cam.up.x, cam.up.y, cam.up.z) {
            v
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CameraNative_getCamera] [ERROR] Unable to create vector3 for up"
            );
            return std::ptr::null_mut();
        };

        let class = if let Ok(v) = env.find_class("com/dropbear/Camera") {
            v
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CameraNative_getCamera] [ERROR] Unable to locate camera class"
            );
            return std::ptr::null_mut();
        };

        let camera_obj = if let Ok(v) = env.new_object(
            class,
            "(Ljava/lang/String;Lcom/dropbear/EntityId;Lcom/dropbear/math/Vector3;Lcom/dropbear/math/Vector3;Lcom/dropbear/math/Vector3;DDDDDDDD)V",
            &[
                JValue::Object(&label),
                JValue::Object(&entity_id),
                JValue::Object(&eye),
                JValue::Object(&target),
                JValue::Object(&up),
                JValue::Double(cam.aspect),
                JValue::Double(cam.settings.fov_y),
                JValue::Double(cam.znear),
                JValue::Double(cam.zfar),
                JValue::Double(cam.yaw),
                JValue::Double(cam.pitch),
                JValue::Double(cam.settings.speed),
                JValue::Double(cam.settings.sensitivity),
            ],
        ) {
            v
        } else {
            eprintln!("[Java_com_dropbear_ffi_components_CameraNative_getCamera] [ERROR] Unable to create the camera object");
            return std::ptr::null_mut();
        };

        return camera_obj.as_raw();
    }

    std::ptr::null_mut()
}



/**
 * Class:     `com_dropbear_ffi_components_CameraNative`
 *
 * Method:    `getAttachedCamera`
 *
 * Signature: `(JJ)Lcom/dropbear/Camera;`
 *
 * `JNIEXPORT jobject JNICALL Java_com_dropbear_ffi_components_CameraNative_getAttachedCamera
 * (JNIEnv *, jclass, jlong, jlong);`
*/
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_CameraNative_getAttachedCamera(
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    entity_id: jlong,
) -> jobject {
    let world = world_handle as *mut World;
    if world.is_null() {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CameraNative_getAttachedCamera] [ERROR] World pointer is null"
        );
        return std::ptr::null_mut();
    }

    let world = unsafe { &*world };
    let entity = convert_jlong_to_entity!(entity_id);

    if let Ok(mut q) = world.query_one::<(&Camera, &CameraComponent)>(entity)
        && let Some((cam, comp)) = q.get()
    {
        if matches!(comp.camera_type, CameraType::Debug) {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CameraNative_getAttachedCamera] [WARN] Querying a CameraType::Debug is illegal, returning null"
            );
            return std::ptr::null_mut();
        }

        let entity_id = if let Ok(v) = env.find_class("com/dropbear/EntityId") {
            v
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CameraNative_getAttachedCamera] [ERROR] Unable to find EntityId class"
            );
            return std::ptr::null_mut();
        };
        let entity_id = if let Ok(v) = env.new_object(
            entity_id,
            "(J)V",
            &[JValue::Long(entity.to_bits().get() as i64)],
        ) {
            v
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CameraNative_getAttachedCamera] [ERROR] Unable to create new entity_id object"
            );
            return std::ptr::null_mut();
        };

        let label = if let Ok(v) = env.new_string(cam.label.as_str()) {
            v
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CameraNative_getAttachedCamera] [ERROR] Unable to create new string for label"
            );
            return std::ptr::null_mut();
        };

        let eye = if let Ok(v) = create_vector3(&mut env, cam.eye.x, cam.eye.y, cam.eye.z) {
            v
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CameraNative_getAttachedCamera] [ERROR] Unable to create vector3 for eye"
            );
            return std::ptr::null_mut();
        };

        let target = if let Ok(v) =
            create_vector3(&mut env, cam.target.x, cam.target.y, cam.target.z)
        {
            v
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CameraNative_getAttachedCamera] [ERROR] Unable to create vector3 for target"
            );
            return std::ptr::null_mut();
        };

        let up = if let Ok(v) = create_vector3(&mut env, cam.up.x, cam.up.y, cam.up.z) {
            v
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CameraNative_getAttachedCamera] [ERROR] Unable to create vector3 for up"
            );
            return std::ptr::null_mut();
        };

        let class = if let Ok(v) = env.find_class("com/dropbear/Camera") {
            v
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CameraNative_getAttachedCamera] [ERROR] Unable to locate camera class"
            );
            return std::ptr::null_mut();
        };

        let camera_obj = if let Ok(v) = env.new_object(
            class,
            "(Ljava/lang/String;Lcom/dropbear/EntityId;Lcom/dropbear/math/Vector3;Lcom/dropbear/math/Vector3;Lcom/dropbear/math/Vector3;DDDDDDDD)V",
            &[
                JValue::Object(&label),
                JValue::Object(&entity_id),
                JValue::Object(&eye),
                JValue::Object(&target),
                JValue::Object(&up),
                JValue::Double(cam.aspect),
                JValue::Double(cam.settings.fov_y),
                JValue::Double(cam.znear),
                JValue::Double(cam.zfar),
                JValue::Double(cam.yaw),
                JValue::Double(cam.pitch),
                JValue::Double(cam.settings.speed),
                JValue::Double(cam.settings.sensitivity),
            ],
        ) {
            v
        } else {
            eprintln!("[Java_com_dropbear_ffi_components_CameraNative_getAttachedCamera] [ERROR] Unable to create the camera object");
            return std::ptr::null_mut();
        };

        return camera_obj.as_raw();
    }

    std::ptr::null_mut()
}


/**
 * Class:     `com_dropbear_ffi_components_CameraNative`
 *
 * Method:    `setCamera`
 *
 * Signature: `(JLcom/dropbear/Camera;)V`
 *
 * `JNIEXPORT void JNICALL Java_com_dropbear_ffi_components_CameraNative_setCamera
 * (JNIEnv *, jclass, jlong, jobject);`
*/
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_components_CameraNative_setCamera(
    mut env: JNIEnv,
    _class: JClass,
    world_handle: jlong,
    camera_obj: JObject,
) {
    let world = world_handle as *mut World;
    if world.is_null() {
        eprintln!("[Java_com_dropbear_ffi_components_CameraNative_setCamera] [ERROR] World pointer is null");
        return;
    }

    let world = unsafe { &mut *world };

    let entity_id_obj = if let Ok(v) =
        env.call_method(&camera_obj, "getId", "()Lcom/dropbear/EntityId;", &[])
    {
        if let Ok(obj) = v.l() {
            obj
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CameraNative_setCamera] [ERROR] Unable to extract EntityId object"
            );
            return;
        }
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CameraNative_setCamera] [ERROR] Unable to get EntityId from camera"
        );
        return;
    };

    let entity_id = if let Ok(v) = env.call_method(&entity_id_obj, "getId", "()J", &[]) {
        if let Ok(id) = v.j() {
            id as u32
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CameraNative_setCamera] [ERROR] Unable to extract entity id value"
            );
            return;
        }
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CameraNative_setCamera] [ERROR] Unable to call getId on EntityId"
        );
        return;
    };

    let entity = unsafe { world.find_entity_from_id(entity_id) };

    let eye_obj = if let Ok(v) =
        env.call_method(&camera_obj, "getEye", "()Lcom/dropbear/math/Vector3;", &[])
    {
        if let Ok(obj) = v.l() {
            obj
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CameraNative_setCamera] [ERROR] Unable to extract eye vector"
            );
            return;
        }
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CameraNative_setCamera] [ERROR] Unable to get eye from camera"
        );
        return;
    };

    let eye = if let Some(v) = extract_vector3(&mut env, &eye_obj) {
        v
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CameraNative_setCamera] [ERROR] Unable to extract eye vector values"
        );
        return;
    };

    let target_obj = if let Ok(v) = env.call_method(
        &camera_obj,
        "getTarget",
        "()Lcom/dropbear/math/Vector3;",
        &[],
    ) {
        if let Ok(obj) = v.l() {
            obj
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CameraNative_setCamera] [ERROR] Unable to extract target vector"
            );
            return;
        }
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CameraNative_setCamera] [ERROR] Unable to get target from camera"
        );
        return;
    };

    let target = if let Some(v) = extract_vector3(&mut env, &target_obj) {
        v
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CameraNative_setCamera] [ERROR] Unable to extract target vector values"
        );
        return;
    };

    let up_obj = if let Ok(v) =
        env.call_method(&camera_obj, "getUp", "()Lcom/dropbear/math/Vector3;", &[])
    {
        if let Ok(obj) = v.l() {
            obj
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CameraNative_setCamera] [ERROR] Unable to extract up vector"
            );
            return;
        }
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CameraNative_setCamera] [ERROR] Unable to get up from camera"
        );
        return;
    };

    let up = if let Some(v) = extract_vector3(&mut env, &up_obj) {
        v
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CameraNative_setCamera] [ERROR] Unable to extract up vector values"
        );
        return;
    };

    let fov_y = if let Ok(v) = env.call_method(&camera_obj, "getFov_y", "()D", &[]) {
        if let Ok(d) = v.d() {
            d
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CameraNative_setCamera] [ERROR] Unable to extract fov_y"
            );
            return;
        }
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CameraNative_setCamera] [ERROR] Unable to get fov_y from camera"
        );
        return;
    };

    let znear = if let Ok(v) = env.call_method(&camera_obj, "getZnear", "()D", &[]) {
        if let Ok(d) = v.d() {
            d
        } else {
            return;
        }
    } else {
        return;
    };

    let zfar = if let Ok(v) = env.call_method(&camera_obj, "getZfar", "()D", &[]) {
        if let Ok(d) = v.d() {
            d
        } else {
            return;
        }
    } else {
        return;
    };

    let yaw = if let Ok(v) = env.call_method(&camera_obj, "getYaw", "()D", &[]) {
        if let Ok(d) = v.d() {
            d
        } else {
            return;
        }
    } else {
        return;
    };

    let pitch = if let Ok(v) = env.call_method(&camera_obj, "getPitch", "()D", &[]) {
        if let Ok(d) = v.d() {
            d
        } else {
            return;
        }
    } else {
        return;
    };

    let speed = if let Ok(v) = env.call_method(&camera_obj, "getSpeed", "()D", &[]) {
        if let Ok(d) = v.d() {
            d
        } else {
            return;
        }
    } else {
        return;
    };

    let sensitivity = if let Ok(v) = env.call_method(&camera_obj, "getSensitivity", "()D", &[]) {
        if let Ok(d) = v.d() {
            d
        } else {
            return;
        }
    } else {
        return;
    };

    if let Ok(mut q) = world.query_one::<&mut Camera>(entity) {
        if let Some(cam) = q.get() {
            cam.eye = eye.as_dvec3();
            cam.target = target.as_dvec3();
            cam.up = up.as_dvec3();
            cam.settings.fov_y = fov_y;
            cam.znear = znear;
            cam.zfar = zfar;
            cam.yaw = yaw;
            cam.pitch = pitch;
            cam.settings.speed = speed;
            cam.settings.sensitivity = sensitivity;
        } else {
            eprintln!(
                "[Java_com_dropbear_ffi_components_CameraNative_setCamera] [ERROR] Entity does not have a Camera component"
            );
        }
    } else {
        eprintln!(
            "[Java_com_dropbear_ffi_components_CameraNative_setCamera] [ERROR] Unable to query camera component"
        );
    }
}
