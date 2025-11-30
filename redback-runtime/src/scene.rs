use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
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
use eucalyptus_core::camera::CameraComponent;
use eucalyptus_core::hierarchy::EntityTransformExt;
use eucalyptus_core::input::InputState;
use eucalyptus_core::ptr::{GraphicsPtr, InputStatePtr, WorldPtr};
use eucalyptus_core::runtime::RuntimeProjectConfig;
use eucalyptus_core::scene::SceneConfig;
use eucalyptus_core::scripting::{ScriptManager, ScriptTarget};
use eucalyptus_core::states::Script;
use eucalyptus_core::traits::registry::ComponentRegistry;
use eucalyptus_core::window::{self, GRAPHICS_COMMAND};
use hecs::{Entity, World};
use parking_lot::Mutex;
use tokio::sync::oneshot;
use tokio::sync::oneshot::error::TryRecvError;

/// The scene that the redback-runtime uses.
pub(crate) struct RuntimeScene {
    #[allow(dead_code)]
    project_config: RuntimeProjectConfig,
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
    scene_command: SceneCommand,

    current_scene: Option<String>,
    pending_scene: Option<PendingSceneLoad>,
    pub window: Option<Arc<Window>>,
}

struct PendingSceneLoad {
    name: String,
    receiver: oneshot::Receiver<anyhow::Result<LoadedScene>>,
    handle: FutureHandle,
}

struct LoadedScene {
    world: World,
    active_camera: Entity,
}

impl RuntimeScene {
    /// Creates a new instance of [`RuntimeScene`]
    pub fn new(project_config: RuntimeProjectConfig) -> anyhow::Result<Self> {
        // checks for any deadlocks in another thread.
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_secs(1));
                let deadlocks = parking_lot::deadlock::check_deadlock();
                if deadlocks.is_empty() {
                    continue;
                }

                for (i, threads) in deadlocks.iter().enumerate() {
                    log::error!("Deadlock #{}", i);
                    for t in threads {
                        log::error!("Thread Id {:#?}", t.thread_id());
                        log::error!("{:#?}", t.backtrace());
                    }
                }
                panic!(
                    "Fatal: {} deadlocks detected, unable to continue on normal process",
                    deadlocks.len()
                );
            }
        });

        let initial_scene = project_config.initial_scene.clone();

        let scenes = project_config
            .scenes
            .iter()
            .map(|scene| (scene.scene_name.clone(), scene.clone()))
            .collect::<HashMap<_, _>>();

        let result = Self {
            project_config: project_config.clone(),
            scenes,
            initial_scene,
            world: Box::new(World::new()),
            input_state: Box::new(InputState::new()),
            active_camera: Arc::new(Mutex::new(None)),
            render_pipeline: None,
            light_manager: LightManager::new(),
            current_scene: None,
            component_registry: Arc::new(ComponentRegistry::new()),
            script_manager: ScriptManager::new()?,
            script_target: None,
            scripts_ready: false,
            scene_command: Default::default(),
            pending_scene: None,
            window: None,
        };

        Ok(result)
    }

    fn start_scene_load(
        &mut self,
        scene_name: &str,
        graphics: &mut RenderContext,
    ) -> anyhow::Result<()> {
        if self.scenes.is_empty() {
            anyhow::bail!("No scenes available in this runtime build");
        }

        if let Some(pending) = self.pending_scene.take() {
            graphics.shared.future_queue.cancel(&pending.handle);
        }

        self.world.clear();
        self.render_pipeline = None;
        self.light_manager = LightManager::new();
        self.scripts_ready = false;
        self.script_target = None;
        *self.active_camera.lock() = None;

        let scene = self
            .scenes
            .get(scene_name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Scene '{}' does not exist", scene_name))?;

        let graphics_shared = graphics.shared.clone();
        let component_registry = self.component_registry.clone();
        let (tx, rx) = oneshot::channel();

        let handle = graphics.shared.future_queue.push(async move {
            let mut temp_world = World::new();
            let result = scene
                .load_into_world(
                    &mut temp_world,
                    graphics_shared,
                    Some(component_registry.as_ref()),
                    None,
                )
                .await
                .map(|active_camera| LoadedScene {
                    world: temp_world,
                    active_camera,
                });

            if tx.send(result).is_err() {
                log::warn!("Scene load result receiver dropped before completion");
            }
        });

        self.pending_scene = Some(PendingSceneLoad {
            name: scene_name.to_string(),
            receiver: rx,
            handle,
        });

        Ok(())
    }

    fn poll_pending_scene(&mut self, graphics: &mut RenderContext) {
        let Some(mut pending) = self.pending_scene.take() else {
            return;
        };

        match pending.receiver.try_recv() {
            Ok(result) => {
                let _ = graphics
                    .shared
                    .future_queue
                    .exchange_owned_as::<()>(&pending.handle);

                match result {
                    Ok(loaded) => {
                        self.on_scene_loaded(loaded, pending.name, graphics);
                    }
                    Err(err) => {
                        log::error!("Failed to load scene: {err:?}");
                        self.scene_command = SceneCommand::Quit;
                    }
                }
            }
            Err(TryRecvError::Empty) => {
                self.pending_scene = Some(pending);
            }
            Err(TryRecvError::Closed) => {
                log::error!("Scene load task for '{}' closed unexpectedly", pending.name);
                let _ = graphics
                    .shared
                    .future_queue
                    .exchange_owned_as::<()>(&pending.handle);
                self.scene_command = SceneCommand::Quit;
            }
        }
    }

    fn on_scene_loaded(
        &mut self,
        loaded: LoadedScene,
        scene_name: String,
        graphics: &mut RenderContext,
    ) {
        self.world = Box::new(loaded.world);
        *self.active_camera.lock() = Some(loaded.active_camera);
        self.current_scene = Some(scene_name);
        self.pending_scene = None;
        self.window = Some(graphics.shared.window.clone());
        self.input_state.window = Some(graphics.shared.window.clone());

        if let Err(err) = self.prepare_render_resources(graphics) {
            log::error!("Failed to prepare render pipeline: {err:?}");
        }

        if let Err(err) = self.initialise_scripts() {
            log::warn!("Unable to initialise scripts: {err:?}");
        }
    }

    fn prepare_render_resources(&mut self, graphics: &mut RenderContext) -> anyhow::Result<()> {
        self.light_manager
            .create_light_array_resources(graphics.shared.clone());

        let active_camera = self
            .active_camera
            .lock()
            .context("Active camera was not set")?;

        let mut query = self
            .world
            .query_one::<&Camera>(active_camera)
            .map_err(|_| anyhow::anyhow!("Unable to query active camera"))?;

        let camera = query
            .get()
            .ok_or_else(|| anyhow::anyhow!("Camera component missing on active entity"))?;

        let shader = Shader::new(
            graphics.shared.clone(),
            shader::shader_wesl::SHADER_SHADER,
            Some("runtime_default"),
        );

        let pipeline = graphics.create_render_pipline(
            &shader,
            vec![
                graphics.shared.texture_bind_layout.as_ref(),
                camera.layout(),
                self.light_manager.layout(),
            ],
            Some("Runtime Scene Pipeline"),
        );

        self.render_pipeline = Some(pipeline);

        self.light_manager.create_render_pipeline(
            graphics.shared.clone(),
            shader::shader_wesl::LIGHT_SHADER,
            camera,
            Some("Runtime Light Pipeline"),
        );

        Ok(())
    }

    fn initialise_scripts(&mut self) -> anyhow::Result<()> {
        let mut tag_database: HashMap<String, Vec<Entity>> = HashMap::new();

        for (entity, script) in self.world.query::<&Script>().iter() {
            for tag in &script.tags {
                tag_database.entry(tag.clone()).or_default().push(entity);
            }
        }

        if tag_database.is_empty() {
            self.scripts_ready = false;
            return Ok(());
        }

        let Some(target) = self.detect_script_target()? else {
            log::warn!(
                "Script components detected but no script artifact found next to the runtime"
            );
            self.scripts_ready = false;
            return Ok(());
        };

        self.script_manager
            .init_script(tag_database, target.clone())?;

        let world_ptr = self.world.as_mut() as WorldPtr;
        let input_ptr = self.input_state.as_mut() as InputStatePtr;
        let graphics_ptr = GRAPHICS_COMMAND.0.as_ref() as GraphicsPtr;

        self.script_manager
            .load_script(world_ptr, input_ptr, graphics_ptr)?;

        self.script_target = Some(target);
        self.scripts_ready = true;

        Ok(())
    }

    fn detect_script_target(&self) -> anyhow::Result<Option<ScriptTarget>> {
        if let Ok(path) = env::var("DROPBEAR_SCRIPT_PATH") {
            let path = PathBuf::from(path);
            if Self::is_native_library(&path) {
                return Ok(Some(ScriptTarget::Native { library_path: path }));
            }
            if Self::is_jvm_artifact(&path) {
                return Ok(Some(ScriptTarget::JVM { library_path: path }));
            }
        }

        let exe_dir = env::current_exe()
            .context("Unable to locate runtime executable path")?
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Executable has no parent directory"))?
            .to_path_buf();

        let search_dirs = [exe_dir.join("libs"), exe_dir.clone()];
        let mut native_candidates = Vec::new();
        let mut jar_candidates = Vec::new();

        for dir in search_dirs {
            if !dir.exists() {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_file() {
                        continue;
                    }
                    if Self::is_native_library(&path) {
                        native_candidates.push(path);
                    } else if Self::is_jvm_artifact(&path) {
                        jar_candidates.push(path);
                    }
                }
            }
        }

        if let Some(native) = Self::pick_preferred(native_candidates) {
            return Ok(Some(ScriptTarget::Native {
                library_path: native,
            }));
        }

        if let Some(jar) = Self::pick_preferred(jar_candidates) {
            return Ok(Some(ScriptTarget::JVM { library_path: jar }));
        }

        Ok(None)
    }

    fn is_native_library(path: &PathBuf) -> bool {
        let ext = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        match env::consts::OS {
            "windows" => ext == "dll",
            "macos" => ext == "dylib",
            _ => ext == "so",
        }
    }

    fn is_jvm_artifact(path: &PathBuf) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("jar"))
            .unwrap_or(false)
    }

    fn pick_preferred(mut candidates: Vec<PathBuf>) -> Option<PathBuf> {
        if candidates.is_empty() {
            return None;
        }

        candidates.sort_by(|a, b| {
            let priority_a = !Self::is_preferred_name(a);
            let priority_b = !Self::is_preferred_name(b);
            priority_a.cmp(&priority_b).then_with(|| {
                b.metadata()
                    .and_then(|m| m.modified())
                    .ok()
                    .cmp(&a.metadata().and_then(|m| m.modified()).ok())
            })
        });

        candidates.into_iter().next()
    }

    fn is_preferred_name(path: &PathBuf) -> bool {
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .map(|name| {
                let lower = name.to_ascii_lowercase();
                lower.contains("dropbear") || lower.contains("native")
            })
            .unwrap_or(false)
    }

    fn update_cameras(&mut self, graphics: &mut RenderContext) {
        let (width, height) = graphics.frame.screen_size;
        if height <= 0.0 {
            return;
        }
        let aspect = f64::from(width / height);

        for (_, (camera, component)) in self
            .world
            .query::<(&mut Camera, &mut CameraComponent)>()
            .iter()
        {
            camera.aspect = aspect;
            component.update(camera);
            camera.update(graphics.shared.clone());
        }
    }

    fn update_render_transforms(&mut self) {
        for (_, (renderer, transform)) in
            self.world.query::<(&mut MeshRenderer, &Transform)>().iter()
        {
            renderer.update(transform);
        }

        let mut propagated = Vec::new();
        for (entity, entity_transform) in self.world.query::<&EntityTransform>().iter() {
            let final_transform = entity_transform.propagate(&self.world, entity);
            propagated.push((entity, final_transform));
        }

        for (entity, final_transform) in propagated {
            if let Ok(mut query) = self.world.query_one::<&mut MeshRenderer>(entity) {
                if let Some(renderer) = query.get() {
                    renderer.update(&final_transform);
                }
            }

            if let Ok(mut query) = self
                .world
                .query_one::<(&mut LightComponent, &mut Light)>(entity)
            {
                if let Some((component, light)) = query.get() {
                    light.update(component, &final_transform);
                }
            }
        }
    }

    fn update_lights(&mut self, graphics: &mut RenderContext) {
        self.light_manager
            .update(graphics.shared.clone(), &self.world);
    }
}

impl Scene for RuntimeScene {
    fn load(&mut self, graphics: &mut RenderContext) {
        self.window = Some(graphics.shared.window.clone());
        self.input_state.window = Some(graphics.shared.window.clone());

        let target_scene = self
            .current_scene
            .clone()
            .or_else(|| Some(self.initial_scene.clone()))
            .or_else(|| self.scenes.keys().next().cloned());

        if let Some(scene_name) = target_scene {
            if let Err(err) = self.start_scene_load(&scene_name, graphics) {
                log::error!("Failed to start scene load: {err:?}");
                self.scene_command = SceneCommand::Quit;
            }
        } else {
            log::error!("Unable to determine a scene to load");
            self.scene_command = SceneCommand::Quit;
        }
    }

    fn update(&mut self, dt: f32, graphics: &mut RenderContext) {
        self.window = Some(graphics.shared.window.clone());
        self.input_state.window = Some(graphics.shared.window.clone());

        self.poll_pending_scene(graphics);

        if self.render_pipeline.is_none() {
            return;
        }

        window::poll(graphics.shared.window.clone());

        if self.scripts_ready {
            let world_ptr = self.world.as_mut() as WorldPtr;
            if let Err(err) = unsafe {
                self.script_manager
                    .update_script(world_ptr, &self.input_state, dt)
            } {
                log::error!("Script runtime error: {err:?}");
            }
        }

        self.update_cameras(graphics);
        self.update_render_transforms();
        self.update_lights(graphics);
    }

    fn render(&mut self, graphics: &mut RenderContext) {
        let Some(pipeline) = &self.render_pipeline else {
            return;
        };

        let color = Color {
            r: 100.0 / 255.0,
            g: 149.0 / 255.0,
            b: 237.0 / 255.0,
            a: 1.0,
        };

        let Some(active_camera) = self.active_camera.lock().clone() else {
            log::warn!("No active camera available for rendering");
            return;
        };

        let camera = match self
            .world
            .query_one::<&Camera>(active_camera)
            .and_then(|mut q| Ok(q.get().cloned()))
        {
            Ok(Some(camera)) => camera,
            _ => {
                log::warn!("Unable to fetch camera component for active camera entity");
                return;
            }
        };

        self.window = Some(graphics.shared.window.clone());

        let lights = self.world.query::<(&Light, &LightComponent)>().iter().map(|(_, (light, component))| (light.clone(), component.clone())).collect::<Vec<_>>();

        {
            let mut render_pass = graphics.clear_colour(color);
            if let Some(light_pipeline) = &self.light_manager.pipeline {
                render_pass.set_pipeline(light_pipeline);
                for (light, component) in &lights
                {
                    if !component.enabled {
                        continue;
                    }
                    if let Some(instance_buffer) = &light.instance_buffer {
                        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                    }
                    render_pass.draw_light_model(
                        light.model(),
                        camera.bind_group(),
                        light.bind_group(),
                    );
                }
            }
        }

        let mut model_batches: HashMap<ModelId, Vec<InstanceRaw>> = HashMap::new();

        for (_, renderer) in self.world.query::<&MeshRenderer>().iter() {
            model_batches
                .entry(renderer.model_id())
                .or_default()
                .push(renderer.instance.to_raw());
        }

        for (model_id, instances) in model_batches {
            let model = {
                let cache = MODEL_CACHE.lock();
                cache.values().find(|model| model.id == model_id).cloned()
            };

            let Some(model) = model else {
                log::warn!("Unable to locate model with id {:?}", model_id);
                continue;
            };

            let instance_buffer =
                graphics
                    .shared
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Runtime Instance Buffer"),
                        contents: bytemuck::cast_slice(&instances),
                        usage: wgpu::BufferUsages::VERTEX,
                    });

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
        self.script_target = None;
        self.pending_scene = None;
    }

    fn run_command(&mut self) -> SceneCommand {
        std::mem::replace(&mut self.scene_command, SceneCommand::None)
    }
}
