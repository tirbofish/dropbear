use egui_ltreeview::{NodeBuilder, TreeViewBuilder};
use eucalyptus_core::{
    component::ComponentRegistry,
    hierarchy::{Children, Hierarchy, Parent},
    physics::{collider::ColliderGroup, rigidbody::RigidBody},
    states::{Label, PROJECT},
};
use hecs::{Entity, World};
use std::collections::{BTreeMap, HashMap, VecDeque};

use crate::editor::page::EditorTabVisibility;
use crate::editor::{
    Editor, EditorTabDock, EditorTabDockDescriptor, EditorTabViewer, Signal, StaticallyKept,
    TABS_GLOBAL,
};

impl<'a> EditorTabViewer<'a> {
    pub(crate) fn entity_list(&mut self, ui: &mut egui::Ui) {
        puffin::profile_function!();
        let mut cfg = TABS_GLOBAL.lock();

        let (_response, action) = {
            puffin::profile_scope!("entity_list.tree_build");
            egui_ltreeview::TreeView::new(egui::Id::new(
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
                        // root node/scene display
                        NodeBuilder::dir(u64::MAX)
                            .label(format!("Scene: {}", current_scene_name))
                            .context_menu(|ui| {
                                if ui.button("New Empty Entity").clicked() {
                                    let label = Editor::unique_label_for_world(self.world, "Blank Entity");
                                    self.world.spawn((label,));
                                    ui.close();
                                }
                                ui.menu_button("Import Template", |_| {});
                                ui.separator();
                                if ui.button("Paste to Root").clicked() {
                                    let copied = self.signal.iter().find_map(|s| {
                                        if let Signal::Copy(e, pm) = s { Some((e.clone(), pm.clone())) } else { None }
                                    });
                                    if let Some((entities, parent_map)) = copied {
                                        self.signal.push_back(Signal::Paste(entities, parent_map, None));
                                    }
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
                    component_ids_by_entity: &HashMap<Entity, Vec<u64>>,
                    rigidbody_component_id: Option<u64>,
                    cfg: &mut StaticallyKept,
                    signal: &mut VecDeque<Signal>,
                ) -> anyhow::Result<()> {
                    puffin::profile_scope!("entity_list.add_entity_to_tree");
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
                                    let mut grouped_components: BTreeMap<String, Vec<(u64, &eucalyptus_core::component::ComponentDescriptor)>> =
                                        BTreeMap::new();

                                    for (id, desc) in registry.iter_available_components() {
                                        if desc.internal { continue; }
                                        let category = desc
                                            .category
                                            .clone()
                                            .unwrap_or_else(|| "Uncategorised".to_string());
                                        grouped_components
                                            .entry(category)
                                            .or_default()
                                            .push((id, desc));
                                    }

                                    for (category, components) in grouped_components.iter_mut() {
                                        components.sort_by(|a, b| a.1.type_name.cmp(&b.1.type_name));

                                        ui.menu_button(category, |ui| {
                                            for (id, desc) in components.iter() {
                                                let response = ui.button(desc.type_name.as_str());

                                                if let Some(description) = desc.description.as_ref() {
                                                    response.clone().on_hover_text(description);
                                                }

                                                if response.clicked() {
                                                    if let Some(component) =
                                                        registry.create_default_component(*id)
                                                    {
                                                        signal.push_back(Signal::AddComponent(entity, component));
                                                    }
                                                    ui.close();
                                                }
                                            }
                                        });
                                    }
                                });
                                if ui.button("Create Template").clicked() {
                                    eucalyptus_core::fatal!("Not implemented yet");
                                }
                                ui.separator();
                                if ui.button("Copy").clicked() {
                                    let (sub_e, sub_pm) = Editor::collect_entity_subtree(world, entity, registry);
                                    if !sub_e.is_empty() {
                                        signal.retain(|s| !matches!(s, Signal::Copy(_, _)));
                                        signal.push_back(Signal::Copy(sub_e, sub_pm));
                                    }
                                    ui.close();
                                }
                                if ui.button("Paste as Child").clicked() {
                                    let copied = signal.iter().find_map(|s| {
                                        if let Signal::Copy(e, pm) = s { Some((e.clone(), pm.clone())) } else { None }
                                    });
                                    if let Some((entities, parent_map)) = copied {
                                        let paste_parent = world.get::<&Label>(entity).ok().map(|l| (*l).clone());
                                        signal.push_back(Signal::Paste(entities, parent_map, paste_parent));
                                    }
                                    ui.close();
                                }
                            }),
                    );

                    if let Some(component_ids) = component_ids_by_entity.get(&entity) {
                        let display_id = crate::features::is_enabled(crate::features::ShowComponentTypeIDInEditor);
                        let has_rigidbody = world.get::<&RigidBody>(entity).is_ok();
                        let has_collider = world.get::<&ColliderGroup>(entity).is_ok();

                        for component_type_id in component_ids {
                            let component_node_id =
                                cfg.component_node_id(entity, *component_type_id);
                            let display = registry
                                .get_descriptor_by_numeric_id(*component_type_id)
                                .map(|desc| {
                                    if display_id {
                                        format!("{} (id #{component_type_id})", desc.type_name)
                                    } else {
                                        desc.type_name.clone()
                                    }
                                })
                                .unwrap_or_else(|| {
                                    if display_id {
                                        format!("Unknown (id #{component_type_id})")
                                    } else {
                                        String::from("Unknown")
                                    }
                                });

                            let node = NodeBuilder::leaf(component_node_id)
                                .label_ui(|ui| {
                                    ui.label(display.clone());

                                    if has_rigidbody
                                        && !has_collider
                                        && Some(*component_type_id) == rigidbody_component_id
                                    {
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
                                            *component_type_id,
                                        );
                                        ui.close();
                                    }
                                });

                            builder.node(node);
                        }
                    }

                    let mut children_entities = if let Ok(children) = world.get::<&Children>(entity) {
                        children.children().to_vec()
                    } else {
                        Vec::new()
                    };
                    children_entities.sort_by_key(|e| e.to_bits().get());

                    for child in children_entities {
                        if let Err(e) =
                            add_entity_to_tree(
                                builder,
                                child,
                                world,
                                registry,
                                component_ids_by_entity,
                                rigidbody_component_id,
                                cfg,
                                signal,
                            )
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

                let mut component_ids_by_entity: HashMap<Entity, Vec<u64>> = HashMap::new();
                let rigidbody_component_id = self
                    .component_registry
                    .iter_available_components()
                    .find_map(|(id, desc)| {
                        if desc.fqtn == "eucalyptus_core::physics::rigidbody::RigidBody" {
                            Some(id)
                        } else {
                            None
                        }
                    });
                {
                    puffin::profile_scope!("entity_list.index_components");
                    for (component_id, desc) in self.component_registry.iter_available_components() {
                        if desc.internal { continue; }
                        for entity in self
                            .component_registry
                            .find_entities_by_numeric_id(self.world, component_id)
                        {
                            component_ids_by_entity
                                .entry(entity)
                                .or_default()
                                .push(component_id);
                        }
                    }
                }

                let mut root_entities: Vec<Entity> = self
                    .world
                    .query::<Entity>()
                    .without::<&Parent>()
                    .iter()
                    .map(|e| e)
                    .collect();
                root_entities.sort_by_key(|e| e.to_bits().get());

                for entity in root_entities {
                    puffin::profile_scope!("entity_list.root_entity");
                    if let Err(e) = add_entity_to_tree(
                        builder,
                        entity,
                        &mut self.world,
                        &self.component_registry,
                        &component_ids_by_entity,
                        rigidbody_component_id,
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
            })
        };

        puffin::profile_scope!("entity_list.actions");
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

pub struct EntityListDock;

impl EditorTabDock for EntityListDock {
    fn desc() -> EditorTabDockDescriptor {
        EditorTabDockDescriptor {
            id: "entity_list",
            title: "Model/Entity List".to_string(),
            visibility: EditorTabVisibility::GameEditor,
        }
    }

    fn display(viewer: &mut EditorTabViewer<'_>, ui: &mut egui::Ui) {
        viewer.entity_list(ui);
    }
}
