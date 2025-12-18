use super::*;
use crate::signal::SignalController;
use crate::spawn::PendingSpawnController;
use dropbear_engine::asset::{ASSET_REGISTRY, PointerKind};
use dropbear_engine::graphics::{InstanceRaw, RenderContext};
use dropbear_engine::model::MODEL_CACHE;
use dropbear_engine::{
    entity::{EntityTransform, MeshRenderer, Transform},
    lighting::{Light, LightComponent},
    model::{DrawLight, DrawModel},
    scene::{Scene, SceneCommand},
};
use eucalyptus_core::hierarchy::EntityTransformExt;
use eucalyptus_core::logging;
use eucalyptus_core::states::{Label, WorldLoadingStatus};
use eucalyptus_core::window::{CommandBufferPoller};
use log;
use parking_lot::Mutex;
use tokio::sync::mpsc::unbounded_channel;
use wgpu::Color;
use wgpu::util::DeviceExt;
use winit::{event_loop::ActiveEventLoop, keyboard::KeyCode};

impl Scene for Editor {
    fn load(&mut self, graphics: &mut RenderContext) {
        self.current_scene_name = {
            let last_opened = {
                let project = PROJECT.read();
                project.last_opened_scene.clone()
            };

            if let Some(scene_name) = last_opened {
                Some(scene_name)
            } else {
                let scenes = SCENES.read();
                scenes.first().map(|scene| scene.scene_name.clone())
            }
        };

        let (tx, rx) = unbounded_channel::<WorldLoadingStatus>();
        let (tx2, rx2) = oneshot::channel::<World>();
        self.progress_tx = Some(rx);
        self.world_receiver = Some(rx2);

        let graphics_shared = graphics.shared.clone();
        let active_camera_clone = self.active_camera.clone();
        let project_path_clone = self.project_path.clone();

        let dock_state_shared = Arc::new(Mutex::new(self.dock_state.clone()));
        let dock_state_for_loading = dock_state_shared.clone();

        let component_registry = self.component_registry.clone();

        let handle = graphics.shared.future_queue.push(async move {
            let mut temp_world = World::new();
            if let Err(e) = Self::load_project_config(
                graphics_shared,
                Some(tx),
                &mut temp_world,
                Some(tx2),
                active_camera_clone,
                project_path_clone,
                dock_state_for_loading,
                component_registry,
            )
            .await
            {
                // todo: change from a panic to another thing
                panic!("Failed to load project config: {}", e);
            }
        });

        self.world_load_handle = Some(handle);

        self.dock_state_shared = Some(dock_state_shared);

        self.window = Some(graphics.shared.window.clone());
        self.is_world_loaded.mark_scene_loaded();
    }

    fn update(&mut self, dt: f32, graphics: &mut RenderContext) {
        if let Some(request) = self.pending_scene_load.take() {
            self.start_async_scene_load(request.scene, graphics);
        }

        if matches!(self.editor_state, EditorState::Playing) {
            if let Some(scene_name) = self.pending_play_scene_load.take() {
                if let Err(e) = self
                    .start_async_play_scene_load_by_name(&scene_name, graphics.shared.clone())
                {
                    fatal!("Failed to start play-world scene load: {}", e);
                    self.signal = Signal::StopPlaying;
                }
            }

            if let Some(mut receiver) = self.play_world_receiver.take() {
                if let Ok(result) = receiver.try_recv() {
                    match result {
                        Ok(loaded_world) => {
                            self.ensure_play_world();
                            if let Some(play_world) = self.play_world.as_mut() {
                                **play_world = loaded_world;
                            }

                            if let Err(e) = self.reload_play_scripts() {
                                fatal!("Failed to initialise play-mode scripts: {}", e);
                                self.signal = Signal::StopPlaying;
                            } else {
                                success!("You are in play mode now! Press Escape (or F12) to exit");
                                log::info!("Play-mode scripts initialised");
                            }
                        }
                        Err(error_msg) => {
                            fatal!("Scene loading failed: {}", error_msg);
                            self.signal = Signal::StopPlaying;
                        }
                    }
                } else {
                    self.play_world_receiver = Some(receiver);
                }
            }
        }

        if let Some(mut receiver) = self.world_receiver.take() {
            self.show_project_loading_window(&graphics.shared.get_egui_context());
            if let Ok(loaded_world) = receiver.try_recv() {
                self.world = Box::new(loaded_world);
                self.is_world_loaded.mark_project_loaded();

                if let Some(dock_state_shared) = &self.dock_state_shared
                    && let Some(loaded_dock_state) = dock_state_shared.try_lock()
                {
                    self.dock_state = loaded_dock_state.clone();
                    log::debug!("Dock state updated from loaded config");
                }

                log::debug!("World received");
            } else {
                self.world_receiver = Some(receiver);
                return;
            }
        }

        if !self.is_world_loaded.is_fully_loaded() {
            log::debug!("Scene is not fully loaded, initialising...");
            return;
        } else {
            log_once::debug_once!("Scene has fully loaded");
        }

        if !self.is_world_loaded.rendering_loaded && self.is_world_loaded.is_fully_loaded() {
            self.load_wgpu_nerdy_stuff(graphics);
            return;
        }

        match self.check_up(
            graphics.shared.clone(),
            graphics.shared.future_queue.clone(),
        ) {
            Ok(_) => {}
            Err(e) => {
                fatal!("{}", e);
            }
        }

        {
            // title to projects name
            let project_title = { PROJECT.read().project_name.clone() };
            let current_scene = self.current_scene_name.clone();
            let title = if let Some(scene) = current_scene {
                format!(
                    "{} - {} | Version {} on commit {}",
                    project_title,
                    scene,
                    env!("CARGO_PKG_VERSION"),
                    env!("GIT_HASH")
                )
            } else {
                format!(
                    "{} | Version {} on commit {}",
                    project_title,
                    env!("CARGO_PKG_VERSION"),
                    env!("GIT_HASH")
                )
            };
            
            graphics.shared.window.set_title(&title);
        }

        self.poll(graphics);

        {
            // basic futurequeue spawn queue management.
            let mut completed = Vec::new();
            for (i, handle) in self.light_spawn_queue.iter().enumerate() {
                if let Some(l) = graphics
                    .shared
                    .future_queue
                    .exchange_owned_as::<Light>(handle)
                {
                    let label_component = Label::from(l.label.clone());
                    self.world.spawn((
                        label_component,
                        l,
                        LightComponent::default(),
                        Transform::default(),
                        ModelProperties::default(),
                    ));
                    success!("Spawned light successfully");
                    completed.push(i);
                }
            }

            for &i in completed.iter().rev() {
                log_once::debug_once!("Removing item {} from pending spawn list", i);
                self.light_spawn_queue.remove(i);
            }
        }

        let cache_mutex_ptr = std::sync::LazyLock::force(&MODEL_CACHE) as *const _;
        ASSET_REGISTRY.add_pointer(PointerKind::Const("model_cache"), cache_mutex_ptr as usize);

        if let Some((_, tab)) = self.dock_state.find_active_focused() {
            self.is_viewport_focused = matches!(tab, EditorTab::Viewport);
        } else {
            self.is_viewport_focused = false;
        }

        if matches!(self.editor_state, EditorState::Playing) {
            if self.input_state.pressed_keys.contains(&KeyCode::Escape) {
                self.signal = Signal::StopPlaying;
            }

            if self.play_scripts_loaded {
                if let Some(play_world) = self.play_world.as_mut() {
                    let world_ptr = play_world.as_mut() as *mut World;

                    if let Err(e) = unsafe {
                        self.script_manager
                            .update_script(world_ptr, &self.input_state, dt)
                    } {
                        fatal!("Failed to update script: {}", e);
                        self.signal = Signal::StopPlaying;
                    }
                }
            }
        }

        if self.is_viewport_focused
            && matches!(self.viewport_mode, ViewportMode::CameraMove)
            && !matches!(self.editor_state, EditorState::Playing)
        // && self.is_using_debug_camera()
        {
            let active_cam = self.active_camera.lock();
            if let Some(active_camera) = *active_cam
                && let Ok(mut query) = self
                    .world
                    .query_one::<(&mut Camera, &CameraComponent)>(active_camera)
                && let Some((camera, _)) = query.get()
            {
                for key in &self.input_state.pressed_keys {
                    match key {
                        KeyCode::KeyW => camera.move_forwards(),
                        KeyCode::KeyA => camera.move_left(),
                        KeyCode::KeyD => camera.move_right(),
                        KeyCode::KeyS => camera.move_back(),
                        KeyCode::ShiftLeft => camera.move_down(),
                        KeyCode::Space => camera.move_up(),
                        _ => {}
                    }
                }
            }
        }

        let _ = self.run_signal(graphics.shared.clone());
        
        if matches!(self.editor_state, EditorState::Playing) {
            if let Some(play_world) = self.play_world.as_mut() {
                let resolve_play_entity = |author_entity: Entity, author_world: &World, play_world: &World| {
                    let label = author_world
                        .get::<&Label>(author_entity)
                        .ok()
                        .map(|l| l.as_str().to_string())?;
                    play_world
                        .query::<&Label>()
                        .iter()
                        .find_map(|(e, l)| (l.as_str() == label).then_some(e))
                };

                if let Some(prev) = self.previously_selected_entity {
                    if let Some(prev_play) = resolve_play_entity(prev, self.world.as_ref(), play_world) {
                        if let Ok(mut q) = play_world.query_one::<&mut MeshRenderer>(prev_play)
                            && let Some(entity) = q.get()
                        {
                            entity.is_selected = false;
                        }
                    }
                }

                if let Some(sel) = self.selected_entity {
                    if let Some(sel_play) = resolve_play_entity(sel, self.world.as_ref(), play_world) {
                        if let Ok(mut q) = play_world.query_one::<&mut MeshRenderer>(sel_play)
                            && let Some(entity) = q.get()
                        {
                            entity.is_selected = true;
                        }
                    }
                }
            }
        } else {
            if let Some(e) = self.previously_selected_entity
                && let Ok(mut q) = self.world.query_one::<&mut MeshRenderer>(e)
                && let Some(entity) = q.get()
            {
                entity.is_selected = false
            }

            if let Some(e) = self.selected_entity
                && let Ok(mut q) = self.world.query_one::<&mut MeshRenderer>(e)
                && let Some(entity) = q.get()
            {
                entity.is_selected = true
            }
        }

        let current_size = graphics.shared.viewport_texture.size;
        self.size = current_size;

        let new_aspect = current_size.width as f64 / current_size.height as f64;

        {
            let (active_camera, world_for_aspect): (Option<Entity>, Option<&mut World>) = if matches!(
                self.editor_state,
                EditorState::Playing
            ) {
                (*self.play_active_camera.lock(), self.play_world.as_deref_mut())
            } else {
                (*self.active_camera.lock(), Some(self.world.as_mut()))
            };

            if let (Some(active_camera), Some(world)) = (active_camera, world_for_aspect)
                && let Ok(mut query) = world.query_one::<&mut Camera>(active_camera)
                && let Some(camera) = query.get()
            {
                camera.aspect = new_aspect;
            }
        }

        {
            let sim_world: &mut World = if matches!(self.editor_state, EditorState::Playing) {
                self.play_world
                    .as_deref_mut()
                    .unwrap_or_else(|| self.world.as_mut())
            } else {
                self.world.as_mut()
            };

            for (_entity_id, (camera, component)) in sim_world
                .query::<(&mut Camera, &mut CameraComponent)>()
                .iter()
            {
                component.update(camera);
                camera.update(graphics.shared.clone());
            }
        }

        {
            let sim_world: &mut World = if matches!(self.editor_state, EditorState::Playing) {
                self.play_world
                    .as_deref_mut()
                    .unwrap_or_else(|| self.world.as_mut())
            } else {
                self.world.as_mut()
            };

            {
                let query = sim_world.query_mut::<(&mut MeshRenderer, &Transform)>();
                for (_, (renderer, transform)) in query {
                    renderer.update(transform);
                }
            }

            {
                let mut updates = Vec::new();
                for (entity, transform) in sim_world.query::<&EntityTransform>().iter() {
                    let final_transform = transform.propagate(sim_world, entity);
                    updates.push((entity, final_transform));
                }

                for (entity, final_transform) in updates {
                    if let Ok(mut renderer) = sim_world.get::<&mut MeshRenderer>(entity) {
                        renderer.update(&final_transform);
                    }
                }
            }

            {
                let light_query = sim_world
                    .query_mut::<(&mut LightComponent, &Transform, &mut Light)>();
                for (_, (light_component, transform, light)) in light_query {
                    light.update(light_component, transform);
                }
            }

            {
                let mut updates = Vec::new();
                for (entity, transform) in sim_world.query::<&EntityTransform>().iter() {
                    let final_transform = transform.propagate(sim_world, entity);
                    updates.push((entity, final_transform));
                }

                for (entity, final_transform) in updates {
                    if let Ok(mut q) =
                        sim_world.query_one::<(&mut LightComponent, &mut Light)>(entity)
                    {
                        if let Some((light_component, light)) = q.get() {
                            light.update(light_component, &final_transform);
                        }
                    }
                }
            }
        }

        {
            let sim_world: &World = if matches!(self.editor_state, EditorState::Playing) {
                self.play_world.as_deref().unwrap_or_else(|| self.world.as_ref())
            } else {
                self.world.as_ref()
            };

            self.light_manager
                .update(graphics.shared.clone(), sim_world);

            self.nerd_stats.update(dt, sim_world.len());
        }

        self.input_state.window = self.window.clone();
        self.previously_selected_entity = self.selected_entity;
    }

    fn render(&mut self, graphics: &mut RenderContext) {
        // cornflower blue
        let color = Color {
            r: 100.0 / 255.0,
            g: 149.0 / 255.0,
            b: 237.0 / 255.0,
            a: 1.0,
        };

        self.color = color;
        self.size = graphics.shared.viewport_texture.size;
        self.texture_id = Some(*graphics.shared.texture_id.clone());
        {
            self.show_ui(&graphics.shared.get_egui_context());
        }
        self.nerd_stats.show(&graphics.shared.get_egui_context());

        self.window = Some(graphics.shared.window.clone());
        logging::render(&graphics.shared.get_egui_context());
        if let Some(pipeline) = &self.render_pipeline {
            log_once::debug_once!("Found render pipeline");
            let (active_camera, world): (Option<Entity>, &World) = if matches!(
                self.editor_state,
                EditorState::Playing
            ) {
                let world = self
                    .play_world
                    .as_deref()
                    .unwrap_or_else(|| self.world.as_ref());
                (*self.play_active_camera.lock(), world)
            } else {
                (*self.active_camera.lock(), self.world.as_ref())
            };

            if let Some(active_camera) = active_camera {
                let cam = {
                    if let Ok(mut query) = world.query_one::<&Camera>(active_camera) {
                        query.get().cloned()
                    } else {
                        None
                    }
                };

                if let Some(camera) = cam {
                    let lights = {
                        let mut lights = Vec::new();
                        let mut light_query = world.query::<(&Light, &LightComponent)>();
                        for (_, (light, comp)) in light_query.iter() {
                            lights.push((light.clone(), comp.clone()));
                        }
                        lights
                    };

                    let entities = {
                        let mut entities = Vec::new();
                        let mut entity_query = world.query::<&MeshRenderer>();
                        for (_, renderer) in entity_query.iter() {
                            entities.push(renderer.clone());
                        }
                        entities
                    };

                    {
                        // light cube rendering
                        let mut render_pass = graphics.clear_colour(color);
                        if let Some(light_pipeline) = &self.light_manager.pipeline {
                            render_pass.set_pipeline(light_pipeline);
                            for (light, _component) in &lights {
                                render_pass.set_vertex_buffer(
                                    1,
                                    light.instance_buffer.as_ref().unwrap().slice(..),
                                );
                                if _component.visible {
                                    render_pass.draw_light_model(
                                        &light.cube_model,
                                        camera.bind_group(),
                                        light.bind_group(),
                                    );
                                }
                            }
                        }
                    }

                    let mut model_batches: HashMap<ModelId, Vec<InstanceRaw>> = HashMap::new();
                    for renderer in &entities {
                        let model_ptr = renderer.model_id();
                        let instance_raw = renderer.instance.to_raw();
                        model_batches
                            .entry(model_ptr)
                            .or_default()
                            .push(instance_raw);
                    }

                    for (model_ptr, instances) in model_batches {
                        {
                            let model_opt = {
                                let cache = MODEL_CACHE.lock();
                                cache.values().find(|m| m.id == model_ptr).cloned()
                            };

                            if let Some(model) = model_opt {
                                let instance_buffer = graphics.shared.device.create_buffer_init(
                                    &wgpu::util::BufferInitDescriptor {
                                        label: Some("Batched Instance Buffer"),
                                        contents: bytemuck::cast_slice(&instances),
                                        usage: wgpu::BufferUsages::VERTEX,
                                    },
                                );

                                {
                                    // normal model rendering
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

                                log_once::debug_once!("Rendered {:?}", model_ptr);
                            } else {
                                log_once::error_once!("No such MODEL as {:?}", model_ptr);
                            }
                        }
                    }
                } else {
                    log_once::error_once!("Camera returned None");
                }
            } else {
                log_once::error_once!("No active camera found");
            }
        } else {
            if self.is_world_loaded.is_fully_loaded() {
                log_once::warn_once!("No render pipeline exists");
            } else {
                log_once::debug_once!("No render pipeline exists, but world not loaded yet");
            }
        }
    }

    fn exit(&mut self, _event_loop: &ActiveEventLoop) {}

    fn run_command(&mut self) -> SceneCommand {
        std::mem::replace(&mut self.scene_command, SceneCommand::None)
    }
}
