//! Deals with scene loading and scene metadata.

pub mod loading;
pub mod scripting;

use crate::camera::CameraComponent;
use crate::hierarchy::{Children, Parent, SceneHierarchy};
use crate::states::{Camera3D, Label, Light, Script, SerializedMeshRenderer, WorldLoadingStatus, PROJECT};
use crate::utils::ResolveReference;
use dropbear_engine::asset::ASSET_REGISTRY;
use dropbear_engine::camera::{Camera, CameraBuilder};
use dropbear_engine::entity::{EntityTransform, MeshRenderer, Transform};
use dropbear_engine::graphics::Texture;
use dropbear_engine::graphics::SharedGraphicsContext;
use dropbear_engine::lighting::{Light as EngineLight, LightComponent};
use dropbear_engine::model::{LoadedModel, Material, Model, ModelId};
use dropbear_engine::model::MODEL_CACHE;
use dropbear_engine::utils::{ResourceReference, ResourceReferenceType};
use dropbear_traits::SerializableComponent;
use dropbear_traits::registry::ComponentRegistry;
use glam::{DQuat, DVec3};
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use crossbeam_channel::Sender;
use hecs::Entity;
use crate::physics::collider::ColliderGroup;
use crate::physics::kcc::KCC;
use crate::physics::PhysicsState;
use crate::physics::rigidbody::RigidBody;
use crate::properties::CustomProperties;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct SceneEntity {
    #[serde(default)]
    pub label: Label,

    #[serde(default)]
    pub components: Vec<Box<dyn SerializableComponent>>,

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

    /// Helper function to load a component and add it to the entity builder
    async fn load_component(
        component: Box<dyn SerializableComponent>,
        builder: &mut hecs::EntityBuilder,
        graphics: Arc<SharedGraphicsContext>,
        registry: Option<&ComponentRegistry>,
        label: &str,
    ) -> anyhow::Result<()> {
        if let Some(transform) = component.as_any().downcast_ref::<EntityTransform>() {
            builder.add(*transform);
        } else if let Some(renderer) = component.as_any().downcast_ref::<SerializedMeshRenderer>() {
            let renderer = renderer.clone();
            let import_scale = renderer.import_scale.unwrap_or(1.0);
            let mut model = match &renderer.handle.ref_type {
                ResourceReferenceType::None => {
                    log::error!(
                        "Resource reference type is None for entity '{}', not supported, skipping",
                        label
                    );
                    return Ok(());
                }
                ResourceReferenceType::Unassigned { id } => {
                    log::debug!("Loading entity '{}' with no model selected", label);

                    let model = std::sync::Arc::new(Model {
                        label: "None".to_string(),
                        path: ResourceReference::from_reference(ResourceReferenceType::Unassigned { id: *id }),
                        meshes: Vec::new(),
                        materials: Vec::new(),
                        id: ModelId(*id),
                    });
                    let loaded = LoadedModel::new_raw(&ASSET_REGISTRY, model);
                    MeshRenderer::from_handle_with_import_scale(loaded, import_scale)
                }
                ResourceReferenceType::File(reference) => {
                    let path = &renderer.handle.resolve()?;

                    log::debug!(
                            "Path for entity {} is {} from reference {}",
                            label,
                            path.display(),
                            reference
                        );

                    let loaded = Model::load(graphics.clone(), path, Some(label), None).await?;
                    MeshRenderer::from_handle_with_import_scale(loaded, import_scale)
                }
                ResourceReferenceType::Bytes(bytes) => {
                    log::info!("Loading entity from bytes [Len: {}]", bytes.len());

                    let loaded =
                        Model::load_from_memory(graphics.clone(), bytes.clone(), Some(label), None)
                            .await?;
                    MeshRenderer::from_handle_with_import_scale(loaded, import_scale)
                }
                ResourceReferenceType::Cuboid { size_bits } => {
                    let size = [
                        f32::from_bits(size_bits[0]),
                        f32::from_bits(size_bits[1]),
                        f32::from_bits(size_bits[2]),
                    ];
                    log::info!("Loading entity from cuboid: {:?}", size);
                    {
                        let mut cache_guard = MODEL_CACHE.lock();
                        cache_guard.remove(label);
                    }

                    let size_vec = glam::DVec3::new(size[0] as f64, size[1] as f64, size[2] as f64);
                    let mut loaded_model = dropbear_engine::procedural::ProcedurallyGeneratedObject::cuboid(size_vec)
                        .build_model(graphics.clone(), None, Some(label));

                    let model = loaded_model.make_mut();
                    model.path = ResourceReference::from_reference(ResourceReferenceType::Cuboid { size_bits: *size_bits });

                    loaded_model.refresh_registry();

                    MeshRenderer::from_handle_with_import_scale(loaded_model, import_scale)
                }
            };

            if !renderer.material_override.is_empty() {
                for override_entry in &renderer.material_override {
                    if ASSET_REGISTRY
                        .model_handle_from_reference(&override_entry.source_model)
                        .is_none()
                    {
                        if matches!(
                            override_entry.source_model.ref_type,
                            ResourceReferenceType::File(_)
                        ) {
                            let source_path = override_entry.source_model.resolve()?;
                            let label_hint = override_entry.source_model.as_uri();
                            Model::load(graphics.clone(), &source_path, label_hint, None).await?;
                        } else {
                            log::warn!(
                                "Material override for '{}' references unsupported resource {:?}",
                                label,
                                override_entry.source_model
                            );
                            continue;
                        }
                    }

                    if let Err(err) = model.apply_material_override(
                        &override_entry.target_material,
                        override_entry.source_model.clone(),
                        &override_entry.source_material,
                    ) {
                        log::warn!(
                            "Failed to apply material override '{}' on '{}': {}",
                            override_entry.target_material,
                            label,
                            err
                        );
                    }
                }
            }

            if !renderer.material_customisation.is_empty() {
                for custom in &renderer.material_customisation {
                    {
                        let model_mut = model.make_model_mut();
                        let name_index = model_mut
                            .materials
                            .iter()
                            .position(|mat| mat.name == custom.target_material);

                        let index = name_index.or(custom.material_index);

                        if let Some(material) = index.and_then(|idx| model_mut.materials.get_mut(idx)) {
                            material.set_tint(graphics.as_ref(), custom.tint);
                            material.set_uv_tiling(graphics.as_ref(), custom.uv_tiling);

                            if let Some(reference) = &custom.diffuse_texture {
                                if let Ok(path) = reference.resolve() {
                                    if let Ok(bytes) = std::fs::read(&path) {
                                        let diffuse = Texture::new_with_wrap_mode(
                                            graphics.clone(),
                                            &bytes,
                                            custom.wrap_mode,
                                        );
                                        let flat_normal = (*ASSET_REGISTRY
                                            .solid_texture_rgba8(
                                                graphics.clone(),
                                                [128, 128, 255, 255],
                                            ))
                                        .clone();

                                        material.diffuse_texture = diffuse;
                                        material.normal_texture = flat_normal;
                                        material.bind_group = Material::create_bind_group(
                                            graphics.as_ref(),
                                            &material.diffuse_texture,
                                            &material.normal_texture,
                                            &material.name,
                                        );
                                        material.texture_tag = reference.as_uri().map(|s| s.to_string());
                                        material.wrap_mode = custom.wrap_mode;
                                        material.set_uv_tiling(graphics.as_ref(), custom.uv_tiling);
                                    } else {
                                        log::warn!(
                                            "Failed to read custom texture '{}' for '{}'",
                                            path.display(),
                                            label
                                        );
                                    }
                                } else {
                                    log::warn!(
                                        "Failed to resolve custom texture reference {:?} for '{}'",
                                        reference,
                                        label
                                    );
                                }
                            }
                        }
                    }

                }

                model.sync_asset_registry();
            }

            builder.add(model);
        } else if let Some(props) = component.as_any().downcast_ref::<CustomProperties>() {
            builder.add(props.clone());
        } else if let Some(camera_comp) = component.as_any().downcast_ref::<Camera3D>() {
            let cam_builder = CameraBuilder::from(camera_comp.clone());
            let comp = CameraComponent::from(camera_comp.clone());
            let camera = Camera::new(graphics.clone(), cam_builder, Some(label));
            builder.add_bundle((camera, comp));
        } else if let Some(light_conf) = component.as_any().downcast_ref::<Light>() {
            let light = EngineLight::new(
                graphics.clone(),
                light_conf.light_component.clone(),
                light_conf.transform,
                Some(label),
            )
            .await;
            builder.add_bundle((light_conf.light_component.clone(), light));
            builder.add(light_conf.clone());
            builder.add(light_conf.transform);
        } else if let Some(script) = component.as_any().downcast_ref::<Script>() {
            builder.add(script.clone());
        } else if let Some(body) = component.as_any().downcast_ref::<RigidBody>() {
            builder.add(body.clone());
        } else if component.as_any().downcast_ref::<Parent>().is_some() {
            log::debug!(
                "Skipping Parent component for '{}' - will be rebuilt from hierarchy_map",
                label
            );
        } else if let Some(registry) = registry {
            if !registry.deserialize_into_builder(component.as_ref(), builder)? {
                log::warn!(
                    "Unknown component type '{}' for entity '{}' - skipping",
                    component.type_name(),
                    label
                );
            }
        } else {
            log::warn!(
                "Unknown component type '{}' for entity '{}' - skipping",
                component.type_name(),
                label
            );
        }

        Ok(())
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
        registry: Option<&ComponentRegistry>,
        progress_sender: Option<Sender<WorldLoadingStatus>>,
        is_play_mode: bool,
    ) -> anyhow::Result<hecs::Entity> {
        if let Some(ref s) = progress_sender {
            let _ = s.send(WorldLoadingStatus::Idle);
        }

        log::info!(
            "Loading scene [{}], clearing world with {} entities",
            self.scene_name,
            world.len()
        );
        world.clear();

        #[allow(unused_variables)]
        let project_config = if cfg!(feature = "editor") {
            let cfg = PROJECT.read();
            cfg.project_path.clone()
        } else {
            log::debug!("Not using the editor feature, returning empty pathbuffer");
            PathBuf::new()
        };

        log::info!("World cleared, now has {} entities", world.len());

        let entity_configs: Vec<(usize, SceneEntity)> = {
            let cloned = self.entities.clone();
            cloned
                .into_par_iter()
                .enumerate()
                .map(|(i, e)| (i, e))
                .collect()
        };

        let mut label_to_entity: HashMap<Label, hecs::Entity> = HashMap::new();

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

            let mut builder = hecs::EntityBuilder::new();

            builder.add(label_for_map.clone());

            let mut has_entity_transform = false;

            for component in components {
                if component
                    .as_any()
                    .downcast_ref::<EntityTransform>()
                    .is_some()
                {
                    has_entity_transform = true;
                }

                Self::load_component(
                    component,
                    &mut builder,
                    graphics.clone(),
                    registry,
                    &label_for_logs,
                )
                .await?;
            }

            let entity = world.spawn(builder.build());

            if has_entity_transform {
                if let Ok((
                              entity_transform, 
                              renderer_opt, 
                              light_opt, 
                              light_comp_opt
                          )) = world.query_one::<(
                    &EntityTransform,
                    Option<&mut MeshRenderer>,
                    Option<&mut EngineLight>,
                    Option<&mut LightComponent>,
                )>(entity).get()
                {
                    let transform = entity_transform.sync();

                    if let Some(renderer) = renderer_opt {
                        renderer.update(&transform);
                        log::debug!("Updated renderer transform for '{}'", label_for_logs);
                    }

                    if let (Some(light), Some(light_comp)) = (light_opt, light_comp_opt) {
                        light.update(graphics.as_ref(), light_comp, &transform);
                        log::debug!("Updated light transform for '{}'", label_for_logs);
                    }
                }
            }

            // adding to physics
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

                // rigidbody
                if let Some(body) = rigid {
                    body.entity = label.clone();

                    self.physics_state.register_rigidbody(body, e_trans.sync());
                }

                // collider group
                if let Some(group) = col_group {
                    for collider in &mut group.colliders {
                        collider.entity = label.clone();

                        self.physics_state.register_collider(collider);
                    }
                }

                // kinematic character controller
                if let Some(kcc) = kcc {
                    kcc.entity = label.clone();
                }
            }

            if let Some(previous) = label_to_entity.insert(label_for_map.clone(), entity) {
                log::warn!(
                    "Duplicate entity label '{}' detected; previous entity {:?} will be overwritten in hierarchy mapping",
                    label_for_logs,
                    previous
                );
            }

            log::debug!("Loaded entity '{}'", label_for_logs);
        }

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

        {
            let mut has_light = false;
            if world
                .query::<(&LightComponent, &EngineLight)>()
                .iter()
                .next()
                .is_some()
            {
                has_light = true;
            }

            if !has_light {
                log::info!("No lights in scene, spawning default light");

                let legacy_lights: Vec<hecs::Entity> = world
                    .query::<(Entity, &Label)>()
                    .iter()
                    .filter_map(|(entity, label)| {
                        if label.as_str() == "Default Light" {
                            Some(entity)
                        } else {
                            None
                        }
                    })
                    .collect();

                for entity in legacy_lights {
                    if let Err(err) = world.despawn(entity) {
                        log::warn!(
                            "Failed to remove legacy 'Default Light' entity {:?}: {}",
                            entity,
                            err
                        );
                    } else {
                        log::debug!(
                            "Removed legacy 'Default Light' placeholder entity {:?}",
                            entity
                        );
                    }
                }

                if let Some(ref s) = progress_sender {
                    let _ = s.send(WorldLoadingStatus::LoadingEntity {
                        index: 0,
                        name: String::from("Default Light"),
                        total: 1,
                    });
                }
                let comp = LightComponent::directional(glam::DVec3::ONE, 1.0);
                let light_direction = LightComponent::default_direction();
                let rotation =
                    DQuat::from_rotation_arc(DVec3::new(0.0, 0.0, -1.0), light_direction);
                let trans = Transform {
                    position: glam::DVec3::new(2.0, 4.0, 2.0),
                    rotation,
                    ..Default::default()
                };
                let light =
                    EngineLight::new(graphics.clone(), comp.clone(), trans, Some("Default Light"))
                        .await;

                let light_config = Light {
                    label: "Default Light".to_string(),
                    transform: trans,
                    light_component: comp.clone(),
                    enabled: true,
                    entity_id: None,
                };

                {
                    world.spawn((
                        Label::from("Default Light"),
                        comp,
                        trans,
                        light,
                        light_config,
                        CustomProperties::default(),
                    ));
                }
            }
        }

        log::info!("Loaded {} entities from scene", self.entities.len());
        {
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

                {
                    if let Some(camera_entity) = debug_camera {
                        log::info!("Using existing debug camera for editor");
                        Ok(camera_entity)
                    } else {
                        log::info!("No debug or starting camera found, creating viewport camera for editor");

                        let legacy_cameras: Vec<hecs::Entity> = world
                            .query::<(Entity, &Label)>()
                            .iter()
                            .filter_map(|(entity, label)| {
                                if label.as_str() == "Viewport Camera" {
                                    log::debug!("Found 'Viewport Camera' entity: {:?}", entity);
                                    Some(entity)
                                } else {
                                    None
                                }
                            })
                            .collect();

                        for entity in legacy_cameras {
                            if let Err(err) = world.despawn(entity) {
                                log::warn!(
                                "Failed to remove legacy 'Viewport Camera' entity {:?}: {}",
                                entity,
                                err
                            );
                            } else {
                                log::debug!(
                                "Removed legacy 'Viewport Camera' placeholder entity {:?}",
                                entity
                            );
                            }
                        }

                        if let Some(ref s) = progress_sender {
                            let _ = s.send(WorldLoadingStatus::LoadingEntity {
                                index: 0,
                                name: String::from("Viewport Camera"),
                                total: 1,
                            });
                        }
                        let camera = Camera::predetermined(graphics.clone(), Some("Viewport Camera"));
                        let component = crate::camera::DebugCamera::new();
                        let label = Label::new("Viewport Camera");
                        let camera_entity = { world.spawn((label, camera, component)) };
                        Ok(camera_entity)
                    }
                }
            }
        }
    }
}