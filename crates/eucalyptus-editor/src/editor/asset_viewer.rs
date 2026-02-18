use std::{cmp::Ordering, fs, hash::DefaultHasher, io, path::Path};
use std::hash::{Hash, Hasher};
use dropbear_engine::{graphics::NO_TEXTURE, utils::ResourceReference};
use dropbear_engine::asset::ASSET_REGISTRY;
use dropbear_engine::entity::MeshRenderer;
use dropbear_engine::model::Model;
use dropbear_engine::texture::Texture;
use eucalyptus_core::utils::ResolveReference;
use egui_ltreeview::{Action, NodeBuilder, TreeViewBuilder};
use eucalyptus_core::states::PROJECT;
use hecs::Entity;
use log::{info, warn};

use crate::editor::{ComponentNodeSelection, DraggedAsset, EditorTabViewer, FsEntry, StaticallyKept, TABS_GLOBAL};
use eucalyptus_core::component::DRAGGED_ASSET_ID;

#[derive(Clone, Copy, Debug)]
enum TextureSlot {
    Diffuse,
    Normal,
    Emissive,
    MetallicRoughness,
    Occlusion,
}

impl<'a> EditorTabViewer<'a> {
    pub(crate) fn show_asset_viewer(&mut self, ui: &mut egui::Ui) {
        let mut cfg = TABS_GLOBAL.lock();

        let project_root = {
            let project = PROJECT.read();
            if project.project_path.as_os_str().is_empty() {
                ui.label("Open a project to browse assets.");
                return;
            }
            project.project_path.clone()
        };

        let (_resp, action) = egui_ltreeview::TreeView::new(egui::Id::new("asset_viewer")).show(ui, |builder| {
            builder.node(Self::dir_node("euca://"));
            self.build_resource_branch(&mut cfg, builder, &project_root);
            Self::build_scripts_branch(&mut cfg, builder, &project_root);
            Self::build_scene_branch(&mut cfg, builder, &project_root);
            builder.close_dir();
        });

        for a in action {
            match a {
                Action::SetSelected(selected) => {
                    log_once::debug_once!("Selected: {:?}", selected);
                }
                Action::Move(moved) => {
                    log_once::debug_once!("Moved: {:?}", moved);
                }
                Action::Drag(dragged) => {
                    log_once::debug_once!("Dragged: {:?}", dragged);

                    if let Some(&node_id) = dragged.source.first() {
                        if let Some(asset) = cfg.asset_node_assets.get(&node_id).cloned() {
                            cfg.dragged_asset = Some(asset.clone());
                            ui.ctx().data_mut(|d| {
                                d.insert_temp(egui::Id::new(DRAGGED_ASSET_ID), Some(asset.path.clone()))
                            });
                        }
                    }
                }
                Action::Activate(activated) => {
                    log_once::debug_once!("Activated: {:?}", activated);
                }
                Action::DragExternal(_) => {}
                Action::MoveExternal(_) => {}
            }
        }
    }

    fn build_resource_branch(
        &mut self,
        cfg: &mut StaticallyKept,
        builder: &mut TreeViewBuilder<u64>,
        project_root: &Path,
    ) {
        let label = "euca://resources";
        builder.node(Self::dir_node_labeled(label, "resources"));
        let resources_root = project_root.join("resources");
        if resources_root.exists() {
            self.walk_resource_directory(cfg, builder, &resources_root, &resources_root);
        } else {
            Self::add_placeholder_leaf(builder, "euca://resources/missing", "missing");
        }
        builder.close_dir();
    }

    fn walk_resource_directory(
        &mut self,
        cfg: &mut StaticallyKept,
        builder: &mut TreeViewBuilder<u64>,
        base_path: &Path,
        current_path: &Path,
    ) {
        let entries = match Self::sorted_entries(current_path) {
            Ok(entries) => entries,
            Err(err) => {
                log_once::warn_once!(
                    "Unable to enumerate resources at '{}': {}",
                    current_path.parent().unwrap_or(current_path).display(),
                    err
                );
                return;
            }
        };

        for entry in entries {
            let full_label = Self::resource_label(base_path, &entry.path);
            if entry.is_dir {
                builder.node(Self::dir_node_labeled(&full_label, &entry.name));
                self.walk_resource_directory(cfg, builder, base_path, &entry.path);
                builder.close_dir();
            } else {
                if entry.name.eq_ignore_ascii_case("resources.eucc") {
                    continue;
                }

                let reference = ResourceReference::from_euca_uri(&full_label)
                    .unwrap_or_else(|_| ResourceReference::default());
                cfg.asset_node_assets.insert(
                    Self::asset_node_id(&full_label),
                    DraggedAsset {
                        name: entry.name.clone(),
                        path: reference.clone(),
                    },
                );
                let is_model = Self::is_model_file(&entry.name);
                let is_texture = Self::is_texture_file(&entry.name);
                let entry_name = entry.name.clone();
                let reference_for_menu = reference.clone();
                let menu = Self::leaf_node_labeled(&full_label, &entry.name).context_menu(|ui| {
                        if is_model {
                            if ui.button("Load to memory").clicked() {
                                ui.close();
                                self.queue_model_load(reference_for_menu.clone(), entry_name.clone());
                            }
                        }

                        if is_texture {
                            if ui.button("Load to memory").clicked() {
                                ui.close();
                                self.queue_texture_load(reference_for_menu.clone(), entry_name.clone());
                            }

                            ui.separator();
                            ui.menu_button("Choose", |ui| {
                                if ui.button("Diffuse").clicked() {
                                    ui.close();
                                    self.apply_texture_slot(
                                        reference_for_menu.clone(),
                                        TextureSlot::Diffuse,
                                    );
                                }
                                if ui.button("Normal").clicked() {
                                    ui.close();
                                    self.apply_texture_slot(
                                        reference_for_menu.clone(),
                                        TextureSlot::Normal,
                                    );
                                }
                                if ui.button("Emissive").clicked() {
                                    ui.close();
                                    self.apply_texture_slot(
                                        reference_for_menu.clone(),
                                        TextureSlot::Emissive,
                                    );
                                }
                                if ui.button("Metal/Rough").clicked() {
                                    ui.close();
                                    self.apply_texture_slot(
                                        reference_for_menu.clone(),
                                        TextureSlot::MetallicRoughness,
                                    );
                                }
                                if ui.button("Occlusion").clicked() {
                                    ui.close();
                                    self.apply_texture_slot(
                                        reference_for_menu.clone(),
                                        TextureSlot::Occlusion,
                                    );
                                }
                            });
                        }
                    });
                builder.node(menu);
            }
        }
    }

    fn resource_label(base_path: &Path, path: &Path) -> String {
        let relative = path
            .strip_prefix(base_path)
            .map(|rel| rel.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"));
        if relative.is_empty() {
            "euca://resources".to_string()
        } else {
            format!("euca://resources/{}", relative)
        }
    }

    fn build_scripts_branch(
        _cfg: &mut StaticallyKept,
        builder: &mut TreeViewBuilder<u64>,
        project_root: &Path,
    ) {
        let label = "euca://scripts";
        builder.node(Self::dir_node_labeled(label, "scripts"));
        let scripts_root = project_root.join("src");
        if !scripts_root.exists() {
            Self::add_placeholder_leaf(builder, "euca://scripts/missing", "missing");
            builder.close_dir();
            return;
        }

        let entries = match Self::sorted_entries(&scripts_root) {
            Ok(entries) => entries,
            Err(err) => {
                log::warn!(
                    "Unable to enumerate scripts at '{}': {}",
                    scripts_root.display(),
                    err
                );
                builder.close_dir();
                return;
            }
        };

        let mut had_content = false;
        for entry in entries {
            if entry.is_dir {
                let source_label = format!("{}/{}", label, entry.name);
                builder.node(Self::dir_node_labeled(&source_label, &entry.name));
                if Self::build_script_source_set(builder, &entry.path, &source_label) {
                    had_content = true;
                }
                builder.close_dir();
            } else if !entry.name.eq_ignore_ascii_case("source.eucc") {
                let file_label = format!("{}/{}", label, entry.name);
                builder.node(Self::leaf_node_labeled(&file_label, &entry.name));
                had_content = true;
            }
        }

        if !had_content {
            Self::add_placeholder_leaf(builder, "euca://scripts/empty", "empty");
        }

        builder.close_dir();
    }

    fn build_script_source_set(
        builder: &mut TreeViewBuilder<u64>,
        source_path: &Path,
        source_label: &str,
    ) -> bool {
        let entries = match Self::sorted_entries(source_path) {
            Ok(entries) => entries,
            Err(err) => {
                log::warn!(
                    "Unable to enumerate source set at '{}': {}",
                    source_path.display(),
                    err
                );
                Self::add_placeholder_leaf(
                    builder,
                    &format!("{source_label}/unreadable"),
                    "unreadable",
                );
                return true;
            }
        };

        let mut had_content = false;
        for entry in entries {
            if entry.is_dir {
                if entry.name.eq_ignore_ascii_case("kotlin") {
                    if Self::build_kotlin_tree(builder, &entry.path, source_label) {
                        had_content = true;
                    }
                } else {
                    let child_label = format!("{}/{}", source_label, entry.name);
                    builder.node(Self::dir_node_labeled(&child_label, &entry.name));
                    Self::build_plain_directory(builder, &entry.path, &child_label);
                    builder.close_dir();
                    had_content = true;
                }
            } else if !entry.name.eq_ignore_ascii_case("source.eucc") {
                let file_label = format!("{}/{}", source_label, entry.name);
                builder.node(Self::leaf_node_labeled(&file_label, &entry.name));
                had_content = true;
            }
        }

        if !had_content {
            Self::add_placeholder_leaf(builder, &format!("{source_label}/empty"), "empty");
            had_content = true;
        }

        had_content
    }

    fn build_plain_directory(
        builder: &mut TreeViewBuilder<u64>,
        dir_path: &Path,
        parent_label: &str,
    ) {
        let entries = match Self::sorted_entries(dir_path) {
            Ok(entries) => entries,
            Err(err) => {
                log::warn!(
                    "Unable to enumerate directory '{}': {}",
                    dir_path.display(),
                    err
                );
                Self::add_placeholder_leaf(
                    builder,
                    &format!("{parent_label}/unreadable"),
                    "unreadable",
                );
                return;
            }
        };

        if entries.is_empty() {
            Self::add_placeholder_leaf(builder, &format!("{parent_label}/empty"), "empty");
            return;
        }

        for entry in entries {
            let child_label = format!("{}/{}", parent_label, entry.name);
            if entry.is_dir {
                builder.node(Self::dir_node_labeled(&child_label, &entry.name));
                Self::build_plain_directory(builder, &entry.path, &child_label);
                builder.close_dir();
            } else {
                builder.node(Self::leaf_node_labeled(&child_label, &entry.name));
            }
        }
    }

    fn build_kotlin_tree(
        builder: &mut TreeViewBuilder<u64>,
        kotlin_path: &Path,
        source_label: &str,
    ) -> bool {
        let entries = match Self::sorted_entries(kotlin_path) {
            Ok(entries) => entries,
            Err(err) => {
                log::warn!(
                    "Unable to enumerate kotlin sources at '{}': {}",
                    kotlin_path.display(),
                    err
                );
                Self::add_placeholder_leaf(
                    builder,
                    &format!("{source_label}/unreadable"),
                    "unreadable",
                );
                return true;
            }
        };

        if entries.is_empty() {
            Self::add_placeholder_leaf(
                builder,
                &format!("{source_label}/no_kotlin_files"),
                "no kotlin files",
            );
            return true;
        }

        let mut had_entries = false;
        for entry in entries {
            if entry.is_dir {
                Self::build_kotlin_package_collapsed(
                    builder,
                    &entry.path,
                    source_label,
                    vec![entry.name.clone()],
                );
                had_entries = true;
            } else {
                let file_id = format!("{}/{}", source_label, entry.name);
                builder.node(Self::leaf_node_labeled(&file_id, &entry.name));
                had_entries = true;
            }
        }

        had_entries
    }

    fn build_kotlin_package_collapsed(
        builder: &mut TreeViewBuilder<u64>,
        dir_path: &Path,
        parent_path_str: &str,
        accumulated_parts: Vec<String>,
    ) {
        let entries = match Self::sorted_entries(dir_path) {
            Ok(entries) => entries,
            Err(err) => {
                let package_suffix = accumulated_parts.join(".");
                let full_path_str = format!("{}/{}", parent_path_str, package_suffix);
                log::warn!(
                    "Unable to enumerate package '{}' ({}): {}",
                    package_suffix,
                    dir_path.display(),
                    err
                );
                Self::add_placeholder_leaf(
                    builder,
                    &format!("{full_path_str}/unreadable"),
                    "unreadable",
                );
                return;
            }
        };

        let subdirs: Vec<&FsEntry> = entries.iter().filter(|e| e.is_dir).collect();
        let files: Vec<&FsEntry> = entries.iter().filter(|e| !e.is_dir).collect();

        if files.is_empty() && subdirs.len() == 1 {
            let subdir = subdirs[0];
            let mut new_parts = accumulated_parts;
            new_parts.push(subdir.name.clone());
            Self::build_kotlin_package_collapsed(builder, &subdir.path, parent_path_str, new_parts);
        } else {
            let package_suffix = accumulated_parts.join(".");
            let full_path_str = format!("{}/{}", parent_path_str, package_suffix);

            builder.node(Self::dir_node_labeled(&full_path_str, &package_suffix));

            for file in files {
                let file_id = format!("{}/{}", full_path_str, file.name);
                builder.node(Self::leaf_node_labeled(&file_id, &file.name));
            }

            for subdir in subdirs {
                Self::build_kotlin_package_collapsed(
                    builder,
                    &subdir.path,
                    &full_path_str,
                    vec![subdir.name.clone()],
                );
            }

            builder.close_dir();
        }
    }

    fn build_scene_branch(
        _cfg: &mut StaticallyKept,
        builder: &mut TreeViewBuilder<u64>,
        project_root: &Path,
    ) {
        let label = "euca://scenes";
        builder.node(Self::dir_node_labeled(label, "scenes"));
        let scenes_root = project_root.join("scenes");
        if !scenes_root.exists() {
            Self::add_placeholder_leaf(builder, "euca://scenes/missing", "missing");
            builder.close_dir();
            return;
        }

        let entries = match Self::sorted_entries(&scenes_root) {
            Ok(entries) => entries,
            Err(err) => {
                log::warn!(
                    "Unable to enumerate scenes at '{}': {}",
                    scenes_root.display(),
                    err
                );
                Self::add_placeholder_leaf(builder, "euca://scenes/unreadable", "unreadable");
                builder.close_dir();
                return;
            }
        };

        let mut had_entries = false;
        for entry in entries {
            if entry.is_dir {
                let child_label = format!("{}/{}", label, entry.name);
                builder.node(Self::dir_node_labeled(&child_label, &entry.name));
                Self::build_plain_directory(builder, &entry.path, &child_label);
                builder.close_dir();
                had_entries = true;
            } else if entry
                .path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("eucs"))
                .unwrap_or(false)
            {
                let file_label = format!("{}/{}", label, entry.name);
                builder.node(Self::leaf_node_labeled(&file_label, &entry.name));
                had_entries = true;
            }
        }

        if !had_entries {
            Self::add_placeholder_leaf(builder, "euca://scenes/no_scenes", "no scenes");
        }

        builder.close_dir();
    }

    fn sorted_entries(path: &Path) -> io::Result<Vec<FsEntry>> {
        let mut entries = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let name = entry.file_name().to_string_lossy().to_string();
            entries.push(FsEntry {
                path: entry.path(),
                name_lower: name.to_lowercase(),
                name,
                is_dir: file_type.is_dir(),
            });
        }

        entries.sort_by(|a, b| match b.is_dir.cmp(&a.is_dir) {
            Ordering::Equal => a.name_lower.cmp(&b.name_lower),
            other => other,
        });

        Ok(entries)
    }

    fn asset_node_id(label: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        label.hash(&mut hasher);
        let mut id = hasher.finish();
        if id == 0 {
            id = 1;
        }
        id
    }

    fn dir_node<'ui>(label: &str) -> NodeBuilder<'ui, u64> {
        Self::with_icon(NodeBuilder::dir(Self::asset_node_id(label)).label(label.to_string()))
    }

    fn dir_node_labeled<'ui>(id_source: &str, label: &str) -> NodeBuilder<'ui, u64> {
        Self::with_icon(NodeBuilder::dir(Self::asset_node_id(id_source)).label(label.to_string()))
    }

    fn leaf_node_labeled<'ui>(id_source: &str, label: &str) -> NodeBuilder<'ui, u64> {
        Self::with_icon(NodeBuilder::leaf(Self::asset_node_id(id_source)).label(label.to_string()))
    }

    fn with_icon<'ui>(builder: NodeBuilder<'ui, u64>) -> NodeBuilder<'ui, u64> {
        builder.icon(|ui| {
            egui_extras::install_image_loaders(ui.ctx());
            Self::draw_asset_icon(ui)
        })
    }

    fn draw_asset_icon(ui: &mut egui::Ui) {
        let image = egui::Image::from_bytes("bytes://asset-viewer-icon", NO_TEXTURE)
            .max_size(egui::vec2(14.0, 14.0));
        ui.add(image);
    }

    fn add_placeholder_leaf(builder: &mut TreeViewBuilder<u64>, id_source: &str, label: &str) {
        builder.node(Self::leaf_node_labeled(id_source, label));
    }


    fn is_model_file(name: &str) -> bool {
        let name = name.to_ascii_lowercase();
        name.ends_with(".glb") || name.ends_with(".gltf")
    }

    fn is_texture_file(name: &str) -> bool {
        let name = name.to_ascii_lowercase();
        name.ends_with(".png")
            || name.ends_with(".jpg")
            || name.ends_with(".jpeg")
            || name.ends_with(".tga")
            || name.ends_with(".bmp")
            || name.ends_with(".webp")
    }

    fn queue_model_load(&self, reference: ResourceReference, label: String) {
        if ASSET_REGISTRY
            .read()
            .get_model_handle_by_reference(&reference)
            .is_some()
        {
            info!("Model already loaded: {}", label);
            return;
        }

        let graphics = self.graphics.clone();
        let queue = graphics.future_queue.clone();
        queue.push(async move {
            let path = reference.resolve()?;
            let buffer = fs::read(&path)?;
            let handle = Model::load_from_memory_raw(
                graphics.clone(),
                buffer,
                None,
                ASSET_REGISTRY.clone(),
            )
            .await?;

            let mut registry = ASSET_REGISTRY.write();
            if let Some(model) = registry.get_model(handle).cloned() {
                let mut model = model;
                model.path = reference.clone();
                model.label = label.clone();
                registry.update_model(handle, model);
                registry.label_model(label.clone(), handle);
            }

            Ok::<(), anyhow::Error>(())
        });
    }

    fn queue_texture_load(&self, reference: ResourceReference, label: String) {
        if ASSET_REGISTRY
            .read()
            .get_texture_handle_by_reference(&reference)
            .is_some()
        {
            info!("Texture already loaded: {}", label);
            return;
        }

        let graphics = self.graphics.clone();
        let queue = graphics.future_queue.clone();
        queue.push(async move {
            let path = reference.resolve()?;
            let texture = Texture::from_file(graphics.clone(), &path, Some(&label)).await?;
            let mut registry = ASSET_REGISTRY.write();
            registry.add_texture_with_label(label.clone(), texture);
            Ok::<(), anyhow::Error>(())
        });
    }

    fn apply_texture_slot(&mut self, reference: ResourceReference, slot: TextureSlot) {
        let Some(entity) = *self.selected_entity else {
            warn!("Unable to apply texture: no entity selected");
            return;
        };

        let Some(handle) = ASSET_REGISTRY
            .read()
            .get_texture_handle_by_reference(&reference)
        else {
            warn!("Texture not loaded in memory, load it first");
            return;
        };

        let texture = {
            let registry = ASSET_REGISTRY.read();
            registry.get_texture(handle).cloned()
        };

        let Some(texture) = texture else {
            warn!("Texture handle missing from registry");
            return;
        };

        if let Ok(renderer) = self.world.query_one::<&mut MeshRenderer>(entity).get() {
            for material in renderer.material_snapshot.values_mut() {
                match slot {
                    TextureSlot::Diffuse => material.diffuse_texture = texture.clone(),
                    TextureSlot::Normal => material.normal_texture = texture.clone(),
                    TextureSlot::Emissive => material.emissive_texture = Some(texture.clone()),
                    TextureSlot::MetallicRoughness => {
                        material.metallic_roughness_texture = Some(texture.clone())
                    }
                    TextureSlot::Occlusion => material.occlusion_texture = Some(texture.clone()),
                }
            }
        } else {
            warn!("Selected entity has no MeshRenderer");
        }
    }

    pub(crate) fn handle_tree_selection(&mut self, cfg: &mut StaticallyKept, items: &[u64]) {
        for node_id in items {
            self.resolve_tree_node(cfg, *node_id);
        }
    }

    pub(crate) fn handle_tree_activate(
        &mut self,
        cfg: &mut StaticallyKept,
        activate: &egui_ltreeview::Activate<u64>,
    ) {
        self.handle_tree_selection(cfg, &activate.selected);
    }

    pub(crate) fn handle_tree_drag(
        &mut self,
        cfg: &mut StaticallyKept,
        drag: &egui_ltreeview::DragAndDrop<u64>,
    ) {
        if let Some(&node_id) = drag.source.first() {
            if let Some(selection) = cfg.component_selection(node_id) {
                cfg.pending_component_drag = Some(selection);
                self.inspect_component_selection(cfg, selection);
            }
        }
    }

    pub(crate) fn handle_tree_move(
        &mut self,
        cfg: &mut StaticallyKept,
        drag: &egui_ltreeview::DragAndDrop<u64>,
    ) {
        let selection = cfg.pending_component_drag.take().or_else(|| {
            drag.source
                .first()
                .and_then(|node_id| cfg.component_selection(*node_id))
        });

        if let Some(selection) = selection {
            self.inspect_component_selection(cfg, selection);
            if let Some(target_entity) = Self::entity_from_node_id(drag.target) {
                log::info!(
                    "Component id #{} ready to drop onto entity {:?}",
                    selection.component_type_id,
                    target_entity
                );
            }
        }
    }

    fn resolve_tree_node(&mut self, cfg: &mut StaticallyKept, node_id: u64) {
        if node_id == u64::MAX {
            log_once::debug_once!("Root node has been selected");
            cfg.root_node_selected = true;
            *self.selected_entity = None;
        } else if let Some(selection) = cfg.component_selection(node_id) {
            cfg.root_node_selected = false;
            self.inspect_component_selection(cfg, selection);
        } else if let Some(entity) = Self::entity_from_node_id(node_id) {
            cfg.root_node_selected = false;
            *self.selected_entity = Some(entity);
        }
    }

    fn inspect_component_selection(
        &mut self,
        cfg: &mut StaticallyKept,
        selection: ComponentNodeSelection,
    ) {
        cfg.remember_component_lookup(selection);
        let component_id = selection.component_type_id;
        let matches = self
            .component_registry
            .find_entities_by_numeric_id(self.world, component_id);
        let descriptor = self
            .component_registry
            .get_descriptor_by_numeric_id(component_id);

        if matches.is_empty() {
            log::warn!("Component id #{} not found in world", component_id);
            return;
        }

        let name = descriptor
            .map(|desc| desc.fqtn.as_str())
            .unwrap_or("<unknown>");
        for entity in matches {
            log::debug!(
                "Serializable component '{}' (id #{}) attached to entity {:?}",
                name,
                component_id,
                entity
            );
        }
    }

    fn entity_from_node_id(node_id: u64) -> Option<Entity> {
        if node_id == u64::MAX {
            None
        } else {
            Entity::from_bits(node_id)
        }
    }
}