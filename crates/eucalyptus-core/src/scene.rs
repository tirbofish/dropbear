//! Deals with scene loading and scene metadata.

pub mod loading;
pub mod scripting;

use crate::camera::CameraComponent;
use crate::hierarchy::{Children, Parent, SceneHierarchy};
use crate::states::{SerializableCamera, Label, SerializedLight, Script, SerializedMeshRenderer, WorldLoadingStatus, PROJECT};
use dropbear_engine::camera::Camera;
use dropbear_engine::entity::{EntityTransform, MeshRenderer, Transform};
use dropbear_engine::graphics::SharedGraphicsContext;
use dropbear_engine::lighting::{Light, LightComponent};
use glam::{DQuat, DVec3};
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use crossbeam_channel::Sender;
use egui::Ui;
use hecs::{Entity, EntityBuilder};
use crate::component::{Component, ComponentRegistry, SerializedComponent};
use crate::physics::collider::ColliderGroup;
use crate::physics::kcc::KCC;
use crate::physics::PhysicsState;
use crate::physics::rigidbody::RigidBody;
use crate::properties::CustomProperties;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct SceneEntity {
    #[serde(default)]
    pub label: Label,

    #[serde(default)]
    pub components: Vec<Box<dyn SerializedComponent>>,

    #[serde(skip)]
    pub entity_id: Option<hecs::Entity>,
}

impl SceneEntity {
    pub fn from_world(
        world: &hecs::World,
        entity: hecs::Entity,
        registry: &ComponentRegistry,
    ) -> Option<Self> {
        let label = if let Ok(label) = world.query_one::<&Label>(entity).get()
        {
            label.clone()
        } else {
            return None;
        };

        let components = registry.extract_all_components(world, entity);

        Some(Self {
            label,
            components,
            entity_id: Some(entity),
        })
    }
}

/// The specific settings of a scene.
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct SceneSettings {
    /// Ensures a scene's assets are preloaded at the start of the game.
    /// 
    /// This is useful for situations where you might need a loading screen
    /// and want to make sure an image is loaded into memory.
    #[serde(default)]
    pub preloaded: bool,

    /// Toggles rendering of collider hitboxes / wireframes.
    #[serde(default)]
    pub show_hitboxes: bool,
}

impl SceneSettings {
    /// Creates a new [`SceneSettings`] config.
    pub fn new() -> Self {
        Self {
            preloaded: false,
            show_hitboxes: false,
        }
    }
}

/// Specifies the configuration of a scene, such as its entities, hierarchies and any settings that
/// may be necessary.
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct SceneConfig {
    #[serde(default)]
    pub scene_name: String,

    #[serde(default)]
    pub entities: Vec<SceneEntity>,

    #[serde(default)]
    pub hierarchy_map: SceneHierarchy,

    #[serde(default)]
    pub physics_state: PhysicsState,

    #[serde(default)]
    pub settings: SceneSettings,

    #[serde(skip)]
    pub path: PathBuf,
}

impl SceneConfig {
    /// Creates a new instance of the scene config
    pub fn new(scene_name: String, path: impl AsRef<Path>) -> Self {
        Self {
            scene_name,
            path: path.as_ref().to_path_buf(),
            entities: Vec::new(),
            hierarchy_map: SceneHierarchy::new(),
            physics_state: PhysicsState::new(),
            settings: SceneSettings::new(),
        }
    }

    /// Write the scene config to a .eucs file
    pub fn write_to(&self, project_path: impl AsRef<Path>) -> anyhow::Result<()> {
        let ron_str = ron::ser::to_string_pretty(&self, PrettyConfig::default())
            .map_err(|e| anyhow::anyhow!("RON serialization error: {}", e))?;

        let scenes_dir = project_path.as_ref().join("scenes");
        fs::create_dir_all(&scenes_dir)?;

        let config_path = scenes_dir.join(format!("{}.eucs", self.scene_name));
        fs::write(&config_path, ron_str).map_err(|e| anyhow::anyhow!(e.to_string()))?;
        Ok(())
    }

    /// Read a scene config from a .eucs file
    pub fn read_from(scene_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let ron_str = fs::read_to_string(scene_path.as_ref())?;
        let mut config: SceneConfig = ron::de::from_str(&ron_str)
            .map_err(|e| anyhow::anyhow!("RON deserialization error: {}", e))?;

        config.path = scene_path.as_ref().to_path_buf();
        Ok(config)
    }

    /// Loads a scene into a world.
    ///
    /// Typically used in conjunction with a [`crossbeam_channel::unbounded`]
    ///
    /// `is_play_mode` is used to specify if the viewport camera (debug camera) is to be used (`false`)
    /// or if the starting camera for the scene is too be used (`true`).
    pub async fn load_into_world(
        &mut self,
        world: &mut hecs::World,
        graphics: Arc<SharedGraphicsContext>,
        registry: &ComponentRegistry,
        progress_sender: Option<Sender<WorldLoadingStatus>>,
        is_play_mode: bool,
    ) -> anyhow::Result<hecs::Entity> {
        if let Some(ref s) = progress_sender {
            let _ = s.send(WorldLoadingStatus::Idle);
        }

        // clear world to make room for new entities
        log::debug!(
            "Loading scene [{}], clearing world with {} entities",
            self.scene_name,
            world.len()
        );
        world.clear();

        log::info!("World cleared, now has {} entities", world.len());

        let mut label_to_entity: HashMap<Label, hecs::Entity> = HashMap::new();

        // gather all entities
        let entity_configs: Vec<(usize, SceneEntity)> = {
            let cloned = self.entities.clone();
            cloned
                .into_par_iter()
                .enumerate()
                .map(|(i, e)| (i, e))
                .collect()
        };

        // fetch all entities
        for (index, entity_config) in entity_configs {
            let SceneEntity {
                label,
                components,
                entity_id: _,
            } = entity_config;

            let label_for_map = label.clone();
            let label_for_logs = label_for_map.to_string();

            log::debug!("Loading entity: {}", label_for_logs);

            let total = self.entities.len();

            if let Some(ref s) = progress_sender {
                let _ = s.send(WorldLoadingStatus::LoadingEntity {
                    index,
                    name: label_for_logs.clone(),
                    total,
                });
            }

            // all entities will ALWAYS have a label. if it doesnt, its not a valid entity
            let mut builder = EntityBuilder::new();
            builder.add(label_for_map.clone());

            for component in components.iter() {

                if component.as_any().downcast_ref::<Parent>().is_some() {
                    log::debug!(
                        "Skipping Parent component for '{}' - will be rebuilt from hierarchy_map",
                        label_for_logs
                    );
                    continue;
                }

                let Some(loader_future) =
                    registry.load_component(component.as_ref(), graphics.clone())
                else {
                    log::warn!(
                        "Skipping unregistered serialized component for '{}'",
                        label_for_logs
                    );
                    continue;
                };

                let applier = loader_future.await?;
                applier(&mut builder);
            }

            let entity = world.spawn(builder.build());

            self.register_physics_for_entity(world, entity);

            if let Some(previous) = label_to_entity.insert(label_for_map.clone(), entity) {
                log::warn!(
                    "Duplicate entity label '{}' detected; previous entity {:?} will be overwritten in hierarchy mapping",
                    label_for_logs,
                    previous
                );
            }

            log::debug!("Loaded entity '{}'", label_for_logs);
        }

        self.rebuild_hierarchy(world, &label_to_entity);
        self.ensure_default_light(world, graphics.clone(), progress_sender.as_ref()).await?;

        log::info!("Loaded {} entities from scene", self.entities.len());

        let camera_entity =
            self.select_active_camera(world, graphics, progress_sender.as_ref(), is_play_mode)?;
        Ok(camera_entity)
    }

    fn register_physics_for_entity(&mut self, world: &mut hecs::World, entity: hecs::Entity) {
        if let Ok((
            label,
            e_trans,
            rigid,
            col_group,
            kcc
        )) = world.query_one::<(
            &Label,
            &EntityTransform,
            Option<&mut RigidBody>,
            Option<&mut ColliderGroup>,
            Option<&mut KCC>
        )>(entity).get() {
            if let Some(body) = rigid {
                body.entity = label.clone();
                self.physics_state.register_rigidbody(body, e_trans.sync());
            }

            if let Some(group) = col_group {
                for collider in &mut group.colliders {
                    collider.entity = label.clone();
                    self.physics_state.register_collider(collider);
                }
            }

            if let Some(kcc) = kcc {
                kcc.entity = label.clone();
            }
        }
    }

    fn rebuild_hierarchy(
        &self,
        world: &mut hecs::World,
        label_to_entity: &HashMap<Label, hecs::Entity>,
    ) {
        let mut parent_children_map: HashMap<Label, Vec<Label>> = HashMap::new();

        for entity_label in label_to_entity.keys() {
            let children: Vec<Label> = self.hierarchy_map.get_children(entity_label).to_vec();
            if !children.is_empty() {
                parent_children_map.insert(entity_label.clone(), children);
            }
        }

        for (parent_label, child_labels) in parent_children_map {
            let Some(&parent_entity) = label_to_entity.get(&parent_label) else {
                log::warn!(
                    "Unable to resolve parent entity '{}' while rebuilding hierarchy",
                    parent_label
                );
                continue;
            };

            let mut resolved_children = Vec::new();
            for child_label in child_labels {
                if let Some(&child_entity) = label_to_entity.get(&child_label) {
                    resolved_children.push(child_entity);
                    if let Err(e) = world.insert_one(child_entity, Parent::new(parent_entity)) {
                        log::error!(
                            "Failed to attach Parent component to child entity {:?}: {}",
                            child_entity,
                            e
                        );
                    }
                } else {
                    log::warn!(
                        "Unable to resolve child '{}' for parent '{}'",
                        child_label,
                        parent_label
                    );
                }
            }

            if resolved_children.is_empty() {
                continue;
            }

            let mut local_insert_one: Option<hecs::Entity> = None;

            match world.query_one::<&mut Children>(parent_entity).get() {
                Ok(child_component) => {
                    child_component.clear();
                    child_component
                        .children_mut()
                        .extend(resolved_children.iter().copied());
                }
                Err(e) => {
                    log::warn!(
                        "Failed to query Parent component for entity {:?}: {}",
                        parent_entity,
                        e
                    );
                    local_insert_one = Some(parent_entity);
                }
            }

            if let Some(parent_entity) = local_insert_one
                && let Err(e) = world.insert_one(parent_entity, Children::new(resolved_children))
            {
                log::error!(
                    "Failed to attach Parent component to entity {:?}: {}",
                    parent_entity,
                    e
                );
            }
        }
    }

    async fn ensure_default_light(
        &self,
        world: &mut hecs::World,
        graphics: Arc<SharedGraphicsContext>,
        progress_sender: Option<&Sender<WorldLoadingStatus>>,
    ) -> anyhow::Result<()> {
        let mut has_light = false;
        if world
            .query::<&Light>()
            .iter()
            .next()
            .is_some()
        {
            has_light = true;
        }

        if !has_light {
            log::info!("No lights in scene, spawning default light");

            if let Some(s) = progress_sender {
                let _ = s.send(WorldLoadingStatus::LoadingEntity {
                    index: 0,
                    name: String::from("Default Light"),
                    total: 1,
                });
            }
            let comp = LightComponent::directional(glam::DVec3::ONE, 1.0);
            let light =
                Light::new(graphics.clone(), comp.clone(), Some("Default Light"))
                    .await;

            let light_config = SerializedLight {
                label: "Default Light".to_string(),
                light_component: comp.clone(),
                entity_id: None,
            };

            let transform = comp.to_transform();

            world.spawn((
                Label::from("Default Light"),
                comp,
                light,
                light_config,
                CustomProperties::default(),
                transform,
            ));
        }

        Ok(())
    }

    fn select_active_camera(
        &self,
        world: &mut hecs::World,
        graphics: Arc<SharedGraphicsContext>,
        progress_sender: Option<&Sender<WorldLoadingStatus>>,
        is_play_mode: bool,
    ) -> anyhow::Result<hecs::Entity> {
        use crate::camera::CameraType;

        if is_play_mode {
            log::debug!("Running in play mode");
            let starting_camera = world
                .query::<(Entity, &Camera, &CameraComponent)>()
                .iter()
                .find_map(|(entity, _, component)| {
                    if component.starting_camera {
                        log::debug!("Found starting camera: {:?}", entity);
                        Some(entity)
                    } else {
                        None
                    }
                });

            if let Some(camera_entity) = starting_camera {
                log::debug!("Using starting camera for play mode");
                Ok(camera_entity)
            } else {
                panic!("Unable to locate any starting camera while playing")
            }
        } else {
            let debug_camera = {
                world
                    .query::<(Entity, &Camera, &CameraComponent)>()
                    .iter()
                    .find_map(|(entity, _, component)| {
                        if matches!(component.camera_type, CameraType::Debug) {
                            log::debug!("Found debug camera: {:?}", entity);
                            Some(entity)
                        } else {
                            None
                        }
                    })
            };

            if let Some(camera_entity) = debug_camera {
                log::info!("Using existing debug camera for editor");
                Ok(camera_entity)
            } else {
                log::info!("No debug or starting camera found, creating viewport camera for editor");

                if let Some(s) = progress_sender {
                    let _ = s.send(WorldLoadingStatus::LoadingEntity {
                        index: 0,
                        name: String::from("Viewport Camera"),
                        total: 1,
                    });
                }
                let camera = Camera::predetermined(graphics.clone(), Some("Viewport Camera"));
                let component = crate::camera::DebugCamera::new();
                let label = Label::new("Viewport Camera");
                let camera_entity = world.spawn((label, camera, component));
                Ok(camera_entity)
            }
        }
    }
}