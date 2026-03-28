use crate::editor::page::EditorTabVisibility;
use crate::editor::{EditorTabDock, EditorTabDockDescriptor, EditorTabViewer, TABS_GLOBAL};
use dropbear_engine::camera::Camera;
use eucalyptus_core::camera::{CameraComponent, CameraType};
use eucalyptus_core::entity_status::EntityStatus;
use hecs::Entity;

impl<'a> EditorTabViewer<'a> {
    pub(crate) fn resource_inspector(&mut self, ui: &mut egui::Ui) {
        let mut cfg = TABS_GLOBAL.lock();

        let local_scene_settings = cfg.root_node_selected;

        if let Some(entity) = self.selected_entity {
            let inspect_entity = *entity;

            if !cfg.root_node_selected {
                ui.label(format!("Entity ID: {}", inspect_entity.id()));
                ui.separator();

                // entity status
                {
                    let (mut hidden, mut disabled) = self
                        .world
                        .get::<&EntityStatus>(inspect_entity)
                        .map(|s| (s.hidden, s.disabled))
                        .unwrap_or((false, false));

                    let mut status_changed = false;
                    ui.horizontal(|ui| {
                        if ui.checkbox(&mut hidden, "Hidden").changed() {
                            status_changed = true;
                        }
                        if ui.checkbox(&mut disabled, "Disabled").changed() {
                            status_changed = true;
                        }
                    });

                    if status_changed {
                        let has_status = self.world.get::<&EntityStatus>(inspect_entity).is_ok();
                        if has_status {
                            if let Ok(mut s) = self.world.get::<&mut EntityStatus>(inspect_entity) {
                                s.hidden = hidden;
                                s.disabled = disabled;
                            }
                        } else {
                            let _ = self
                                .world
                                .insert_one(inspect_entity, EntityStatus { hidden, disabled });
                        }
                    }
                }
                ui.separator();

                let mut local_unset_comp = false;
                if let Ok((_, comp)) = self
                    .world
                    .query_one::<(&Camera, &CameraComponent)>(inspect_entity)
                    .get()
                {
                    let is_active = self
                        .active_camera
                        .lock()
                        .map_or(false, |active| active == inspect_entity);
                    ui.horizontal(|ui| {
                        let label = if is_active {
                            "Viewing Through This Camera"
                        } else {
                            "View Through This Camera"
                        };
                        if ui
                            .add_enabled(!is_active, egui::Button::new(label))
                            .clicked()
                        {
                            let mut active_camera = self.active_camera.lock();
                            *active_camera = Some(inspect_entity);
                        }

                        let mut is_starting = comp.starting_camera;
                        let is_starting_label = if comp.camera_type == CameraType::Debug {
                            is_starting = true;
                            "Cannot set a Debug camera as starting"
                        } else if is_starting {
                            "Already set as starting"
                        } else {
                            "Set as starting"
                        };

                        if ui
                            .add_enabled(!is_starting, egui::Button::new(is_starting_label))
                            .clicked()
                        {
                            local_unset_comp = true;
                        }
                    });
                    ui.separator();
                }

                if local_unset_comp {
                    for (e, comp) in self.world.query::<(Entity, &mut CameraComponent)>().iter() {
                        if e == inspect_entity {
                            comp.starting_camera = true;
                            continue;
                        }
                        log::debug!("Unset starting camera for entity {:?}", e);
                        comp.starting_camera = false;
                    }
                }

                self.component_registry.inspect_components(
                    self.world,
                    inspect_entity,
                    ui,
                    self.graphics.clone(),
                );
            }
        } else if !local_scene_settings {
            ui.label("No entity selected, therefore no info to provide. Go on, what are you waiting for? Click an entity!");
        }

        if local_scene_settings {
            log_once::debug_once!("Rendering scene settings");
            self.scene_settings(&mut cfg, ui);
        }
    }
}

pub struct ResourceInspectorDock;

impl EditorTabDock for ResourceInspectorDock {
    fn desc() -> EditorTabDockDescriptor {
        EditorTabDockDescriptor {
            id: "inspector",
            title: "Resource Inspector".to_string(),
            visibility: EditorTabVisibility::GameEditor,
        }
    }

    fn display(viewer: &mut EditorTabViewer<'_>, ui: &mut egui::Ui) {
        viewer.resource_inspector(ui);
    }
}
