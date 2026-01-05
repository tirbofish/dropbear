//! Allows you to a launch play mode as another window.

use std::sync::Arc;
use crossbeam_channel::{unbounded, Receiver};
use dropbear_engine::buffer::ResizableBuffer;
use eucalyptus_core::physics::collider::shader::ColliderWireframePipeline;
use eucalyptus_core::physics::collider::shader::ColliderInstanceRaw;
use eucalyptus_core::physics::collider::{ColliderShapeKey, WireframeGeometry};
use futures::executor;
use hecs::{Entity, World};
use wgpu::{RenderPipeline, SurfaceConfiguration};
use dropbear_engine::camera::Camera;
use dropbear_engine::future::FutureHandle;
use dropbear_engine::graphics::RenderContext;
use dropbear_engine::lighting::LightManager;
use dropbear_engine::scene::SceneCommand;
use dropbear_engine::shader::Shader;
use eucalyptus_core::camera::CameraComponent;
use eucalyptus_core::input::InputState;
use eucalyptus_core::scripting::{ScriptManager, ScriptTarget};
use eucalyptus_core::states::{WorldLoadingStatus, SCENES, Script, PROJECT};
use eucalyptus_core::scene::loading::SCENE_LOADER;
use eucalyptus_core::traits::registry::ComponentRegistry;
use eucalyptus_core::ptr::{CommandBufferPtr, InputStatePtr, PhysicsStatePtr, WorldPtr};
use eucalyptus_core::command::COMMAND_BUFFER;
use eucalyptus_core::scene::loading::IsSceneLoaded;
use std::collections::HashMap;
use std::path::PathBuf;
use winit::window::Fullscreen;
use eucalyptus_core::physics::PhysicsState;
use eucalyptus_core::rapier3d::prelude::*;
use eucalyptus_core::register_components;

mod scene;
mod input;
mod command;

fn find_jvm_library_path() -> PathBuf {
    let proj = PROJECT.read();
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

pub struct PlayMode {
    scene_command: SceneCommand,
    input_state: InputState,
    script_manager: ScriptManager,
    world: Box<World>,
    component_registry: Arc<ComponentRegistry>,
    active_camera: Option<Entity>,
    
    // rendering
    render_pipeline: Option<RenderPipeline>,
    light_manager: LightManager,

    display_settings: DisplaySettings,

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

    collider_wireframe_pipeline: Option<ColliderWireframePipeline>,
    collider_wireframe_geometry_cache: HashMap<ColliderShapeKey, WireframeGeometry>,
    collider_instance_buffer: Option<ResizableBuffer<ColliderInstanceRaw>>,
}

impl PlayMode {
    pub fn new(initial_scene: Option<String>) -> anyhow::Result<Self> {

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
            render_pipeline: None,
            light_manager: Default::default(),
            scripts_ready: false,
            has_initial_resize_done: false,
            display_settings: DisplaySettings {
                window_mode: WindowMode::Windowed,
                maintain_aspect_ratio: true,
                vsync: true,
            },
            physics_pipeline: Default::default(),
            physics_state: Box::new(PhysicsState::new()),
            pending_physics_state: Default::default(),
            physics_receiver: Default::default(),
            collider_wireframe_pipeline: None,
            collider_wireframe_geometry_cache: HashMap::new(),
            collider_instance_buffer: None,
            collision_event_receiver: Some(ce_r),
            collision_force_event_receiver: Some(cfe_r),
            event_collector,
        };

        log::debug!("Created new play mode instance");

        Ok(result)
    }

    pub fn load_wgpu_nerdy_stuff<'a>(&mut self, graphics: &mut RenderContext<'a>) {
        let shader = Shader::new(
            graphics.shared.clone(),
            dropbear_engine::shader::shader_wesl::SHADER_SHADER,
            Some("viewport_shader"),
        );

        self.light_manager
            .create_light_array_resources(graphics.shared.clone());

        if let Some(active_camera) = self.active_camera {
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
                            &graphics.shared.material_tint_bind_layout.clone(),
                        ],
                        None,
                    );
                    self.render_pipeline = Some(pipeline);

                    self.light_manager.create_render_pipeline(
                        graphics.shared.clone(),
                        dropbear_engine::shader::shader_wesl::LIGHT_SHADER,
                        camera,
                        Some("Light Pipeline"),
                    );

                    self.light_manager.create_shadow_pipeline(
                        graphics.shared.clone(),
                        dropbear_engine::shader::shader_wesl::SHADOW_SHADER,
                        Some("Shadow Pipeline"),
                    );

                    let collider_pipeline = ColliderWireframePipeline::new(graphics.shared.clone(), camera.layout());
                    self.collider_wireframe_pipeline = Some(collider_pipeline);
                } else {
                    log_once::warn_once!(
                        "Unable to fetch the query result of camera: {:?}",
                        active_camera
                    )
                }
            } else {
                log_once::warn_once!(
                    "Unable to query camera, component for active camera: {:?}",
                    active_camera
                );
            }
        } else {
            log_once::warn_once!("No active camera found");
        }
    }

    fn reload_scripts_for_current_world(&mut self) {
        let mut entity_tag_map: HashMap<String, Vec<Entity>> = HashMap::new();
        for (entity_id, script) in self.world.query::<&Script>().iter() {
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
    pub fn request_async_scene_load(&mut self, graphics: &RenderContext, requested_scene: IsSceneLoaded) {
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
                    if entry.status.is_none() {
                        entry.status = self.world_loading_progress.as_ref().cloned();
                    }
                }
            }
        }

        let mut scene_to_load = {
            let scenes = SCENES.read();
            let scene = scenes.iter().find(|s| s.scene_name == scene_name).unwrap().clone();
            scene
        };

        let graphics_cloned = graphics.shared.clone();
        let component_registry = self.component_registry.clone();

        let handle = graphics.shared.future_queue.push(async move {
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
                Err(e) => {panic!("Failed to load scene [{}]: {}", scene_to_load.scene_name, e);}
            }
        });

        log::debug!("Created future handle for scene loading: {:?}", handle);

        self.scene_loading_handle = Some(handle);
        if let Some(ref mut progress) = self.scene_progress {
            progress.scene_handle_requested = true;
        }
    }

    /// Requests an immediate scene load, blocking the current thread until the scene is fully loaded.
    pub fn request_immediate_scene_load(&mut self, graphics: &mut RenderContext, requested_scene: IsSceneLoaded) {
        let scene_name = requested_scene.requested_scene.clone();
        log::debug!("Immediate scene load requested: {}", scene_name);

        self.world = Box::new(World::new());
        self.physics_state = Box::new(PhysicsState::new());
        self.physics_receiver = None;
        self.active_camera = None;
        self.render_pipeline = None;
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

        let graphics_cloned = graphics.shared.clone();
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
    pub fn switch_to(&mut self, scene_progress: IsSceneLoaded, graphics: &mut RenderContext) {
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
}

impl DisplaySettings {
    pub fn update(&mut self, graphics: &RenderContext) {
        let window = graphics.shared.window.clone();

        let is_maximized = window.is_maximized();
        let is_fullscreen = window.fullscreen().is_some();

        self.window_mode = if is_fullscreen {
            WindowMode::BorderlessFullscreen
        } else if is_maximized {
            WindowMode::Maximized
        } else {
            WindowMode::Windowed
        };

        match self.window_mode {
            WindowMode::Windowed => {
                window.set_fullscreen(None);
                window.set_maximized(false);
            }
            WindowMode::Maximized => {
                window.set_fullscreen(None);
                window.set_maximized(true);
            }
            WindowMode::Fullscreen | WindowMode::BorderlessFullscreen => {
                let monitor = window.current_monitor();
                window.set_fullscreen(Some(Fullscreen::Borderless(monitor)));
                window.set_maximized(false);
            }
        }

        if self.vsync {
            let config = SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: graphics.shared.surface_format,
                width: graphics.frame.screen_size.0 as u32,
                height: graphics.frame.screen_size.1 as u32,
                present_mode: if self.vsync {
                    wgpu::PresentMode::Fifo
                } else {
                    wgpu::PresentMode::Immediate
                },
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };

            graphics.shared.surface.configure(&graphics.shared.device, &config);
        }
    }
}

pub enum WindowMode {
    Windowed,
    Maximized,
    Fullscreen,
    BorderlessFullscreen,
}