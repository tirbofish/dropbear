use dropbear_engine::asset::ASSET_REGISTRY;
use dropbear_engine::model::Model;
use dropbear_engine::texture::Texture;
use dropbear_engine::{graphics::NO_TEXTURE, utils::ResourceReference};
use egui_ltreeview::{Action, NodeBuilder, TreeViewBuilder};
use eucalyptus_core::states::PROJECT;
use eucalyptus_core::utils::ResolveReference;
use hecs::Entity;
use log::{info, warn};
use std::hash::{Hash, Hasher};
use std::{cmp::Ordering, fs, hash::DefaultHasher, io, path::Path};

use crate::editor::{
    AssetDivision, AssetNodeInfo, AssetNodeKind, ComponentNodeSelection, DraggedAsset,
    EditorTabDock, EditorTabDockDescriptor, EditorTabViewer, FsEntry, ResourceDivision,
    SceneDivision, ScriptDivision, Signal, StaticallyKept, TABS_GLOBAL,
};
use eucalyptus_core::component::DRAGGED_ASSET_ID;

impl<'a> EditorTabViewer<'a> {
    pub(crate) fn show_asset_viewer(&mut self, ui: &mut egui::Ui) {
        let mut cfg = TABS_GLOBAL.lock();
        cfg.asset_node_assets.clear();
        cfg.asset_node_info.clear();
        if let Some(rename) = &cfg.asset_rename {
            if !rename.original_path.exists() {
                cfg.asset_rename = None;
            }
        }

        if let Some(mut rename) = cfg.asset_rename.take() {
            let rename_id = egui::Id::new("asset_rename_input");
            let mut should_apply = false;

            ui.horizontal(|ui| {
                ui.label("Rename");
                let response = ui.add(egui::TextEdit::singleline(&mut rename.buffer).id(rename_id));
                if rename.just_started {
                    ui.ctx().memory_mut(|m| m.request_focus(rename_id));
                    rename.just_started = false;
                }

                let enter = ui.input(|input| input.key_pressed(egui::Key::Enter));
                should_apply = (enter && response.has_focus()) || response.lost_focus();
            });

            if should_apply {
                let is_dir = rename.original_path.is_dir();
                self.apply_asset_rename(&rename, is_dir);
            } else {
                cfg.asset_rename = Some(rename);
            }

            ui.separator();
        }

        let project_root = {
            let project = PROJECT.read();
            if project.project_path.as_os_str().is_empty() {
                ui.label("Open a project to browse assets.");
                return;
            }
            project.project_path.clone()
        };

        let (_resp, action) =
            egui_ltreeview::TreeView::new(egui::Id::new("asset_viewer")).show(ui, |builder| {
                builder.node(Self::dir_node("euca://"));
                self.build_resource_branch(&mut cfg, builder, &project_root);
                self.build_scripts_branch(&mut cfg, builder, &project_root);
                self.build_scene_branch(&mut cfg, builder, &project_root);
                builder.close_dir();
            });

        for a in action {
            match a {
                Action::SetSelected(selected) => {
                    log_once::debug_once!("Selected: {:?}", selected);
                }
                Action::Move(moved) => {
                    log_once::debug_once!("Moved: {:?}", moved);
                    self.handle_asset_move(&mut cfg, &moved);
                }
                Action::Drag(dragged) => {
                    log_once::debug_once!("Dragged: {:?}", dragged);

                    if let Some(&node_id) = dragged.source.first() {
                        if let Some(asset) = cfg.asset_node_assets.get(&node_id).cloned() {
                            cfg.dragged_asset = Some(asset.clone());
                            ui.ctx().data_mut(|d| {
                                d.insert_temp(
                                    egui::Id::new(DRAGGED_ASSET_ID),
                                    Some(asset.path.clone()),
                                )
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
        let resources_root = project_root.join("resources");
        let root_info = AssetNodeInfo {
            path: resources_root.clone(),
            division: AssetDivision::Resources,
            kind: AssetNodeKind::Resource(ResourceDivision::Folder),
            is_dir: true,
            is_division_root: true,
            allow_add_folder: true,
        };
        Self::register_asset_node(cfg, label, root_info.clone());
        let node_id = Self::asset_node_id(label);
        let menu = Self::dir_node_kind(label, "resources", root_info.kind).context_menu(|ui| {
            self.asset_dir_context_menu(cfg, ui, node_id, &root_info, "New Folder")
        });
        builder.node(menu);
        if resources_root.exists() {
            self.walk_resource_directory(cfg, builder, &resources_root, &resources_root);
        } else {
            Self::add_placeholder_leaf(
                builder,
                "euca://resources/missing",
                "missing",
                AssetNodeKind::Resource(ResourceDivision::File),
            );
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
                let dir_info = AssetNodeInfo {
                    path: entry.path.clone(),
                    division: AssetDivision::Resources,
                    kind: AssetNodeKind::Resource(ResourceDivision::Folder),
                    is_dir: true,
                    is_division_root: false,
                    allow_add_folder: true,
                };
                Self::register_asset_node(cfg, &full_label, dir_info.clone());
                let node_id = Self::asset_node_id(&full_label);
                let menu = Self::dir_node_kind(&full_label, &entry.name, dir_info.kind)
                    .context_menu(|ui| {
                        self.asset_dir_context_menu(cfg, ui, node_id, &dir_info, "New Folder")
                    });
                builder.node(menu);
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
                let file_info = AssetNodeInfo {
                    path: entry.path.clone(),
                    division: AssetDivision::Resources,
                    kind: AssetNodeKind::Resource(ResourceDivision::File),
                    is_dir: false,
                    is_division_root: false,
                    allow_add_folder: false,
                };
                Self::register_asset_node(cfg, &full_label, file_info.clone());
                let node_id = Self::asset_node_id(&full_label);
                let menu = Self::leaf_node_kind(&full_label, &entry.name, file_info.kind)
                    .context_menu(|ui| {
                        self.asset_file_context_menu(cfg, ui, node_id, &file_info);
                        ui.separator();

                        if is_model {
                            if ui.button("Load to memory").clicked() {
                                ui.close();
                                self.queue_model_load(
                                    reference_for_menu.clone(),
                                    entry_name.clone(),
                                );
                                info!("Loading model {}", entry_name);
                            }
                        }

                        if is_texture {
                            if ui.button("Load to memory").clicked() {
                                ui.close();
                                self.queue_texture_load(
                                    reference_for_menu.clone(),
                                    entry_name.clone(),
                                );
                                info!("Loading texture {}", entry_name);
                            }
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
        &mut self,
        cfg: &mut StaticallyKept,
        builder: &mut TreeViewBuilder<u64>,
        project_root: &Path,
    ) {
        let label = "euca://scripts";
        let scripts_root = project_root.join("src");
        let root_info = AssetNodeInfo {
            path: scripts_root.clone(),
            division: AssetDivision::Scripts,
            kind: AssetNodeKind::Script(ScriptDivision::Package),
            is_dir: true,
            is_division_root: true,
            allow_add_folder: false,
        };
        Self::register_asset_node(cfg, label, root_info.clone());
        let node_id = Self::asset_node_id(label);
        let menu = Self::dir_node_kind(label, "scripts", root_info.kind).context_menu(|ui| {
            self.asset_dir_context_menu(cfg, ui, node_id, &root_info, "New Package")
        });
        builder.node(menu);
        if !scripts_root.exists() {
            Self::add_placeholder_leaf(
                builder,
                "euca://scripts/missing",
                "missing",
                AssetNodeKind::Script(ScriptDivision::Script),
            );
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
                let kotlin_root = entry.path.join("kotlin");
                let source_info = AssetNodeInfo {
                    path: kotlin_root,
                    division: AssetDivision::Scripts,
                    kind: AssetNodeKind::Script(ScriptDivision::Package),
                    is_dir: true,
                    is_division_root: true,
                    allow_add_folder: true,
                };
                Self::register_asset_node(cfg, &source_label, source_info.clone());
                let node_id = Self::asset_node_id(&source_label);
                let menu = Self::dir_node_kind(&source_label, &entry.name, source_info.kind)
                    .context_menu(|ui| {
                        self.asset_dir_context_menu(cfg, ui, node_id, &source_info, "New Package")
                    });
                builder.node(menu);
                if self.build_script_source_set(cfg, builder, &entry.path, &source_label) {
                    had_content = true;
                }
                builder.close_dir();
            } else if !entry.name.eq_ignore_ascii_case("source.eucc") {
                let file_label = format!("{}/{}", label, entry.name);
                let file_info = AssetNodeInfo {
                    path: entry.path.clone(),
                    division: AssetDivision::Scripts,
                    kind: AssetNodeKind::Script(ScriptDivision::Script),
                    is_dir: false,
                    is_division_root: false,
                    allow_add_folder: false,
                };
                Self::register_asset_node(cfg, &file_label, file_info.clone());
                let node_id = Self::asset_node_id(&file_label);
                let menu = Self::leaf_node_kind(&file_label, &entry.name, file_info.kind)
                    .context_menu(|ui| self.asset_file_context_menu(cfg, ui, node_id, &file_info));
                builder.node(menu);
                had_content = true;
            }
        }

        if !had_content {
            Self::add_placeholder_leaf(
                builder,
                "euca://scripts/empty",
                "empty",
                AssetNodeKind::Script(ScriptDivision::Script),
            );
        }

        builder.close_dir();
    }

    fn build_script_source_set(
        &mut self,
        cfg: &mut StaticallyKept,
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
                    AssetNodeKind::Script(ScriptDivision::Script),
                );
                return true;
            }
        };

        let mut had_content = false;
        for entry in entries {
            if entry.is_dir {
                if entry.name.eq_ignore_ascii_case("kotlin") {
                    if self.build_kotlin_tree(cfg, builder, &entry.path, source_label) {
                        had_content = true;
                    }
                } else {
                    let child_label = format!("{}/{}", source_label, entry.name);
                    let dir_info = AssetNodeInfo {
                        path: entry.path.clone(),
                        division: AssetDivision::Scripts,
                        kind: AssetNodeKind::Script(ScriptDivision::Package),
                        is_dir: true,
                        is_division_root: false,
                        allow_add_folder: true,
                    };
                    Self::register_asset_node(cfg, &child_label, dir_info.clone());
                    let node_id = Self::asset_node_id(&child_label);
                    let menu = Self::dir_node_kind(&child_label, &entry.name, dir_info.kind)
                        .context_menu(|ui| {
                            self.asset_dir_context_menu(cfg, ui, node_id, &dir_info, "New Package")
                        });
                    builder.node(menu);
                    self.build_plain_directory(
                        cfg,
                        builder,
                        &entry.path,
                        &child_label,
                        AssetDivision::Scripts,
                        AssetNodeKind::Script(ScriptDivision::Package),
                        AssetNodeKind::Script(ScriptDivision::Script),
                        "New Package",
                    );
                    builder.close_dir();
                    had_content = true;
                }
            } else if !entry.name.eq_ignore_ascii_case("source.eucc") {
                let file_label = format!("{}/{}", source_label, entry.name);
                let file_info = AssetNodeInfo {
                    path: entry.path.clone(),
                    division: AssetDivision::Scripts,
                    kind: AssetNodeKind::Script(ScriptDivision::Script),
                    is_dir: false,
                    is_division_root: false,
                    allow_add_folder: false,
                };
                Self::register_asset_node(cfg, &file_label, file_info.clone());
                let node_id = Self::asset_node_id(&file_label);
                let menu = Self::leaf_node_kind(&file_label, &entry.name, file_info.kind)
                    .context_menu(|ui| self.asset_file_context_menu(cfg, ui, node_id, &file_info));
                builder.node(menu);
                had_content = true;
            }
        }

        if !had_content {
            Self::add_placeholder_leaf(
                builder,
                &format!("{source_label}/empty"),
                "empty",
                AssetNodeKind::Script(ScriptDivision::Script),
            );
            had_content = true;
        }

        had_content
    }

    fn build_plain_directory(
        &mut self,
        cfg: &mut StaticallyKept,
        builder: &mut TreeViewBuilder<u64>,
        dir_path: &Path,
        parent_label: &str,
        division: AssetDivision,
        dir_kind: AssetNodeKind,
        file_kind: AssetNodeKind,
        new_folder_label: &str,
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
                    file_kind,
                );
                return;
            }
        };

        if entries.is_empty() {
            Self::add_placeholder_leaf(
                builder,
                &format!("{parent_label}/empty"),
                "empty",
                file_kind,
            );
            return;
        }

        for entry in entries {
            let child_label = format!("{}/{}", parent_label, entry.name);
            if entry.is_dir {
                let dir_info = AssetNodeInfo {
                    path: entry.path.clone(),
                    division,
                    kind: dir_kind,
                    is_dir: true,
                    is_division_root: false,
                    allow_add_folder: true,
                };
                Self::register_asset_node(cfg, &child_label, dir_info.clone());
                let node_id = Self::asset_node_id(&child_label);
                let menu = Self::dir_node_kind(&child_label, &entry.name, dir_info.kind)
                    .context_menu(|ui| {
                        self.asset_dir_context_menu(cfg, ui, node_id, &dir_info, new_folder_label)
                    });
                builder.node(menu);
                self.build_plain_directory(
                    cfg,
                    builder,
                    &entry.path,
                    &child_label,
                    division,
                    dir_kind,
                    file_kind,
                    new_folder_label,
                );
                builder.close_dir();
            } else {
                let file_info = AssetNodeInfo {
                    path: entry.path.clone(),
                    division,
                    kind: file_kind,
                    is_dir: false,
                    is_division_root: false,
                    allow_add_folder: false,
                };
                Self::register_asset_node(cfg, &child_label, file_info.clone());
                let node_id = Self::asset_node_id(&child_label);
                let menu = Self::leaf_node_kind(&child_label, &entry.name, file_info.kind)
                    .context_menu(|ui| self.asset_file_context_menu(cfg, ui, node_id, &file_info));
                builder.node(menu);
            }
        }
    }

    fn build_kotlin_tree(
        &mut self,
        cfg: &mut StaticallyKept,
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
                    AssetNodeKind::Script(ScriptDivision::Script),
                );
                return true;
            }
        };

        if entries.is_empty() {
            Self::add_placeholder_leaf(
                builder,
                &format!("{source_label}/no_kotlin_files"),
                "no kotlin files",
                AssetNodeKind::Script(ScriptDivision::Script),
            );
            return true;
        }

        let mut had_entries = false;
        for entry in entries {
            if entry.is_dir {
                self.build_kotlin_package_collapsed(
                    cfg,
                    builder,
                    &entry.path,
                    source_label,
                    vec![entry.name.clone()],
                );
                had_entries = true;
            } else {
                let file_id = format!("{}/{}", source_label, entry.name);
                let file_info = AssetNodeInfo {
                    path: entry.path.clone(),
                    division: AssetDivision::Scripts,
                    kind: AssetNodeKind::Script(ScriptDivision::Script),
                    is_dir: false,
                    is_division_root: false,
                    allow_add_folder: false,
                };
                Self::register_asset_node(cfg, &file_id, file_info.clone());
                let node_id = Self::asset_node_id(&file_id);
                let menu = Self::leaf_node_kind(&file_id, &entry.name, file_info.kind)
                    .context_menu(|ui| self.asset_file_context_menu(cfg, ui, node_id, &file_info));
                builder.node(menu);
                had_entries = true;
            }
        }

        had_entries
    }

    fn build_kotlin_package_collapsed(
        &mut self,
        cfg: &mut StaticallyKept,
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
                    AssetNodeKind::Script(ScriptDivision::Script),
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
            self.build_kotlin_package_collapsed(
                cfg,
                builder,
                &subdir.path,
                parent_path_str,
                new_parts,
            );
        } else {
            let package_suffix = accumulated_parts.join(".");
            let full_path_str = format!("{}/{}", parent_path_str, package_suffix);

            let dir_info = AssetNodeInfo {
                path: dir_path.to_path_buf(),
                division: AssetDivision::Scripts,
                kind: AssetNodeKind::Script(ScriptDivision::Package),
                is_dir: true,
                is_division_root: false,
                allow_add_folder: true,
            };
            Self::register_asset_node(cfg, &full_path_str, dir_info.clone());
            let node_id = Self::asset_node_id(&full_path_str);
            let menu = Self::dir_node_kind(&full_path_str, &package_suffix, dir_info.kind)
                .context_menu(|ui| {
                    self.asset_dir_context_menu(cfg, ui, node_id, &dir_info, "New Package")
                });
            builder.node(menu);

            for file in files {
                let file_id = format!("{}/{}", full_path_str, file.name);
                let file_info = AssetNodeInfo {
                    path: file.path.clone(),
                    division: AssetDivision::Scripts,
                    kind: AssetNodeKind::Script(ScriptDivision::Script),
                    is_dir: false,
                    is_division_root: false,
                    allow_add_folder: false,
                };
                Self::register_asset_node(cfg, &file_id, file_info.clone());
                let node_id = Self::asset_node_id(&file_id);
                let menu = Self::leaf_node_kind(&file_id, &file.name, file_info.kind)
                    .context_menu(|ui| self.asset_file_context_menu(cfg, ui, node_id, &file_info));
                builder.node(menu);
            }

            for subdir in subdirs {
                self.build_kotlin_package_collapsed(
                    cfg,
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
        &mut self,
        cfg: &mut StaticallyKept,
        builder: &mut TreeViewBuilder<u64>,
        project_root: &Path,
    ) {
        let label = "euca://scenes";
        let scenes_root = project_root.join("scenes");
        let root_info = AssetNodeInfo {
            path: scenes_root.clone(),
            division: AssetDivision::Scenes,
            kind: AssetNodeKind::Scene(SceneDivision::Folder),
            is_dir: true,
            is_division_root: true,
            allow_add_folder: true,
        };
        Self::register_asset_node(cfg, label, root_info.clone());
        let node_id = Self::asset_node_id(label);
        let menu = Self::dir_node_kind(label, "scenes", root_info.kind).context_menu(|ui| {
            self.asset_dir_context_menu(cfg, ui, node_id, &root_info, "New Folder")
        });
        builder.node(menu);
        if !scenes_root.exists() {
            Self::add_placeholder_leaf(
                builder,
                "euca://scenes/missing",
                "missing",
                AssetNodeKind::Scene(SceneDivision::Scene),
            );
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
                Self::add_placeholder_leaf(
                    builder,
                    "euca://scenes/unreadable",
                    "unreadable",
                    AssetNodeKind::Scene(SceneDivision::Scene),
                );
                builder.close_dir();
                return;
            }
        };

        let mut had_entries = false;
        for entry in entries {
            if entry.is_dir {
                let child_label = format!("{}/{}", label, entry.name);
                let dir_info = AssetNodeInfo {
                    path: entry.path.clone(),
                    division: AssetDivision::Scenes,
                    kind: AssetNodeKind::Scene(SceneDivision::Folder),
                    is_dir: true,
                    is_division_root: false,
                    allow_add_folder: true,
                };
                Self::register_asset_node(cfg, &child_label, dir_info.clone());
                let node_id = Self::asset_node_id(&child_label);
                let menu = Self::dir_node_kind(&child_label, &entry.name, dir_info.kind)
                    .context_menu(|ui| {
                        self.asset_dir_context_menu(cfg, ui, node_id, &dir_info, "New Folder")
                    });
                builder.node(menu);
                self.build_plain_directory(
                    cfg,
                    builder,
                    &entry.path,
                    &child_label,
                    AssetDivision::Scenes,
                    AssetNodeKind::Scene(SceneDivision::Folder),
                    AssetNodeKind::Scene(SceneDivision::Scene),
                    "New Folder",
                );
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
                let file_info = AssetNodeInfo {
                    path: entry.path.clone(),
                    division: AssetDivision::Scenes,
                    kind: AssetNodeKind::Scene(SceneDivision::Scene),
                    is_dir: false,
                    is_division_root: false,
                    allow_add_folder: false,
                };
                Self::register_asset_node(cfg, &file_label, file_info.clone());
                let node_id = Self::asset_node_id(&file_label);
                let menu = Self::leaf_node_kind(&file_label, &entry.name, file_info.kind)
                    .context_menu(|ui| self.asset_file_context_menu(cfg, ui, node_id, &file_info));
                builder.node(menu);
                had_entries = true;
            }
        }

        if !had_entries {
            Self::add_placeholder_leaf(
                builder,
                "euca://scenes/no_scenes",
                "no scenes",
                AssetNodeKind::Scene(SceneDivision::Scene),
            );
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

    fn register_asset_node(cfg: &mut StaticallyKept, id_source: &str, info: AssetNodeInfo) -> u64 {
        let node_id = Self::asset_node_id(id_source);
        cfg.asset_node_info.insert(node_id, info);
        node_id
    }

    fn dir_node<'ui>(label: &str) -> NodeBuilder<'ui, u64> {
        Self::with_icon_kind(
            NodeBuilder::dir(Self::asset_node_id(label)).label(label.to_string()),
            AssetNodeKind::Resource(ResourceDivision::Folder),
        )
    }

    fn dir_node_kind<'ui>(
        id_source: &str,
        label: &str,
        kind: AssetNodeKind,
    ) -> NodeBuilder<'ui, u64> {
        Self::with_icon_kind(
            NodeBuilder::dir(Self::asset_node_id(id_source)).label(label.to_string()),
            kind,
        )
    }

    fn leaf_node_kind<'ui>(
        id_source: &str,
        label: &str,
        kind: AssetNodeKind,
    ) -> NodeBuilder<'ui, u64> {
        Self::with_icon_kind(
            NodeBuilder::leaf(Self::asset_node_id(id_source)).label(label.to_string()),
            kind,
        )
    }

    fn with_icon_kind(
        builder: NodeBuilder<u64>,
        kind: AssetNodeKind,
    ) -> NodeBuilder<u64> {
        builder.icon(move |ui| {
            egui_extras::install_image_loaders(ui.ctx());
            Self::draw_asset_icon(ui, kind)
        })
    }

    fn draw_asset_icon(ui: &mut egui::Ui, _kind: AssetNodeKind) {
        let image = egui::Image::from_bytes("bytes://asset-viewer-icon", NO_TEXTURE)
            .max_size(egui::vec2(14.0, 14.0));
        ui.add(image);
    }

    fn add_placeholder_leaf(
        builder: &mut TreeViewBuilder<u64>,
        id_source: &str,
        label: &str,
        kind: AssetNodeKind,
    ) {
        builder.node(Self::leaf_node_kind(id_source, label, kind));
    }

    fn asset_dir_context_menu(
        &mut self,
        cfg: &mut StaticallyKept,
        ui: &mut egui::Ui,
        node_id: u64,
        info: &AssetNodeInfo,
        new_folder_label: &str,
    ) {
        if info.allow_add_folder {
            if ui.button(new_folder_label).clicked() {
                ui.close();
                let base_name = if info.division == AssetDivision::Scripts {
                    "newpackage"
                } else {
                    "New Folder"
                };
                self.create_asset_folder(&info.path, base_name);
            }
        }

        if ui.button("Paste").clicked() {
            ui.close();
            if !info.path.exists() {
                if let Err(err) = fs::create_dir_all(&info.path) {
                    warn!("Unable to create folder '{}': {}", info.path.display(), err);
                    return;
                }
            }
            *self.signal = Signal::AssetPaste {
                target_dir: info.path.clone(),
                division: info.division,
            };
        }

        if ui.button("Reveal Folder").clicked() {
            ui.close();
            if let Err(err) = open::that(&info.path) {
                warn!("Unable to reveal folder: {}", err);
            }
        }

        if !info.is_division_root && ui.button("Rename").clicked() {
            let current_name = info
                .path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("")
                .to_string();
            cfg.asset_rename = Some(crate::editor::AssetRenameState {
                node_id,
                original_path: info.path.clone(),
                buffer: current_name,
                just_started: true,
            });
        }

        if !info.is_division_root && ui.button("Delete").clicked() {
            ui.close();
            self.delete_asset_entry(info);
        }
    }

    fn asset_file_context_menu(
        &mut self,
        cfg: &mut StaticallyKept,
        ui: &mut egui::Ui,
        node_id: u64,
        info: &AssetNodeInfo,
    ) {
        if matches!(info.kind, AssetNodeKind::Script(ScriptDivision::Script))
            && ui.button("Open Script").clicked()
        {
            ui.close();
            if let Err(err) = open::that(&info.path) {
                warn!("Unable to open script: {}", err);
            }
        }

        if ui.button("Rename").clicked() {
            let current_name = info
                .path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("")
                .to_string();
            cfg.asset_rename = Some(crate::editor::AssetRenameState {
                node_id,
                original_path: info.path.clone(),
                buffer: current_name,
                just_started: true,
            });
        }

        if ui.button("Copy").clicked() {
            ui.close();
            *self.signal = Signal::AssetCopy {
                source: info.path.clone(),
                division: info.division,
            };
        }

        if ui.button("Delete").clicked() {
            ui.close();
            self.delete_asset_entry(info);
        }
    }

    fn apply_asset_rename(&self, rename: &crate::editor::AssetRenameState, is_dir: bool) {
        let trimmed = rename.buffer.trim();
        if trimmed.is_empty() {
            warn!("Rename cancelled: empty name");
            return;
        }

        let Some(parent) = rename.original_path.parent() else {
            warn!("Unable to rename: missing parent directory");
            return;
        };

        let mut new_name = trimmed.to_string();
        if !is_dir {
            if let Some(ext) = rename.original_path.extension().and_then(|e| e.to_str()) {
                let has_ext = Path::new(&new_name)
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some();
                if !has_ext {
                    new_name = format!("{new_name}.{ext}");
                }
            }
        }

        let target_path = parent.join(&new_name);
        if target_path == rename.original_path {
            return;
        }

        if target_path.exists() {
            warn!("Rename target already exists: {}", target_path.display());
            return;
        }

        if let Err(err) = fs::rename(&rename.original_path, &target_path) {
            warn!(
                "Failed to rename '{}': {}",
                rename.original_path.display(),
                err
            );
        } else {
            info!("Renamed to {}", target_path.display());
        }
    }

    fn delete_asset_entry(&self, info: &AssetNodeInfo) {
        if info.is_division_root {
            warn!("Cannot delete division root");
            return;
        }

        let result = if info.is_dir {
            fs::remove_dir_all(&info.path)
        } else {
            fs::remove_file(&info.path)
        };

        if let Err(err) = result {
            warn!("Failed to delete '{}': {}", info.path.display(), err);
        } else {
            info!("Deleted {}", info.path.display());
        }
    }

    fn create_asset_folder(&self, base_dir: &Path, base_name: &str) {
        if let Err(err) = fs::create_dir_all(base_dir) {
            warn!("Unable to create folder '{}': {}", base_dir.display(), err);
            return;
        }

        let mut candidate = base_dir.join(base_name);
        if candidate.exists() {
            let mut index = 1;
            loop {
                let suffix = if base_name.contains(' ') {
                    format!(" {}", index)
                } else {
                    format!("{}", index)
                };
                candidate = base_dir.join(format!("{base_name}{suffix}"));
                if !candidate.exists() {
                    break;
                }
                index += 1;
            }
        }

        if let Err(err) = fs::create_dir_all(&candidate) {
            warn!("Unable to create folder '{}': {}", candidate.display(), err);
        } else {
            info!("Created folder {}", candidate.display());
        }
    }

    fn handle_asset_move(
        &mut self,
        cfg: &mut StaticallyKept,
        drag: &egui_ltreeview::DragAndDrop<u64>,
    ) {
        let Some(&source_id) = drag.source.first() else {
            return;
        };
        let Some(source_info) = cfg.asset_node_info.get(&source_id).cloned() else {
            return;
        };
        let Some(target_info) = cfg.asset_node_info.get(&drag.target).cloned() else {
            return;
        };

        if source_info.is_division_root {
            warn!("Cannot move division root");
            return;
        }

        if source_info.division != target_info.division {
            warn!("Cannot move assets across divisions");
            return;
        }

        let target_dir = if target_info.is_dir {
            target_info.path.clone()
        } else {
            target_info
                .path
                .parent()
                .unwrap_or(&target_info.path)
                .to_path_buf()
        };

        if !target_dir.exists() {
            if let Err(err) = fs::create_dir_all(&target_dir) {
                warn!("Target directory does not exist: {}", err);
                return;
            }
        }

        if source_info.is_dir && target_dir.starts_with(&source_info.path) {
            warn!("Cannot move a folder into itself");
            return;
        }

        let Some(name) = source_info.path.file_name() else {
            warn!("Unable to move: invalid file name");
            return;
        };

        let target_path = target_dir.join(name);
        if target_path == source_info.path {
            return;
        }

        if target_path.exists() {
            warn!("Target already exists: {}", target_path.display());
            return;
        }

        if let Err(err) = fs::rename(&source_info.path, &target_path) {
            warn!("Failed to move '{}': {}", source_info.path.display(), err);
        } else {
            info!("Moved to {}", target_path.display());
        }
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
            let handle = match Model::load_from_memory_raw(
                graphics.clone(),
                buffer,
                Some(reference.clone()),
                None,
                ASSET_REGISTRY.clone(),
            )
            .await
            {
                Ok(v) => v,
                Err(e) => {
                    eucalyptus_core::warn!("Unable to load model {}: {}", reference, e);
                    return Err(e);
                }
            };

            let mut registry = ASSET_REGISTRY.write();
            if let Some(model) = registry.get_model_mut(handle) {
                model.path = reference.clone();
                model.label = label.clone();
            }
            registry.label_model(label.clone(), handle);

            Ok::<(), anyhow::Error>(())
        });
    }

    fn queue_texture_load(&self, reference: ResourceReference, label: String) {
        if ASSET_REGISTRY
            .read()
            .get_texture_handle_by_reference(&reference)
            .is_some()
        {
            eucalyptus_core::info!("Texture already loaded: {}", label);
            return;
        }

        let graphics = self.graphics.clone();
        let queue = graphics.future_queue.clone();
        queue.push(async move {
            let path = reference.resolve()?;
            let texture = match Texture::from_file(graphics.clone(), &path, Some(&label)).await {
                Ok(v) => v,
                Err(e) => {
                    eucalyptus_core::warn!("Unable to load texture {}: {}", reference, e);
                    return Err(e);
                }
            };
            let mut registry = ASSET_REGISTRY.write();
            registry.add_texture_with_label(label.clone(), texture);
            Ok::<(), anyhow::Error>(())
        });
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
                let v = if crate::features::is_enabled(crate::features::ShowComponentTypeIDInEditor) { format!(" id #{}", selection.component_type_id) } else { "".to_string() };
                eucalyptus_core::info!(
                    "Component{} ready to drop onto entity {:?}",
                    v,
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

pub struct AssetViewerDock;

impl EditorTabDock for AssetViewerDock {
    fn desc() -> EditorTabDockDescriptor {
        EditorTabDockDescriptor {
            title: "Asset Viewer".to_string(),
        }
    }

    fn display(viewer: &mut EditorTabViewer<'_>, ui: &mut egui::Ui) {
        viewer.show_asset_viewer(ui);
    }
}
