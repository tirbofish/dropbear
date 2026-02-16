use crate::editor::Editor;
use dropbear_engine::entity::{EntityTransform, MeshRenderer};
use dropbear_engine::future::FutureQueue;
use dropbear_engine::graphics::{SharedGraphicsContext};
use dropbear_engine::model::{Model};
pub(crate) use eucalyptus_core::spawn::{PendingSpawnController, PENDING_SPAWNS};
use eucalyptus_core::{fatal, success};
use hecs::Entity;
use std::sync::Arc;
use dropbear_engine::asset::Handle;
use dropbear_engine::component::{ComponentInitContext, ComponentInsert, ComponentResources};

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
                let entity = self.world.spawn((spawn.scene_entity.label.clone(),));
                let components = spawn.scene_entity.components.clone();

                let mut resources = ComponentResources::new();
                resources.insert(graphics.clone());
                let resources = Arc::new(resources);

                let future = async move {
                    let mut inserts: Vec<Box<dyn ComponentInsert>> = Vec::new();
                    for component in components {
                        let ctx = ComponentInitContext {
                            entity,
                            resources: resources.clone(),
                        };
                        let insert = component.init(ctx).await?;
                        inserts.push(insert);
                    }

                    Ok::<(Entity, Vec<Box<dyn ComponentInsert>>), anyhow::Error>((entity, inserts))
                };

                let handle = queue.push(Box::pin(future));
                spawn.handle = Some(handle);
            }

            if let Some(handle) = &spawn.handle {
                if let Some(result) = queue.exchange_owned(handle) {
                    if let Ok(r) = result.downcast::<anyhow::Result<(Entity, Vec<Box<dyn ComponentInsert>>)>>() {
                        match Arc::try_unwrap(r) {
                            Ok(Ok((entity, inserts))) => {
                                for insert in inserts {
                                    insert.insert(&mut self.world, entity)?;
                                }

                                if self.world.get::<&EntityTransform>(entity).is_err() {
                                    let _ = self.world.insert_one(entity, EntityTransform::default());
                                }

                                success!(
                                    "Spawned '{}' from pending queue",
                                    spawn.scene_entity.label
                                );
                                completed.push(index);
                            }
                            Ok(Err(err)) => {
                                fatal!("Unable to init components for '{}': {}", spawn.scene_entity.label, err);
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
                if let Ok(r) = result.downcast::<anyhow::Result<Box<dyn ComponentInsert>>>() {
                    match Arc::try_unwrap(r) {
                        Ok(Ok(insert)) => {
                            insert.insert(&mut self.world, *entity)?;
                            success!("Added component to entity {:?}", entity);
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
