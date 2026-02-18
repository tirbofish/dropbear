use egui_ltreeview::{NodeBuilder, TreeViewBuilder};
use eucalyptus_core::{component::ComponentRegistry, hierarchy::{Children, Hierarchy, Parent}, physics::{collider::ColliderGroup, rigidbody::RigidBody}, states::{Label, PROJECT}};
use hecs::{Entity, World};

use crate::editor::{Editor, EditorTabViewer, Signal, StaticallyKept, TABS_GLOBAL};

impl<'a> EditorTabViewer<'a> {
    pub(crate) fn entity_list(&mut self, ui: &mut egui::Ui) {
        let mut cfg = TABS_GLOBAL.lock();

        let (_response, action) = egui_ltreeview::TreeView::new(egui::Id::new(
            "model_entity_list",
        ))
            .show(ui, |builder| {
                let current_scene_name = {
                    PROJECT
                        .read()
                        .last_opened_scene
                        .clone()
                        .unwrap_or("Scene".to_string())
                };
                builder.node(
                    NodeBuilder::dir(u64::MAX)
                        .label(format!("Scene: {}", current_scene_name))
                        .context_menu(|ui| {
                            if ui.button("New Empty Entity").clicked() {
                                let label = Editor::unique_label_for_world(self.world, "Blank Entity");
                                self.world.spawn((label,));
                                ui.close();
                            }
                        }),
                );
                // the root scene must be the biggest number possible to remove any ambiguity

                fn add_entity_to_tree(
                    builder: &mut TreeViewBuilder<u64>,
                    entity: Entity,
                    world: &mut World,
                    registry: &ComponentRegistry,
                    cfg: &mut StaticallyKept,
                    signal: &mut Signal,
                ) -> anyhow::Result<()> {
                    let entity_id = entity.to_bits().get();
                    let label = if let Ok(label) = world.query_one::<&Label>(entity).get()
                    {
                        label.clone()
                    } else {
                        anyhow::bail!(
                                "This entity [{}] is expected to contain Label",
                                entity_id
                            );
                    };

                    builder.node(
                        NodeBuilder::dir(entity_id)
                            .label(label.as_str())
                            .context_menu(|ui| {
                                ui.menu_button("New", |ui| {
                                    if ui.button("Child").clicked() {
                                        let label = Editor::unique_label_for_world(world, "New Entity");
                                        let child = world.spawn((label,));
                                        Hierarchy::set_parent(world, child, entity);
                                        ui.close();
                                    }
                                });
                                ui.menu_button("Add", |ui| {
                                    log_once::debug_once!("Available components: ");
                                    for (id, desc) in registry.iter_available_components() {
                                        log_once::debug_once!("id: {}, name: {}", id, desc.fqtn);

                                        if ui.button(desc.type_name.as_str()).clicked() {
                                            if let Some(component) = registry.create_default_component(id) {
                                                *signal = Signal::AddComponent(entity, component);
                                            }
                                            ui.close();
                                        }
                                    }
                                });
                            }),
                    );

                    let components = registry.extract_all_components(world, entity);

                    for component in components.iter() {
                        let Some(component_type_id) =
                            registry.id_for_component(component.as_ref())
                        else {
                            log_once::warn_once!(
                                    "Component missing registry id, skipping tree entry"
                                );
                            continue;
                        };
                        let component_node_id =
                            cfg.component_node_id(entity, component_type_id as u64);
                        let display = registry
                            .get_descriptor_by_numeric_id(component_type_id)
                            .map(|desc| format!("{} (id #{component_type_id})", desc.type_name))
                            .unwrap_or_else(|| format!("Unknown (id #{component_type_id})"));

                        let has_rigidbody = world.get::<&RigidBody>(entity).is_ok();
                        let has_collider = world.get::<&ColliderGroup>(entity).is_ok();

                        let node = NodeBuilder::leaf(component_node_id)
                            .label_ui(|ui| {
                                ui.label(display.clone());

                                if has_rigidbody && !has_collider && component.typetag_name().contains("RigidBody") {
                                    ui.add_space(4.0);
                                    ui.small_button("⚠")
                                        .on_hover_text("RigidBody has no colliders! Add the ColliderGroup component");
                                }
                            })
                            .context_menu(|ui| {
                                if ui.button("Remove Component").clicked() {
                                    registry.remove_component_by_id(
                                        world,
                                        entity,
                                        component_type_id,
                                    );
                                    ui.close();
                                }
                            });

                        builder.node(node);
                    }

                    let children_entities = if let Ok(children) = world.get::<&Children>(entity) {
                        children.children().to_vec()
                    } else {
                        Vec::new()
                    };

                    for child in children_entities {
                        if let Err(e) =
                            add_entity_to_tree(builder, child, world, registry, cfg, signal)
                        {
                            log_once::error_once!(
                                    "Failed to add child entity to tree, skipping: {}",
                                    e
                                );
                            continue;
                        }
                    }

                    builder.close_dir();
                    Ok(())
                }

                let root_entities: Vec<Entity> = self
                    .world
                    .query::<Entity>()
                    .without::<&Parent>()
                    .iter()
                    .map(|e| e)
                    .collect();

                for entity in root_entities {
                    if let Err(e) = add_entity_to_tree(
                        builder,
                        entity,
                        &mut self.world,
                        &self.component_registry,
                        &mut cfg,
                        self.signal,
                    ) {
                        log_once::error_once!(
                                "Failed to add child entity to tree, skipping: {}",
                                e
                            );
                    }
                }

                builder.close_dir();
            });

        for i in action {
            match i {
                egui_ltreeview::Action::SetSelected(items) => {
                    log_once::debug_once!("Selected: {:?}", items);
                    self.handle_tree_selection(&mut cfg, &items);
                }
                egui_ltreeview::Action::Move(drag_and_drop) => {
                    log_once::debug_once!("Moved: {:?}", drag_and_drop);
                    self.handle_tree_move(&mut cfg, &drag_and_drop);
                }
                egui_ltreeview::Action::Drag(drag_and_drop) => {
                    log_once::debug_once!("Dragged: {:?}", drag_and_drop);
                    self.handle_tree_drag(&mut cfg, &drag_and_drop);
                }
                egui_ltreeview::Action::Activate(activate) => {
                    log_once::debug_once!("Activated: {:?}", activate);
                    self.handle_tree_activate(&mut cfg, &activate);
                }
                egui_ltreeview::Action::DragExternal(_drag_and_drop_external) => {}
                egui_ltreeview::Action::MoveExternal(_drag_and_drop_external) => {}
            }
        }
    }
}