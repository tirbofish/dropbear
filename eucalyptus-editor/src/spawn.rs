use crate::editor::Editor;
use dropbear_engine::asset::ASSET_REGISTRY;
use dropbear_engine::camera::Camera;
use dropbear_engine::entity::{EntityTransform, MeshRenderer, Transform};
use dropbear_engine::future::FutureQueue;
use dropbear_engine::graphics::{SharedGraphicsContext, Texture};
use dropbear_engine::lighting::{Light, LightComponent};
use dropbear_engine::model::{LoadedModel, Material, Model, ModelId};
use dropbear_engine::utils::{ResourceReference, ResourceReferenceType};
use eucalyptus_core::camera::CameraComponent;
use eucalyptus_core::scene::SceneEntity;
pub(crate) use eucalyptus_core::spawn::{PendingSpawnController, PENDING_SPAWNS};
use eucalyptus_core::states::{
    Light as LightConfig, Script, SerializedMeshRenderer,
};
use eucalyptus_core::utils::ResolveReference;
use eucalyptus_core::{fatal, success};
use hecs::EntityBuilder;
use std::sync::Arc;
use eucalyptus_core::properties::CustomProperties;

fn component_ref<'a, T: 'static>(entity: &'a SceneEntity) -> Option<&'a T> {
    entity
        .components
        .iter()
        .find_map(|component| component.as_any().downcast_ref::<T>())
}

fn component_cloned<T: Clone + 'static>(entity: &SceneEntity) -> Option<T> {
    component_ref::<T>(entity).cloned()
}

impl PendingSpawnController for Editor {
    fn check_up(
        &mut self,
        graphics: Arc<SharedGraphicsContext>,
        queue: Arc<FutureQueue>,
    ) -> anyhow::Result<()> {
        queue.poll();
        let mut spawn_list = PENDING_SPAWNS.lock();
        let mut completed = Vec::new();

        for (index, spawn) in spawn_list.iter_mut().enumerate() {
            log_once::debug_once!(
                "Processing pending spawn for '{}'",
                spawn.scene_entity.label
            );

            let serialized_renderer =
                component_cloned::<SerializedMeshRenderer>(&spawn.scene_entity);

            if serialized_renderer.is_none() && spawn.handle.is_none() {
                log::debug!(
                    "No renderer component found for '{}', spawning immediately",
                    spawn.scene_entity.label
                );
                self.spawn_scene_entity(&spawn.scene_entity, None);
                completed.push(index);
                continue;
            }

            if spawn.handle.is_none() {
                if let Some(renderer) = serialized_renderer.clone() {
                    let graphics_clone = graphics.clone();
                    let label = spawn.scene_entity.label.to_string();
                    let future = async move {
                        load_renderer_from_serialized(renderer, graphics_clone, label).await
                    };
                    let handle = queue.push(Box::pin(future));
                    spawn.handle = Some(handle);
                }
            }

            if let Some(handle) = &spawn.handle {
                if let Some(result) = queue.exchange_owned(handle) {
                    if let Ok(r) = result.downcast::<anyhow::Result<MeshRenderer>>() {
                        match Arc::try_unwrap(r) {
                            Ok(outcome) => match outcome {
                                Ok(renderer) => {
                                    self.spawn_scene_entity(&spawn.scene_entity, Some(renderer));
                                    success!(
                                        "Spawned '{}' from pending queue",
                                        spawn.scene_entity.label
                                    );
                                    completed.push(index);
                                }
                                Err(err) => {
                                    fatal!("Unable to load mesh renderer: {}", err);
                                    completed.push(index);
                                }
                            },
                            Err(_) => {
                                log_once::warn_once!(
                                    "Renderer future for '{}' still shared, deferring",
                                    spawn.scene_entity.label
                                );
                            }
                        }
                    } else {
                        fatal!(
                            "Future result for '{}' could not be downcasted",
                            spawn.scene_entity.label
                        );
                        completed.push(index);
                    }
                }
            }
        }

        for &i in completed.iter().rev() {
            spawn_list.remove(i);
        }

        let mut completed_components = Vec::new();
        for (index, (entity, handle)) in self.pending_components.iter().enumerate() {
            if let Some(result) = queue.exchange_owned(handle) {
                match result.downcast::<anyhow::Result<MeshRenderer>>() {
                    Ok(r) => {
                        match Arc::try_unwrap(r) {
                            Ok(Ok(renderer)) => {
                                let _ = self.world.insert_one(*entity, renderer);
                                let _ = self.world.insert_one(*entity, EntityTransform::default());
                                success!("Added MeshRenderer to entity {:?}", entity);
                                completed_components.push(index);
                            }
                            Ok(Err(e)) => {
                                fatal!("Failed to load MeshRenderer: {}", e);
                                completed_components.push(index);
                            }
                            Err(_) => {} // Still shared
                        }
                    }
                    Err(result) => {
                        match result.downcast::<anyhow::Result<(Camera, CameraComponent)>>() {
                            Ok(r) => {
                                match Arc::try_unwrap(r) {
                                    Ok(Ok((camera, component))) => {
                                        let _ = self.world.insert(*entity, (camera, component));
                                        success!("Added Camera to entity {:?}", entity);
                                        completed_components.push(index);
                                    }
                                    Ok(Err(e)) => {
                                        fatal!("Failed to create Camera: {}", e);
                                        completed_components.push(index);
                                    }
                                    Err(_) => {} // Still shared
                                }
                            }
                            Err(result) => {
                                if let Ok(r) = result.downcast::<anyhow::Result<(
                                    LightComponent,
                                    Light,
                                    LightConfig,
                                    Transform,
                                )>>() {
                                    match Arc::try_unwrap(r) {
                                        Ok(Ok((
                                            light_comp,
                                            engine_light,
                                            light_config,
                                            transform,
                                        ))) => {
                                            let _ = self.world.insert(
                                                *entity,
                                                (light_comp, engine_light, light_config, transform),
                                            );
                                            success!("Added Light to entity {:?}", entity);
                                            completed_components.push(index);
                                        }
                                        Ok(Err(e)) => {
                                            fatal!("Failed to create Light: {}", e);
                                            completed_components.push(index);
                                        }
                                        Err(_) => {} // Still shared
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        for &i in completed_components.iter().rev() {
            self.pending_components.remove(i);
        }

        let mut completed_swaps = Vec::new();
        for (index, (entity, handle)) in self.pending_model_swaps.iter().enumerate() {
            if let Some(result) = queue.exchange_owned(handle) {
                match result.downcast::<anyhow::Result<LoadedModel>>() {
                    Ok(r) => match Arc::try_unwrap(r) {
                        Ok(Ok(loaded_model)) => {
                            if let Ok(mut renderer) = self.world.get::<&mut MeshRenderer>(*entity) {
                                renderer.set_handle(loaded_model);
                            } else {
                                let renderer = MeshRenderer::from_handle(loaded_model);
                                let _ = self.world.insert_one(*entity, renderer);
                            }

                            success!("Swapped MeshRenderer model for entity {:?}", entity);
                            completed_swaps.push(index);
                        }
                        Ok(Err(err)) => {
                            fatal!("Failed to load model for swap: {}", err);
                            completed_swaps.push(index);
                        }
                        Err(_) => {} // Still shared
                    },
                    Err(_) => {
                        fatal!("Model swap future result could not be downcasted");
                        completed_swaps.push(index);
                    }
                }
            }
        }

        for &i in completed_swaps.iter().rev() {
            self.pending_model_swaps.remove(i);
        }

        Ok(())
    }
}

impl Editor {
    fn spawn_scene_entity(
        &mut self,
        scene_entity: &SceneEntity,
        mesh_renderer: Option<MeshRenderer>,
    ) {
        let mut builder = EntityBuilder::new();
        builder.add(scene_entity.label.clone());

        if let Some(transform) = component_ref::<EntityTransform>(scene_entity).copied() {
            builder.add(transform);
        } else {
            builder.add(EntityTransform::default());
        }

        if let Some(renderer) = mesh_renderer {
            builder.add(renderer);
        }

        if let Some(props) = component_cloned::<CustomProperties>(scene_entity) {
            builder.add(props);
        }

        if let Some(script) = component_cloned::<Script>(scene_entity) {
            builder.add(script);
        }

        if let Some(camera) = component_cloned::<CameraComponent>(scene_entity) {
            builder.add(camera);
        }

        self.world.spawn(builder.build());
    }
}

async fn load_renderer_from_serialized(
    renderer: SerializedMeshRenderer,
    graphics: Arc<SharedGraphicsContext>,
    label: String,
) -> anyhow::Result<MeshRenderer> {
    fn is_legacy_internal_cube_uri(uri: &str) -> bool {
        let uri = uri.replace('\\', "/");
        uri.ends_with("internal/dropbear/models/cube")
    }

    let mut mesh_renderer = match &renderer.handle.ref_type {
        ResourceReferenceType::None => anyhow::bail!(
            "Renderer for '{}' does not specify an asset reference",
            label
        ),
        ResourceReferenceType::Unassigned { id } => {
            let model = std::sync::Arc::new(Model {
                label: "None".to_string(),
                path: ResourceReference::from_reference(ResourceReferenceType::Unassigned { id: *id }),
                meshes: Vec::new(),
                materials: Vec::new(),
                id: ModelId(*id),
            });

            let loaded = LoadedModel::new_raw(&ASSET_REGISTRY, model);
            MeshRenderer::from_handle(loaded)
        }
        ResourceReferenceType::File(reference) => {
            if is_legacy_internal_cube_uri(reference) {
                let size = glam::DVec3::new(1.0, 1.0, 1.0);
                let size_bits = [1.0f32.to_bits(), 1.0f32.to_bits(), 1.0f32.to_bits()];
                let mut loaded_model = dropbear_engine::procedural::ProcedurallyGeneratedObject::cuboid(size)
                    .build_model(graphics.clone(), None, Some(&label));

                let model = loaded_model.make_mut();
                model.path = ResourceReference::from_reference(ResourceReferenceType::Cuboid { size_bits });

                loaded_model.refresh_registry();

                MeshRenderer::from_handle(loaded_model)
            } else {
                let path = renderer.handle.resolve()?;
                MeshRenderer::from_path(graphics.clone(), &path, Some(&label)).await?
            }
        }
        ResourceReferenceType::Bytes(bytes) => {
            let model =
                Model::load_from_memory(graphics.clone(), bytes.clone(), Some(&label)).await?;
            MeshRenderer::from_handle(model)
        }
        ResourceReferenceType::Cuboid { size_bits } => {
            let size = [
                f32::from_bits(size_bits[0]),
                f32::from_bits(size_bits[1]),
                f32::from_bits(size_bits[2]),
            ];
            let size_vec = glam::DVec3::new(size[0] as f64, size[1] as f64, size[2] as f64);
            let mut loaded_model = dropbear_engine::procedural::ProcedurallyGeneratedObject::cuboid(size_vec)
                .build_model(graphics.clone(), None, Some(&label));

            let model = loaded_model.make_mut();
            model.path = ResourceReference::from_reference(ResourceReferenceType::Cuboid { size_bits: *size_bits });

            loaded_model.refresh_registry();

            MeshRenderer::from_handle(loaded_model)
        }
    };

    for override_entry in renderer.material_override {
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
                if let Err(err) = Model::load(graphics.clone(), &source_path, label_hint).await {
                    log::warn!(
                        "Failed to preload source model {:?} for override '{}': {}",
                        override_entry.source_model,
                        override_entry.target_material,
                        err
                    );
                    continue;
                }
            } else {
                log::warn!(
                    "Unsupported override source {:?} for '{}'",
                    override_entry.source_model,
                    label
                );
                continue;
            }
        }

        if let Err(err) = mesh_renderer.apply_material_override(
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

    if !renderer.material_customisation.is_empty() {
        for custom in &renderer.material_customisation {
            let model_mut = mesh_renderer.make_model_mut();
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
                        match std::fs::read(&path) {
                            Ok(bytes) => {
                                let diffuse = Texture::new_with_wrap_mode(
                                    graphics.clone(),
                                    &bytes,
                                    custom.wrap_mode,
                                );
                                let flat_normal = (*ASSET_REGISTRY
                                    .solid_texture_rgba8(graphics.clone(), [128, 128, 255, 255]))
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
                            }
                            Err(err) => {
                                log::warn!(
                                    "Failed to read custom texture '{}' for '{}': {}",
                                    path.display(),
                                    label,
                                    err
                                );
                            }
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

        mesh_renderer.sync_asset_registry();
    }

    Ok(mesh_renderer)
}
