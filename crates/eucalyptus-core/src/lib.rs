pub mod lighting;
pub mod camera;
pub mod component;
pub mod config;
pub mod hierarchy;
pub mod input;
pub mod logging;
pub mod ptr;
pub mod result;
pub mod runtime;
pub mod scene;
pub mod scripting;
pub mod spawn;
pub mod states;
pub mod utils;
pub mod command;
pub mod physics;
pub mod asset;
pub mod types;
pub mod properties;
pub mod mesh;
pub mod entity;
pub mod engine;
pub mod transform;
pub mod ui;

pub use dropbear_macro as macros;
pub use dropbear_traits as traits;

pub use egui;
pub use rapier3d;
use dropbear_engine::animation::AnimationComponent;
use dropbear_engine::camera::Camera;
use dropbear_engine::entity::{EntityTransform, MeshRenderer};
use dropbear_traits::registry::ComponentRegistry;
use properties::CustomProperties;
use crate::camera::CameraComponent;
use crate::physics::collider::ColliderGroup;
use crate::physics::kcc::KCC;
use crate::physics::rigidbody::RigidBody;
use crate::states::{Camera3D, Light, Script, SerializedMeshRenderer};
use crate::ui::UIComponent;

/// The appdata directory for storing any information.
///
/// By default, most of its items are located in [`app_dirs2::AppDataType::UserData`].
pub const APP_INFO: app_dirs2::AppInfo = app_dirs2::AppInfo {
    name: "Eucalyptus",
    author: "tirbofish",
};

#[unsafe(no_mangle)]
pub extern "C" fn get_rustc_version() -> *const u8 {
    let meta = rustc_version_runtime::version_meta();
    let meta_string = format!("{:?}", meta);
    Box::leak(meta_string.into_boxed_str()).as_ptr()
}

/// Registers all available and potential serializers and deserializers of an entity.
pub fn register_components(
    component_registry: &mut ComponentRegistry,
) {
    component_registry.register_with_default::<EntityTransform>();
    component_registry.register_with_default::<CustomProperties>();
    component_registry.register_with_default::<Light>();
    component_registry.register_with_default::<Script>();
    component_registry.register_with_default::<SerializedMeshRenderer>();
    component_registry.register_with_default::<Camera3D>();
    component_registry.register_with_default::<RigidBody>();
    component_registry.register_with_default::<ColliderGroup>();
    component_registry.register_with_default::<KCC>();
    component_registry.register_with_default::<UIComponent>();
    component_registry.register_with_default::<AnimationComponent>();

    component_registry.register_converter::<MeshRenderer, SerializedMeshRenderer, _>(
        |_, _, renderer| {
            Some(SerializedMeshRenderer::from_renderer(renderer))
        },
    );

    component_registry.register_converter::<CameraComponent, Camera3D, _>(
        |world, entity, component| {
            let Ok(camera) = world.get::<&Camera>(entity) else {
                log::debug!(
                            "Camera component without matching Camera found on entity {:?}",
                            entity
                        );
                return None;
            };

            Some(Camera3D::from_ecs_camera(&camera, component))
        },
    );

    // // register plugin defined structs
    // if let Err(e) = plugin_registry.load_plugins() {
    //     fatal!("Failed to load plugins: {}", e);
    //     return;
    // }
    //
    // for p in plugin_registry.list_plugins() {
    //     log::info!("Plugin {} has been loaded", p.display_name);
    // }
    //
    // log::info!("Total plugins added: {}", plugin_registry.plugins.len());
    //
    // for plugin in plugin_registry.list_plugins() {
    //     if let Some(p) = plugin_registry.get_mut(&plugin.display_name) {
    //         p.register_component(component_registry);
    //         log::info!(
    //                     "Components for plugin [{}] has been registered to component registry",
    //                     plugin.display_name
    //                 );
    //     }
    // }
}