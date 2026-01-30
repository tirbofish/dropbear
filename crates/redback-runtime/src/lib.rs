//! Allows you to a launch play mode as another window.

use std::sync::Arc;
use crossbeam_channel::{unbounded, Receiver};
use dropbear_engine::buffer::ResizableBuffer;
use dropbear_engine::pipelines::DropbearShaderPipeline;
use dropbear_engine::pipelines::GlobalsUniform;
use dropbear_engine::pipelines::light_cube::LightCubePipeline;
use dropbear_engine::pipelines::shader::MainRenderPipeline;
use eucalyptus_core::physics::collider::shader::ColliderWireframePipeline;
use eucalyptus_core::physics::collider::shader::ColliderInstanceRaw;
use eucalyptus_core::physics::collider::{ColliderShapeKey, WireframeGeometry};
use futures::executor;
use hecs::{Entity, World};
use dropbear_engine::future::{FutureHandle, FutureQueue};
use dropbear_engine::graphics::SharedGraphicsContext;
use dropbear_engine::scene::SceneCommand;
use eucalyptus_core::input::InputState;
use eucalyptus_core::scripting::{ScriptManager, ScriptTarget};
use eucalyptus_core::states::{WorldLoadingStatus, SCENES, Script};
use eucalyptus_core::scene::loading::{SceneLoadResult, SCENE_LOADER};
use eucalyptus_core::traits::registry::ComponentRegistry;
use eucalyptus_core::ptr::{CommandBufferPtr, InputStatePtr, PhysicsStatePtr, WorldPtr};
use eucalyptus_core::command::COMMAND_BUFFER;
use eucalyptus_core::scene::loading::IsSceneLoaded;
use std::collections::HashMap;
use std::path::PathBuf;
use wgpu::SurfaceConfiguration;
use winit::window::Fullscreen;
use yakui_winit::YakuiWinit;
use dropbear_engine::texture::Texture;
use eucalyptus_core::physics::PhysicsState;
use eucalyptus_core::rapier3d::prelude::*;
use eucalyptus_core::register_components;
use kino_ui::KinoState;
use kino_ui::rendering::KinoWGPURenderer;

mod scene;
mod input;
mod command;

#[cfg(feature = "debug")]
fn find_jvm_library_path() -> PathBuf {
    let proj = eucalyptus_core::states::PROJECT.read();
    let project_path = if !proj.project_path.is_dir() {
        proj.project_path
            .parent()
            .expect("Unable to locate parent of project")
            .to_path_buf()
    } else {
        proj.project_path.clone()
    }
        .join("build/libs");

    let mut latest_jar: Option<(PathBuf, std::time::SystemTime)> = None;

    for entry in std::fs::read_dir(&project_path).expect("Unable to read directory") {
        let entry = entry.expect("Unable to get directory entry");
        let path = entry.path();

        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if filename.ends_with("-all.jar") {
                let metadata = entry.metadata().expect("Unable to get file metadata");
                let modified = metadata.modified().expect("Unable to get file modified time");

                match latest_jar {
                    None => latest_jar = Some((path.clone(), modified)),
                    Some((_, latest_time)) if modified > latest_time => {
                        latest_jar = Some((path.clone(), modified));
                    }
                    _ => {}
                }
            }
        }
    }

    latest_jar
        .map(|(path, _)| path)
        .expect("No suitable candidate for a JVM targeted play mode session available")
}

#[cfg(not(feature = "debug"))]
fn find_jvm_library_path() -> PathBuf {
    let mut latest_jar: Option<(PathBuf, std::time::SystemTime)> = None;

    for entry in std::fs::read_dir(std::env::current_exe().unwrap().parent().unwrap()).expect("Unable to read directory") {
        let entry = entry.expect("Unable to get directory entry");
        let path = entry.path();

        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if filename.ends_with("-all.jar") {
                let metadata = entry.metadata().expect("Unable to get file metadata");
                let modified = metadata.modified().expect("Unable to get file modified time");

                match latest_jar {
                    None => latest_jar = Some((path.clone(), modified)),
                    Some((_, latest_time)) if modified > latest_time => {
                        latest_jar = Some((path.clone(), modified));
                    }
                    _ => {}
                }
            }
        }
    }

    latest_jar
        .map(|(path, _)| path)
        .expect("No suitable candidate for a JVM targeted play mode session available")
}

pub struct PlayMode {
    scene_command: SceneCommand,
    input_state: InputState,
    script_manager: ScriptManager,
    world: Box<World>,
    component_registry: Arc<ComponentRegistry>,
    active_camera: Option<Entity>,
    display_settings: DisplaySettings,
    
    // rendering
    light_cube_pipeline: Option<LightCubePipeline>,
    main_pipeline: Option<MainRenderPipeline>,
    shader_globals: Option<GlobalsUniform>,
    collider_wireframe_pipeline: Option<ColliderWireframePipeline>,

    initial_scene: Option<String>,
    current_scene: Option<String>,
    world_loading_progress: Option<Receiver<WorldLoadingStatus>>,
    world_receiver: Option<tokio::sync::oneshot::Receiver<World>>,
    physics_receiver: Option<tokio::sync::oneshot::Receiver<PhysicsState>>,
    scene_loading_handle: Option<FutureHandle>,
    scene_progress: Option<IsSceneLoaded>,
    pending_world: Option<Box<World>>,
    pending_camera: Option<Entity>,
    pending_physics_state: Option<Box<PhysicsState>>,
    pub(crate) scripts_ready: bool,
    has_initial_resize_done: bool,

    // physics
    physics_pipeline: PhysicsPipeline,
    physics_state: Box<PhysicsState>,
    collision_event_receiver: Option<std::sync::mpsc::Receiver<CollisionEvent>>,
    collision_force_event_receiver: Option<std::sync::mpsc::Receiver<ContactForceEvent>>,
    event_collector: ChannelEventCollector,

    collider_wireframe_geometry_cache: HashMap<ColliderShapeKey, WireframeGeometry>,
    collider_instance_buffer: Option<ResizableBuffer<ColliderInstanceRaw>>,
    viewport_offset: (f32, f32),

    // ui
    yakui_winit: Option<YakuiWinit>,
    kino: Option<kino_ui::KinoState>,
}

impl PlayMode {
    pub fn new(initial_scene: Option<String>) -> anyhow::Result<Self> {
        eucalyptus_core::utils::start_deadlock_detector();

        let mut component_registry = ComponentRegistry::new();

        register_components(&mut component_registry);

        let (collision_event_sender, ce_r) = std::sync::mpsc::channel::<CollisionEvent>();
        let (contact_force_event_sender, cfe_r) = std::sync::mpsc::channel::<ContactForceEvent>();

        let event_collector = ChannelEventCollector::new(collision_event_sender, contact_force_event_sender);

        let result = Self {
            scene_command: SceneCommand::None,
            input_state: InputState::new(),
            script_manager: ScriptManager::new()?,
            world: Box::new(World::new()),
            initial_scene,
            current_scene: None,
            world_loading_progress: None,
            world_receiver: None,
            component_registry: Arc::new(component_registry),
            scene_loading_handle: None,
            scene_progress: None,
            pending_world: None,
            pending_camera: None,
            active_camera: None,
            main_pipeline: None,
            light_cube_pipeline: None,
            shader_globals: None,
            scripts_ready: false,
            has_initial_resize_done: false,
            physics_pipeline: Default::default(),
            physics_state: Box::new(PhysicsState::new()),
            pending_physics_state: Default::default(),
            physics_receiver: Default::default(),
            collider_wireframe_pipeline: None,
            collider_wireframe_geometry_cache: HashMap::new(),
            collider_instance_buffer: None,
            viewport_offset: (0.0, 0.0),
            collision_event_receiver: Some(ce_r),
            collision_force_event_receiver: Some(cfe_r),
            event_collector,
            yakui_winit: None,
            display_settings: DisplaySettings {
                window_mode: WindowMode::Windowed,
                maintain_aspect_ratio: true,
                vsync: false,
                last_window_mode: WindowMode::BorderlessFullscreen,
                last_vsync: true,
                last_size: (0, 0),
            },
            kino: None,
        };

        log::debug!("Created new play mode instance");

        Ok(result)
    }

    pub fn load_wgpu_nerdy_stuff<'a>(&mut self, graphics: Arc<SharedGraphicsContext>) {
        self.light_cube_pipeline = Some(LightCubePipeline::new(graphics.clone()));
        self.main_pipeline = Some(MainRenderPipeline::new(graphics.clone()));
        self.shader_globals = Some(GlobalsUniform::new(graphics.clone(), Some("runtime shader globals")));
        self.collider_wireframe_pipeline = Some(ColliderWireframePipeline::new(graphics.clone()));
        
        self.kino = Some(KinoState::new(KinoWGPURenderer::new(&graphics.device, &graphics.queue, Texture::TEXTURE_FORMAT, [graphics.viewport_texture.size.width as f32, graphics.viewport_texture.size.height as f32])))
    }

    fn reload_scripts_for_current_world(&mut self) {
        let mut entity_tag_map: HashMap<String, Vec<Entity>> = HashMap::new();
        for (entity_id, script) in self.world.query::<(Entity, &Script)>().iter() {
            for tag in &script.tags {
                entity_tag_map.entry(tag.clone()).or_default().push(entity_id);
            }
        }

        let target = ScriptTarget::JVM {
            library_path: find_jvm_library_path(),
        };

        self.scripts_ready = false;

        if let Err(e) = self
            .script_manager
            .init_script(None, entity_tag_map.clone(), target.clone())
        {
            panic!("Failed to initialise scripts: {}", e);
        } else {
            log::debug!("Initialised scripts successfully!");
        }

        let world_ptr = self.world.as_mut() as WorldPtr;
        let input_ptr = &mut self.input_state as InputStatePtr;
        let graphics_ptr = COMMAND_BUFFER.0.as_ref() as CommandBufferPtr;
        let physics_ptr = self.physics_state.as_mut() as PhysicsStatePtr;
        
        if let Err(e) = self
            .script_manager
            .load_script(world_ptr, input_ptr, graphics_ptr, physics_ptr)
        {
            panic!("Failed to load scripts: {}", e);
        } else {
            log::debug!("Loaded scripts successfully!");
        }

        self.scripts_ready = true;
        log::debug!("Scripts reloaded successfully!");
    }

    /// Requests an asynchronous scene load, returning immediately and loading the scene in the background.
    ///
    /// It will not request the scene load if the currently rendered scene is the same as the requested scene.
    pub fn request_async_scene_load(&mut self, graphics: Arc<SharedGraphicsContext>, requested_scene: IsSceneLoaded) {
        log::debug!("Requested async scene load: {}", requested_scene.requested_scene);
        let scene_name = requested_scene.requested_scene.clone();
        self.scene_progress = Some(requested_scene);

        let (tx, rx) = unbounded::<WorldLoadingStatus>();
        let (world_tx, world_rx) = tokio::sync::oneshot::channel::<World>();
        let (physics_tx, physics_rx) = tokio::sync::oneshot::channel::<PhysicsState>();

        self.world_loading_progress = Some(rx);
        self.world_receiver = Some(world_rx);
        self.physics_receiver = Some(physics_rx);

        if let Some(ref progress) = self.scene_progress {
            if let Some(id) = progress.id {
                let mut loader = SCENE_LOADER.lock();
                if let Some(entry) = loader.get_entry_mut(id) {
                    if let Some(scene) = &self.current_scene && scene == &self.scene_progress.as_ref().unwrap().requested_scene {
                        log::debug!("Load scene async request cancelled because scene name is current");
                        entry.result = SceneLoadResult::Error("Currently rendered scene name is the same as the requested scene".to_string());
                        self.world_loading_progress = None;
                        self.world_receiver = None;
                        self.physics_receiver = None;
                        self.scene_progress = None;
                        return
                    } else {
                        if entry.status.is_none() {
                            entry.status = self.world_loading_progress.as_ref().cloned();
                        }
                    }
                }
            }
        }

        let mut scene_to_load = {
            let scenes = SCENES.read();
            let scene = scenes.iter().find(|s| s.scene_name == scene_name).unwrap().clone();
            scene
        };

        let graphics_cloned = graphics.clone();
        let component_registry = self.component_registry.clone();

        let handle = FutureQueue::push(&graphics.future_queue, async move {
            let mut temp_world = World::new();
            let load_status = scene_to_load.load_into_world(
                &mut temp_world,
                graphics_cloned,
                Some(&component_registry),
                Some(tx),
                true,
            ).await;
            match load_status {
                Ok(v) => {
                    if world_tx.send(temp_world).is_err() {
                        log::warn!("Unable to send world: Receiver has been deallocated. This usually means a new scene load was requested before this one finished.");
                    };

                    if physics_tx.send(scene_to_load.physics_state.clone()).is_err() {
                        log::warn!("Unable to send physics state: Receiver has been deallocated");
                    }

                    v
                }
                Err(e) => { panic!("Failed to load scene [{}]: {}", scene_to_load.scene_name, e); }
            }
        });

        log::debug!("Created future handle for scene loading: {:?}", handle);

        self.scene_loading_handle = Some(handle);
        if let Some(ref mut progress) = self.scene_progress {
            progress.scene_handle_requested = true;
        }
    }

    /// Requests an immediate scene load, blocking the current thread until the scene is fully loaded.
    pub fn request_immediate_scene_load(&mut self, graphics: Arc<SharedGraphicsContext>, requested_scene: IsSceneLoaded) {
        if let Some(scene) = &self.current_scene && scene == &requested_scene.requested_scene {
            log::debug!("Immediate scene load request cancelled because scene name is current");
            return
        }

        let scene_name = requested_scene.requested_scene.clone();
        log::debug!("Immediate scene load requested: {}", scene_name);

        self.world = Box::new(World::new());
        self.physics_state = Box::new(PhysicsState::new());
        self.physics_receiver = None;
        self.active_camera = None;
        self.main_pipeline = None;
        self.light_cube_pipeline = None;
        self.current_scene = None;
        self.world_loading_progress = None;
        self.world_receiver = None;
        self.scene_loading_handle = None;
        self.scene_progress = None;

        let mut scene_to_load = {
            let scenes = SCENES.read();
            scenes.iter()
                .find(|s| s.scene_name == scene_name)
                .cloned()
                .expect(&format!("Scene '{}' not found", scene_name))
        };

        let graphics_cloned = graphics.clone();
        let component_registry = self.component_registry.clone();

        let (tx, _rx) = unbounded::<WorldLoadingStatus>();

        let (loaded_world, camera_entity, physics_state) = executor::block_on(async move {
            let mut temp_world = World::new();
            let camera = scene_to_load.load_into_world(
                &mut temp_world,
                graphics_cloned,
                Some(&component_registry),
                Some(tx),
                true,
            ).await;

            match camera {
                Ok(cam) => (temp_world, cam, scene_to_load.physics_state),
                Err(e) => panic!("Failed to immediately load scene [{}]: {}", scene_to_load.scene_name, e),
            }
        });

        self.world = Box::new(loaded_world);
        self.physics_state = Box::new(physics_state);
        self.active_camera = Some(camera_entity);
        self.current_scene = Some(scene_name.clone());

        let mut progress = requested_scene;
        progress.scene_handle_requested = true;
        progress.world_loaded = true;
        progress.camera_received = true;
        self.scene_progress = Some(progress);

        self.load_wgpu_nerdy_stuff(graphics);

        self.reload_scripts_for_current_world();

        log::debug!("Scene '{}' loaded", scene_name);
    }

    /// Switches to a new scene, clearing the current world and preparing to load the new scene.
    pub fn switch_to(&mut self, scene_progress: IsSceneLoaded, graphics: Arc<SharedGraphicsContext>) {
        log::debug!("Switching to new scene requested: {}", scene_progress.requested_scene);

        if scene_progress.is_everything_loaded() {
            if let Some(new_world) = self.pending_world.take() {
                self.world = new_world;
            }
            if let Some(physics_state) = self.pending_physics_state.take() {
                self.physics_state = physics_state;
            }
            self.has_initial_resize_done = false;
            if let Some(new_camera) = self.pending_camera.take() {
                self.active_camera = Some(new_camera);
            }

            self.load_wgpu_nerdy_stuff(graphics);
            self.reload_scripts_for_current_world();

            self.current_scene = Some(scene_progress.requested_scene.clone());
        }
    }
}

pub struct DisplaySettings {
    pub window_mode: WindowMode,
    pub maintain_aspect_ratio: bool,
    pub vsync: bool,
    last_window_mode: WindowMode,
    last_vsync: bool,
    last_size: (u32, u32),
}

impl DisplaySettings {
    pub fn update(&mut self, graphics: Arc<SharedGraphicsContext>) {
        let window = graphics.window.clone();
        let size = (
            graphics.viewport_texture.size.width,
            graphics.viewport_texture.size.height,
        );

        let needs_update = self.window_mode != self.last_window_mode
            || self.vsync != self.last_vsync
            || size != self.last_size;

        if !needs_update {
            return;
        }

        match self.window_mode {
            WindowMode::Windowed => {
                window.set_fullscreen(None);
                window.set_maximized(false);
            }
            WindowMode::Maximized => {
                window.set_fullscreen(None);
                window.set_maximized(true);
            }
            WindowMode::Fullscreen => {
                let monitor = window.current_monitor();
                let fullscreen = monitor
                    .as_ref()
                    .and_then(|m| m.video_modes().next())
                    .map(Fullscreen::Exclusive)
                    .or_else(|| Some(Fullscreen::Borderless(monitor)));

                window.set_fullscreen(fullscreen);
                window.set_maximized(false);
            }
            WindowMode::BorderlessFullscreen => {
                let monitor = window.current_monitor();
                window.set_fullscreen(Some(Fullscreen::Borderless(monitor)));
                window.set_maximized(false);
            }
        }

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: graphics.surface_format,
            width: graphics.viewport_texture.size.width,
            height: graphics.viewport_texture.size.height,
            present_mode: if self.vsync {
                wgpu::PresentMode::AutoVsync
            } else {
                wgpu::PresentMode::AutoNoVsync
            },
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        {
            let mut cfg = graphics.surface_config.write();
            *cfg = config;
        }

        self.last_window_mode = self.window_mode;
        self.last_vsync = self.vsync;
        self.last_size = size;
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WindowMode {
    Windowed,
    Maximized,
    Fullscreen,
    BorderlessFullscreen,
}