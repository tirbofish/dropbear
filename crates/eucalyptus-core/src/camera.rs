//! Additional information and context for cameras from the [`dropbear_engine::camera`]
use crate::states::Camera3D;
use crate::traits::SerializableComponent;
use dropbear_engine::camera::{Camera, CameraBuilder, CameraSettings};
use dropbear_macro::SerializableComponent;
use glam::DVec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, SerializableComponent)]
pub struct CameraComponent {
    pub settings: CameraSettings,
    pub camera_type: CameraType,
    pub starting_camera: bool,
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl CameraComponent {
    pub fn new() -> Self {
        Self {
            settings: CameraSettings::default(),
            camera_type: CameraType::Normal,
            starting_camera: false,
        }
    }

    pub fn update(&mut self, camera: &mut Camera) {
        camera.settings = self.settings;
    }
}

impl From<Camera3D> for CameraBuilder {
    fn from(value: Camera3D) -> Self {
        let forward = value.transform.rotation * DVec3::Z;
        let up = if matches!(value.camera_type, CameraType::Debug | CameraType::Normal) {
            DVec3::Y
        } else {
            value.transform.rotation * DVec3::Y
        };

        Self {
            eye: value.transform.position,
            target: value.transform.position + forward,
            up,
            aspect: value.aspect,
            znear: value.near as f64,
            zfar: value.far as f64,
            settings: CameraSettings {
                speed: value.speed as f64,
                sensitivity: value.sensitivity as f64,
                fov_y: value.fov as f64,
            },
        }
    }
}

impl From<Camera3D> for CameraComponent {
    fn from(value: Camera3D) -> Self {
        let settings = CameraSettings::new(
            value.speed as f64,
            value.sensitivity as f64,
            value.fov as f64,
        );
        Self {
            settings,
            camera_type: value.camera_type,
            starting_camera: value.starting_camera,
        }
    }
}

pub struct PlayerCamera;

impl PlayerCamera {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> CameraComponent {
        CameraComponent {
            camera_type: CameraType::Player,
            ..CameraComponent::new()
        }
    }
}

pub struct DebugCamera;

impl DebugCamera {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> CameraComponent {
        CameraComponent {
            camera_type: CameraType::Debug,
            ..CameraComponent::new()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CameraType {
    Normal,
    Debug,
    Player,
}

impl Default for CameraType {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone)]
pub enum CameraAction {
    SetPlayerTarget { entity: hecs::Entity, offset: DVec3 },
    ClearPlayerTarget,
    SetCurrentPositionAsOffset(hecs::Entity),
}

pub mod shared {
    use dropbear_engine::camera::Camera;

    pub fn camera_exists_for_entity(world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&Camera>(entity).is_ok()
    }
}

pub mod jni {
    #![allow(non_snake_case)]

    use glam::DVec3;
    use jni::JNIEnv;
    use jni::objects::{JClass, JObject};
    use jni::sys::{jboolean, jdouble, jlong, jobject};
    use dropbear_engine::camera::Camera;
    use crate::convert_jlong_to_entity;
    use crate::scripting::jni::utils::{FromJObject, ToJObject};
    use crate::types::NVector3;

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_CameraNative_cameraExistsForEntity(
        _env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
    ) -> jboolean {
        let world = crate::convert_ptr!(world_ptr => hecs::World);
        let entity = convert_jlong_to_entity!(entity_id);
        if world.get::<&Camera>(entity).is_ok() {
            true.into()
        } else {
            false.into()
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_CameraNative_getCameraEye(
        mut env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
    ) -> jobject {
        let world = crate::convert_ptr!(world_ptr => hecs::World);
        let entity = convert_jlong_to_entity!(entity_id);
        if let Ok(camera) = world.get::<&Camera>(entity) {
            let eye: NVector3 = NVector3::from(camera.eye);
            return match eye.to_jobject(&mut env) {
                Ok(val) => val.into_raw(),
                Err(_) => std::ptr::null_mut()
            };
        } else {
            let _ = env.throw_new("java/lang/IllegalArgumentException", format!("Entity {} missing Camera", entity_id));
            std::ptr::null_mut()
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_CameraNative_setCameraEye(
        mut env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
        eye_obj: JObject,
    ) {
        let world = crate::convert_ptr!(world_ptr => hecs::World);
        let entity = convert_jlong_to_entity!(entity_id);
        let new_eye = match NVector3::from_jobject(&mut env, &eye_obj) {
            Ok(v) => v,
            Err(e) => {
                let _ = env.throw_new("java/lang/IllegalArgumentException", format!("Invalid Vector3d: {:?}", e));
                return;
            }
        };
        if let Ok(mut camera) = world.get::<&mut Camera>(entity) {
            camera.eye = DVec3::from(new_eye);
        } else {
            let _ = env.throw_new("java/lang/IllegalArgumentException", "Entity missing Camera component");
        }
    }

    // TARGET
    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_CameraNative_getCameraTarget(
        mut env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
    ) -> jobject {
        let world = crate::convert_ptr!(world_ptr => hecs::World);
        let entity = convert_jlong_to_entity!(entity_id);
        if let Ok(camera) = world.get::<&Camera>(entity) {
            let target: NVector3 = NVector3::from(camera.target);
            return match target.to_jobject(&mut env) {
                Ok(val) => val.into_raw(),
                Err(_) => std::ptr::null_mut()
            };
        } else {
            let _ = env.throw_new("java/lang/IllegalArgumentException", "Entity missing Camera component");
            std::ptr::null_mut()
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_CameraNative_setCameraTarget(
        mut env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
        target_obj: JObject,
    ) {
        let world = crate::convert_ptr!(world_ptr => hecs::World);
        let entity = convert_jlong_to_entity!(entity_id);
        let new_target = match NVector3::from_jobject(&mut env, &target_obj) {
            Ok(v) => v,
            Err(_) => return,
        };
        if let Ok(mut camera) = world.get::<&mut Camera>(entity) {
            camera.target = DVec3::from(new_target);
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_CameraNative_getCameraUp(
        mut env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
    ) -> jobject {
        let world = crate::convert_ptr!(world_ptr => hecs::World);
        let entity = convert_jlong_to_entity!(entity_id);
        if let Ok(camera) = world.get::<&Camera>(entity) {
            let up = NVector3::from(camera.up);
            return match up.to_jobject(&mut env) {
                Ok(val) => val.into_raw(),
                Err(_) => std::ptr::null_mut()
            };
        } else {
            let _ = env.throw_new("java/lang/IllegalArgumentException", "Entity missing Camera component");
            std::ptr::null_mut()
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_CameraNative_setCameraUp(
        mut env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
        up_obj: JObject,
    ) {
        let world = crate::convert_ptr!(world_ptr => hecs::World);
        let entity = convert_jlong_to_entity!(entity_id);
        let new_up = match NVector3::from_jobject(&mut env, &up_obj) {
            Ok(v) => v,
            Err(_) => return,
        };
        if let Ok(mut camera) = world.get::<&mut Camera>(entity) {
            camera.up = DVec3::from(new_up);
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_CameraNative_getCameraAspect(
        _env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
    ) -> jdouble {
        let world = crate::convert_ptr!(world_ptr => hecs::World);
        let entity = convert_jlong_to_entity!(entity_id);
        if let Ok(camera) = world.get::<&Camera>(entity) {
            camera.aspect
        } else {
            0.0
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_CameraNative_getCameraFovY(
        _env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
    ) -> jdouble {
        let world = crate::convert_ptr!(world_ptr => hecs::World);
        let entity = convert_jlong_to_entity!(entity_id);
        if let Ok(camera) = world.get::<&Camera>(entity) {
            camera.settings.fov_y
        } else {
            0.0
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_CameraNative_setCameraFovY(
        _env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
        value: jdouble,
    ) {
        let world = crate::convert_ptr!(world_ptr => hecs::World);
        let entity = convert_jlong_to_entity!(entity_id);
        if let Ok(mut camera) = world.get::<&mut Camera>(entity) {
            camera.settings.fov_y = value;
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_CameraNative_getCameraZNear(
        _env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
    ) -> jdouble {
        let world = crate::convert_ptr!(world_ptr => hecs::World);
        let entity = convert_jlong_to_entity!(entity_id);
        if let Ok(camera) = world.get::<&Camera>(entity) {
            camera.znear
        } else {
            0.0
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_CameraNative_setCameraZNear(
        _env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
        value: jdouble,
    ) {
        let world = crate::convert_ptr!(world_ptr => hecs::World);
        let entity = convert_jlong_to_entity!(entity_id);
        if let Ok(mut camera) = world.get::<&mut Camera>(entity) {
            camera.znear = value;
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_CameraNative_getCameraZFar(
        _env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
    ) -> jdouble {
        let world = crate::convert_ptr!(world_ptr => hecs::World);
        let entity = convert_jlong_to_entity!(entity_id);
        if let Ok(camera) = world.get::<&Camera>(entity) {
            camera.zfar
        } else {
            0.0
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_CameraNative_setCameraZFar(
        _env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
        value: jdouble,
    ) {
        let world = crate::convert_ptr!(world_ptr => hecs::World);
        let entity = convert_jlong_to_entity!(entity_id);
        if let Ok(mut camera) = world.get::<&mut Camera>(entity) {
            camera.zfar = value;
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_CameraNative_getCameraYaw(
        _env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
    ) -> jdouble {
        let world = crate::convert_ptr!(world_ptr => hecs::World);
        let entity = convert_jlong_to_entity!(entity_id);
        if let Ok(camera) = world.get::<&Camera>(entity) {
            camera.yaw
        } else {
            0.0
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_CameraNative_setCameraYaw(
        _env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
        value: jdouble,
    ) {
        let world = crate::convert_ptr!(world_ptr => hecs::World);
        let entity = convert_jlong_to_entity!(entity_id);
        if let Ok(mut camera) = world.get::<&mut Camera>(entity) {
            camera.yaw = value;
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_CameraNative_getCameraPitch(
        _env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
    ) -> jdouble {
        let world = crate::convert_ptr!(world_ptr => hecs::World);
        let entity = convert_jlong_to_entity!(entity_id);
        if let Ok(camera) = world.get::<&Camera>(entity) {
            camera.pitch
        } else {
            0.0
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_CameraNative_setCameraPitch(
        _env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
        value: jdouble,
    ) {
        let world = crate::convert_ptr!(world_ptr => hecs::World);
        let entity = convert_jlong_to_entity!(entity_id);
        if let Ok(mut camera) = world.get::<&mut Camera>(entity) {
            camera.pitch = value;
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_CameraNative_getCameraSpeed(
        _env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
    ) -> jdouble {
        let world = crate::convert_ptr!(world_ptr => hecs::World);
        let entity = convert_jlong_to_entity!(entity_id);
        if let Ok(camera) = world.get::<&Camera>(entity) {
            camera.settings.speed
        } else {
            0.0
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_CameraNative_setCameraSpeed(
        _env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
        value: jdouble,
    ) {
        let world = crate::convert_ptr!(world_ptr => hecs::World);
        let entity = convert_jlong_to_entity!(entity_id);
        if let Ok(mut camera) = world.get::<&mut Camera>(entity) {
            camera.settings.speed = value;
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_CameraNative_getCameraSensitivity(
        _env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
    ) -> jdouble {
        let world = crate::convert_ptr!(world_ptr => hecs::World);
        let entity = convert_jlong_to_entity!(entity_id);
        if let Ok(camera) = world.get::<&Camera>(entity) {
            camera.settings.sensitivity
        } else {
            0.0
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_components_CameraNative_setCameraSensitivity(
        _env: JNIEnv,
        _class: JClass,
        world_ptr: jlong,
        entity_id: jlong,
        value: jdouble,
    ) {
        let world = crate::convert_ptr!(world_ptr => hecs::World);
        let entity = convert_jlong_to_entity!(entity_id);
        if let Ok(mut camera) = world.get::<&mut Camera>(entity) {
            camera.settings.sensitivity = value;
        }
    }
}

#[dropbear_macro::impl_c_api]
pub mod native {
    use crate::ptr::WorldPtr;
    use crate::convert_ptr;
    use crate::scripting::native::{DropbearNativeError};
    use hecs::Entity;
    use glam::DVec3;
    use dropbear_engine::camera::Camera;
    use crate::scripting::result::DropbearNativeResult;
    use crate::types::NVector3;

    pub fn dropbear_camera_exists_for_entity(
        world_ptr: WorldPtr,
        entity_id: u64,
    ) -> DropbearNativeResult<bool> {
        let world = convert_ptr!(world_ptr => hecs::World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if world.get::<&Camera>(entity).is_ok() {
            DropbearNativeResult::Ok(true)
        } else {
            DropbearNativeResult::Ok(false)
        }
    }

    pub fn dropbear_get_camera_eye(
        world_ptr: WorldPtr,
        entity_id: u64,
    ) -> DropbearNativeResult<NVector3> {
        let world = convert_ptr!(world_ptr => hecs::World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(camera) = world.get::<&Camera>(entity) {
            DropbearNativeResult::Ok(NVector3::from(camera.eye))
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_set_camera_eye(
        world_ptr: WorldPtr,
        entity_id: u64,
        value: NVector3,
    ) -> DropbearNativeResult<()> {
        let world = convert_ptr!(world_ptr => hecs::World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(mut camera) = world.get::<&mut Camera>(entity) {
            camera.eye = DVec3::from(value);
            DropbearNativeResult::Ok(())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_get_camera_target(
        world_ptr: WorldPtr,
        entity_id: u64,
    ) -> DropbearNativeResult<NVector3> {
        let world = convert_ptr!(world_ptr => hecs::World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(camera) = world.get::<&Camera>(entity) {
            DropbearNativeResult::Ok(NVector3::from(camera.target))
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_set_camera_target(
        world_ptr: WorldPtr,
        entity_id: u64,
        value: NVector3,
    ) -> DropbearNativeResult<()> {
        let world = convert_ptr!(world_ptr => hecs::World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(mut camera) = world.get::<&mut Camera>(entity) {
            camera.target = DVec3::from(value);
            DropbearNativeResult::Ok(())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_get_camera_up(
        world_ptr: WorldPtr,
        entity_id: u64,
    ) -> DropbearNativeResult<NVector3> {
        let world = convert_ptr!(world_ptr => hecs::World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(camera) = world.get::<&Camera>(entity) {
            DropbearNativeResult::Ok(NVector3::from(camera.up))
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_set_camera_up(
        world_ptr: WorldPtr,
        entity_id: u64,
        value: NVector3,
    ) -> DropbearNativeResult<()> {
        let world = convert_ptr!(world_ptr => hecs::World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(mut camera) = world.get::<&mut Camera>(entity) {
            camera.up = DVec3::from(value);
            Ok(())
        } else {
            Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_get_camera_aspect(
        world_ptr: WorldPtr,
        entity_id: u64,
    ) -> DropbearNativeResult<f64> {
        let world = convert_ptr!(world_ptr => hecs::World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(camera) = world.get::<&Camera>(entity) {
            DropbearNativeResult::Ok(camera.aspect)
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_get_camera_fov_y(
        world_ptr: WorldPtr,
        entity_id: u64,
    ) -> DropbearNativeResult<f64> {
        let world = convert_ptr!(world_ptr => hecs::World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(camera) = world.get::<&Camera>(entity) {
            DropbearNativeResult::Ok(camera.settings.fov_y)
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_set_camera_fov_y(
        world_ptr: WorldPtr,
        entity_id: u64,
        value: f64,
    ) -> DropbearNativeResult<()> {
        let world = convert_ptr!(world_ptr => hecs::World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(mut camera) = world.get::<&mut Camera>(entity) {
            camera.settings.fov_y = value;
            DropbearNativeResult::Ok(())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_get_camera_znear(
        world_ptr: WorldPtr,
        entity_id: u64,
    ) -> DropbearNativeResult<f64> {
        let world = convert_ptr!(world_ptr => hecs::World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(camera) = world.get::<&Camera>(entity) {
            DropbearNativeResult::Ok(camera.znear)
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_set_camera_znear(
        world_ptr: WorldPtr,
        entity_id: u64,
        value: f64,
    ) -> DropbearNativeResult<()> {
        let world = convert_ptr!(world_ptr => hecs::World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(mut camera) = world.get::<&mut Camera>(entity) {
            camera.znear = value;
            DropbearNativeResult::Ok(())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_get_camera_zfar(
        world_ptr: WorldPtr,
        entity_id: u64,
    ) -> DropbearNativeResult<f64> {
        let world = convert_ptr!(world_ptr => hecs::World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(camera) = world.get::<&Camera>(entity) {
            DropbearNativeResult::Ok(camera.zfar)
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_set_camera_zfar(
        world_ptr: WorldPtr,
        entity_id: u64,
        value: f64,
    ) -> DropbearNativeResult<()> {
        let world = convert_ptr!(world_ptr => hecs::World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(mut camera) = world.get::<&mut Camera>(entity) {
            camera.zfar = value;
            DropbearNativeResult::Ok(())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_get_camera_yaw(
        world_ptr: WorldPtr,
        entity_id: u64,
    ) -> DropbearNativeResult<f64> {
        let world = convert_ptr!(world_ptr => hecs::World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(camera) = world.get::<&Camera>(entity) {
            DropbearNativeResult::Ok(camera.yaw)
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_set_camera_yaw(
        world_ptr: WorldPtr,
        entity_id: u64,
        value: f64,
    ) -> DropbearNativeResult<()> {
        let world = convert_ptr!(world_ptr => hecs::World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(mut camera) = world.get::<&mut Camera>(entity) {
            camera.yaw = value;
            DropbearNativeResult::Ok(())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_get_camera_pitch(
        world_ptr: WorldPtr,
        entity_id: u64,
    ) -> DropbearNativeResult<f64> {
        let world = convert_ptr!(world_ptr => hecs::World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(camera) = world.get::<&Camera>(entity) {
            DropbearNativeResult::Ok(camera.pitch)
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_set_camera_pitch(
        world_ptr: WorldPtr,
        entity_id: u64,
        value: f64,
    ) -> DropbearNativeResult<()> {
        let world = convert_ptr!(world_ptr => hecs::World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(mut camera) = world.get::<&mut Camera>(entity) {
            camera.pitch = value;
            DropbearNativeResult::Ok(())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_get_camera_speed(
        world_ptr: WorldPtr,
        entity_id: u64,
    ) -> DropbearNativeResult<f64> {
        let world = convert_ptr!(world_ptr => hecs::World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(camera) = world.get::<&Camera>(entity) {
            DropbearNativeResult::Ok(camera.settings.speed)
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_set_camera_speed(
        world_ptr: WorldPtr,
        entity_id: u64,
        value: f64,
    ) -> DropbearNativeResult<()> {
        let world = convert_ptr!(world_ptr => hecs::World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(mut camera) = world.get::<&mut Camera>(entity) {
            camera.settings.speed = value;
            DropbearNativeResult::Ok(())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_get_camera_sensitivity(
        world_ptr: WorldPtr,
        entity_id: u64,
    ) -> DropbearNativeResult<f64> {
        let world = convert_ptr!(world_ptr => hecs::World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(camera) = world.get::<&Camera>(entity) {
            DropbearNativeResult::Ok(camera.settings.sensitivity)
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }

    pub fn dropbear_set_camera_sensitivity(
        world_ptr: WorldPtr,
        entity_id: u64,
        value: f64,
    ) -> DropbearNativeResult<()> {
        let world = convert_ptr!(world_ptr => hecs::World);
        let entity = Entity::from_bits(entity_id).ok_or(DropbearNativeError::InvalidEntity)?;

        if let Ok(mut camera) = world.get::<&mut Camera>(entity) {
            camera.settings.sensitivity = value;
            DropbearNativeResult::Ok(())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchComponent)
        }
    }
}
