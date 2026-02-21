//! Different states and objects that exist in the scene.
//!
//! It's really just a "throw everything in here, organise later".

use crate::camera::{CameraComponent, CameraType};
use crate::component::{
    Component, ComponentDescriptor, ComponentInitFuture, InspectableComponent, SerializedComponent,
};
use crate::config::{ProjectConfig, ResourceConfig, SourceConfig};
use crate::properties::Value;
use crate::scene::SceneConfig;
use dropbear_engine::camera::Camera;
use dropbear_engine::entity::Transform;
use dropbear_engine::graphics::SharedGraphicsContext;
use dropbear_engine::lighting::LightComponent;
use dropbear_engine::model::AlphaMode;
use dropbear_engine::texture::TextureWrapMode;
use dropbear_engine::utils::ResourceReference;
use egui::{CollapsingHeader, TextEdit, Ui};
use hecs::{Entity, World};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::Arc;

/// A global "singleton" that contains the configuration of a project.
pub static PROJECT: Lazy<RwLock<ProjectConfig>> =
    Lazy::new(|| RwLock::new(ProjectConfig::default()));

pub static RESOURCES: Lazy<RwLock<ResourceConfig>> =
    Lazy::new(|| RwLock::new(ResourceConfig::default()));

pub static SOURCE: Lazy<RwLock<SourceConfig>> = Lazy::new(|| RwLock::new(SourceConfig::default()));

pub static SCENES: Lazy<RwLock<Vec<SceneConfig>>> = Lazy::new(|| RwLock::new(Vec::new()));

/// Removes a scene with the provided name from the in-memory scene cache.
/// Returns `true` when a scene was removed and `false` when no matching scene existed.
pub fn unload_scene(scene_name: &str) -> bool {
    let mut scenes = SCENES.write();
    let initial_len = scenes.len();
    scenes.retain(|scene| scene.scene_name != scene_name);
    let removed = scenes.len() != initial_len;

    if removed {
        log::info!("Unloaded scene '{}' from memory", scene_name);
    } else {
        log::debug!("Scene '{}' was not loaded; nothing to unload", scene_name);
    }

    removed
}

/// Reads a scene configuration from disk based on the active project's path.
pub fn load_scene(scene_name: &str) -> anyhow::Result<SceneConfig> {
    let scene_path = {
        let project = PROJECT.read();
        if project.project_path.as_os_str().is_empty() {
            return Err(anyhow::anyhow!(
                "Project path is not set; cannot load scenes"
            ));
        }

        project
            .project_path
            .join("scenes")
            .join(format!("{}.eucs", scene_name))
    };

    let scene = SceneConfig::read_from(&scene_path)?;
    log::info!(
        "Loaded scene '{}' from {}",
        scene_name,
        scene_path.display()
    );
    Ok(scene)
}

/// Reloads a scene into the in-memory cache by unloading any existing copy first.
pub fn load_scene_into_memory(scene_name: &str) -> anyhow::Result<()> {
    unload_scene(scene_name);

    let scene = load_scene(scene_name)?;
    {
        let mut scenes = SCENES.write();
        scenes.insert(0, scene);
    }

    log::info!("Scene '{}' loaded into memory", scene_name);

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Node {
    File(File),
    Folder(Folder),
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub enum File {
    #[default]
    Unknown,
    ResourceFile {
        name: String,
        path: PathBuf,
        resource_type: ResourceType,
    },
    SourceFile {
        name: String,
        path: PathBuf,
    },
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Folder {
    pub name: String,
    pub path: PathBuf,
    pub nodes: Vec<Node>,
}

/// The type of resource
#[derive(Debug, Serialize, Deserialize, Clone, Hash)]
pub enum ResourceType {
    Unknown,
    Config,
    Script,
    Model,
    Thumbnail,
    Texture,
    Shader,
}

impl Display for ResourceType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let str = match self {
            ResourceType::Unknown => "unknown",
            ResourceType::Model => "model",
            ResourceType::Texture => "texture",
            ResourceType::Shader => "shaders",
            ResourceType::Thumbnail => "thumbnail",
            ResourceType::Script => "script",
            ResourceType::Config => "eucalyptus project config",
        };
        write!(f, "{}", str)
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Script {
    pub tags: Vec<String>,
}

#[typetag::serde]
impl SerializedComponent for Script {}

impl Component for Script {
    type SerializedForm = Self;
    type RequiredComponentTypes = (Self,);

    fn descriptor() -> ComponentDescriptor {
        ComponentDescriptor {
            fqtn: "eucalyptus_core::states::Script".to_string(),
            type_name: "Script".to_string(),
            category: Some("Logic".to_string()),
            description: Some("A script component that can be attached to entities.".to_string()),
        }
    }

    fn init<'a>(
        ser: &'a Self::SerializedForm,
        _graphics: Arc<SharedGraphicsContext>,
    ) -> ComponentInitFuture<'a, Self> {
        Box::pin(async move { Ok((ser.clone(),)) })
    }

    fn update_component(
        &mut self,
        _world: &World,
        _physics: &mut crate::physics::PhysicsState,
        _entity: Entity,
        _dt: f32,
        _graphics: Arc<SharedGraphicsContext>,
    ) {
    }

    fn save(&self, _world: &World, _entity: Entity) -> Box<dyn SerializedComponent> {
        Box::new(self.clone())
    }
}

impl InspectableComponent for Script {
    fn inspect(
        &mut self,
        _world: &World,
        _entity: Entity,
        ui: &mut Ui,
        _graphics: Arc<SharedGraphicsContext>,
    ) {
        CollapsingHeader::new("Scripting")
            .default_open(true)
            .show(ui, |ui| {
                CollapsingHeader::new("Tags")
                    .default_open(true)
                    .show(ui, |ui| {
                        let mut local_del: Option<usize> = None;
                        for (i, tag) in self.tags.iter_mut().enumerate() {
                            let current_width = ui.available_width();
                            ui.horizontal(|ui| {
                                ui.add_sized(
                                    [current_width * 70.0 / 100.0, 20.0],
                                    TextEdit::singleline(tag),
                                );
                                if ui.button("🗑️").clicked() {
                                    local_del = Some(i);
                                }
                            });
                        }
                        if let Some(i) = local_del {
                            self.tags.remove(i);
                        }
                        if ui.button("➕ Add").clicked() {
                            self.tags.push(String::new())
                        }
                    });
            });
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SerializableCamera {
    pub label: String,
    pub transform: Transform,
    pub camera_type: CameraType,

    pub aspect: f64,
    pub fov: f32,
    pub near: f32,
    pub far: f32,

    pub speed: f32,
    pub sensitivity: f32,

    pub starting_camera: bool,
}

impl Default for SerializableCamera {
    fn default() -> Self {
        let default = CameraComponent::new();
        Self {
            transform: Transform::default(),
            aspect: 16.0 / 9.0,
            fov: 45.0,
            near: 0.1,
            far: 100.0,
            label: String::new(),
            camera_type: CameraType::Normal,
            speed: default.settings.speed as f32,
            sensitivity: default.settings.sensitivity as f32,
            starting_camera: false,
        }
    }
}

impl SerializableCamera {
    pub fn from_ecs_camera(camera: &Camera, component: &CameraComponent) -> Self {
        let position = glam::DVec3::from_array(camera.eye.to_array());
        let target = glam::DVec3::from_array(camera.target.to_array());
        let up = glam::DVec3::from_array(camera.up.to_array());

        let rotation = if (target - position).length_squared() > 0.0001 {
            glam::DQuat::from_mat4(&glam::DMat4::look_at_lh(position, target, up)).inverse()
        } else {
            glam::DQuat::IDENTITY
        };

        let transform = Transform {
            position,
            rotation,
            scale: glam::DVec3::ONE,
        };

        Self {
            transform,
            label: camera.label.clone(),
            camera_type: component.camera_type,
            aspect: camera.aspect,
            fov: camera.settings.fov_y as f32,
            near: camera.znear as f32,
            far: camera.zfar as f32,
            speed: camera.settings.speed as f32,
            sensitivity: camera.settings.sensitivity as f32,
            starting_camera: component.starting_camera,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Property {
    pub id: u64,
    pub key: String,
    pub value: Value,
}

// A serializable configuration struct for the [Light] type
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SerializedLight {
    pub label: String,
    pub light_component: LightComponent,

    #[serde(skip)]
    pub entity_id: Option<hecs::Entity>,
}

impl Default for SerializedLight {
    fn default() -> Self {
        Self {
            label: "Default Light".to_string(),
            light_component: LightComponent::default(),
            entity_id: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum EditorTab {
    AssetViewer,       // bottom side,
    ResourceInspector, // left side,
    ModelEntityList,   // right side,
    Viewport,          // middle,
    ErrorConsole,
    Console,
    Plugin(usize),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PluginInfo {
    pub display_name: String,
}

/// An enum that describes the status of loading the world.
///
/// This is enum is used by [`SceneConfig::load_into_world`] heavily. This enum
/// is recommended to be used with an [`UnboundedSender`]
pub enum WorldLoadingStatus {
    Idle,
    LoadingEntity {
        index: usize,
        name: String,
        total: usize,
    },
    Completed,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct Label(String);

impl Default for Label {
    fn default() -> Self {
        Self(String::from("No Label"))
    }
}

impl Label {
    /// Creates a new label component from any type that can be converted into a [`String`].
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the underlying string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns a mutable reference to the underlying [`String`].
    pub fn as_mut_string(&mut self) -> &mut String {
        &mut self.0
    }

    /// Replaces the underlying value with the provided one.
    pub fn set(&mut self, value: impl Into<String>) {
        self.0 = value.into();
    }

    /// Consumes the label and returns the owned [`String`].
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Returns whether the underlying label is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn locate_entity(&self, world: &World) -> Option<hecs::Entity> {
        world
            .query::<(Entity, &Label)>()
            .iter()
            .find_map(|(e, l)| if l == self { Some(e.clone()) } else { None })
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Label {
    fn from(value: String) -> Self {
        Label::new(value)
    }
}

impl From<&str> for Label {
    fn from(value: &str) -> Self {
        Label::new(value)
    }
}

impl AsRef<str> for Label {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for Label {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Deref for Label {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl DerefMut for Label {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_string()
    }
}

/// A [MeshRenderer] that is serialized into a file to be stored as a value for config.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SerializedMeshRenderer {
    pub label: String,
    pub handle: ResourceReference,
    pub import_scale: Option<f32>,
    pub texture_override: HashMap<String, SerializedMaterialCustomisation>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SerializedMaterialCustomisation {
    pub label: String,
    pub diffuse_texture: Option<ResourceReference>,
    pub tint: [f32; 4],
    pub emissive_factor: [f32; 3],
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub alpha_mode: AlphaMode,
    pub alpha_cutoff: Option<f32>,
    pub double_sided: bool,
    pub occlusion_strength: f32,
    pub normal_scale: f32,
    pub uv_tiling: [f32; 2],
    pub texture_tag: Option<String>,
    pub wrap_mode: TextureWrapMode,
    pub emissive_texture: Option<ResourceReference>,
    pub normal_texture: Option<ResourceReference>,
    pub occlusion_texture: Option<ResourceReference>,
    pub metallic_roughness_texture: Option<ResourceReference>,
}
