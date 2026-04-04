pub mod animation;
pub mod billboard;
pub mod bundle;
pub mod camera;
pub mod command;
pub mod component;
pub mod config;
pub mod debug;
pub mod entity_status;
pub mod hierarchy;
pub mod input;
pub mod lighting;
pub mod logging;
pub mod mesh;
pub mod metadata;
pub mod physics;
pub mod plugin;
pub mod properties;
pub mod ptr;
pub mod resource;
pub mod runtime;
pub mod scene;
pub mod scripting;
pub mod ser;
pub mod states;
pub mod transform;
pub mod types;
pub mod ui;
pub mod utils;
pub mod uuid;
pub mod rendering;
pub mod asset;

pub use dropbear_macro as macros;
pub use dropbear_engine as engine;
pub use kino_ui as kino;

use crate::billboard::BillboardComponent;
use crate::component::ComponentRegistry;
use crate::entity_status::EntityStatus;
use crate::physics::collider::ColliderGroup;
use crate::physics::kcc::KCC;
use crate::physics::rigidbody::RigidBody;
use crate::scripting::types::KotlinComponents;
use crate::states::Script;
use crate::transform::OnRails;
use crate::ui::HUDComponent;
use dropbear_engine::animation::AnimationComponent;
use dropbear_engine::camera::Camera;
use dropbear_engine::entity::{EntityTransform, MeshRenderer};
use dropbear_engine::lighting::Light;
use properties::CustomProperties;


pub mod third_party {
    pub use rapier3d;
    pub use egui;
    pub use hecs;
}

/// The appdata directory for storing any information.
///
/// By default, most of its items are located in [`app_dirs2::AppDataType::UserData`].
pub const APP_INFO: app_dirs2::AppInfo = app_dirs2::AppInfo {
    name: "Eucalyptus",
    author: "tirbofish",
};

/// Registers all available and potential serializers and deserializers of an entity.
pub fn register_components(component_registry: &mut ComponentRegistry) {
    component_registry.register::<EntityTransform>();
    component_registry.register::<EntityStatus>();
    component_registry.register::<CustomProperties>();
    component_registry.register::<Light>();
    component_registry.register::<Script>();
    component_registry.register::<MeshRenderer>();
    component_registry.register::<Camera>();
    component_registry.register::<RigidBody>();
    component_registry.register::<ColliderGroup>();
    component_registry.register::<KCC>();
    component_registry.register::<AnimationComponent>();
    component_registry.register::<BillboardComponent>();
    component_registry.register::<HUDComponent>();
    component_registry.register::<OnRails>();
    component_registry.register::<KotlinComponents>();
}
