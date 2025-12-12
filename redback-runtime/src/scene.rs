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
use eucalyptus_core::states::{ConfigFile, Script};
use eucalyptus_core::traits::registry::ComponentRegistry;
use eucalyptus_core::window::{self, GRAPHICS_COMMAND};
use hecs::{Entity, World};
use parking_lot::Mutex;
use tokio::sync::oneshot;
use tokio::sync::oneshot::error::TryRecvError;
use eucalyptus_core::egui;
use eucalyptus_core::egui::CentralPanel;

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
    pub fn new(project_config: RuntimeProjectConfig, window_config: ConfigFile) -> anyhow::Result<Self> {
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
            window_config: window_config.clone(),
            scenes,
            initial_scene,
            world: Box::new(World::new()),
            input_state: Box::new(InputState::new()),
            active_camera: Arc::new(Mutex::new(None)),
            render_pipeline: None,
            light_manager: LightManager::new(),
            current_scene: None,
            component_registry: Arc::new(ComponentRegistry::new()),
            script_manager: ScriptManager::new(window_config.jvm_args)?,
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
                        panic!("Failed to load scene: {err:?}");
                    }
                }
            }
            Err(TryRecvError::Empty) => {
                self.pending_scene = Some(pending);
            }
            Err(TryRecvError::Closed) => {
                let _ = graphics
                    .shared
                    .future_queue
                    .exchange_owned_as::<()>(&pending.handle);
                panic!("Scene load task for '{}' closed unexpectedly", pending.name);
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
        self.current_scene = Some(scene_name.clone());
        self.pending_scene = None;
        self.window = Some(graphics.shared.window.clone());
        self.input_state.window = Some(graphics.shared.window.clone());

        if let Err(err) = self.prepare_render_resources(graphics) {
            panic!("Failed to prepare render pipeline: {err:?}");
        }

        if let Err(err) = self.initialise_scripts() {
            panic!("Unable to initialise scripts: {err:?}");
        }
        log::debug!("Scene [{}] loaded", scene_name);

        self.display_all_entities()
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

        let target = self.detect_script_target()?;

        self.script_manager
            .init_script(self.window_config.jvm_args.clone(), tag_database, target.clone())?;

        let world_ptr = self.world.as_mut() as WorldPtr;
        let input_ptr = self.input_state.as_mut() as InputStatePtr;
        let graphics_ptr = GRAPHICS_COMMAND.0.as_ref() as GraphicsPtr;

        self.script_manager
            .load_script(world_ptr, input_ptr, graphics_ptr)?;

        self.script_target = Some(target);
        self.scripts_ready = true;

        Ok(())
    }

    fn detect_script_target(&self) -> anyhow::Result<ScriptTarget> {
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
                        log::debug!("Found native library: {}", path.display());
                        native_candidates.push(path);
                    } else if Self::is_jvm_artifact(&path) {
                        log::debug!("Found jvm artifact: {}", path.display());
                        jar_candidates.push(path);
                    }
                }
            }
        }

        let project_name = &self.project_config.project_name;
        if let Some(native) =
            Self::pick_native_by_project_name(native_candidates.clone(), project_name)
        {
            return Ok(ScriptTarget::Native {
                library_path: native,
            });
        }

        if let Some(native) = Self::pick_preferred(native_candidates) {
            log::warn!(
                "Using fallback native script '{}' (no project-name match for '{}')",
                native.display(),
                project_name
            );
            return Ok(ScriptTarget::Native {
                library_path: native,
            });
        }

        // For JVM, any JAR works
        if let Some(_jar) = Self::pick_preferred(jar_candidates) {
            #[cfg(feature = "jvm")]
            return Ok(ScriptTarget::JVM { library_path: _jar });
            #[cfg(not(feature = "jvm"))]
            return Err(anyhow::anyhow!("\
                JVM support is not enabled in this build. If you are the developer, please enable it \
                with the `jvm` feature flag on redback-runtime. \n\
                If you are a user, please contact the \
                developer to enable JVM support. Maybe they will respond 🤷‍♂️?\
                For now, you will have to stick with a .{}...\
            ", Self::native_extension()));
        }

        anyhow::bail!(
            "Unable to locate a suitable script target. \n\
            You must place either a .{} or a .jar file in the same directory as the runtime executable",
            Self::native_extension()
        )
    }

    fn pick_native_by_project_name(candidates: Vec<PathBuf>, project_name: &str) -> Option<PathBuf> {
        if candidates.is_empty() {
            return None;
        }

        let mut exact = Vec::new();
        let mut fuzzy = Vec::new();

        for path in candidates {
            let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };

            if Self::is_exact_project_match(stem, project_name) {
                exact.push(path);
            } else if Self::is_fuzzy_project_match(stem, project_name) {
                fuzzy.push(path);
            }
        }

        Self::pick_most_recent(exact).or_else(|| Self::pick_most_recent(fuzzy))
    }

    fn pick_most_recent(paths: Vec<PathBuf>) -> Option<PathBuf> {
        if paths.is_empty() {
            return None;
        }

        paths
            .into_iter()
            .max_by_key(|path| path.metadata().and_then(|m| m.modified()).ok())
    }

    fn is_exact_project_match(candidate: &str, project_name: &str) -> bool {
        candidate.eq_ignore_ascii_case(project_name)
    }

    fn is_fuzzy_project_match(candidate: &str, project_name: &str) -> bool {
        let candidate_norm = Self::normalise_token(candidate);
        let trimmed = candidate_norm
            .strip_prefix("lib")
            .unwrap_or(candidate_norm.as_str());
        let project_norm = Self::normalise_token(project_name);

        Self::token_contains(&candidate_norm, &project_norm)
            || Self::token_contains(trimmed, &project_norm)
    }

    fn token_contains(candidate: &str, project: &str) -> bool {
        if candidate == project {
            return true;
        }

        candidate.starts_with(project)
            || candidate
                .split(['-', '_', '.'])
                .any(|segment| segment == project)
    }

    fn normalise_token(value: &str) -> String {
        value
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() {
                    c.to_ascii_lowercase()
                } else if matches!(c, '-' | '_' | '.') {
                    c
                } else {
                    '-'
                }
            })
            .collect()
    }

    fn native_extension() -> &'static str {
        match env::consts::OS {
            "windows" => "dll",
            "macos" => "dylib",
            _ => "so",
        }
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

        for (e, (camera, component)) in self
            .world
            .query::<(&mut Camera, &mut CameraComponent)>()
            .iter()
        {
            camera.aspect = aspect;
            component.update(camera);
            camera.update(graphics.shared.clone());
            if e == *self.active_camera.lock().as_ref().unwrap() {
                camera.debug_camera_state();
            }
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
        CentralPanel::default().show(&graphics.shared.get_egui_context(), |ui| {
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
                    panic!("Script runtime error: {err:?}");
                }
            }

            self.update_cameras(graphics);
            self.update_render_transforms();
            self.update_lights(graphics);

            let size = graphics.shared.window.outer_size();
            let texture_id = Some(*graphics.shared.texture_id.clone());
            if let Some(view) = texture_id {
                ui.add_sized(
                    [size.width as f32, size.height as f32],
                    egui::Image::new((
                        view,
                        egui::vec2(size.width as f32, size.height as f32)
                    ))
                );
            }
        });
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
