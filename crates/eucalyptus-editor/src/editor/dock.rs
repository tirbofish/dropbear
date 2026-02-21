use super::*;
use crate::editor::ViewportMode;
use std::hash::Hasher;
use std::{any::TypeId, collections::HashMap, hash::Hash, path::PathBuf, sync::LazyLock};
use crate::editor::console::EucalyptusConsole;
use crate::plugin::PluginRegistry;
use dropbear_engine::entity::{EntityTransform, Transform};
use dropbear_engine::utils::ResourceReference;
use egui::{self};
use egui_dock::TabViewer;
use hecs::{Entity, World};
use parking_lot::Mutex;
use transform_gizmo_egui::{EnumSet, Gizmo, GizmoMode, GizmoOrientation};

pub struct EditorTabViewer<'a> {
    pub view: egui::TextureId,
    pub tex_size: Extent3d,
    pub graphics: Arc<SharedGraphicsContext>,
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
    pub tab_registry: &'a EditorTabRegistry,
    pub build_logs: &'a mut Vec<String>,
    pub eucalyptus_console: &'a mut EucalyptusConsole,
    pub current_scene_name: &'a mut Option<String>,
}

pub type EditorTabId = u64;

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
pub(crate) struct ComponentNodeKey {
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
    context_menu_tab: Option<EditorTabId>,
    pub(crate) is_focused: bool,
    pub(crate) old_pos: Transform,

    pub(crate) entity_transform_original: Option<EntityTransform>,

    pub(crate) dragged_asset: Option<DraggedAsset>,
    pub(crate) asset_node_assets: HashMap<u64, DraggedAsset>,
    pub(crate) asset_node_info: HashMap<u64, AssetNodeInfo>,
    pub(crate) asset_rename: Option<AssetRenameState>,

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

pub struct EditorTabRegistry {
    pub title_to_id: HashMap<String, EditorTabId>,
    pub descriptors: HashMap<EditorTabId, EditorTabDockDescriptor>,
    pub displayers: HashMap<EditorTabId, EditorTabDisplayer>,
}

pub type EditorTabDisplayer = Box<
    dyn for<'a> Fn(&mut EditorTabViewer<'a>, &mut egui::Ui) + Send + Sync + 'static,
>;

impl EditorTabRegistry {
    pub fn new() -> Self {
        Self {
            title_to_id: HashMap::new(),
            descriptors: HashMap::new(),
            displayers: HashMap::new(),
        }
    }

    pub fn register<D>(&mut self)
    where
        D: EditorTabDock + Send + Sync + 'static,
    {
        let desc = D::desc();
        let id = Self::id_for_type::<D>();

        self.title_to_id.insert(desc.title.to_string(), id);
        self.descriptors.insert(id, desc);
        self.displayers
            .insert(id, Box::new(|viewer, ui| D::display(viewer, ui)));
    }

    pub fn get_descriptor_by_title(&self, title: &str) -> Option<&EditorTabDockDescriptor> {
        self.title_to_id
            .get(title)
            .and_then(|tab_id| self.descriptors.get(tab_id))
    }

    pub fn id_for_title(&self, title: &str) -> Option<EditorTabId> {
        self.title_to_id.get(title).copied()
    }

    pub fn display_by_id(
        &self,
        tab_id: EditorTabId,
        viewer: &mut EditorTabViewer<'_>,
        ui: &mut egui::Ui,
    ) -> bool {
        let Some(displayer) = self.displayers.get(&tab_id) else {
            return false;
        };
        displayer(viewer, ui);
        true
    }

    fn id_for_type<T: 'static>() -> EditorTabId {
        Self::numeric_id(TypeId::of::<T>())
    }

    fn numeric_id(type_id: TypeId) -> EditorTabId {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        type_id.hash(&mut hasher);
        Self::normalize_id(hasher.finish())
    }

    fn normalize_id(id: u64) -> u64 {
        if id == 0 {
            1
        } else {
            id
        }
    }
}

impl Default for EditorTabRegistry {
    fn default() -> Self {
        Self::new()
    }
}


pub struct EditorTabDockDescriptor {
    pub title: String,
}

pub trait EditorTabDock {
    fn desc() -> EditorTabDockDescriptor;
    fn display(viewer: &mut EditorTabViewer<'_>, ui: &mut egui::Ui);
}

impl<'a> TabViewer for EditorTabViewer<'a> {
    type Tab = EditorTabId;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        self.tab_registry
            .descriptors
            .get(tab)
            .map(|desc| desc.title.clone().into())
            .unwrap_or_else(|| "Unknown Tab".into())
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

        if !self.tab_registry.display_by_id(*tab, self, ui) {
            ui.label("Unknown dock");
        }
    }
}

impl<'a> EditorTabViewer<'a> {
    pub(crate) fn console_tab(&mut self, ui: &mut egui::Ui) {
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

                if is_error && !self.eucalyptus_console.show_error {
                    continue;
                }
                if is_warn && !self.eucalyptus_console.show_warning {
                    continue;
                }
                if is_debug && !self.eucalyptus_console.show_debug {
                    continue;
                }
                if is_trace && !self.eucalyptus_console.show_trace {
                    continue;
                }
                if is_info && !self.eucalyptus_console.show_info {
                    continue;
                }

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
                    egui::RichText::new(log).color(color).monospace(),
                ));
            }
        });
    }
}

pub struct ConsoleDock;

impl EditorTabDock for ConsoleDock {
    fn desc() -> EditorTabDockDescriptor {
        EditorTabDockDescriptor {
            title: "Console".to_string(),
        }
    }

    fn display(viewer: &mut EditorTabViewer<'_>, ui: &mut egui::Ui) {
        viewer.console_tab(ui);
    }
}

#[derive(Clone)]
pub(crate) struct FsEntry {
    pub(crate) path: PathBuf,
    pub(crate) name: String,
    pub(crate) name_lower: String,
    pub(crate) is_dir: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct AssetRenameState {
    #[allow(dead_code)] // cbb to refactor just to remove this
    pub(crate) node_id: u64,
    pub(crate) original_path: PathBuf,
    pub(crate) buffer: String,
    pub(crate) just_started: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetDivision {
    Resources,
    Scripts,
    Scenes,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceDivision {
    File,
    Folder,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptDivision {
    Package,
    Script,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneDivision {
    Scene,
    Folder,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetNodeKind {
    Resource(ResourceDivision),
    Script(ScriptDivision),
    Scene(SceneDivision),
}

#[derive(Clone, Debug)]
pub(crate) struct AssetNodeInfo {
    pub(crate) path: PathBuf,
    pub(crate) division: AssetDivision,
    pub(crate) kind: AssetNodeKind,
    pub(crate) is_dir: bool,
    pub(crate) is_division_root: bool,
    pub(crate) allow_add_folder: bool,
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
