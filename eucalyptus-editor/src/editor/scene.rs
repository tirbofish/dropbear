use crossbeam_channel::unbounded;
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
use eucalyptus_core::states::{CustomProperties, Label, WorldLoadingStatus};
use log;
use parking_lot::Mutex;
use wgpu::Color;
use wgpu::util::DeviceExt;
use winit::{event_loop::ActiveEventLoop, keyboard::KeyCode};
use eucalyptus_core::physics::collider::ColliderGroup;
use eucalyptus_core::physics::collider::shader::ColliderUniform;

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

        log::debug!("Current scene name = {:?}", self.current_scene_name);

        let (tx, rx) = unbounded::<WorldLoadingStatus>();
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

    fn physics_update(&mut self, _dt: f32, _graphics: &mut RenderContext) {}

    fn update(&mut self, dt: f32, graphics: &mut RenderContext) {
        if let Some(rx) = &self.play_mode_exit_rx {
            if rx.try_recv().is_ok() {
                log::info!("Play mode process has exited, returning to editing mode");
                success!("Exited play mode");
                self.editor_state = EditorState::Editing;
                self.play_mode_exit_rx = None;
                self.play_mode_pid = None;
            }
        }

        if let Some(request) = self.pending_scene_load.take() {
            self.start_async_scene_load(request.scene, graphics);
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
                        CustomProperties::default(),
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

        let current_size = graphics.shared.viewport_texture.size;
        self.size = current_size;

        let new_aspect = current_size.width as f64 / current_size.height as f64;

        {
            let active_camera = *self.active_camera.lock();
            let world = self.world.as_mut();

            if let Some(active_camera) = active_camera
                && let Ok(mut query) = world.query_one::<&mut Camera>(active_camera)
                && let Some(camera) = query.get()
            {
                camera.aspect = new_aspect;
            }
        }

        {
            let sim_world = self.world.as_mut();

            for (_entity_id, (camera, component)) in sim_world
                .query::<(&mut Camera, &mut CameraComponent)>()
                .iter()
            {
                component.update(camera);
                camera.update(graphics.shared.clone());
            }
        }

        {
            let sim_world = self.world.as_mut();

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
        }

        {
            let sim_world = self.world.as_ref();

            self.light_manager
                .update(graphics.shared.clone(), sim_world);

            self.nerd_stats.write().record_stats(dt, sim_world.len() as u32);
        }

        self.input_state.window = self.window.clone();
        self.previously_selected_entity = self.selected_entity;
        self.input_state.mouse_delta = None;
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

        self.window = Some(graphics.shared.window.clone());
        logging::render(&graphics.shared.get_egui_context());
        if let Some(pipeline) = &self.render_pipeline {
            log_once::debug_once!("Found render pipeline");
            let (active_camera, world): (Option<Entity>, &World) =
                (*self.active_camera.lock(), self.world.as_ref());

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

                    // mipmap gen
                    // LOL THIS DOESNT WORK FUCK WHY CANT YOU BE LIKE OPENGL AND MAKE MIPMAPPING SIMPLER FML
                    if let Some(mipmap) = &self.mipmap_generator {
                        let graphics = graphics.shared.clone();
                        let mut encoder = graphics.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("mip map generator command encoder descriptor"),
                        });

                        for i in ASSET_REGISTRY.iter_material() {
                            mipmap.generate(&graphics.device, &mut encoder, &i.diffuse_texture.texture);
                        }

                        graphics.queue.submit(Some(encoder.finish()));
                    }

                    { // light cube rendering
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

                    { // standard model rendering
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
                    }

                    {
                        if let Some(collider_pipeline) = &self.collider_wireframe_pipeline {
                            let mut render_pass = graphics.continue_pass();
                            render_pass.set_pipeline(&collider_pipeline.pipeline);
                            render_pass.set_bind_group(0, camera.bind_group(), &[]);

                            let colliders_to_render = {
                                let mut colliders = Vec::new();

                                let mut q = world.query::<(&Label, &ColliderGroup)>();
                                for (entity, (label, group)) in q.iter() {
                                    for collider in &group.colliders {
                                        let transform = Transform::new().with_offset(collider.translation, collider.rotation);
                                        colliders.push((entity, collider.clone(), transform.clone(), label.clone()))
                                    }
                                }

                                colliders
                            };

                            for (_entity, collider, transform, _label) in colliders_to_render {

                                let color = [1.0, 1.0, 0.0, 1.0]; // yellow

                                let collider_uniform = ColliderUniform::new(&transform, color);

                                let collider_buffer = graphics.shared.device.create_buffer_init(
                                    &wgpu::util::BufferInitDescriptor {
                                        label: Some("Collider Uniform Buffer"),
                                        contents: bytemuck::cast_slice(&[collider_uniform]),
                                        usage: wgpu::BufferUsages::UNIFORM,
                                    },
                                );

                                let collider_bind_group = graphics.shared.device.create_bind_group(
                                    &wgpu::BindGroupDescriptor {
                                        layout: &collider_pipeline.bind_group_layout,
                                        entries: &[wgpu::BindGroupEntry {
                                            binding: 0,
                                            resource: collider_buffer.as_entire_binding(),
                                        }],
                                        label: Some("collider bind group"),
                                    },
                                );

                                render_pass.set_bind_group(1, &collider_bind_group, &[]);

                                let geometry = Self::create_wireframe_geometry(
                                    graphics.shared.clone(),
                                    &collider.shape,
                                );

                                render_pass.set_vertex_buffer(0, geometry.vertex_buffer.slice(..));
                                render_pass.set_index_buffer(
                                    geometry.index_buffer.slice(..),
                                    wgpu::IndexFormat::Uint16,
                                );
                                render_pass.draw_indexed(0..geometry.index_count, 0, 0..1);
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

impl Editor {
    fn create_wireframe_geometry(
        graphics: Arc<SharedGraphicsContext>,
        shape: &ColliderShape,
    ) -> WireframeGeometry {
        match shape {
            ColliderShape::Box { half_extents } => {
                WireframeGeometry::box_wireframe(graphics, *half_extents)
            }
            ColliderShape::Sphere { radius } => {
                WireframeGeometry::sphere_wireframe(graphics, *radius, 16, 16)
            }
            ColliderShape::Capsule { half_height, radius } => {
                WireframeGeometry::capsule_wireframe(graphics, *half_height, *radius, 16)
            }
            ColliderShape::Cylinder { half_height, radius } => {
                WireframeGeometry::cylinder_wireframe(graphics, *half_height, *radius, 16)
            }
            ColliderShape::Cone { half_height, radius } => {
                WireframeGeometry::cone_wireframe(graphics, *half_height, *radius, 16)
            }
        }
    }
}