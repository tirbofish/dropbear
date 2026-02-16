use super::*;
use crate::editor::ViewportMode;
use std::{
    collections::HashMap
    ,
    hash::Hash
    ,
    path::PathBuf,
    sync::LazyLock,
};

use crate::editor::console::EucalyptusConsole;
use crate::plugin::PluginRegistry;
use dropbear_engine::utils::ResourceReference;
use dropbear_engine::entity::{EntityTransform, Transform};
use egui::{self};
use egui_dock::TabViewer;
use eucalyptus_core::traits::registry::ComponentRegistry;
use hecs::{Entity, World};
use parking_lot::Mutex;
use transform_gizmo_egui::{EnumSet, Gizmo, GizmoMode, GizmoOrientation};

pub struct EditorTabViewer<'a> {
    pub view: egui::TextureId,
    pub tex_size: Extent3d,
    pub gizmo: &'a mut Gizmo,
    pub world: &'a mut World,
    pub selected_entity: &'a mut Option<Entity>,
    pub viewport_mode: &'a mut ViewportMode,
    pub undo_stack: &'a mut Vec<UndoableAction>,
    pub signal: &'a mut Signal,
    pub gizmo_mode: &'a mut EnumSet<GizmoMode>,
    pub gizmo_orientation: &'a mut GizmoOrientation,
    pub editor_mode: &'a mut EditorState,
    pub active_camera: &'a mut Arc<Mutex<Option<Entity>>>,
    pub plugin_registry: &'a mut PluginRegistry,
    pub component_registry: &'a ComponentRegistry,
    pub build_logs: &'a mut Vec<String>,
    pub eucalyptus_console: &'a mut EucalyptusConsole,

    // "wah wah its unsafe, its using raw pointers" shut the fuck up if it breaks i will know
    pub editor: *mut Editor,
}

#[derive(Clone, Debug)]
pub struct DraggedAsset {
    pub name: String,
    pub path: ResourceReference,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComponentNodeSelection {
    pub node_id: u64,
    entity_bits: u64,
    pub component_type_id: u64,
}

impl ComponentNodeSelection {
    pub fn entity(&self) -> Option<Entity> {
        Entity::from_bits(self.entity_bits)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ComponentNodeKey {
    entity_bits: u64,
    component_type_id: u64,
}

impl ComponentNodeKey {
    fn new(entity: Entity, component_type_id: u64) -> Self {
        Self {
            entity_bits: entity.to_bits().get(),
            component_type_id,
        }
    }

    fn as_selection(&self, node_id: u64) -> ComponentNodeSelection {
        ComponentNodeSelection {
            node_id,
            entity_bits: self.entity_bits,
            component_type_id: self.component_type_id,
        }
    }
}

pub static TABS_GLOBAL: LazyLock<Mutex<StaticallyKept>> =
    LazyLock::new(|| Mutex::new(StaticallyKept::default()));

/// Variables kept statically.
///
/// The entire module (including the tab viewer) due to it
/// being part of an update/render function, therefore this is used to ensure
/// progress is not lost.
#[derive(Default)]
pub struct StaticallyKept {
    show_context_menu: bool,
    context_menu_pos: egui::Pos2,
    context_menu_tab: Option<EditorTab>,
    pub(crate) is_focused: bool,
    pub(crate) old_pos: Transform,
    pub(crate) scale_locked: bool,

    pub(crate) old_label_entity: Option<hecs::Entity>,
    pub(crate) label_original: Option<String>,
    pub(crate) label_last_edit: Option<Instant>,

    pub(crate) transform_old_entity: Option<hecs::Entity>,
    pub(crate) transform_original_transform: Option<Transform>,
    pub(crate) entity_transform_original: Option<EntityTransform>,

    pub(crate) transform_in_progress: bool,
    pub(crate) transform_rotation_cache: HashMap<Entity, glam::DVec3>,

    pub(crate) dragged_asset: Option<DraggedAsset>,
    pub(crate) asset_node_assets: HashMap<u64, DraggedAsset>,

    pub(crate) component_node_ids: HashMap<ComponentNodeKey, u64>,
    pub(crate) component_node_lookup: HashMap<u64, ComponentNodeKey>,
    pub(crate) next_component_node_id: u64,
    pub(crate) last_component_lookup: Option<ComponentNodeSelection>,
    pub(crate) pending_component_drag: Option<ComponentNodeSelection>,
    pub(crate) root_node_selected: bool,
}

impl StaticallyKept {
    pub(crate) fn next_component_node_id(&mut self) -> u64 {
        if self.next_component_node_id == 0 {
            self.next_component_node_id = 1;
        }
        let id = self.next_component_node_id;
        self.next_component_node_id = self.next_component_node_id.wrapping_add(1);
        if self.next_component_node_id == 0 {
            self.next_component_node_id = 1;
        }
        id
    }

    pub(crate) fn component_node_id(&mut self, entity: Entity, component_type_id: u64) -> u64 {
        let key = ComponentNodeKey::new(entity, component_type_id);
        if let Some(id) = self.component_node_ids.get(&key) {
            *id
        } else {
            let id = self.next_component_node_id();
            self.component_node_ids.insert(key, id);
            self.component_node_lookup.insert(id, key);
            id
        }
    }

    pub(crate) fn component_selection(&self, node_id: u64) -> Option<ComponentNodeSelection> {
        self.component_node_lookup
            .get(&node_id)
            .map(|key| key.as_selection(node_id))
    }

    pub(crate) fn remember_component_lookup(&mut self, selection: ComponentNodeSelection) {
        self.last_component_lookup = Some(selection);
    }
}

impl<'a> TabViewer for EditorTabViewer<'a> {
    type Tab = EditorTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            EditorTab::Viewport => "Viewport".into(),
            EditorTab::ModelEntityList => "Model/Entity List".into(),
            EditorTab::AssetViewer => "Asset Viewer".into(),
            EditorTab::ResourceInspector => "Resource Inspector".into(),
            EditorTab::Plugin(dock_index) => {
                if let Some((_, plugin)) = self.plugin_registry.plugins.get_index_mut(*dock_index) {
                    plugin.display_name().into()
                } else {
                    "Unknown Plugin Name".into()
                }
            }
            EditorTab::ErrorConsole => "Build Output".into(),
            EditorTab::Console => "Console".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        ui.ctx().input(|i| {
            if i.pointer.button_pressed(egui::PointerButton::Secondary)
                && let Some(pos) = i.pointer.hover_pos()
                && ui.available_rect_before_wrap().contains(pos)
            {
                let mut cfg = TABS_GLOBAL.lock();
                cfg.show_context_menu = true;
                cfg.context_menu_pos = pos;
                cfg.context_menu_tab = Some(tab.clone());
            }
        });

        match tab {
            EditorTab::Viewport => {
                self.viewport_tab(ui);
            }
            EditorTab::ModelEntityList => {
                self.entity_list(ui);
            }
            EditorTab::AssetViewer => {
                self.show_asset_viewer(ui);
            }
            EditorTab::ResourceInspector => {
                self.resource_inspector(ui);
            }
            EditorTab::Plugin(dock_info) => {
                if self.editor.is_null() {
                    panic!("Editor pointer is null, unexpected behaviour");
                }
                let editor = unsafe { &mut *self.editor };
                if let Some((_, plugin)) = self.plugin_registry.plugins.get_index_mut(*dock_info) {
                    plugin.ui(ui, editor);
                } else {
                    ui.colored_label(
                        egui::Color32::RED,
                        format!("Plugin at index '{}' not found", *dock_info),
                    );
                }
            }
            EditorTab::ErrorConsole => {
                self.build_console(ui);
            }
            EditorTab::Console => {
                ui.heading("Console");
                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Clear").clicked() {
                        self.eucalyptus_console.history.clear();
                    }

                    ui.separator();

                    ui.checkbox(&mut self.eucalyptus_console.show_info, "Info");
                    ui.checkbox(&mut self.eucalyptus_console.show_warning, "Warning");
                    ui.checkbox(&mut self.eucalyptus_console.show_error, "Error");
                    ui.checkbox(&mut self.eucalyptus_console.show_debug, "Debug");
                    ui.checkbox(&mut self.eucalyptus_console.show_trace, "Trace");

                    ui.separator();

                    ui.checkbox(&mut self.eucalyptus_console.auto_scroll, "Auto-scroll");

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("Logs: {}", self.eucalyptus_console.history.len()));
                    });
                });

                ui.separator();

                let _ = self.eucalyptus_console.take();

                let scroll = egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(self.eucalyptus_console.auto_scroll);
                
                scroll.show(ui, |ui| {
                    for log in &self.eucalyptus_console.history {
                        let is_error = log.contains("[ERROR]") || log.contains("[FATAL]");
                        let is_warn = log.contains("[WARN]");
                        let is_debug = log.contains("[DEBUG]");
                        let is_trace = log.contains("[TRACE]");
                        let is_info = !is_error && !is_warn && !is_debug && !is_trace;

                        if is_error && !self.eucalyptus_console.show_error { continue; }
                        if is_warn && !self.eucalyptus_console.show_warning { continue; }
                        if is_debug && !self.eucalyptus_console.show_debug { continue; }
                        if is_trace && !self.eucalyptus_console.show_trace { continue; }
                        if is_info && !self.eucalyptus_console.show_info { continue; }

                        let color = if is_error {
                            egui::Color32::from_rgb(255, 100, 100)
                        } else if is_warn {
                            egui::Color32::from_rgb(255, 200, 50)
                        } else if is_debug {
                            egui::Color32::from_rgb(100, 200, 255)
                        } else if is_trace {
                            egui::Color32::from_rgb(150, 150, 150)
                        } else {
                            egui::Color32::LIGHT_GRAY
                        };

                        ui.add(egui::Label::new(
                            egui::RichText::new(log)
                                .color(color)
                                .monospace()
                        ));
                    }
                });
            }
        }
    }
}

#[derive(Clone)]
pub(crate) struct FsEntry {
    pub(crate) path: PathBuf,
    pub(crate) name: String,
    pub(crate) name_lower: String,
    pub(crate) is_dir: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum EditorTabMenuAction {
    ImportResource,
    RefreshAssets,
    AddEntity,
    DeleteEntity,
    AddComponent,
    RemoveComponent,
    ViewportOption,
}
