use std::ffi::{c_char, CStr, CString};
use glam::DVec3;
use hecs::World;
use dropbear_engine::camera::Camera;
use crate::camera::{CameraComponent, CameraType};
use crate::ptr::WorldPtr;
use crate::scripting::native::DropbearNativeError;
use crate::scripting::native::exports::dropbear_common::{DropbearNativeReturn, Handle};
use crate::scripting::native::exports::dropbear_math::Vector3D;

/// A camera that represents the [`Camera`] type in Rust in a C struct form. 
#[repr(C)]
pub struct NativeCamera {
    pub label: *const c_char,
    pub entity_id: i64,
    pub eye: Vector3D,
    pub target: Vector3D,
    pub up: Vector3D,
    pub aspect: f64,
    pub fov_y: f64,
    pub znear: f64,
    pub zfar: f64,
    pub yaw: f64,
    pub pitch: f64,
    pub speed: f64,
    pub sensitivity: f64,
}

/// Fetches a camera from the world by label.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_camera(
    world_ptr: *const World,
    label: *const c_char,
    out_camera: *mut NativeCamera,
) -> DropbearNativeReturn {
    if world_ptr.is_null() || label.is_null() || out_camera.is_null() {
        eprintln!("[dropbear_get_camera] [ERROR] Null pointer received");
        return DropbearNativeError::NullPointer as i32;
    }

    let world = unsafe { &*world_ptr };

    let label_str = match unsafe { CStr::from_ptr(label) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            eprintln!("[dropbear_get_camera] [ERROR] Invalid UTF-8 in label");
            return DropbearNativeError::InvalidUTF8 as i32;
        }
    };

    if let Some((id, (cam, comp))) = world
        .query::<(&Camera, &CameraComponent)>()
        .iter()
        .find(|(_, (cam, _))| cam.label == label_str)
    {
        if matches!(comp.camera_type, CameraType::Debug) {
            eprintln!("[dropbear_get_camera] [WARN] Querying a CameraType::Debug is illegal");
            return DropbearNativeError::NoSuchEntity as i32;
        }

        let label_cstring = CString::new(cam.label.as_str()).unwrap();

        unsafe {
            (*out_camera).label = label_cstring.into_raw();
            (*out_camera).entity_id = id.id() as i64;

            (*out_camera).eye = Vector3D {
                x: cam.eye.x,
                y: cam.eye.y,
                z: cam.eye.z,
            };

            (*out_camera).target = Vector3D {
                x: cam.target.x,
                y: cam.target.y,
                z: cam.target.z,
            };

            (*out_camera).up = Vector3D {
                x: cam.up.x,
                y: cam.up.y,
                z: cam.up.z,
            };

            (*out_camera).aspect = cam.aspect;
            (*out_camera).fov_y = cam.settings.fov_y;
            (*out_camera).znear = cam.znear;
            (*out_camera).zfar = cam.zfar;
            (*out_camera).yaw = cam.yaw;
            (*out_camera).pitch = cam.pitch;
            (*out_camera).speed = cam.settings.speed;
            (*out_camera).sensitivity = cam.settings.sensitivity;
        }

        return DropbearNativeError::Success as i32;
    }

    eprintln!(
        "[dropbear_get_camera] [ERROR] Camera with label '{}' not found",
        label_str
    );
    DropbearNativeError::EntityNotFound as i32
}

/// Fetches the camera attached to an entity as a component. 
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_get_attached_camera(
    world_ptr: *const World,
    entity_handle: Handle,
    out_camera: *mut NativeCamera,
) -> i32 {
    if world_ptr.is_null() || out_camera.is_null() {
        eprintln!("[dropbear_get_attached_camera] [ERROR] Null pointer received");
        return DropbearNativeError::NullPointer as i32;
    }

    let world = unsafe { &*world_ptr };
    let entity = unsafe { world.find_entity_from_id(entity_handle as u32) };

    match world.query_one::<(&Camera, &CameraComponent)>(entity) {
        Ok(mut q) => {
            if let Some((cam, comp)) = q.get() {
                if matches!(comp.camera_type, CameraType::Debug) {
                    eprintln!(
                        "[dropbear_get_attached_camera] [WARN] Querying a CameraType::Debug is illegal"
                    );
                    return DropbearNativeError::NoSuchEntity as i32;
                }

                let label_cstring = CString::new(cam.label.as_str()).unwrap();

                unsafe {
                    (*out_camera).label = label_cstring.into_raw();
                    (*out_camera).entity_id = entity_handle;

                    (*out_camera).eye = Vector3D {
                        x: cam.eye.x,
                        y: cam.eye.y,
                        z: cam.eye.z,
                    };

                    (*out_camera).target = Vector3D {
                        x: cam.target.x,
                        y: cam.target.y,
                        z: cam.target.z,
                    };

                    (*out_camera).up = Vector3D {
                        x: cam.up.x,
                        y: cam.up.y,
                        z: cam.up.z,
                    };

                    (*out_camera).aspect = cam.aspect;
                    (*out_camera).fov_y = cam.settings.fov_y;
                    (*out_camera).znear = cam.znear;
                    (*out_camera).zfar = cam.zfar;
                    (*out_camera).yaw = cam.yaw;
                    (*out_camera).pitch = cam.pitch;
                    (*out_camera).speed = cam.settings.speed;
                    (*out_camera).sensitivity = cam.settings.sensitivity;
                }

                0
            } else {
                eprintln!("[dropbear_get_attached_camera] [ERROR] Entity has no Camera component");
                DropbearNativeError::NoSuchEntity as i32
            }
        }
        Err(_) => {
            eprintln!("[dropbear_get_attached_camera] [ERROR] Failed to query entity");
            DropbearNativeError::EntityNotFound as i32
        }
    }
}

/// Sets the camera attached to an entity as a component.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dropbear_set_camera(
    world_ptr: WorldPtr,
    camera: *const NativeCamera,
) -> i32 {
    if world_ptr.is_null() || camera.is_null() {
        eprintln!("[dropbear_set_camera] [ERROR] Null pointer received");
        return DropbearNativeError::NullPointer as i32;
    }

    let world = unsafe { &mut *(world_ptr) };
    let cam_data = unsafe { &*camera };

    let entity = unsafe { world.find_entity_from_id(cam_data.entity_id as u32) };

    match world.query_one_mut::<&mut Camera>(entity) {
        Ok(cam) => {
            cam.eye = DVec3::new(
                cam_data.eye.x,
                cam_data.eye.y,
                cam_data.eye.z,
            );

            cam.target = DVec3::new(
                cam_data.target.x,
                cam_data.target.y,
                cam_data.target.z,
            );

            cam.up = DVec3::new(
                cam_data.up.x,
                cam_data.up.y,
                cam_data.up.z,
            );

            cam.aspect = cam_data.aspect;
            cam.settings.fov_y = cam_data.fov_y;
            cam.znear = cam_data.znear;
            cam.zfar = cam_data.zfar;
            cam.yaw = cam_data.yaw;
            cam.pitch = cam_data.pitch;
            cam.settings.speed = cam_data.speed;
            cam.settings.sensitivity = cam_data.sensitivity;

            DropbearNativeError::Success as i32
        }
        Err(_) => {
            eprintln!("[dropbear_set_camera] [ERROR] Unable to query camera component");
            DropbearNativeError::EntityNotFound as i32
        }
    }
}