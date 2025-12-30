pub mod component;
pub mod console_error;
pub mod dock;
pub mod input;
pub mod scene;
pub mod settings;

pub(crate) use crate::editor::dock::*;

use crate::build::build;
use crate::debug;
use crate::graphics::OutlineShader;
use crate::plugin::PluginRegistry;
use crate::stats::NerdStats;
use crossbeam_channel::{unbounded, Receiver, Sender};
use dropbear_engine::entity::EntityTransform;
use dropbear_engine::mipmap::MipMapGenerator;
use dropbear_engine::shader::Shader;
use dropbear_engine::{camera::Camera, entity::{MeshRenderer, Transform}, future::FutureHandle, graphics::{RenderContext, SharedGraphicsContext}, lighting::LightManager, model::{ModelId, MODEL_CACHE}, scene::SceneCommand, DropbearWindowBuilder, WindowData};
use egui::{self, Context};
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use eucalyptus_core::{register_components, APP_INFO};
use eucalyptus_core::hierarchy::{Children, SceneHierarchy};
use eucalyptus_core::scene::{SceneConfig, SceneEntity};
use eucalyptus_core::states::{Label, SerializedMeshRenderer};
use eucalyptus_core::traits::SerializableComponent;
use eucalyptus_core::traits::registry::ComponentRegistry;
use eucalyptus_core::{
    camera::{CameraComponent, CameraType, DebugCamera},
    fatal, info,
    input::InputState,
    scripting::BuildStatus,
    states,
    states::{
        EditorTab, Script, WorldLoadingStatus, PROJECT, SCENES,
    },
    success,
    utils::ViewportMode,
    warn,
};
use hecs::{Entity, World};
use parking_lot::{Mutex, RwLock};
use rfd::FileDialog;
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::Arc,
    time::Instant,
};
use std::rc::Rc;
use tokio::sync::oneshot;
use transform_gizmo_egui::{EnumSet, Gizmo, GizmoMode, GizmoOrientation};
use wgpu::{Color, Extent3d, RenderPipeline};
use winit::window::{CursorGrabMode, WindowAttributes};
use winit::{keyboard::KeyCode, window::Window};
use winit::dpi::PhysicalSize;
use eucalyptus_core::physics::collider::{ColliderShape, WireframeGeometry};
use eucalyptus_core::physics::collider::shader::ColliderWireframePipeline;
use eucalyptus_core::properties::CustomProperties;
use crate::about::AboutWindow;

pub struct Editor {
    pub scene_command: SceneCommand,
    pub world: Box<World>,
    pub dock_state: DockState<EditorTab>,
    pub texture_id: Option<egui::TextureId>,
    pub size: Extent3d,
    pub render_pipeline: Option<RenderPipeline>,
    pub outline_pipeline: Option<OutlineShader>,
    pub collider_wireframe_pipeline: Option<ColliderWireframePipeline>,
    pub light_manager: LightManager,
    pub color: Color,

    pub mipmap_generator: Option<MipMapGenerator>,

    pub active_camera: Arc<Mutex<Option<Entity>>>,

    pub is_viewport_focused: bool,
    // is_cursor_locked: bool,
    pub window: Option<Arc<Window>>,

    pub show_new_project: bool,
    pub project_name: String,
    pub(crate) project_path: Arc<Mutex<Option<PathBuf>>>,
    pub pending_scene_switch: bool,

    pub gizmo: Gizmo,
    pub previously_selected_entity: Option<hecs::Entity>,
    pub selected_entity: Option<hecs::Entity>,
    pub viewport_mode: ViewportMode,

    pub(crate) signal: Signal,
    pub(crate) undo_stack: Vec<UndoableAction>,
    // todo: add redo (later)
    // redo_stack: Vec<UndoableAction>,
    pub(crate) editor_state: EditorState,
    pub gizmo_mode: EnumSet<GizmoMode>,
    pub gizmo_orientation: GizmoOrientation,

    // might as well save some memory if its not required...
    // #[allow(unused)] // unused to allow for JVM to startup
    // pub(crate) script_manager: ScriptManager,
    pub play_mode_backup: Option<PlayModeBackup>,

    /// State of the input
    pub(crate) input_state: Box<InputState>,

    // channels
    /// A threadsafe Unbounded Receiver, typically used for checking the status of the world loading
    pub progress_tx: Option<crossbeam_channel::Receiver<WorldLoadingStatus>>,
    /// Used to check if the world has been loaded in
    is_world_loaded: IsWorldLoadedYet,
    /// Used to fetch the current status of the loading, so it can be used for different
    /// egui loading windows or splash screens and such.
    pub current_state: WorldLoadingStatus,

    // handles for futures
    pub world_load_handle: Option<FutureHandle>,
    pub(crate) light_spawn_queue: Vec<FutureHandle>,
    pub(crate) pending_components: Vec<(hecs::Entity, FutureHandle)>,
    pub world_receiver: Option<oneshot::Receiver<hecs::World>>,

    // building
    pub progress_rx: Option<Receiver<BuildStatus>>,
    pub handle_created: Option<FutureHandle>,
    pub build_logs: Vec<String>,
    pub build_progress: f32,
    pub show_build_window: bool,
    pub last_build_error: Option<String>,
    pub show_build_error_window: bool,

    // plugins
    pub plugin_registry: PluginRegistry,

    pub dock_state_shared: Option<Arc<Mutex<DockState<EditorTab>>>>,

    // scene creation
    open_new_scene_window: bool,
    new_scene_name: String,
    current_scene_name: Option<String>,
    pending_scene_load: Option<PendingSceneLoad>,
    pending_scene_creation: Option<String>,

    // about
    nerd_stats: Rc<RwLock<NerdStats>>,

    // component registry
    component_registry: Arc<ComponentRegistry>,

    // play mode process tracking
    pub(crate) play_mode_process: Option<std::process::Child>,
    pub(crate) play_mode_pid: Option<u32>,
    pub(crate) play_mode_exit_rx: Option<std::sync::mpsc::Receiver<()>>,
}

impl Editor {
    pub fn new() -> anyhow::Result<Self> {
        let tabs = vec![EditorTab::Viewport];
        let mut dock_state = DockState::new(tabs);

        let surface = dock_state.main_surface_mut();
        let [_old, right] =
            surface.split_right(NodeIndex::root(), 0.25, vec![EditorTab::ModelEntityList]);
        let [_old, _] =
            surface.split_left(NodeIndex::root(), 0.20, vec![EditorTab::ResourceInspector]);
        let [_old, _] = surface.split_below(right, 0.5, vec![EditorTab::AssetViewer]);

        eucalyptus_core::utils::start_deadlock_detector();

        let plugin_registry = PluginRegistry::new();
        let mut component_registry = ComponentRegistry::new();

        register_components(/*&mut plugin_registry,*/ &mut component_registry);

        let component_registry = Arc::new(component_registry);

        Ok(Self {
            scene_command: SceneCommand::None,
            dock_state,
            texture_id: None,
            size: Extent3d::default(),
            render_pipeline: None,
            color: Color::default(),
            is_viewport_focused: false,
            // is_cursor_locked: false,
            window: None,
            world: Box::new(World::new()),
            show_new_project: false,
            project_name: String::new(),
            project_path: Arc::new(Mutex::new(None)),
            pending_scene_switch: false,
            gizmo: Gizmo::default(),
            previously_selected_entity: None,
            selected_entity: None,
            viewport_mode: ViewportMode::None,
            signal: Signal::None,
            undo_stack: Vec::new(),
            // script_manager: ScriptManager::new()?,
            editor_state: EditorState::Editing,
            gizmo_mode: EnumSet::empty(),
            gizmo_orientation: GizmoOrientation::Global,
            play_mode_backup: None,
            input_state: Box::new(InputState::new()),
            light_manager: LightManager::new(),
            active_camera: Arc::new(Mutex::new(None)),
            progress_tx: None,
            is_world_loaded: IsWorldLoadedYet::new(),
            current_state: WorldLoadingStatus::Idle,
            world_load_handle: None,
            light_spawn_queue: vec![],
            pending_components: vec![],
            world_receiver: None,
            progress_rx: None,
            handle_created: None,
            build_logs: Vec::new(),
            build_progress: 0.0,
            show_build_window: false,
            last_build_error: None,
            show_build_error_window: false,
            plugin_registry,
            dock_state_shared: None,
            outline_pipeline: None,
            open_new_scene_window: false,
            new_scene_name: String::new(),
            current_scene_name: None,
            pending_scene_load: None,
            pending_scene_creation: None,
            nerd_stats: Rc::new(RwLock::new(NerdStats::default())),
            component_registry,
            play_mode_process: None,
            play_mode_pid: None,
            play_mode_exit_rx: None,
            collider_wireframe_pipeline: None,
            mipmap_generator: None,
        })
    }

    fn double_key_pressed(&mut self, key: KeyCode) -> bool {
        let now = Instant::now();

        if let Some(last_time) = self.input_state.last_key_press_times.get(&key) {
            let time_diff = now.duration_since(*last_time);

            if time_diff <= self.input_state.double_press_threshold {
                self.input_state.last_key_press_times.remove(&key);
                return true;
            }
        }

        self.input_state.last_key_press_times.insert(key, now);
        false
    }

    /// Save the current world state to the active scene
    pub fn save_current_scene(&mut self) -> anyhow::Result<()> {
        let mut scenes = SCENES.write();

        if scenes.is_empty() {
            return Err(anyhow::anyhow!("No scenes loaded to save"));
        }

        let target_scene_name = self
            .current_scene_name
            .clone()
            .or_else(|| scenes.first().map(|scene| scene.scene_name.clone()))
            .ok_or_else(|| anyhow::anyhow!("Unable to determine active scene"))?;

        let scene = scenes
            .iter_mut()
            .find(|scene| scene.scene_name == target_scene_name)
            .ok_or_else(|| anyhow::anyhow!("Active scene '{}' is not loaded", target_scene_name))?;

        scene.entities.clear();
        scene.hierarchy_map = SceneHierarchy::new();
        log::debug!(
            "Reset internal hierarchy map for scene {}",
            scene.scene_name
        );

        let labels = self
            .world
            .query::<&Label>()
            .iter()
            .map(|(e, l)| (e, l.clone()))
            .collect::<Vec<_>>();

        for (id, label) in labels {
            let entity_label = label.clone();

            let components = self
                .component_registry
                .extract_all_components(&self.world, id);

            if let Ok(children_comp) = self.world.get::<&Children>(id) {
                for &child_entity in children_comp.children() {
                    if let Ok(child_label) = self.world.get::<&Label>(child_entity) {
                        scene
                            .hierarchy_map
                            .set_parent(Label::new(child_label.as_str()), entity_label.clone());
                    } else {
                        log::warn!(
                            "Unable to resolve child entity {:?} for parent '{}' when saving scene",
                            child_entity,
                            entity_label
                        );
                    }
                }
            }

            let scene_entity = SceneEntity {
                label: entity_label.clone(),
                components,
                entity_id: Some(id),
            };

            scene.entities.push(scene_entity);
            log::debug!("Saved entity: {}", entity_label);
        }

        log::info!(
            "Saved {} entities to scene '{}'",
            scene.entities.len(),
            scene.scene_name
        );

        Ok(())
    }

    fn persist_active_scene_to_disk(&self) -> anyhow::Result<()> {
        let target_scene_name = self.current_scene_name.clone().or_else(|| {
            let scenes = SCENES.read();
            scenes.first().map(|scene| scene.scene_name.clone())
        });

        let Some(scene_name) = target_scene_name else {
            return Ok(());
        };

        let scene_clone = {
            let scenes = SCENES.read();
            scenes
                .iter()
                .find(|scene| scene.scene_name == scene_name)
                .cloned()
        };

        let Some(scene_clone) = scene_clone else {
            log::warn!(
                "Attempted to persist scene '{}' but it is not loaded",
                scene_name
            );
            return Ok(());
        };

        let project_path = {
            let project = PROJECT.read();
            project.project_path.clone()
        };

        scene_clone.write_to(&project_path)?;
        Ok(())
    }

    pub fn save_project_config(&mut self) -> anyhow::Result<()> {
        self.save_current_scene()?;
        self.persist_active_scene_to_disk()?;

        {
            let mut config = PROJECT.write();
            let dock_state = self.dock_state.clone();
            config.dock_layout = Some(dock_state);
        }

        {
            let mut config = PROJECT.write();
            config.write_to_all()?;
        }

        Ok(())
    }

    /// The window when loading a project or a scene or anything that uses [`WorldLoadingStatus`]
    fn show_project_loading_window(&mut self, ctx: &egui::Context) {
        if let Some(ref mut rx) = self.progress_tx {
            match rx.try_recv() {
                Ok(status) => match status {
                    WorldLoadingStatus::LoadingEntity { index, name, total } => {
                        log::debug!("Loading entity: {} ({}/{})", name, index + 1, total);
                        self.current_state =
                            WorldLoadingStatus::LoadingEntity { index, name, total };
                    }
                    WorldLoadingStatus::Completed => {
                        log::debug!(
                            "Received WorldLoadingStatus::Completed - project loading finished"
                        );
                        self.is_world_loaded.mark_project_loaded();
                        self.current_state = WorldLoadingStatus::Completed;
                        self.progress_tx = None;
                        log::debug!("Returning back");
                        return;
                    }
                    WorldLoadingStatus::Idle => {
                        log::debug!("Project loading is idle");
                    }
                },
                Err(_) => {
                    // log::debug!("Unable to receive the progress: {}", e);
                }
            }
        } else {
            log::debug!("No progress receiver available");
        }

        egui::Window::new("Loading Project")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .fixed_size([300.0, 100.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Loading...");
                    });
                    // ui.add_space(5.0);
                    // ui.add(egui::ProgressBar::new(progress).text(format!("{:.0}%", progress * 100.0)));
                    match &self.current_state {
                        WorldLoadingStatus::Idle => {
                            ui.label("Initialising...");
                        }
                        WorldLoadingStatus::LoadingEntity { name, .. } => {
                            ui.label(format!("Loading entity: {}", name));
                        }
                        WorldLoadingStatus::Completed => {
                            ui.label("Done!");
                        }
                    }
                });
            });
    }

    /// Loads the project config.
    ///
    /// It uses an unbounded sender to send messages back to the receiver so it can
    /// be used within threads.
    pub async fn load_project_config(
        graphics: Arc<SharedGraphicsContext>,
        sender: Option<Sender<WorldLoadingStatus>>,
        world: &mut World,
        world_sender: Option<oneshot::Sender<hecs::World>>,
        active_camera: Arc<Mutex<Option<hecs::Entity>>>,
        project_path: Arc<Mutex<Option<PathBuf>>>,
        dock_state: Arc<Mutex<DockState<EditorTab>>>,
        component_registry: Arc<ComponentRegistry>,
    ) -> anyhow::Result<()> {
        {
            let config = PROJECT.read();
            let mut path = project_path.lock();
            *path = Some(config.project_path.clone());

            if let Some(layout) = &config.dock_layout {
                let mut dock = dock_state.lock();
                let layout = layout.clone();
                *dock = layout.clone();
            }
        }

        let last_scene = {
            let config = PROJECT.read();
            config.last_opened_scene.clone()
        };

        let first_scene_opt = {
            let scenes = SCENES.read();
            if let Some(scene_name) = last_scene {
                scenes
                    .iter()
                    .find(|scene| scene.scene_name == scene_name)
                    .cloned()
            } else {
                scenes.first().cloned()
            }
        };

        {
            if let Some(mut first_scene) = first_scene_opt {
                let cam = first_scene
                    .load_into_world(
                        world,
                        graphics,
                        Some(component_registry.as_ref()),
                        sender.clone(),
                        false,
                    )
                    .await?;
                let mut a_c = active_camera.lock();
                *a_c = Some(cam);

                log::info!(
                    "Successfully loaded scene with {} entities",
                    first_scene.entities.len(),
                );
            } else {
                let existing_debug_camera = {
                    world
                        .query::<(&Camera, &CameraComponent)>()
                        .iter()
                        .find_map(|(entity, (_, component))| {
                            if matches!(component.camera_type, CameraType::Debug) {
                                Some(entity)
                            } else {
                                None
                            }
                        })
                };

                if let Some(camera_entity) = existing_debug_camera {
                    log::info!("Using existing debug camera");
                    let mut a_c = active_camera.lock();
                    *a_c = Some(camera_entity);
                } else {
                    log::info!("No scenes found, creating default debug camera");

                    let debug_camera = Camera::predetermined(graphics, Some("Debug Camera"));
                    let component = DebugCamera::new();

                    {
                        let e = world.spawn((Label::from("Debug Camera"), debug_camera, component));
                        let mut a_c = active_camera.lock();
                        *a_c = Some(e);
                    }
                }
            }
        }

        if let Some(ref s) = sender.clone() {
            let _ = s.send(WorldLoadingStatus::Completed);
        }

        if let Some(ws) = world_sender {
            let _ = ws.send(std::mem::take(world));
        }

        Ok(())
    }

    pub(crate) fn queue_scene_load_by_name(&mut self, scene_name: &str) -> anyhow::Result<()> {
        if scene_name.trim().is_empty() {
            return Err(anyhow::anyhow!("Scene name cannot be empty"));
        }

        let should_persist_current = self.current_scene_name.is_some()
            && self.is_world_loaded.is_fully_loaded()
            && self.world.len() > 0
            && {
                let scenes = SCENES.read();
                !scenes.is_empty()
            };

        if should_persist_current {
            self.save_current_scene()?;
            self.persist_active_scene_to_disk()?;
        }

        if let Some(current) = self.current_scene_name.as_deref() {
            states::unload_scene(current);
        }

        let scene = states::load_scene(scene_name)?;

        {
            let mut scenes = SCENES.write();
            scenes.retain(|existing| existing.scene_name != scene.scene_name);
            scenes.insert(0, scene.clone());
        }

        {
            let mut project = PROJECT.write();
            project.last_opened_scene = Some(scene.scene_name.clone());
            project.write_to_all()?;
        }

        log::info!("Scene '{}' staged for loading", scene.scene_name);

        self.current_scene_name = Some(scene.scene_name.clone());
        self.pending_scene_load = Some(PendingSceneLoad { scene });

        Ok(())
    }

    fn cleanup_scene_resources(&mut self, graphics: &mut RenderContext) {
        if let Some(handle) = self.world_load_handle.take() {
            graphics.shared.future_queue.cancel(&handle);
        }

        self.light_spawn_queue.clear();
        self.progress_tx = None;
        self.world_receiver = None;
        self.current_state = WorldLoadingStatus::Idle;

        self.world.clear();
        self.selected_entity = None;
        self.previously_selected_entity = None;
        self.active_camera.lock().take();

        self.render_pipeline = None;
        self.outline_pipeline = None;
        self.texture_id = None;
        self.light_manager = LightManager::new();

        {
            let mut cache = MODEL_CACHE.lock();
            cache.clear();
        }
    }

    fn start_async_scene_load(&mut self, mut scene: SceneConfig, graphics: &mut RenderContext) {
        self.cleanup_scene_resources(graphics);

        let (progress_sender, progress_receiver) =
            unbounded::<WorldLoadingStatus>();
        self.progress_tx = Some(progress_receiver);
        self.current_state = WorldLoadingStatus::Idle;

        let (world_sender, world_receiver) = oneshot::channel();
        self.world_receiver = Some(world_receiver);

        self.is_world_loaded = IsWorldLoadedYet::new();
        self.is_world_loaded.mark_scene_loaded();

        let graphics_shared = graphics.shared.clone();
        let active_camera = self.active_camera.clone();
        let scene_name = scene.scene_name.clone();
        let component_registry_clone = self.component_registry.clone();

        let handle = graphics.shared.future_queue.push(async move {
            let mut temp_world = World::new();

            let load_result = scene
                .load_into_world(
                    &mut temp_world,
                    graphics_shared.clone(),
                    Some(component_registry_clone.as_ref()),
                    Some(progress_sender.clone()),
                    false,
                )
                .await;

            match load_result {
                Ok(active_entity) => {
                    let mut camera_lock = active_camera.lock();
                    *camera_lock = Some(active_entity);
                }
                Err(err) => {
                    log::error!("Failed to load scene '{}': {}", scene_name, err);
                }
            }

            let _ = progress_sender.send(WorldLoadingStatus::Completed);

            if world_sender.send(temp_world).is_err() {
                log::error!("Failed to deliver loaded world for scene '{}'", scene_name);
            }
        });

        self.world_load_handle = Some(handle);
    }

    fn create_new_scene(&mut self, name: &str) -> anyhow::Result<()> {
        let trimmed_name = name.trim();
        if trimmed_name.is_empty() {
            return Err(anyhow::anyhow!("Scene name cannot be empty"));
        }

        if trimmed_name.contains('/') || trimmed_name.contains('\\') || trimmed_name.contains(':') {
            return Err(anyhow::anyhow!(
                "Scene name cannot contain path separator characters"
            ));
        }

        let scene_name_owned = trimmed_name.to_string();

        let project_root = {
            let cfg = PROJECT.read();
            cfg.project_path.clone()
        };

        if project_root.as_os_str().is_empty() {
            return Err(anyhow::anyhow!("Project path is not set"));
        }

        let scenes_dir = project_root.join("scenes");
        if !scenes_dir.exists() {
            fs::create_dir_all(&scenes_dir)?;
        }

        let target_path = scenes_dir.join(format!("{}.eucs", scene_name_owned));
        if target_path.exists() {
            return Err(anyhow::anyhow!(
                "Scene '{}' already exists",
                scene_name_owned
            ));
        }

        let scene_config = SceneConfig::new(scene_name_owned.clone(), &target_path);
        scene_config.write_to(&project_root)?;

        self.queue_scene_load_by_name(&scene_name_owned)?;
        success!("Created scene '{}'", scene_name_owned);
        Ok(())
    }

    fn open_scene_from_path(&mut self, path: PathBuf) -> anyhow::Result<()> {
        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("eucs"))
            != Some(true)
        {
            return Err(anyhow::anyhow!("Selected file is not an .eucs scene"));
        }

        let project_root = {
            let cfg = PROJECT.read();
            cfg.project_path.clone()
        };

        if project_root.as_os_str().is_empty() {
            return Err(anyhow::anyhow!("Project path is not set"));
        }

        let scenes_dir = project_root.join("scenes");
        if !path.starts_with(&scenes_dir) {
            return Err(anyhow::anyhow!(
                "Scene '{}' is outside of the current project",
                path.display()
            ));
        }

        let scene_name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| anyhow::anyhow!("Scene file name is invalid"))?;

        self.queue_scene_load_by_name(scene_name)?;
        info!("Queued scene '{}' for loading", scene_name);
        Ok(())
    }

    pub fn show_ui(&mut self, ctx: &Context) {
        if let Some(scene_name) = self.pending_scene_creation.take() {
            let result = self.create_new_scene(scene_name.as_str());
            self.new_scene_name.clear();
            if let Err(e) = result {
                fatal!("Failed to create scene '{}': {}", scene_name, e);
            }
        }

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {

                    if ui.button("New Scene").clicked() {
                        self.open_new_scene_window = true;
                    }

                    if ui.button("Open Scene").clicked() {
                        let scenes_dir = {
                            let project = PROJECT.read();
                            project.project_path.join("scenes")
                        };

                        let mut dialog = FileDialog::new();
                        if scenes_dir.exists() {
                            dialog = dialog.set_directory(&scenes_dir);
                        }

                        let dialog = dialog.add_filter("Eucalyptus Scenes", &["eucs"]);

                        if let Some(path) = dialog.pick_file() {
                            if let Err(e) = self.open_scene_from_path(path) {
                                fatal!("Failed to open scene: {}", e);
                            }
                        }
                    }

                    if ui.button("Save").clicked() {
                        match self.save_project_config() {
                            Ok(_) => {}
                            Err(e) => {
                                fatal!("Error saving project: {}", e);
                            }
                        }
                        success!("Successfully saved project");
                    }
                    if ui.button("Reveal project").clicked() {
                        let project_path = {
                            PROJECT.read().project_path.clone()
                        };
                        match open::that(project_path) {
                            Ok(()) => info!("Revealed project"),
                            Err(e) => warn!("Unable to open project: {}", e),
                        }
                    }
                    if ui.button("Project Settings").clicked() {};
                    if matches!(self.editor_state, EditorState::Playing) {
                        if ui.button("Stop").clicked() {
                            self.signal = Signal::StopPlaying;
                        }
                    } else if ui.button("Play").clicked() {
                        self.signal = Signal::Play;
                    }
                    ui.menu_button("Export", |ui| {
                        // todo: create a window for better build menu
                        if ui.button("Build").clicked() {
                            {
                                let proj = PROJECT.read();
                                match build(proj.project_path.join(format!("{}.eucp", proj.project_name.clone())).clone()) {
                                    Ok(thingy) => success!("Project output at {}", thingy.display()),
                                    Err(e) => {
                                        fatal!("Unable to build project [{}]: {}", proj.project_path.clone().display(), e);
                                    },
                                }
                            }
                        }
                        ui.label("Package"); // todo: create a window for label
                    });
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        match self.save_project_config() {
                            Ok(_) => {
                                log::info!("Saved, quitting...");
                                std::process::exit(0);
                            }
                            Err(e) => {
                                fatal!("Error saving project: {}", e);
                            }
                        }
                        success!("Successfully saved project");
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Copy").clicked() {
                        if let Some(entity) = &self.selected_entity {
                            let query = self.world.query_one::<(
                                &Label,
                                &MeshRenderer,
                                &EntityTransform,
                                &CustomProperties,
                            )>(*entity);
                            if let Ok(mut q) = query {
                                if let Some((entity_label, renderer, transform, props)) = q.get() {
                                    let mut components: Vec<Box<dyn SerializableComponent>> = Vec::new();

                                    components.push(Box::new(*transform));

                                    let serialized_renderer = SerializedMeshRenderer {
                                        handle: renderer.handle().path.clone(),
                                        material_override: renderer.material_overrides().to_vec(),
                                    };
                                    components.push(Box::new(serialized_renderer));

                                    components.push(Box::new(props.clone()));

                                    let s_entity = SceneEntity {
                                        label: entity_label.clone(),
                                        components,
                                        entity_id: None,
                                    };
                                    self.signal = Signal::Copy(s_entity);

                                    info!("Copied selected entity!");
                                } else {
                                    warn!("Unable to copy entity: Unable to fetch world entity properties");
                                }
                            } else {
                                warn!("Unable to copy entity: Unable to obtain lock");
                            }
                        } else {
                            warn!("Unable to copy entity: None selected");
                        }

                    }

                    if ui.button("Paste").clicked() {
                        match &self.signal {
                            Signal::Copy(entity) => {
                                self.signal = Signal::Paste(entity.clone());
                            }
                            _ => {
                                warn!("Unable to paste: You haven't selected anything!");
                            }
                        }
                    }

                    if ui.button("Undo").clicked() {
                        self.signal = Signal::Undo;
                    }
                    ui.label("Redo");
                });

                ui.menu_button("Window", |ui_window| {
                    if ui_window.button("Open Asset Viewer").clicked() {
                        self.dock_state.push_to_focused_leaf(EditorTab::AssetViewer);
                    }
                    if ui_window.button("Open Resource Inspector").clicked() {
                        self.dock_state
                            .push_to_focused_leaf(EditorTab::ResourceInspector);
                    }
                    if ui_window.button("Open Entity List").clicked() {
                        self.dock_state
                            .push_to_focused_leaf(EditorTab::ModelEntityList);
                    }
                    if ui_window.button("Open Viewport").clicked() {
                        self.dock_state.push_to_focused_leaf(EditorTab::Viewport);
                    }
                    if ui_window.button("Open Error Console").clicked() {
                        self.dock_state.push_to_focused_leaf(EditorTab::ErrorConsole);
                    }
                    if self.plugin_registry.plugins.len() == 0 {
                        ui_window.label(
                            egui::RichText::new("No plugins ")
                                .color(ui_window.visuals().weak_text_color())
                        );
                    }
                    for (i, (_, plugin)) in self.plugin_registry.plugins.iter().enumerate() {
                        if ui_window.button(format!("Open {}", plugin.display_name())).clicked() {
                            self.dock_state.push_to_focused_leaf(EditorTab::Plugin(i));
                        }
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("Show AppData folder").clicked() {
                        match app_dirs2::app_root(app_dirs2::AppDataType::UserData, &APP_INFO) {
                            Ok(val) => {
                                match open::that(&val) {
                                    Ok(()) => info!("Opened logs folder"),
                                    Err(e) => fatal!("Unable to open {}: {}", val.display(), e)
                                }
                            },
                            Err(e) => {
                                fatal!("Unable to show logs: {}", e);
                            },
                        };
                    }

                    if ui.button("Nerd Stats").clicked() {
                        log::debug!("Requested nerd stats window");
                        let window_data = DropbearWindowBuilder::new()
                            .with_attributes(
                                WindowAttributes::default()
                                    .with_title("Nerd Stats")
                                    .with_inner_size(PhysicalSize::new(500, 600))
                            )
                            .add_scene_with_input(self.nerd_stats.clone(), "nerd_stats")
                            .set_initial_scene("nerd_stats")
                            .build();

                        self.scene_command = SceneCommand::RequestWindow(window_data);
                    }

                    if ui.button("About").clicked() {
                        log::debug!("About window requested to be opened");
                        let about = Rc::new(RwLock::new(AboutWindow::new()));
                        let window = DropbearWindowBuilder::new()
                            .with_attributes(
                                WindowAttributes::default()
                                    .with_title("About eucalyptus editor")
                                    .with_inner_size(PhysicalSize::new(500, 300))
                                    .with_resizable(false)
                            )
                            .add_scene_with_input(about, "about")
                            .set_initial_scene("about")
                            .build();
                        self.scene_command = SceneCommand::RequestWindow(window);
                    }
                });

                {
                    let cfg = PROJECT.read();
                    if cfg.editor_settings.is_debug_menu_shown {
                        debug::show_menu_bar(ui, &mut self.signal);
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let can_play = matches!(self.editor_state, EditorState::Playing);
                    ui.group(|ui| {
                        ui.add_enabled_ui(can_play, |ui| {
                            if ui.button("⏹").clicked() {
                                log::debug!("Menu button Stop button pressed");
                                self.signal = Signal::StopPlaying;
                            }
                        });

                        ui.add_enabled_ui(!can_play, |ui| {
                            if ui.button("▶").clicked() {
                                log::debug!("Menu Button Play button pressed");
                                self.signal = Signal::Play;
                            }
                        });
                    });
                });
            });
        });

        let editor_ptr = self as *mut Editor;

        egui::CentralPanel::default().show(ctx, |ui| {
            DockArea::new(&mut self.dock_state)
                .style(Style::from_egui(ui.style().as_ref()))
                .show_inside(
                    ui,
                    &mut EditorTabViewer {
                        view: self.texture_id.unwrap(),
                        gizmo: &mut self.gizmo,
                        tex_size: self.size,
                        world: &mut self.world,
                        selected_entity: &mut self.selected_entity,
                        viewport_mode: &mut self.viewport_mode,
                        undo_stack: &mut self.undo_stack,
                        signal: &mut self.signal,
                        active_camera: &mut self.active_camera,
                        gizmo_mode: &mut self.gizmo_mode,
                        gizmo_orientation: &mut self.gizmo_orientation,
                        editor_mode: &mut self.editor_state,
                        plugin_registry: &mut self.plugin_registry,
                        editor: editor_ptr,
                        build_logs: &mut self.build_logs,
                        component_registry: &self.component_registry,
                    },
                );
        });

        let mut project_path = self.project_path.lock();
        crate::utils::show_new_project_window(
            ctx,
            &mut self.show_new_project,
            &mut self.project_name,
            &mut project_path,
            |name, path| {
                crate::utils::start_project_creation(name.to_string(), Some(path.clone()));
                self.pending_scene_switch = true;
            },
        );

        if self.pending_scene_switch {
            self.scene_command = SceneCommand::SwitchScene("editor".to_string());
            self.pending_scene_switch = false;
        }

        let mut open_flag = self.open_new_scene_window;
        let mut close_requested = false;
        if open_flag {
            egui::Window::new("New Scene")
                .open(&mut open_flag)
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.label("Name: ");
                        ui.text_edit_singleline(&mut self.new_scene_name);
                        if ui.button("Create").clicked() {
                            self.pending_scene_creation = Some(self.new_scene_name.clone());
                            close_requested = true;
                        }
                    });
                });
        }

        if close_requested {
            open_flag = false;
        }

        self.open_new_scene_window = open_flag;
    }

    /// Restores transform components back to its original state before PlayMode.
    pub fn restore(&mut self) -> anyhow::Result<()> {
        if let Some(window) = &self.window {
            let _ = window.set_cursor_grab(CursorGrabMode::None);
        }

        if let Some(backup) = &self.play_mode_backup {
            for (
                entity_id,
                original_mesh_renderer,
                original_transform,
                original_properties,
                original_script,
            ) in &backup.entities
            {
                if let Ok(mut mesh_renderer) = self.world.get::<&mut MeshRenderer>(*entity_id) {
                    mesh_renderer.clone_from(original_mesh_renderer);
                    mesh_renderer.sync_asset_registry();
                }

                if let Ok(mut transform) = self.world.get::<&mut EntityTransform>(*entity_id) {
                    *transform = *original_transform;
                }

                if let Ok(mut properties) = self.world.get::<&mut CustomProperties>(*entity_id) {
                    properties.clone_from(original_properties);
                }

                let has_script = self.world.get::<&Script>(*entity_id).is_ok();
                match (has_script, original_script) {
                    (true, Some(original)) => {
                        if let Ok(mut script) = self.world.get::<&mut Script>(*entity_id) {
                            *script = original.clone();
                        }
                    }
                    (true, None) => {
                        let _ = self.world.remove_one::<Script>(*entity_id);
                    }
                    (false, Some(original)) => {
                        let _ = self.world.insert_one(*entity_id, original.clone());
                    }
                    (false, None) => {}
                }
            }

            for (entity_id, original_camera, original_camera_component) in &backup.camera_data {
                if let Ok(mut camera) = self.world.get::<&mut Camera>(*entity_id) {
                    *camera = original_camera.clone();
                }

                if let Ok(mut camera_component) = self.world.get::<&mut CameraComponent>(*entity_id)
                {
                    *camera_component = original_camera_component.clone();
                }
            }

            log::info!("Restored scene from play mode backup");

            self.play_mode_backup = None;
            Ok(())
        } else {
            Err(anyhow::anyhow!("No play mode backup found to restore"))
        }
    }

    pub fn create_backup(&mut self) -> anyhow::Result<()> {
        let mut entities = Vec::new();

        for (entity_id, (mesh_renderer, transform, properties)) in self
            .world
            .query::<(&MeshRenderer, &EntityTransform, &CustomProperties)>()
            .iter()
        {
            let script = self
                .world
                .query_one::<&Script>(entity_id)
                .ok()
                .and_then(|mut s| s.get().cloned());
            entities.push((
                entity_id,
                mesh_renderer.clone(),
                *transform,
                properties.clone(),
                script,
            ));
        }

        let mut camera_data = Vec::new();

        for (entity_id, (camera, component)) in
            self.world.query::<(&Camera, &CameraComponent)>().iter()
        {
            camera_data.push((entity_id, camera.clone(), component.clone()));
        }

        let backup = PlayModeBackup {
            entities,
            camera_data,
        };

        let entity_count = backup.entities.len();
        let camera_count = backup.camera_data.len();
        self.play_mode_backup = Some(backup);

        log::info!(
            "Created play mode backup with {} entities and {} cameras",
            entity_count,
            camera_count
        );
        Ok(())
    }

    pub fn switch_to_debug_camera(&mut self) {
        let debug_camera = self
            .world
            .query::<(&Camera, &CameraComponent)>()
            .iter()
            .find_map(|(e, (_cam, comp))| {
                if matches!(comp.camera_type, CameraType::Debug) {
                    Some(e)
                } else {
                    None
                }
            });

        if let Some(camera_entity) = debug_camera {
            let mut active_camera = self.active_camera.lock();
            *active_camera = Some(camera_entity);
            info!("Switched to debug camera");
        } else {
            warn!("No debug camera found in the world");
        }
    }

    pub fn switch_to_player_camera(&mut self) {
        let player_camera = self
            .world
            .query::<(&Camera, &CameraComponent)>()
            .iter()
            .find_map(
                |(e, (_cam, comp))| {
                    if comp.starting_camera { Some(e) } else { None }
                },
            );

        if let Some(camera_entity) = player_camera {
            let mut active_camera = self.active_camera.lock();
            *active_camera = Some(camera_entity);
            info!("Switched to player camera");
        } else {
            warn!("No player camera found in the world");
        }
    }

    pub fn is_using_debug_camera(&self) -> bool {
        let active_camera = self.active_camera.lock();
        if let Some(active_camera_entity) = *active_camera
            && let Ok(mut query) = self
                .world
                .query_one::<&CameraComponent>(active_camera_entity)
            && let Some(component) = query.get()
        {
            return matches!(component.camera_type, CameraType::Debug);
        }
        false
    }

    /// Loads all the wgpu resources such as renderer.
    ///
    /// **Note**: To be ran AFTER [`Editor::load_project_config`]
    pub fn load_wgpu_nerdy_stuff<'a>(&mut self, graphics: &mut RenderContext<'a>) {
        // log::debug!("Contents of viewport shader: \n{:#?}", dropbear_engine::shader::shader_wesl::SHADER_SHADER);
        let shader = Shader::new(
            graphics.shared.clone(),
            dropbear_engine::shader::shader_wesl::SHADER_SHADER,
            Some("viewport_shader"),
        );

        self.light_manager
            .create_light_array_resources(graphics.shared.clone());

        if let Some(active_camera) = *self.active_camera.lock() {
            if let Ok(mut q) = self
                .world
                .query_one::<(&Camera, &CameraComponent)>(active_camera)
            {
                if let Some((camera, _component)) = q.get() {
                    let pipeline = graphics.create_render_pipline(
                        &shader,
                        vec![
                            &graphics.shared.texture_bind_layout.clone(),
                            camera.layout(),
                            self.light_manager.layout(),
                        ],
                        None,
                    );
                    self.render_pipeline = Some(pipeline);

                    // log::debug!("Contents of light shader: \n{:#?}", dropbear_engine::shader::shader_wesl::LIGHT_SHADER);
                    self.light_manager.create_render_pipeline(
                        graphics.shared.clone(),
                        dropbear_engine::shader::shader_wesl::LIGHT_SHADER,
                        camera,
                        Some("Light Pipeline"),
                    );

                    // log::debug!("Contents of outline shader: \n{:#?}", dropbear_engine::shader::shader_wesl::OUTLINE_SHADER);
                    let outline_shader =
                        OutlineShader::init(graphics.shared.clone(), camera.layout());
                    self.outline_pipeline = Some(outline_shader);

                    let collider_pipeline = ColliderWireframePipeline::new(graphics.shared.clone(), camera.layout());
                    self.collider_wireframe_pipeline = Some(collider_pipeline);

                    self.mipmap_generator = Some(MipMapGenerator::new(graphics.shared.clone()))
                } else {
                    log_once::warn_once!(
                        "Unable to fetch the query result of camera: {:?}",
                        active_camera
                    )
                }
            } else {
                log_once::warn_once!(
                    "Unable to query camera, component and option<camerafollowtarget> for active camera: {:?}",
                    active_camera
                );
            }
        } else {
            log_once::warn_once!("No active camera found");
        }

        self.window = Some(graphics.shared.window.clone());
        self.is_world_loaded.mark_rendering_loaded();
    }

    /// Initialises another eucalyptus-editor play mode app as a separate process and monitors it in a separate thread.
    pub fn load_play_mode(&mut self) -> anyhow::Result<()> {
        use std::process::{Command, Stdio};
        use std::sync::mpsc::channel;
        use std::thread;

        let current_exe = std::env::current_exe()
            .map_err(|e| anyhow::anyhow!("Failed to get current executable path: {}", e))?;

        let project_dir = {
            let cfg = PROJECT.read();
            cfg.project_path.clone()
        };

        let current_scene = self.current_scene_name.clone().ok_or_else(|| {
            anyhow::anyhow!("No current scene loaded; cannot launch play mode")
        })?;

        log::info!("Launching play mode: {} play --project {:?}",
            current_exe.display(), project_dir);

        let mut child = Command::new(&current_exe)
            .arg("play")
            .arg(&project_dir)
            .arg(&current_scene)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to spawn play mode process: {}", e))?;

        let pid = child.id();
        log::info!("Play mode process started with PID: {}", pid);
        success!("Play mode launched (PID: {})", pid);

        let (tx, rx) = channel();

        thread::spawn(move || {
            log::debug!("Watch thread started for play mode process {}", pid);

            match child.wait() {
                Ok(status) => {
                    log::info!("Play mode process {} exited with status: {}", pid, status);

                    if !status.success() {
                        fatal!("Play mode exited early: {:?}", status.code())
                    }

                    if let Err(e) = tx.send(()) {
                        log::error!("Failed to send play mode exit notification: {}", e);
                    }
                }
                Err(e) => {
                    log::error!("Error waiting for play mode process {}: {}", pid, e);
                    let _ = tx.send(());
                }
            }

            log::debug!("Watch thread for play mode process {} terminated", pid);
        });

        self.play_mode_process = None;
        self.play_mode_pid = Some(pid);
        self.play_mode_exit_rx = Some(rx);
        self.editor_state = EditorState::Playing;

        Ok(())
    }
}

/// Describes an action that is undoable
#[derive(Debug)]
pub enum UndoableAction {
    /// A change in transform. The entity + the old transform. Undoing will revert the transform
    Transform(hecs::Entity, Transform),
    /// A change in EntityTransform. The entity + the old transform. Undoing will revert the transform
    EntityTransform(hecs::Entity, EntityTransform),
    #[allow(dead_code)] // don't know why its considered dead code, todo: check the cause
    /// A spawn of the entity. Undoing will delete the entity
    Spawn(hecs::Entity),
    /// A change of label of the entity. Undoing will revert its label
    Label(hecs::Entity, String),
    RemoveStartingCamera(Entity),
}

impl UndoableAction {
    pub fn push_to_undo(undo_stack: &mut Vec<UndoableAction>, action: Self) {
        undo_stack.push(action);
        // log::debug!("Undo Stack contents: {:#?}", undo_stack);
    }

    pub fn undo(&self, world: &mut World) -> anyhow::Result<()> {
        match self {
            UndoableAction::Transform(entity, transform) => {
                if let Ok(mut q) = world.query_one::<&mut Transform>(*entity) {
                    if let Some(e_t) = q.get() {
                        *e_t = *transform;
                        log::debug!("Reverted transform");
                        Ok(())
                    } else {
                        Err(anyhow::anyhow!("Unable to query the entity"))
                    }
                } else {
                    Err(anyhow::anyhow!("Could not find an entity to query"))
                }
            }
            UndoableAction::EntityTransform(entity, transform) => {
                if let Ok(mut q) = world.query_one::<&mut EntityTransform>(*entity) {
                    if let Some(e_t) = q.get() {
                        *e_t = *transform;
                        log::debug!("Reverted entity transform");
                        Ok(())
                    } else {
                        Err(anyhow::anyhow!("Unable to query the entity"))
                    }
                } else {
                    Err(anyhow::anyhow!("Could not find an entity to query"))
                }
            }
            UndoableAction::Spawn(entity) => {
                if world.despawn(*entity).is_ok() {
                    log::debug!("Undid spawn by despawning entity {:?}", entity);
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Failed to despawn entity {:?}", entity))
                }
            }
            UndoableAction::Label(entity, original_label) => {
                if let Ok(label) = world.query_one_mut::<&mut Label>(*entity) {
                    label.set(original_label.clone());
                    Ok(())
                } else {
                    anyhow::bail!("No entity found (with or without the Label property)");
                }
            }
            UndoableAction::RemoveStartingCamera(old) => {
                for (_i, comp) in &mut world.query::<&mut CameraComponent>() {
                    comp.starting_camera = false;
                }
                if let Ok((cam, comp)) =
                    world.query_one_mut::<(&Camera, &mut CameraComponent)>(*old)
                {
                    comp.starting_camera = true;
                    log::debug!("Reverted starting camera back to true for '{}'", cam.label);
                }
                Ok(())
            }
        }
    }
}

/// This enum will be used to describe the type of command/signal. This is only between
/// the editor and unlike SceneCommand, this will ping a signal everywhere in that scene
pub enum Signal {
    None,
    Copy(SceneEntity),
    Paste(SceneEntity),
    Delete,
    Undo,
    Play,
    StopPlaying,
    CreateEntity,
    LogEntities,
    Spawn(PendingSpawnType),
    /// This only applies to builders with specific behaviours that the standard component
    /// registry is unable to have. Most don't apply to this signal, however some are supported,
    /// such as [`MeshRenderer`] (which uses async loading). 
    AddComponent(hecs::Entity, String),
    LoadModel(hecs::Entity, String),
    RequestNewWindow(WindowData),
}

#[derive(Clone)]
pub struct PlayModeBackup {
    entities: Vec<(
        Entity,
        MeshRenderer,
        EntityTransform,
        CustomProperties,
        Option<Script>,
    )>,
    camera_data: Vec<(Entity, Camera, CameraComponent)>,
}

#[derive(Debug)]
pub enum EditorState {
    Editing,
    Building,
    Playing,
}

struct PendingSceneLoad {
    scene: SceneConfig,
}

pub enum PendingSpawnType {
    Light,
    Plane,
    Cube,
    Camera,
}

pub(crate) struct IsWorldLoadedYet {
    /// Whether the project configuration and world data has been loaded
    pub project_loaded: bool,
    /// Whether the scene rendering and UI setup is complete
    pub scene_loaded: bool,
    /// Checks if the wgpu rendering contexts have been initialised for rendering
    pub rendering_loaded: bool,
}

impl IsWorldLoadedYet {
    pub fn new() -> Self {
        Self {
            project_loaded: false,
            scene_loaded: false,
            rendering_loaded: false,
        }
    }

    pub fn is_fully_loaded(&self) -> bool {
        self.project_loaded && self.scene_loaded
    }

    pub fn mark_project_loaded(&mut self) {
        self.project_loaded = true;
    }

    pub fn mark_scene_loaded(&mut self) {
        self.scene_loaded = true;
    }

    pub fn mark_rendering_loaded(&mut self) {
        self.rendering_loaded = true;
    }
}

impl Default for IsWorldLoadedYet {
    fn default() -> Self {
        Self::new()
    }
}
