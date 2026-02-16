pub mod lighting;
pub mod camera;
pub mod config;
pub mod hierarchy;
pub mod input;
pub mod logging;
pub mod ptr;
pub mod runtime;
pub mod scene;
pub mod scripting;
pub mod states;
pub mod utils;
pub mod command;
pub mod physics;
pub mod types;
pub mod properties;
pub mod mesh;
pub mod entity;
pub mod engine;
pub mod transform;
pub mod asset;
pub mod component;
pub mod animation;

pub use dropbear_macro as macros;

pub use egui;
pub use rapier3d;
use dropbear_engine::animation::AnimationComponent;
use dropbear_engine::camera::Camera;
use dropbear_engine::entity::{EntityTransform, MeshRenderer};
use dropbear_engine::lighting::Light;
use properties::CustomProperties;
use crate::camera::CameraComponent;
use crate::component::ComponentRegistry;
use crate::physics::collider::ColliderGroup;
use crate::physics::kcc::KCC;
use crate::physics::rigidbody::RigidBody;
use crate::states::{SerializableCamera, SerializedLight, Script, SerializedMeshRenderer};

/// The appdata directory for storing any information.
///
/// By default, most of its items are located in [`app_dirs2::AppDataType::UserData`].
pub const APP_INFO: app_dirs2::AppInfo = app_dirs2::AppInfo {
    name: "Eucalyptus",
    author: "tirbofish",
};

/// Registers all available and potential serializers and deserializers of an entity.
pub fn register_components(
    component_registry: &mut ComponentRegistry,
) {
    component_registry.register::<EntityTransform>();
    component_registry.register::<CustomProperties>();
    component_registry.register::<Light>();
    component_registry.register::<Script>();
    component_registry.register::<MeshRenderer>();
    component_registry.register::<Camera>();
    component_registry.register::<RigidBody>();
    component_registry.register::<ColliderGroup>();
    component_registry.register::<KCC>();
    component_registry.register::<AnimationComponent>();
}