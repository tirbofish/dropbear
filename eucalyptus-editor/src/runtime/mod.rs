//! Allows you to a launch play mode as another window.

use std::sync::Arc;
use crossbeam_channel::{unbounded, Receiver};
use futures::executor;
use hecs::{Entity, World};
use wgpu::RenderPipeline;
use dropbear_engine::camera::Camera;
use dropbear_engine::future::{FutureHandle, FutureQueue};
use dropbear_engine::graphics::RenderContext;
use dropbear_engine::lighting::LightManager;
use dropbear_engine::scene::SceneCommand;
use dropbear_engine::shader::Shader;
use eucalyptus_core::camera::CameraComponent;
use eucalyptus_core::input::InputState;
use eucalyptus_core::scripting::ScriptManager;
use eucalyptus_core::states::{WorldLoadingStatus, SCENES};
use eucalyptus_core::traits::registry::ComponentRegistry;
use crate::runtime::scene::IsSceneLoaded;

mod scene;
mod input;
mod command;

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
    scene_loading_handle: Option<FutureHandle>,
    scene_progress: Option<IsSceneLoaded>,
    pub(crate) scripts_ready: bool,
    has_initial_resize_done: bool,
}

impl PlayMode {
    pub fn new(initial_scene: Option<String>) -> anyhow::Result<Self> {
        let result = Self {
            scene_command: SceneCommand::None,
            input_state: InputState::new(),
            script_manager: ScriptManager::new()?,
            world: Box::new(World::new()),
            initial_scene,
            current_scene: None,
            world_loading_progress: None,
            world_receiver: None,
            component_registry: Arc::new(Default::default()),
            scene_loading_handle: None,
            scene_progress: None,
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

    /// Requests an asynchronous scene load, returning immediately and loading the scene in the background.
    pub fn request_async_scene_load(&mut self, graphics: &mut RenderContext, requested_scene: IsSceneLoaded) {
        log::debug!("Requested async scene load: {}", requested_scene.requested_scene);
        let scene_name = requested_scene.requested_scene.clone();
        self.scene_progress = Some(requested_scene);

        let (tx, rx) = unbounded::<WorldLoadingStatus>();
        let (world_tx, world_rx) = tokio::sync::oneshot::channel::<World>();

        self.world_loading_progress = Some(rx);
        self.world_receiver = Some(world_rx);

        let scene_to_load = {
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
                        panic!("Unable to send world: Receiver has been deallocated")
                    };
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
        self.active_camera = None;
        self.render_pipeline = None;
        self.scripts_ready = false;
        self.current_scene = None;
        self.world_loading_progress = None;
        self.world_receiver = None;
        self.scene_loading_handle = None;
        self.scene_progress = None;

        let scene_to_load = {
            let scenes = SCENES.read();
            scenes.iter()
                .find(|s| s.scene_name == scene_name)
                .cloned()
                .expect(&format!("Scene '{}' not found", scene_name))
        };

        let graphics_cloned = graphics.shared.clone();
        let component_registry = self.component_registry.clone();

        let (tx, _rx) = unbounded::<WorldLoadingStatus>();

        let (loaded_world, camera_entity) = executor::block_on(async move {
            let mut temp_world = World::new();
            let camera = scene_to_load.load_into_world(
                &mut temp_world,
                graphics_cloned,
                Some(&component_registry),
                Some(tx),
                true,
            ).await;

            match camera {
                Ok(cam) => (temp_world, cam),
                Err(e) => panic!("Failed to immediately load scene [{}]: {}", scene_to_load.scene_name, e),
            }
        });

        self.world = Box::new(loaded_world);
        self.active_camera = Some(camera_entity);
        self.current_scene = Some(scene_name.clone());

        self.load_wgpu_nerdy_stuff(graphics);

        let mut progress = requested_scene;
        progress.scene_handle_requested = true;
        progress.world_loaded = true;
        progress.camera_received = true;
        self.scene_progress = Some(progress);

        log::debug!("Scene '{}' loaded immediately", scene_name);
    }

    /// Switches to a new scene, clearing the current world and preparing to load the new scene.
    pub fn switch_to(&mut self, scene_progress: IsSceneLoaded, _future_queue: Arc<FutureQueue>) {
        // todo: fix this
        log::debug!("Switching to new scene requested: {}", scene_progress.requested_scene);

        self.world = Box::new(World::new());
        self.active_camera = None;
        self.render_pipeline = None;
        self.scripts_ready = false;
        self.current_scene = None;

        self.world_loading_progress = None;
        self.world_receiver = None;
        self.scene_loading_handle = None;

        self.scene_progress = Some(scene_progress);
    }
}

pub struct DisplaySettings {
    pub window_mode: WindowMode,
    pub maintain_aspect_ratio: bool,
    pub vsync: bool,
}

pub enum WindowMode {
    Windowed,
    Maximized,
    Fullscreen,
    BorderlessFullscreen,
}