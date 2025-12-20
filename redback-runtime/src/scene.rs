use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use dropbear_engine::camera::Camera;
use dropbear_engine::entity::{EntityTransform, MeshRenderer, Transform};
use dropbear_engine::future::FutureHandle;
use dropbear_engine::graphics::{InstanceRaw, RenderContext};
use dropbear_engine::lighting::{Light, LightComponent, LightManager};
use dropbear_engine::model::{DrawLight, DrawModel, MODEL_CACHE, ModelId};
use dropbear_engine::scene::{Scene, SceneCommand};
use dropbear_engine::shader::{self, Shader};
use dropbear_engine::wgpu::util::DeviceExt;
use dropbear_engine::wgpu::{self, Color, RenderPipeline};
use dropbear_engine::winit::event_loop::ActiveEventLoop;
use dropbear_engine::winit::window::Window;
use dropbear_engine::asset::ASSET_REGISTRY;
use eucalyptus_core::camera::CameraComponent;
use eucalyptus_core::egui::{self, CentralPanel, Frame, UiBuilder};
use eucalyptus_core::hierarchy::EntityTransformExt;
use eucalyptus_core::input::InputState;
use eucalyptus_core::ptr::{CommandBufferPtr, InputStatePtr, WorldPtr};
use eucalyptus_core::runtime::RuntimeProjectConfig;
use eucalyptus_core::scene::SceneConfig;
use eucalyptus_core::scripting::{ScriptManager, ScriptTarget};
use eucalyptus_core::states::{Camera3D, ConfigFile, Light as LightConfig, CustomProperties, Script, SerializedMeshRenderer};
use eucalyptus_core::traits::registry::ComponentRegistry;
use eucalyptus_core::window::{CommandBufferPoller, COMMAND_BUFFER};
use hecs::{Entity, World};
use parking_lot::Mutex;
use tokio::sync::oneshot;
use tokio::sync::oneshot::error::TryRecvError;

/// The scene that the redback-runtime uses.
pub(crate) struct RuntimeScene {
    #[allow(dead_code)]
    project_config: RuntimeProjectConfig,
    window_config: ConfigFile,
    scenes: HashMap<String, SceneConfig>,
    initial_scene: String,

    pub world: Box<World>,
    pub input_state: Box<InputState>,
    pub active_camera: Arc<Mutex<Option<Entity>>>,
    render_pipeline: Option<RenderPipeline>,
    light_manager: LightManager,
    component_registry: Arc<ComponentRegistry>,
    script_manager: ScriptManager,
    script_target: Option<ScriptTarget>,
    scripts_ready: bool,
    pub scene_command: SceneCommand,

    current_scene: Option<String>,
    pub(crate) pending_scene_switch: Option<String>,
    world_receiver: Option<oneshot::Receiver<World>>,
    world_load_handle: Option<FutureHandle>,
    pub window: Option<Arc<Window>>,
    viewport_resolution: (u32, u32),
}

impl RuntimeScene {
    /// Creates a new instance of [`RuntimeScene`]
    pub fn new(project_config: RuntimeProjectConfig, window_config: ConfigFile) -> anyhow::Result<Self> {
        eucalyptus_core::utils::start_deadlock_detector();

        let initial_scene = project_config.initial_scene.clone();

        let scenes = project_config
            .scenes
            .iter()
            .map(|scene| (scene.scene_name.clone(), scene.clone()))
            .collect::<HashMap<_, _>>();

        let component_registry = Self::build_component_registry();
        let script_target = Self::detect_script_target(&project_config.project_name);

        let result = Self {
            project_config: project_config.clone(),
            window_config: window_config.clone(),
            scenes,
            initial_scene,
            world: Box::new(World::new()),
            input_state: Box::new(InputState::new()),
            active_camera: Arc::new(Mutex::new(None)),
            render_pipeline: None,
            light_manager: LightManager::new(),
            current_scene: None,
            pending_scene_switch: None,
            component_registry,
            script_manager: ScriptManager::new(window_config.jvm_args)?,
            script_target,
            scripts_ready: false,
            scene_command: Default::default(),
            world_receiver: None,
            world_load_handle: None,
            window: None,
            viewport_resolution: window_config.window_configuration.viewport_resolution,
        };

        Ok(result)
    }

    fn build_component_registry() -> Arc<ComponentRegistry> {
        let mut component_registry = ComponentRegistry::new();
        component_registry.register_with_default::<EntityTransform>();
        component_registry.register_with_default::<CustomProperties>();
        component_registry.register_with_default::<LightConfig>();
        component_registry.register_with_default::<Script>();
        component_registry.register_with_default::<SerializedMeshRenderer>();

        component_registry.register_converter::<MeshRenderer, SerializedMeshRenderer, _>(
            |_, _, renderer| {
                Some(SerializedMeshRenderer {
                    handle: renderer.handle().path.clone(),
                    material_override: renderer.material_overrides().to_vec(),
                })
            },
        );

        component_registry.register_converter::<CameraComponent, Camera3D, _>(
            |world, entity, component| {
                let Ok(camera) = world.get::<&Camera>(entity) else {
                    log::debug!(
                        "Camera component without matching Camera found on entity {:?}",
                        entity
                    );
                    return None;
                };

                Some(Camera3D::from_ecs_camera(&camera, component))
            },
        );

        Arc::new(component_registry)
    }

    fn detect_script_target(project_name: &str) -> Option<ScriptTarget> {
        if let Ok(path) = env::var("REDBACK_SCRIPT_PATH") {
            let candidate = PathBuf::from(path);
            if candidate.exists() {
                return if candidate
                    .extension()
                    .map_or(false, |ext| ext.eq_ignore_ascii_case("jar"))
                {
                    Some(ScriptTarget::JVM {
                        library_path: candidate,
                    })
                } else {
                    Some(ScriptTarget::Native {
                        library_path: candidate,
                    })
                };
            }
        }

        if let Ok(mut exe_path) = env::current_exe() {
            exe_path.pop();
            let project_root = exe_path;

            let preferred_jar = project_root.join(format!("{project_name}.jar"));
            if preferred_jar.exists() {
                return Some(ScriptTarget::JVM {
                    library_path: preferred_jar,
                });
            }

            if let Ok(entries) = std::fs::read_dir(&project_root) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path
                        .extension()
                        .map_or(false, |ext| ext.eq_ignore_ascii_case("jar"))
                    {
                        return Some(ScriptTarget::JVM { library_path: path });
                    }
                }
            }

            let native_path = project_root.join(format!("{project_name}.dll"));
            if native_path.exists() {
                return Some(ScriptTarget::Native {
                    library_path: native_path,
                });
            }

            #[cfg(not(target_os = "windows"))]
            {
                let so_path = project_root.join(format!("lib{project_name}.so"));
                if so_path.exists() {
                    return Some(ScriptTarget::Native {
                        library_path: so_path,
                    });
                }
            }
        }

        None
    }

    fn start_initial_scene_load(&mut self, graphics: &mut RenderContext) -> anyhow::Result<()> {
        let scene_name = if self.scenes.contains_key(&self.initial_scene) {
            self.initial_scene.clone()
        } else if let Some(first_scene) = self.scenes.values().next() {
            log::warn!(
                "Initial scene '{}' not found, falling back to '{}'",
                self.initial_scene,
                first_scene.scene_name
            );
            first_scene.scene_name.clone()
        } else {
            anyhow::bail!("No scenes packaged with runtime config");
        };

        self.queue_scene_load(&scene_name, graphics)
    }

    fn queue_scene_load(&mut self, scene_name: &str, graphics: &mut RenderContext) -> anyhow::Result<()> {
        self.cleanup_scene_resources(graphics);

        let scene = self
            .scenes
            .get(scene_name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Scene '{}' not found in runtime package", scene_name))?;

        // self.reset_state_for_scene_load(graphics);

        let (world_sender, world_receiver) = oneshot::channel();
        self.world_receiver = Some(world_receiver);

        let graphics_shared = graphics.shared.clone();
        let active_camera = self.active_camera.clone();
        let component_registry = self.component_registry.clone();
        let scene_name_owned = scene.scene_name.clone();

        let handle = graphics.shared.future_queue.push(async move {
            let mut temp_world = World::new();
            match scene
                .load_into_world(
                    &mut temp_world,
                    graphics_shared.clone(),
                    Some(component_registry.as_ref()),
                    None,
                    true,
                )
                .await
            {
                Ok(camera_entity) => {
                    let mut active = active_camera.lock();
                    *active = Some(camera_entity);
                    log::info!("Loaded scene '{}'", scene_name_owned);
                    log::debug!("Checkpoint 2: Camera entity: {:?}", camera_entity);
                }
                Err(err) => {
                    panic!("Failed to load scene '{}': {}", scene_name_owned, err);
                }
            }

            if world_sender.send(temp_world).is_err() {
                panic!(
                    "Scene loader dropped before world delivery for '{}'",
                    scene_name_owned
                );
            }
        });

        self.world_load_handle = Some(handle);
        self.current_scene = Some(scene_name.to_string());

        Ok(())
    }

    fn cleanup_scene_resources(&mut self, graphics: &mut RenderContext) {
        if let Some(handle) = self.world_load_handle.take() {
            graphics.shared.future_queue.cancel(&handle);
        }

        self.world_receiver = None;
        self.scripts_ready = false;
        self.active_camera.lock().take();
        self.world.clear();

        self.render_pipeline = None;
        self.light_manager = LightManager::new();

        {
            let mut cache = MODEL_CACHE.lock();
            cache.clear();
        }

        // Drop cached asset registry entries so models/materials/meshes from the previous scene
        // do not linger across scene loads.
        ASSET_REGISTRY.clear_cached_assets();
    }

    fn poll_scene_loading(&mut self, graphics: &mut RenderContext) {
        if let Some(mut receiver) = self.world_receiver.take() {
            match receiver.try_recv() {
                Ok(world) => {
                    self.world = Box::new(world);
                    self.initialise_rendering(graphics);
                    self.prepare_scripts();
                }
                Err(TryRecvError::Empty) => {
                    self.world_receiver = Some(receiver);
                }
                Err(TryRecvError::Closed) => {
                    panic!("Scene loading task ended before delivering world");
                }
            }
        }
    }

    fn update_world_state(&mut self, graphics: &mut RenderContext) {
        {
            let mut query = self.world.query::<(&mut MeshRenderer, &Transform)>();
            for (_entity, (renderer, transform)) in query.iter() {
                renderer.update(transform);
            }
        }

        {
            let mut updates = Vec::new();
            for (entity, transform) in self.world.query::<&EntityTransform>().iter() {
                let final_transform = transform.propagate(&self.world, entity);
                updates.push((entity, final_transform));
            }

            for (entity, final_transform) in updates {
                if let Ok(mut renderer) = self.world.get::<&mut MeshRenderer>(entity) {
                    renderer.update(&final_transform);
                }
            }
        }

        {
            // Update lights using their standalone Transform (not EntityTransform)
            let mut light_query = self.world.query::<(&mut LightComponent, &Transform, &mut Light)>();
            for (_, (light_comp, transform, light)) in light_query.iter() {
                light.update(light_comp, transform);
            }
        }

        {
            for (_entity_id, (camera, component)) in self
                .world
                .query::<(&mut Camera, &mut CameraComponent)>()
                .iter()
            {
                component.update(camera);
                camera.update(graphics.shared.clone());
            }
        }

        self.light_manager
            .update(graphics.shared.clone(), &self.world);
    }

    fn initialise_rendering(&mut self, graphics: &mut RenderContext) {
        if self.render_pipeline.is_some() {
            return;
        }

        let Some(active_camera) = *self.active_camera.lock() else {
            return;
        };

        let camera = if let Ok(mut q) = self
            .world
            .query_one::<(&Camera, &CameraComponent)>(active_camera)
        {
            q.get().map(|(cam, _)| cam.clone())
        } else {
            None
        };

        let Some(camera) = camera else {
            return;
        };

        self.light_manager
            .create_light_array_resources(graphics.shared.clone());

        let shader = Shader::new(
            graphics.shared.clone(),
            shader::shader_wesl::SHADER_SHADER,
            Some("runtime_viewport"),
        );

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
            shader::shader_wesl::LIGHT_SHADER,
            &camera,
            Some("light_pipeline"),
        );

        self.window = Some(graphics.shared.window.clone());
    }

    fn prepare_scripts(&mut self) {
        self.scripts_ready = false;
        let Some(target) = self.script_target.clone() else {
            log::debug!("No script target detected; skipping script setup");
            return;
        };

        let mut entity_tag_map: HashMap<String, Vec<Entity>> = HashMap::new();
        for (entity_id, script) in self.world.query::<&Script>().iter() {
            for tag in &script.tags {
                entity_tag_map.entry(tag.clone()).or_default().push(entity_id);
            }
        }

        log::debug!("Awaiting for script library to be initialised");

        if let Err(err) = self.script_manager.init_script(
            self.window_config.jvm_args.clone(),
            entity_tag_map.clone(),
            target.clone(),
        ) {
            panic!("Failed to init script manager: {}", err);
        }

        log::debug!("Loaded!");

        let world_ptr = self.world.as_mut() as WorldPtr;
        let input_ptr = self.input_state.as_mut() as InputStatePtr;
        let graphics_ptr = COMMAND_BUFFER.0.as_ref() as CommandBufferPtr;

        if let Err(err) = self
            .script_manager
            .load_script(world_ptr, input_ptr, graphics_ptr)
        {
            panic!("Failed to load scripts: {}", err);
        }

        self.scripts_ready = true;
    }
}

impl Scene for RuntimeScene {
    fn load(&mut self, graphics: &mut RenderContext) {
        self.window = Some(graphics.shared.window.clone());

        if let Err(err) = self.start_initial_scene_load(graphics) {
            panic!("Unable to load initial scene: {}", err);
        }
    }

    fn update(&mut self, dt: f32, graphics: &mut RenderContext) {
        graphics.shared.future_queue.poll();

        self.poll_scene_loading(graphics);        
        self.poll(graphics);

        if self.world_receiver.is_none() {
            if let Some(scene_name) = self.pending_scene_switch.take() {
                if let Err(err) = self.queue_scene_load(&scene_name, graphics) {
                    log::error!("Failed to switch scene contents to '{}': {}", scene_name, err);
                }
            }
        }

        CentralPanel::default().frame(Frame::new()).show(&graphics.shared.get_egui_context(), |ui| {
            if self.render_pipeline.is_none() {
                ui.label("Loading scene...");
            }

            if self.render_pipeline.is_none() {
                return;
            }

            self.update_world_state(graphics);

            let egui_ctx = graphics.shared.get_egui_context();
            let egui_wants_input = egui_ctx.wants_pointer_input() || egui_ctx.wants_keyboard_input();
            
            if egui_wants_input {
                self.input_state.pressed_keys.clear();
                self.input_state.mouse_button.clear();
            }

            if self.scripts_ready {
                let world_ptr = self.world.as_mut() as WorldPtr;
                if let Err(err) = unsafe { self.script_manager.update_script(world_ptr, &self.input_state, dt) } {
                    panic!("Script update failed: {}", err);
                }
            }

            let texture_id = *graphics.shared.texture_id;
            let available_size = ui.available_rect_before_wrap().size();
            
            let is_fullscreen = self.window_config.window_configuration.windowed_mode.is_fullscreen();
            
            let viewport_aspect = self.viewport_resolution.0 as f32 / self.viewport_resolution.1 as f32;
            let available_aspect = available_size.x / available_size.y;
            
            let active_camera: Option<Entity> = *self.active_camera.lock();
            if let Some(cam_ent) = active_camera {
                if let Ok(mut q) = self.world.query_one::<&mut Camera>(cam_ent) {
                    if let Some(camera) = q.get() {
                        if is_fullscreen {
                            camera.aspect = viewport_aspect as f64;
                        } else {
                            camera.aspect = available_aspect as f64;
                        }
                        camera.update_view_proj();
                        camera.update(graphics.shared.clone());
                    }
                }
            }
            
            let (display_width, display_height) = if is_fullscreen {
                if available_aspect > viewport_aspect {
                    (available_size.x, available_size.x / viewport_aspect)
                } else {
                    (available_size.y * viewport_aspect, available_size.y)
                }
            } else {
                (available_size.x, available_size.y)
            };
            
            let rect = ui.available_rect_before_wrap();
            let x_offset = (available_size.x - display_width) / 2.0;
            let y_offset = (available_size.y - display_height) / 2.0;
            let image_rect = egui::Rect::from_min_size(
                egui::pos2(rect.min.x + x_offset, rect.min.y + y_offset),
                egui::vec2(display_width, display_height),
            );
            
            ui.scope_builder(UiBuilder::new().max_rect(image_rect), |ui| {
                ui.add(egui::Image::new(egui::load::SizedTexture {
                    id: texture_id,
                    size: egui::vec2(display_width, display_height),
                }));
            });

            self.input_state.window = self.window.clone();
            self.input_state.mouse_delta = None;
        });
    }

    fn render(&mut self, graphics: &mut RenderContext) {
        if self.render_pipeline.is_none() {
            self.initialise_rendering(graphics);
        }

        let Some(active_camera) = *self.active_camera.lock() else {
            return;
        };

        let q = if let Ok(mut query) = self.world.query_one::<&Camera>(active_camera) {
            query.get().cloned()
        } else {
            None
        };

        let Some(camera) = q else {
            return;
        };

        // camera.debug_camera_state();
        // println!("{:#?}", self.project_config);

        let Some(pipeline) = &self.render_pipeline else {
            return;
        };

        let clear_color = Color {
            r: 0.05,
            g: 0.07,
            b: 0.10,
            a: 1.0,
        };

        let lights = {
            let mut lights = Vec::new();
            let mut query = self.world.query::<(&Light, &LightComponent)>();
            for (_, (light, comp)) in query.iter() {
                lights.push((light.clone(), comp.clone()));
            }
            lights
        };

        let renderers = {
            let mut renderers = Vec::new();
            let mut query = self.world.query::<&MeshRenderer>();
            for (_, renderer) in query.iter() {
                renderers.push(renderer.clone());
            }
            renderers
        };

        {
            let mut render_pass = graphics.clear_colour(clear_color);
            if let Some(light_pipeline) = &self.light_manager.pipeline {
                render_pass.set_pipeline(light_pipeline);
                for (light, component) in &lights {
                    if let Some(buffer) = &light.instance_buffer {
                        render_pass.set_vertex_buffer(1, buffer.slice(..));
                        if component.visible {
                            render_pass.draw_light_model(
                                &light.cube_model,
                                camera.bind_group(),
                                light.bind_group(),
                            );
                        }
                    }
                }
            }
        }

        let mut model_batches: HashMap<ModelId, Vec<InstanceRaw>> = HashMap::new();
        for renderer in &renderers {
            model_batches
                .entry(renderer.model_id())
                .or_default()
                .push(renderer.instance.to_raw());
        }

        for (model_id, instances) in model_batches {
            let model_opt = {
                let cache = MODEL_CACHE.lock();
                cache.values().find(|model| model.id == model_id).cloned()
            };

            let Some(model) = model_opt else {
                log_once::error_once!("Missing model {:?} in cache", model_id);
                continue;
            };

            let instance_buffer = graphics.shared.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Runtime Instance Buffer"),
                    contents: bytemuck::cast_slice(&instances),
                    usage: wgpu::BufferUsages::VERTEX,
                },
            );

            let mut render_pass = graphics.continue_pass();
            render_pass.set_pipeline(pipeline);
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
            render_pass.draw_model_instanced(
                &model,
                0..instances.len() as u32,
                camera.bind_group(),
                self.light_manager.bind_group(),
            );
        }
    }

    fn exit(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
        self.scene_command = SceneCommand::None;
        if let Some(window) = &self.window {
            window.set_cursor_visible(true);
        }
        self.world.clear();
        self.render_pipeline = None;
        self.scripts_ready = false;
        self.world_receiver = None;
        self.world_load_handle = None;
        self.script_target = None;
    }

    fn run_command(&mut self) -> SceneCommand {
        std::mem::replace(&mut self.scene_command, SceneCommand::None)
    }
}
