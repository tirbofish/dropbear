use crate::editor::Editor;
use dropbear_engine::asset::Handle;
use dropbear_engine::entity::{EntityTransform, MeshRenderer};
use dropbear_engine::future::{FutureHandle, FutureQueue};
use dropbear_engine::graphics::SharedGraphicsContext;
use dropbear_engine::model::Model;
use eucalyptus_core::hierarchy::Parent;
use eucalyptus_core::scene::SceneEntity;
use eucalyptus_core::states::Label;
use eucalyptus_core::{fatal, success, warn};
use hecs::EntityBuilder;
use parking_lot::Mutex;
use std::sync::{Arc, LazyLock};

#[derive(Clone)]
pub struct PendingSpawn {
    pub scene_entity: SceneEntity,
    pub handle: Option<FutureHandle>,
}

pub static PENDING_SPAWNS: LazyLock<Mutex<Vec<PendingSpawn>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

pub fn push_pending_spawn(spawn: PendingSpawn) {
    PENDING_SPAWNS.lock().push(spawn);
}

pub trait PendingSpawnController {
    fn check_up(
        &mut self,
        graphics: Arc<SharedGraphicsContext>,
        queue: Arc<FutureQueue>,
    ) -> anyhow::Result<()>;
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

            if spawn.handle.is_none() {
                let components = spawn.scene_entity.components.clone();
                let registry = self.component_registry.clone();
                let label = spawn.scene_entity.label.clone();
                let graphics = graphics.clone();

                let future = async move {
                    let mut appliers: Vec<
                        Box<dyn for<'a> FnOnce(&'a mut EntityBuilder) + Send + Sync>,
                    > = Vec::new();
                    for component in components {
                        if component.as_any().downcast_ref::<Parent>().is_some() {
                            continue;
                        }

                        let Some(loader_future) =
                            registry.load_component(component.as_ref(), graphics.clone())
                        else {
                            warn!("Skipping unregistered serialized component for '{}'", label);
                            continue;
                        };

                        let applier = loader_future.await?;
                        appliers.push(applier);
                    }

                    Ok::<
                        (
                            Label,
                            Vec<Box<dyn for<'a> FnOnce(&'a mut EntityBuilder) + Send + Sync>>,
                        ),
                        anyhow::Error,
                    >((label, appliers))
                };

                let handle = queue.push(Box::pin(future));
                spawn.handle = Some(handle);
            }

            if let Some(handle) = &spawn.handle {
                if let Some(result) = queue.exchange_owned(handle) {
                    if let Ok(r) = result.downcast::<anyhow::Result<(
                        Label,
                        Vec<Box<dyn for<'a> FnOnce(&'a mut EntityBuilder) + Send + Sync>>,
                    )>>() {
                        match Arc::try_unwrap(r) {
                            Ok(Ok((label, appliers))) => {
                                let mut builder = EntityBuilder::new();
                                builder.add(label.clone());
                                for applier in appliers {
                                    applier(&mut builder);
                                }

                                let entity = self.world.spawn(builder.build());
                                if self.world.get::<&EntityTransform>(entity).is_err() {
                                    let _ =
                                        self.world.insert_one(entity, EntityTransform::default());
                                }

                                success!("Spawned '{}' from pending queue", label);
                                completed.push(index);
                            }
                            Ok(Err(err)) => {
                                fatal!(
                                    "Unable to init components for '{}': {}",
                                    spawn.scene_entity.label,
                                    err
                                );
                                completed.push(index);
                            }
                            Err(_) => {
                                log_once::warn_once!(
                                    "Spawn future for '{}' still shared, deferring",
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
                if let Ok(r) = result.downcast::<anyhow::Result<Box<dyn for<'a> FnOnce(&'a mut EntityBuilder) + Send + Sync>>>() {
                    match Arc::try_unwrap(r) {
                        Ok(Ok(applier)) => {
                            let mut builder = EntityBuilder::new();
                            applier(&mut builder);
                            if let Err(e) = self.world.insert(*entity, builder.build()) {
                                fatal!("Failed to add component bundle: {}", e);
                            } else {
                                success!("Added component to entity {:?}", entity);
                            }
                            completed_components.push(index);
                        }
                        Ok(Err(e)) => {
                            fatal!("Failed to add component: {}", e);
                            completed_components.push(index);
                        }
                        Err(_) => {} // Still shared
                    }
                } else {
                    fatal!("Pending component result could not be downcasted");
                    completed_components.push(index);
                }
            }
        }

        for &i in completed_components.iter().rev() {
            self.pending_components.remove(i);
        }

        let mut completed_swaps = Vec::new();
        for (index, (entity, handle)) in self.pending_model_swaps.iter().enumerate() {
            if let Some(result) = queue.exchange_owned(handle) {
                match result.downcast::<anyhow::Result<Handle<Model>>>() {
                    Ok(r) => match Arc::try_unwrap(r) {
                        Ok(Ok(loaded_model)) => {
                            if let Ok(mut renderer) = self.world.get::<&mut MeshRenderer>(*entity) {
                                renderer.set_model(loaded_model)
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
