//! Allows you to a launch play mode as another window.

use crossbeam_channel::{Receiver, unbounded};
use dropbear_engine::animation::MorphTargetInfo;
use dropbear_engine::billboarding::BillboardPipeline;
use dropbear_engine::buffer::ResizableBuffer;
use dropbear_engine::camera::Camera;
use dropbear_engine::future::{FutureHandle, FutureQueue};
use dropbear_engine::graphics::{InstanceRaw, SharedGraphicsContext};
use dropbear_engine::pipelines::DropbearShaderPipeline;
use dropbear_engine::pipelines::GlobalsUniform;
use dropbear_engine::pipelines::light_cube::LightCubePipeline;
use dropbear_engine::pipelines::shader::MainRenderPipeline;
use dropbear_engine::scene::SceneCommand;
use dropbear_engine::sky::{DEFAULT_SKY_TEXTURE, HdrLoader, SkyPipeline};
use eucalyptus_core::command::COMMAND_BUFFER;
use eucalyptus_core::component::ComponentRegistry;
use eucalyptus_core::input::InputState;
use eucalyptus_core::physics::PhysicsState;
use eucalyptus_core::ptr::{
    CommandBufferPtr, GraphicsContextPtr, InputStatePtr, PhysicsStatePtr, UiBufferPtr, WorldPtr,
};
use eucalyptus_core::rapier3d::prelude::*;
use eucalyptus_core::register_components;
use eucalyptus_core::scene::loading::IsSceneLoaded;
use eucalyptus_core::scene::loading::{SCENE_LOADER, SceneLoadResult};
use eucalyptus_core::scripting::{ScriptManager, ScriptTarget};
use eucalyptus_core::states::{SCENES, Script, WorldLoadingStatus};
use futures::executor;
use glam::Mat4;
use hecs::{Entity, World};
use kino_ui::KinoState;
use kino_ui::rendering::KinoWGPURenderer;
use kino_ui::windowing::KinoWinitWindowing;
use log::error;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use wgpu::SurfaceConfiguration;
use wgpu::util::DeviceExt;
use winit::window::Fullscreen;

mod command;
mod input;
mod scene;

const MAX_MORPH_WEIGHTS: usize = 4096;

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
                let modified = metadata
                    .modified()
                    .expect("Unable to get file modified time");

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

    for entry in std::fs::read_dir(std::env::current_exe().unwrap().parent().unwrap())
        .expect("Unable to read directory")
    {
        let entry = entry.expect("Unable to get directory entry");
        let path = entry.path();

        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if filename.ends_with("-all.jar") {
                let metadata = entry.metadata().expect("Unable to get file metadata");
                let modified = metadata
                    .modified()
                    .expect("Unable to get file modified time");

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
    instance_buffer_cache: HashMap<u64, ResizableBuffer<InstanceRaw>>,
    animated_instance_buffers: HashMap<Entity, ResizableBuffer<InstanceRaw>>,
    sky_pipeline: Option<SkyPipeline>,
    default_skinning_buffer: Option<wgpu::Buffer>,
    default_morph_deltas_buffer: Option<wgpu::Buffer>,
    default_morph_weights_buffer: Option<wgpu::Buffer>,
    default_morph_info_buffer: Option<wgpu::Buffer>,
    default_animation_bind_group: Option<wgpu::BindGroup>,
    billboard_pipeline: Option<BillboardPipeline>,
    pub(crate) static_batches: HashMap<u64, Vec<(Entity, InstanceRaw)>>,
    pub(crate) animated_instances: Vec<(
        Entity,
        u64,
        InstanceRaw,
        wgpu::Buffer,
        wgpu::Buffer,
        wgpu::Buffer,
        u32,
    )>,
    pub(crate) animated_bind_group_cache: HashMap<Entity, (u64, wgpu::BindGroup)>,
    pub(crate) last_morph_info_per_mesh: HashMap<u32, MorphTargetInfo>,

    last_active_camera_for_per_frame: Option<Entity>,

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

    viewport_offset: (f32, f32),

    // ui
    kino: Option<kino_ui::KinoState>,
}

impl PlayMode {
    pub fn new(initial_scene: Option<String>) -> anyhow::Result<Self> {
        eucalyptus_core::utils::start_deadlock_detector();

        let mut component_registry = ComponentRegistry::new();

        register_components(&mut component_registry);

        let (collision_event_sender, ce_r) = std::sync::mpsc::channel::<CollisionEvent>();
        let (contact_force_event_sender, cfe_r) = std::sync::mpsc::channel::<ContactForceEvent>();

        let event_collector =
            ChannelEventCollector::new(collision_event_sender, contact_force_event_sender);

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
            instance_buffer_cache: HashMap::new(),
            animated_instance_buffers: HashMap::new(),
            scripts_ready: false,
            has_initial_resize_done: false,
            physics_pipeline: Default::default(),
            physics_state: Box::new(PhysicsState::new()),
            pending_physics_state: Default::default(),
            physics_receiver: Default::default(),
            viewport_offset: (0.0, 0.0),
            collision_event_receiver: Some(ce_r),
            collision_force_event_receiver: Some(cfe_r),
            event_collector,
            display_settings: DisplaySettings {
                window_mode: WindowMode::Windowed,
                maintain_aspect_ratio: false,
                vsync: false,
                last_window_mode: WindowMode::BorderlessFullscreen,
                last_vsync: true,
                last_size: (0, 0),
            },
            kino: None,
            sky_pipeline: None,
            default_skinning_buffer: None,
            default_morph_deltas_buffer: None,
            default_morph_weights_buffer: None,
            default_morph_info_buffer: None,
            default_animation_bind_group: None,
            billboard_pipeline: None,
            static_batches: Default::default(),
            animated_instances: vec![],
            animated_bind_group_cache: Default::default(),
            last_morph_info_per_mesh: Default::default(),
            last_active_camera_for_per_frame: None,
        };

        log::debug!("Created new play mode instance");

        Ok(result)
    }

    pub fn reload_wgpu(
        &mut self,
        graphics: Arc<SharedGraphicsContext>,
        sky_texture: Option<&Vec<u8>>,
    ) {
        self.light_cube_pipeline = None;
        self.main_pipeline = None;
        self.shader_globals = None;
        self.kino = None;
        self.sky_pipeline = None;
        self.default_skinning_buffer = None;
        self.default_morph_deltas_buffer = None;
        self.default_morph_weights_buffer = None;
        self.default_morph_info_buffer = None;
        self.default_animation_bind_group = None;

        self.load_wgpu_nerdy_stuff(graphics, sky_texture);
    }

    pub fn load_wgpu_nerdy_stuff<'a>(
        &mut self,
        graphics: Arc<SharedGraphicsContext>,
        sky_texture: Option<&Vec<u8>>,
    ) {
        self.light_cube_pipeline = Some(LightCubePipeline::new(graphics.clone()));
        self.main_pipeline = Some(MainRenderPipeline::new(graphics.clone()));
        self.shader_globals = Some(GlobalsUniform::new(
            graphics.clone(),
            Some("runtime shader globals"),
        ));

        let mut pending_sky_pipeline = None;

        if self.default_skinning_buffer.is_none() {
            let max_skinning_matrices = 256usize;
            let identity = vec![Mat4::IDENTITY; max_skinning_matrices];
            let skinning_buffer =
                graphics
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("runtime default skinning buffer"),
                        contents: bytemuck::cast_slice(&identity),
                        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    });

            let morph_deltas_buffer =
                graphics
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("runtime default morph deltas buffer"),
                        contents: bytemuck::cast_slice(&[0.0f32]),
                        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    });

            let morph_weights = vec![0.0f32; MAX_MORPH_WEIGHTS];
            let morph_weights_buffer =
                graphics
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("runtime default morph weights buffer"),
                        contents: bytemuck::cast_slice(&morph_weights),
                        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    });

            let morph_info = dropbear_engine::animation::MorphTargetInfo::default();
            let morph_info_buffer =
                graphics
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("runtime default morph info buffer"),
                        contents: bytemuck::bytes_of(&morph_info),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    });

            self.default_skinning_buffer = Some(skinning_buffer);
            self.default_morph_deltas_buffer = Some(morph_deltas_buffer);
            self.default_morph_weights_buffer = Some(morph_weights_buffer);
            self.default_morph_info_buffer = Some(morph_info_buffer);
        }

        if self.default_animation_bind_group.is_none() {
            let skinning_buffer = self
                .default_skinning_buffer
                .as_ref()
                .expect("Default skinning buffer missing");
            let morph_deltas_buffer = self
                .default_morph_deltas_buffer
                .as_ref()
                .expect("Default morph deltas buffer missing");
            let morph_weights_buffer = self
                .default_morph_weights_buffer
                .as_ref()
                .expect("Default morph weights buffer missing");
            let morph_info_buffer = self
                .default_morph_info_buffer
                .as_ref()
                .expect("Default morph info buffer missing");

            self.default_animation_bind_group = Some(graphics.device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some("runtime default animation bind group"),
                    layout: &graphics.layouts.animation_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: skinning_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: morph_deltas_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: morph_weights_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: morph_info_buffer.as_entire_binding(),
                        },
                    ],
                },
            ));
        }

        self.kino = Some(kino_ui::KinoState::new(
            KinoWGPURenderer::new(
                &graphics.device,
                &graphics.queue,
                graphics.hdr.read().format(),
                [
                    graphics.viewport_texture.size.width as f32,
                    graphics.viewport_texture.size.height as f32,
                ],
            ),
            KinoWinitWindowing::new(graphics.window.clone(), None),
        ));

        self.billboard_pipeline = Some(BillboardPipeline::new(graphics.clone()));
        *graphics.debug_draw.lock() =
            Some(dropbear_engine::debug::DebugDraw::new(graphics.clone()));

        let sky_texture_result = HdrLoader::from_equirectangular_bytes(
            &graphics.device,
            &graphics.queue,
            sky_texture.map_or(DEFAULT_SKY_TEXTURE, |v| v.as_slice()),
            1080,
            Some("sky texture"),
        );

        if let Some(camera_entity) = self.active_camera {
            if let Ok(camera) = self.world.query_one::<&Camera>(camera_entity).get() {
                match sky_texture_result {
                    Ok(sky_texture) => {
                        pending_sky_pipeline = Some(SkyPipeline::new(
                            graphics.clone(),
                            sky_texture,
                            camera.buffer(),
                        ));
                    }
                    Err(e) => {
                        error!("Failed to load sky texture: {}", e);
                    }
                }

                if let (Some(main_pipeline), Some(globals), Some(light_pipeline)) = (
                    self.main_pipeline.as_mut(),
                    self.shader_globals.as_ref(),
                    self.light_cube_pipeline.as_ref(),
                ) {
                    let _ = main_pipeline.per_frame_bind_group(
                        graphics.clone(),
                        globals.buffer.buffer(),
                        camera.buffer(),
                        light_pipeline.light_buffer(),
                    );
                }
            } else {
                error!("Unable to create bind groups without an active camera component");
            }
        } else {
            error!("Unable to create bind groups without an active camera");
        }

        if let Some(sky_pipeline) = pending_sky_pipeline {
            self.sky_pipeline = Some(sky_pipeline);
        }
    }

    fn reload_scripts_for_current_world(&mut self, graphics: Arc<SharedGraphicsContext>) {
        let mut entity_tag_map: HashMap<String, Vec<Entity>> = HashMap::new();
        for (entity_id, script) in self.world.query::<(Entity, &Script)>().iter() {
            for tag in &script.tags {
                entity_tag_map
                    .entry(tag.clone())
                    .or_default()
                    .push(entity_id);
            }
        }

        let target = ScriptTarget::JVM {
            library_path: find_jvm_library_path(),
        };

        self.scripts_ready = false;

        if let Err(e) =
            self.script_manager
                .init_script(None, entity_tag_map.clone(), target.clone())
        {
            panic!("Failed to initialise scripts: {}", e);
        } else {
            log::debug!("Initialised scripts successfully!");
        }

        let world_ptr = self.world.as_mut() as WorldPtr;
        let input_ptr = &mut self.input_state as InputStatePtr;
        let graphics_ptr = COMMAND_BUFFER.0.as_ref() as CommandBufferPtr;
        let graphics_context_ptr = Arc::as_ptr(&graphics) as GraphicsContextPtr;
        let physics_ptr = self.physics_state.as_mut() as PhysicsStatePtr;
        let ui_ptr = self
            .kino
            .as_mut()
            .map(|kino| kino as *mut KinoState as UiBufferPtr)
            .unwrap_or(std::ptr::null_mut());

        if let Err(e) = self.script_manager.load_script(
            world_ptr,
            input_ptr,
            graphics_ptr,
            graphics_context_ptr,
            physics_ptr,
            ui_ptr,
        ) {
            panic!("Failed to load scripts: {}", e);
        } else {
            log::debug!("Loaded scripts successfully!");
        }

        if let Err(_e) = self.script_manager.discover_components() {}

        // // todo: this wont work for native contexts.
        // if let Some(registry) = Arc::get_mut(&mut self.component_registry) {
        //     registry.drain_kotlin_queue();
        //
        //     if let Some(jvm) = eucalyptus_core::scripting::jni::GLOBAL_JVM.get() {
        //         let jvm = jvm.clone();
        //         registry.set_kotlin_update_fn(move |fqcn: &str, entity_id: u64, dt: f32| {
        //             let Ok(mut env) = jvm.attach_current_thread() else {
        //                 log::warn!("kotlin_update_fn: failed to attach JVM thread");
        //                 return;
        //             };
        //             let Ok(fqcn_jstr) = env.new_string(fqcn) else { return };
        //             let Ok(class) = env.load_class("com/dropbear/decl/ComponentManager") else {
        //                 log::warn!(
        //                     "kotlin_update_fn: ComponentManager class not found - has the project JAR been loaded?"
        //                 );
        //                 return;
        //             };
        //             if let Err(e) = env.call_static_method(
        //                 class,
        //                 "updateKotlinComponent",
        //                 "(Ljava/lang/String;JD)V",
        //                 &[
        //                     JValue::Object(&fqcn_jstr),
        //                     JValue::Long(entity_id as i64),
        //                     JValue::Double(dt as f64),
        //                 ],
        //             ) {
        //                 log::warn!(
        //                     "kotlin_update_fn: updateKotlinComponent('{}', {}) failed: {:?}",
        //                     fqcn, entity_id, e
        //                 );
        //                 let _ = env.exception_clear();
        //             }
        //         });
        //     }
        // } else {
        //     log::warn!("drain_kotlin_queue: could not get exclusive access to component_registry; Kotlin component descriptors were not registered");
        // }

        self.scripts_ready = true;
        log::debug!("Scripts reloaded successfully!");
    }

    /// Requests an asynchronous scene load, returning immediately and loading the scene in the background.
    ///
    /// It will not request the scene load if the currently rendered scene is the same as the requested scene.
    pub fn request_async_scene_load(
        &mut self,
        graphics: Arc<SharedGraphicsContext>,
        requested_scene: IsSceneLoaded,
    ) {
        log::debug!(
            "Requested async scene load: {}",
            requested_scene.requested_scene
        );
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
                    if let Some(scene) = &self.current_scene
                        && scene == &self.scene_progress.as_ref().unwrap().requested_scene
                    {
                        log::debug!(
                            "Load scene async request cancelled because scene name is current"
                        );
                        entry.result = SceneLoadResult::Error(
                            "Currently rendered scene name is the same as the requested scene"
                                .to_string(),
                        );
                        self.world_loading_progress = None;
                        self.world_receiver = None;
                        self.physics_receiver = None;
                        self.scene_progress = None;
                        return;
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
            let scene = scenes
                .iter()
                .find(|s| s.scene_name == scene_name)
                .unwrap()
                .clone();
            scene
        };

        let graphics_cloned = graphics.clone();
        let component_registry = self.component_registry.clone();

        let handle = FutureQueue::push(&graphics.future_queue, async move {
            let mut temp_world = World::new();
            let load_status = scene_to_load
                .load_into_world(
                    &mut temp_world,
                    graphics_cloned,
                    &component_registry.clone(),
                    Some(tx),
                    true,
                )
                .await;
            match load_status {
                Ok(v) => {
                    if world_tx.send(temp_world).is_err() {
                        log::warn!(
                            "Unable to send world: Receiver has been deallocated. This usually means a new scene load was requested before this one finished."
                        );
                    };

                    if physics_tx
                        .send(scene_to_load.physics_state.clone())
                        .is_err()
                    {
                        log::warn!("Unable to send physics state: Receiver has been deallocated");
                    }

                    v
                }
                Err(e) => {
                    panic!("Failed to load scene [{}]: {}", scene_to_load.scene_name, e);
                }
            }
        });

        log::debug!("Created future handle for scene loading: {:?}", handle);

        self.scene_loading_handle = Some(handle);
        if let Some(ref mut progress) = self.scene_progress {
            progress.scene_handle_requested = true;
        }
    }

    /// Requests an immediate scene load, blocking the current thread until the scene is fully loaded.
    pub fn request_immediate_scene_load(
        &mut self,
        graphics: Arc<SharedGraphicsContext>,
        requested_scene: IsSceneLoaded,
    ) {
        if let Some(scene) = &self.current_scene
            && scene == &requested_scene.requested_scene
        {
            log::debug!("Immediate scene load request cancelled because scene name is current");
            return;
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
            scenes
                .iter()
                .find(|s| s.scene_name == scene_name)
                .cloned()
                .expect(&format!("Scene '{}' not found", scene_name))
        };

        let graphics_cloned = graphics.clone();
        let component_registry = self.component_registry.clone();

        let (tx, _rx) = unbounded::<WorldLoadingStatus>();

        let (loaded_world, camera_entity, physics_state) = executor::block_on(async move {
            let mut temp_world = World::new();
            let camera = scene_to_load
                .load_into_world(
                    &mut temp_world,
                    graphics_cloned,
                    &component_registry.clone(),
                    Some(tx),
                    true,
                )
                .await;

            match camera {
                Ok(cam) => (temp_world, cam, scene_to_load.physics_state),
                Err(e) => panic!(
                    "Failed to immediately load scene [{}]: {}",
                    scene_to_load.scene_name, e
                ),
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

        self.load_wgpu_nerdy_stuff(graphics.clone(), None);

        self.reload_scripts_for_current_world(graphics.clone());

        log::debug!("Scene '{}' loaded", scene_name);
    }

    /// Switches to a new scene, clearing the current world and preparing to load the new scene.
    pub fn switch_to(
        &mut self,
        scene_progress: IsSceneLoaded,
        graphics: Arc<SharedGraphicsContext>,
    ) {
        log::debug!(
            "Switching to new scene requested: {}",
            scene_progress.requested_scene
        );

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

            self.load_wgpu_nerdy_stuff(graphics.clone(), None);
            self.reload_scripts_for_current_world(graphics.clone());

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
        let mut window_mode_changed = self.window_mode != self.last_window_mode;

        if !window_mode_changed {
            let actual_mode = if let Some(fullscreen) = window.fullscreen() {
                match fullscreen {
                    Fullscreen::Exclusive(_) => WindowMode::Fullscreen,
                    Fullscreen::Borderless(_) => WindowMode::BorderlessFullscreen,
                }
            } else if window.is_maximized() {
                WindowMode::Maximized
            } else {
                WindowMode::Windowed
            };

            if actual_mode != self.window_mode {
                self.window_mode = actual_mode;
                window_mode_changed = true;
            }
        }

        let size_changed = size != self.last_size;
        let vsync_changed = self.vsync != self.last_vsync;
        let needs_config_update = window_mode_changed || vsync_changed || size_changed;

        if !window_mode_changed && !needs_config_update {
            return;
        }

        if window_mode_changed {
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
        }

        if needs_config_update {
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
