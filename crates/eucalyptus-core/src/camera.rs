//! Additional information and context for cameras from the [`dropbear_engine::camera`]
use crate::states::Camera3D;
use dropbear_engine::camera::{Camera, CameraBuilder, CameraSettings};
use dropbear_traits::{ComponentInitContext, ComponentInitFuture, InsertBundle, SerializableComponent};
use glam::DVec3;
use serde::{Deserialize, Serialize};
use std::any::Any;
use crate::ptr::WorldPtr;
use crate::scripting::result::DropbearNativeResult;
use crate::types::NVector3;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[typetag::serde]
impl SerializableComponent for CameraComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn SerializableComponent> {
        Box::new(self.clone())
    }

    fn init(&self, _ctx: ComponentInitContext) -> ComponentInitFuture {
        let value = self.clone();
        Box::pin(async move {
            let insert: Box<dyn dropbear_traits::ComponentInsert> =
                Box::new(InsertBundle((value,)));
            Ok(insert)
        })
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
    pub fn camera_exists_for_entity(world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&dropbear_engine::camera::Camera>(entity).is_ok()
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "cameraExistsForEntity"),
    c
)]
fn exists_for_entity(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropebear_macro::entity]
    entity: hecs::Entity,
) -> DropbearNativeResult<bool> {
    Ok(shared::camera_exists_for_entity(world, entity))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "getCameraEye"),
    c
)]
fn get_eye(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropebear_macro::entity]
    entity: hecs::Entity,
) -> DropbearNativeResult<NVector3> {
    match world.get::<&Camera>(entity) {
        Ok(camera) => Ok(camera.eye.into()),
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "setCameraEye"),
    c
)]
fn set_eye(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropebear_macro::entity]
    entity: hecs::Entity,
    eye: &NVector3,
) -> DropbearNativeResult<()> {
    match world.get::<&mut Camera>(entity) {
        Ok(mut camera) => {
            camera.eye = (*eye).into();
            Ok(())
        },
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "getCameraTarget"),
    c
)]
fn get_target(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropebear_macro::entity]
    entity: hecs::Entity,
) -> DropbearNativeResult<NVector3> {
    match world.get::<&Camera>(entity) {
        Ok(camera) => Ok(camera.target.into()),
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "setCameraTarget"),
    c
)]
fn set_target(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropebear_macro::entity]
    entity: hecs::Entity,
    target: &NVector3,
) -> DropbearNativeResult<()> {
    match world.get::<&mut Camera>(entity) {
        Ok(mut camera) => {
            camera.target = target.into();
            Ok(())
        },
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "getCameraUp"),
    c
)]
fn get_up(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropebear_macro::entity]
    entity: hecs::Entity,
) -> DropbearNativeResult<NVector3> {
    match world.get::<&Camera>(entity) {
        Ok(camera) => Ok(camera.up.into()),
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "setCameraUp"),
    c
)]
fn set_up(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropebear_macro::entity]
    entity: hecs::Entity,
    up: &NVector3,
) -> DropbearNativeResult<()> {
    match world.get::<&mut Camera>(entity) {
        Ok(mut camera) => {
            camera.up = up.into();
            Ok(())
        },
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "getCameraAspect"),
    c
)]
fn get_aspect(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropebear_macro::entity]
    entity: hecs::Entity,
) -> DropbearNativeResult<f64> {
    match world.get::<&Camera>(entity) {
        Ok(camera) => Ok(camera.aspect.into()),
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "getCameraFovY"),
    c
)]
fn get_fovy(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropebear_macro::entity]
    entity: hecs::Entity,
) -> DropbearNativeResult<f64> {
    match world.get::<&Camera>(entity) {
        Ok(camera) => Ok(camera.settings.fov_y.into()),
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "setCameraFovY"),
    c
)]
fn set_fovy(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropebear_macro::entity]
    entity: hecs::Entity,
    fovy: f64,
) -> DropbearNativeResult<()> {
    match world.get::<&mut Camera>(entity) {
        Ok(mut camera) => {
            camera.settings.fov_y = fovy.into();
            Ok(())
        },
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "getCameraZNear"),
    c
)]
fn get_znear(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropebear_macro::entity]
    entity: hecs::Entity,
) -> DropbearNativeResult<f64> {
    match world.get::<&Camera>(entity) {
        Ok(camera) => Ok(camera.znear.into()),
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "setCameraZNear"),
    c
)]
fn set_znear(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropebear_macro::entity]
    entity: hecs::Entity,
    znear: f64,
) -> DropbearNativeResult<()> {
    match world.get::<&mut Camera>(entity) {
        Ok(mut camera) => {
            camera.znear = znear.into();
            Ok(())
        },
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "getCameraZFar"),
    c
)]
fn get_zfar(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropebear_macro::entity]
    entity: hecs::Entity,
) -> DropbearNativeResult<f64> {
    match world.get::<&Camera>(entity) {
        Ok(camera) => Ok(camera.zfar.into()),
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "setCameraZFar"),
    c
)]
fn set_zfar(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropebear_macro::entity]
    entity: hecs::Entity,
    zfar: f64,
) -> DropbearNativeResult<()> {
    match world.get::<&mut Camera>(entity) {
        Ok(mut camera) => {
            camera.zfar = zfar.into();
            Ok(())
        },
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "getCameraYaw"),
    c
)]
fn get_yaw(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropebear_macro::entity]
    entity: hecs::Entity,
) -> DropbearNativeResult<f64> {
    match world.get::<&Camera>(entity) {
        Ok(camera) => Ok(camera.yaw.into()),
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "setCameraYaw"),
    c
)]
fn set_yaw(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropebear_macro::entity]
    entity: hecs::Entity,
    yaw: f64,
) -> DropbearNativeResult<()> {
    match world.get::<&mut Camera>(entity) {
        Ok(mut camera) => {
            camera.yaw = yaw.into();
            Ok(())
        },
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "getCameraPitch"),
    c
)]
fn get_pitch(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropebear_macro::entity]
    entity: hecs::Entity,
) -> DropbearNativeResult<f64> {
    match world.get::<&Camera>(entity) {
        Ok(camera) => Ok(camera.pitch.into()),
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "setCameraPitch"),
    c
)]
fn set_pitch(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropebear_macro::entity]
    entity: hecs::Entity,
    pitch: f64,
) -> DropbearNativeResult<()> {
    match world.get::<&mut Camera>(entity) {
        Ok(mut camera) => {
            camera.pitch = pitch.into();
            Ok(())
        },
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "getCameraSpeed"),
    c
)]
fn get_speed(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropebear_macro::entity]
    entity: hecs::Entity,
) -> DropbearNativeResult<f64> {
    match world.get::<&Camera>(entity) {
        Ok(camera) => Ok(camera.settings.speed.into()),
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "setCameraSpeed"),
    c
)]
fn set_speed(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropebear_macro::entity]
    entity: hecs::Entity,
    speed: f64,
) -> DropbearNativeResult<()> {
    match world.get::<&mut Camera>(entity) {
        Ok(mut camera) => {
            camera.settings.speed = speed.into();
            Ok(())
        },
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "getCameraSensitivity"),
    c
)]
fn get_sensitivity(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropebear_macro::entity]
    entity: hecs::Entity,
) -> DropbearNativeResult<f64> {
    match world.get::<&Camera>(entity) {
        Ok(camera) => Ok(camera.settings.sensitivity.into()),
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "setCameraSensitivity"),
    c
)]
fn set_sensitivity(
    #[dropbear_macro::define(WorldPtr)]
    world: &hecs::World,
    #[dropebear_macro::entity]
    entity: hecs::Entity,
    sensitivity: f64,
) -> DropbearNativeResult<()> {
    match world.get::<&mut Camera>(entity) {
        Ok(mut camera) => {
            camera.settings.sensitivity = sensitivity.into();
            Ok(())
        },
        Err(e) => Err(e.into()),
    }
}