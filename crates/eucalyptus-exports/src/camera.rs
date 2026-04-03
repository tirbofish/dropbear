use eucalyptus_core::ptr::WorldPtr;
use eucalyptus_core::scripting::result::DropbearNativeResult;
use eucalyptus_core::types::NVector3;
use dropbear_engine::camera::Camera;

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "cameraExistsForEntity"),
    c
)]
fn exists_for_entity(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<bool> {
    Ok(world.get::<&Camera>(entity).is_ok())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "getCameraEye"),
    c
)]
fn get_eye(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
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
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    eye: &NVector3,
) -> DropbearNativeResult<()> {
    match world.get::<&mut Camera>(entity) {
        Ok(mut camera) => {
            camera.eye = (*eye).into();
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "getCameraTarget"),
    c
)]
fn get_target(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
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
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    target: &NVector3,
) -> DropbearNativeResult<()> {
    match world.get::<&mut Camera>(entity) {
        Ok(mut camera) => {
            camera.target = target.into();
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "getCameraUp"),
    c
)]
fn get_up(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
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
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    up: &NVector3,
) -> DropbearNativeResult<()> {
    match world.get::<&mut Camera>(entity) {
        Ok(mut camera) => {
            camera.up = up.into();
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "getCameraAspect"),
    c
)]
fn get_aspect(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
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
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
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
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    fovy: f64,
) -> DropbearNativeResult<()> {
    match world.get::<&mut Camera>(entity) {
        Ok(mut camera) => {
            camera.settings.fov_y = fovy.into();
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "getCameraZNear"),
    c
)]
fn get_znear(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
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
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    znear: f64,
) -> DropbearNativeResult<()> {
    match world.get::<&mut Camera>(entity) {
        Ok(mut camera) => {
            camera.znear = znear.into();
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "getCameraZFar"),
    c
)]
fn get_zfar(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
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
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    zfar: f64,
) -> DropbearNativeResult<()> {
    match world.get::<&mut Camera>(entity) {
        Ok(mut camera) => {
            camera.zfar = zfar.into();
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "getCameraYaw"),
    c
)]
fn get_yaw(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
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
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    yaw: f64,
) -> DropbearNativeResult<()> {
    match world.get::<&mut Camera>(entity) {
        Ok(mut camera) => {
            camera.yaw = yaw.into();
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "getCameraPitch"),
    c
)]
fn get_pitch(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
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
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    pitch: f64,
) -> DropbearNativeResult<()> {
    match world.get::<&mut Camera>(entity) {
        Ok(mut camera) => {
            camera.pitch = pitch.into();
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "getCameraSpeed"),
    c
)]
fn get_speed(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
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
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    speed: f64,
) -> DropbearNativeResult<()> {
    match world.get::<&mut Camera>(entity) {
        Ok(mut camera) => {
            camera.settings.speed = speed.into();
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.CameraNative", func = "getCameraSensitivity"),
    c
)]
fn get_sensitivity(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
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
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    sensitivity: f64,
) -> DropbearNativeResult<()> {
    match world.get::<&mut Camera>(entity) {
        Ok(mut camera) => {
            camera.settings.sensitivity = sensitivity.into();
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
