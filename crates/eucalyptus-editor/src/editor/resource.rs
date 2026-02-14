use egui::CollapsingHeader;
use hecs::Entity;
use indexmap::Equivalent;
use dropbear_engine::camera::Camera;
use dropbear_engine::entity::{EntityTransform, MeshRenderer, Transform};
use dropbear_engine::lighting::LightComponent;
use eucalyptus_core::camera::{CameraComponent, CameraType};
use eucalyptus_core::properties::CustomProperties;
use eucalyptus_core::states::{Label, Light, Script};
use eucalyptus_core::{success, warn};
use eucalyptus_core::physics::collider::{Collider, ColliderGroup};
use eucalyptus_core::physics::kcc::KCC;
use eucalyptus_core::physics::rigidbody::RigidBody;
use crate::editor::{EditorTabViewer, UndoableAction, TABS_GLOBAL};

impl<'a> EditorTabViewer<'a> {
    pub(crate) fn resource_inspector(&mut self, ui: &mut egui::Ui) {
        let mut cfg = TABS_GLOBAL.lock();

        let local_scene_settings = cfg.root_node_selected;
        let mut local_add_collider: Option<Entity> = None;

        if let Some(entity) = self.selected_entity {
            let mut local_set_initial_camera = false;
            let mut inspect_entity = *entity;
            let world = &self.world;

            if !cfg.root_node_selected {
                if let Ok((label, )) = world.query_one::<(&mut Label,)>(inspect_entity).get() {
                    label.inspect(
                        &mut inspect_entity,
                        &mut cfg,
                        ui,
                        self.undo_stack,
                        self.signal,
                        &mut String::new(),
                    );

                    ui.label(format!("Entity ID: {}", inspect_entity.id()));

                    ui.separator();

                    // mesh renderer
                    if let Ok(e) = world.query_one::<&mut MeshRenderer>(inspect_entity).get()
                    {
                        CollapsingHeader::new("MeshRenderer").default_open(true).show(ui, |ui| {
                            e.inspect(
                                &mut inspect_entity,
                                &mut cfg,
                                ui,
                                self.undo_stack,
                                self.signal,
                                &mut String::new(),
                            );
                        });
                    }

                    // entity transform
                    if let Ok(t) = world.query_one::<&mut EntityTransform>(inspect_entity).get()
                    {
                        CollapsingHeader::new("Transform").default_open(true).show(ui, |ui| {
                            t.inspect(
                                &mut inspect_entity,
                                &mut cfg,
                                ui,
                                self.undo_stack,
                                self.signal,
                                &mut String::new(),
                            );
                        });
                        ui.separator();
                    }

                    // custom properties
                    if let Ok(props) = world.query_one::<&mut CustomProperties>(inspect_entity).get()
                    {
                        CollapsingHeader::new("Custom Properties").default_open(true).show(ui, |ui| {
                            props.inspect(
                                &mut inspect_entity,
                                &mut cfg,
                                ui,
                                self.undo_stack,
                                self.signal,
                                label.as_mut_string(),
                            );
                        });
                        ui.separator();
                    }

                    // camera
                    if let Ok((camera, camera_component)) = world
                        .query_one::<(&mut Camera, &mut CameraComponent)>(inspect_entity).get()
                    {
                        CollapsingHeader::new("Camera").default_open(true).show(ui, |ui| {
                            camera.inspect(
                                &mut inspect_entity,
                                &mut cfg,
                                ui,
                                self.undo_stack,
                                self.signal,
                                &mut String::new(),
                            );

                            ui.separator();

                            camera_component.inspect(
                                &mut inspect_entity,
                                &mut cfg,
                                ui,
                                self.undo_stack,
                                self.signal,
                                &mut camera.label.clone(),
                            );

                            ui.separator();

                            // camera controller
                            let mut active_camera = self.active_camera.lock();

                            if active_camera.equivalent(&Some(*entity)) {
                                ui.label("Status: Currently viewing through camera");
                            } else {
                                ui.label("Status: Not viewing through this camera");
                            }

                            if ui.button("Set as active camera").clicked() {
                                *active_camera = Some(*entity);
                                log::info!(
                                            "Currently viewing from camera angle '{}'",
                                            camera.label
                                        );
                            }

                            if camera_component.starting_camera {
                                if ui.button("Stop making camera initial").clicked() {
                                    log::debug!("'Stop making camera initial' button clicked");
                                    camera_component.starting_camera = false;
                                    success!("Removed {} from starting camera", camera.label);
                                }
                            } else if ui.button("Set as initial camera").clicked() {
                                log::debug!("'Set as initial camera' button clicked");
                                if matches!(camera_component.camera_type, CameraType::Debug) {
                                    warn!(
                                                "Cannot set any cameras of type 'Debug' to initial camera"
                                            );
                                } else {
                                    local_set_initial_camera = true
                                }
                            }
                        });
                        ui.separator();
                    }

                    // light
                    if let Ok((light, comp, transform)) = world.query_one::<(&mut Light, &mut LightComponent, &mut Transform)>(inspect_entity).get()
                    {
                        light.transform = *transform;
                        light.light_component = comp.clone();

                        light.inspect(
                            entity,
                            &mut cfg,
                            ui,
                            self.undo_stack,
                            self.signal,
                            &mut String::new(),
                        );

                        *transform = light.transform;
                        *comp = light.light_component.clone();
                        ui.separator();
                    }

                    // script
                    if let Ok((script, ui_c)) = world.query_one::<(Option<&mut Script>, Option<&mut UIComponent>)>(*entity).get()
                    {
                        CollapsingHeader::new("Script").default_open(true).show(ui, |ui| {
                            if let Some(s) = script {
                                s.inspect(
                                    entity,
                                    &mut cfg,
                                    ui,
                                    self.undo_stack,
                                    self.signal,
                                    label.as_mut_string(),
                                );
                            }

                            if let Some(ui_c) = ui_c {
                                CollapsingHeader::new("UI").default_open(true).show(ui, |ui| {
                                    ui_c.inspect(
                                        entity,
                                        &mut cfg,
                                        ui,
                                        self.undo_stack,
                                        self.signal,
                                        label.as_mut_string(),
                                    );
                                });
                            }
                        });
                        ui.separator();
                    }

                    // physics
                    if let Ok((rigid, colliders, kcc)) = world.query_one::<(Option<&mut RigidBody>, Option<&mut ColliderGroup>, Option<&mut KCC>)>(*entity).get()
                    {
                        if rigid.is_some() || colliders.is_some() || kcc.is_some() {
                            CollapsingHeader::new("Physics").default_open(true).show(ui, |ui| {

                                if let Some(kcc) = kcc {
                                    CollapsingHeader::new("Kinematic Character Controller").default_open(true).show(ui, |ui| {
                                        kcc.inspect(
                                            entity,
                                            &mut cfg,
                                            ui,
                                            self.undo_stack,
                                            self.signal,
                                            label.as_mut_string(),
                                        );
                                    });
                                    ui.separator();
                                }

                                if let Some(rigid) = rigid {
                                    CollapsingHeader::new("RigidBody").default_open(true).show(ui, |ui| {
                                        rigid.inspect(
                                            entity,
                                            &mut cfg,
                                            ui,
                                            self.undo_stack,
                                            self.signal,
                                            label.as_mut_string(),
                                        );
                                    });
                                    ui.separator();
                                }

                                if let Some(col) = colliders {
                                    CollapsingHeader::new("Colliders").default_open(true).show(ui, |ui| {
                                        let mut to_remove: Option<usize> = None;

                                        for (index, c) in col.colliders.iter_mut().enumerate() {
                                            ui.horizontal(|ui| {
                                                let header = CollapsingHeader::new(format!("Collider {}", index + 1))
                                                    .default_open(true);

                                                header.show(ui, |ui| {
                                                    c.inspect(
                                                        entity,
                                                        &mut cfg,
                                                        ui,
                                                        self.undo_stack,
                                                        self.signal,
                                                        label.as_mut_string(),
                                                    );
                                                });

                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    if ui.button("🗑").on_hover_text("Remove collider").clicked() {
                                                        to_remove = Some(index);
                                                    }
                                                });
                                            });

                                            ui.separator();
                                        }

                                        if let Some(index) = to_remove {
                                            col.colliders.remove(index);
                                        }

                                        if ui.button("➕ Add new collider").clicked() {
                                            local_add_collider = Some(*entity);
                                        }
                                    });
                                }

                                ui.separator();
                            });
                        }
                        ui.separator();
                    }
                } else {
                    log_once::debug_once!("Unable to query entity inside resource inspector");
                }
            }

            if local_set_initial_camera {
                for (id, comp) in self.world.query::<(Entity, &mut CameraComponent)>().iter() {
                    comp.starting_camera = false;
                    self.undo_stack
                        .push(UndoableAction::RemoveStartingCamera(id))
                }

                if let Ok(comp) = self.world.query_one_mut::<&mut CameraComponent>(*entity)
                {
                    success!("This camera is currently set as the initial camera");
                    comp.starting_camera = true;
                }
            }
        } else if !local_scene_settings {
            ui.label("No entity selected, therefore no info to provide. Go on, what are you waiting for? Click an entity!");
        }

        if local_scene_settings {
            log_once::debug_once!("Rendering scene settings");
            self.scene_settings(&mut cfg, ui);
        }

        if let Some(e) = local_add_collider {
            let mut manual_edit = false;
            if let Ok(col) = self.world.query_one::<Option<&mut ColliderGroup>>(e).get()
            {
                if let Some(col) = col {
                    let mut collider = Collider::new();
                    collider.id = col.colliders.len() as u32 + 1;
                    col.insert(collider);
                } else {
                    manual_edit = true;
                }
            }

            if manual_edit {
                let _ = self.world.insert_one(e, ColliderGroup::new());
            }
        }
    }
}