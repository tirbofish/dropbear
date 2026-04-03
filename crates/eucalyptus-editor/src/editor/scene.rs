use super::*;
use crate::signal::SignalController;
use crate::spawn::PendingSpawnController;
use crossbeam_channel::unbounded;
use dropbear_engine::graphics::CommandEncoder;
use dropbear_engine::{
    entity::{EntityTransform, MeshRenderer, Transform},
    lighting::Light,
    scene::{Scene, SceneCommand},
};
use eucalyptus_core::billboard::BillboardComponent;
use eucalyptus_core::component::KotlinComponentDecl;
use eucalyptus_core::properties::CustomProperties;
use eucalyptus_core::states::{Label, SCENES, WorldLoadingStatus};
use eucalyptus_core::transform::OnRails;
use eucalyptus_core::ui::HUDComponent;
use hecs::Entity;
use kino_ui::rendering::KinoRenderTargetId;
use log;
use magna_carta::ScriptManifest;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::{
    fs,
    path::{Path, PathBuf},
};
use winit::event::{MouseScrollDelta, TouchPhase};
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, keyboard::KeyCode};
use eucalyptus_core::rendering::RendererCommon;

impl Scene for Editor {
    fn load(&mut self, graphics: Arc<SharedGraphicsContext>, _ui: &mut Ui) {
        {
            let src_path = {
                let project = PROJECT.read();
                project.project_path.join("src")
            };
            if src_path.exists() {
                if let Some(registry) = Arc::get_mut(&mut self.component_registry) {
                    let mut processor = match magna_carta::KotlinProcessor::new() {
                        Ok(p) => p,
                        Err(e) => {
                            log::warn!(
                                "Failed to create KotlinProcessor for component scan: {}",
                                e
                            );
                            return;
                        }
                    };
                    let mut manifest = ScriptManifest::new();
                    if let Err(e) =
                        magna_carta::visit_kotlin_files(&src_path, &mut processor, &mut manifest)
                    {
                        log::warn!("Kotlin component scan failed: {}", e);
                    } else {
                        let count = manifest.components().len();
                        for item in manifest.components() {
                            registry.register_kotlin_descriptor(KotlinComponentDecl {
                                fqcn: item.fqcn().to_owned(),
                                type_name: item.simple_name().to_owned(),
                                category: None,
                                description: None,
                            });
                        }
                        log::info!(
                            "Registered {} Kotlin component descriptor(s) from project sources",
                            count
                        );
                    }
                } else {
                    log::warn!(
                        "Could not obtain exclusive access to component_registry for Kotlin component scan"
                    );
                }
            }
        }

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

        let graphics_shared = graphics.clone();
        let active_camera_clone = self.active_camera.clone();
        let project_path_clone = self.project_path.clone();

        let game_dock_state_shared = Arc::new(Mutex::new(self.game_editor_dock_state.clone()));
        let game_dock_state_for_loading = game_dock_state_shared.clone();

        let ui_dock_state_shared = Arc::new(Mutex::new(self.ui_editor_dock_state.clone()));
        let ui_dock_state_for_loading = ui_dock_state_shared.clone();

        let component_registry = self.component_registry.clone();

        let handle = graphics.future_queue.push(async move {
            let mut temp_world = World::new();
            if let Err(e) = Self::load_project_config(
                graphics_shared,
                Some(tx),
                &mut temp_world,
                Some(tx2),
                active_camera_clone,
                project_path_clone,
                game_dock_state_for_loading,
                ui_dock_state_for_loading,
                component_registry,
            )
            .await
            {
                // todo: change from a panic to another thing
                panic!("Failed to load project config: {}", e);
            }
        });

        self.world_load_handle = Some(handle);

        self.game_dock_state_shared = Some(game_dock_state_shared);
        self.ui_dock_state_shared = Some(ui_dock_state_shared);

        self.window = Some(graphics.window.clone());
        self.is_world_loaded.mark_scene_loaded();
    }

    fn physics_update(
        &mut self,
        _dt: f32,
        _graphics: std::sync::Arc<dropbear_engine::graphics::SharedGraphicsContext>,
        _ui: &mut Ui,
    ) {
    }

    fn update(
        &mut self,
        dt: f32,
        graphics: std::sync::Arc<dropbear_engine::graphics::SharedGraphicsContext>,
        ui: &mut Ui,
    ) {
        self.dt = dt;

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
            self.start_async_scene_load(request.scene, graphics.clone());
        }

        if let Some(mut receiver) = self.world_receiver.take() {
            self.show_project_loading_window(ui.ctx());
            if let Ok(loaded_world) = receiver.try_recv() {
                self.world = Box::new(loaded_world);
                self.is_world_loaded.mark_project_loaded();

                if let Some(dock_state_shared) = &self.game_dock_state_shared
                    && let Some(loaded_dock_state) = dock_state_shared.try_lock()
                {
                    self.game_editor_dock_state = loaded_dock_state.clone();
                    log::debug!("Game dock state updated from loaded config");
                }

                if let Some(dock_state_shared) = &self.ui_dock_state_shared
                    && let Some(loaded_dock_state) = dock_state_shared.try_lock()
                {
                    self.ui_editor_dock_state = loaded_dock_state.clone();
                    log::debug!("UI dock state updated from loaded config");
                }

                log::debug!("World received");
            } else {
                self.world_receiver = Some(receiver);
                return;
            }
        }

        {
            for (rails, et) in
                self.world.query_mut::<(&mut OnRails, &mut EntityTransform)>()
            {
                if let Some((pos, rot)) = rails.pending_transform.take() {
                    et.world_mut().position = pos;
                    et.world_mut().rotation = rot;
                }
            }
        }

        self.component_registry.update_components(
            self.world.as_mut(),
            &mut self.physics_state,
            dt,
            graphics.clone(),
        );

        if !self.is_world_loaded.is_fully_loaded() {
            log::debug!("Scene is not fully loaded, initialising...");
            return;
        } else {
            log_once::debug_once!("Scene has fully loaded");
        }

        {
            let desired = EDITOR_SETTINGS.read().anti_aliasing_mode;
            let current = *graphics.antialiasing.read();

            if desired != current {
                if self.pending_aa_reload != Some(desired)
                    && matches!(self.scene_command, SceneCommand::None)
                {
                    log::debug!("Anti aliasing mode changed, requesting WGPU update");
                    self.scene_command = SceneCommand::SetAntialiasing(desired);
                    self.pending_aa_reload = Some(desired);
                }
            } else if self.pending_aa_reload.is_some() && self.signal.is_empty() {
                log::debug!("Anti aliasing mode applied, reloading WGPU data");
                self.signal.push_back(Signal::ReloadWGPUData {
                    skybox_texture: None,
                });
                self.pending_aa_reload = None;
            }
        }

        if !self.is_world_loaded.rendering_loaded && self.is_world_loaded.is_fully_loaded() {
            self.load_wgpu_nerdy_stuff(graphics, None);
            return;
        }

        match self.check_up(graphics.clone(), graphics.future_queue.clone()) {
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

            graphics.window.set_title(&title);
        }

        {
            if let Some(fps) = EDITOR_SETTINGS.read().target_fps.get() {
                log_once::debug_once!("setting new fps for editor: {}", fps);
                if matches!(self.scene_command, SceneCommand::None) {
                    self.scene_command = SceneCommand::SetFPS(*fps);
                }
            }
        }

        {
            // basic futurequeue spawn queue management.
            let mut completed = Vec::new();
            for (i, handle) in self.light_spawn_queue.iter().enumerate() {
                if let Some(l) = graphics.future_queue.exchange_owned_as::<Light>(handle) {
                    let label_component = Label::from(l.label.clone());
                    self.world.spawn((
                        label_component,
                        l,
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

        if let Some((_, tab)) = self.game_editor_dock_state.find_active_focused() {
            let viewport_tab = self.tab_registry.id_for_title("Viewport");
            self.is_viewport_focused = viewport_tab.map_or(false, |id| *tab == id);
        } else {
            self.is_viewport_focused = false;
        }

        if self.is_viewport_focused
            && matches!(self.viewport_mode, ViewportMode::CameraMove)
            && !matches!(self.editor_state, EditorState::Playing)
        // && self.is_using_debug_camera()
        {
            let active_cam = self.active_camera.lock();
            if let Some(active_camera) = *active_cam
                && let Ok((camera, _)) = self
                    .world
                    .query_one::<(&mut Camera, &CameraComponent)>(active_camera)
                    .get()
            {
                for key in &self.input_state.pressed_keys {
                    match key {
                        KeyCode::KeyW => camera.move_forwards(dt),
                        KeyCode::KeyA => camera.move_left(dt),
                        KeyCode::KeyD => camera.move_right(dt),
                        KeyCode::KeyS => camera.move_back(dt),
                        KeyCode::ShiftLeft => camera.move_down(dt),
                        KeyCode::Space => camera.move_up(dt),
                        _ => {}
                    }
                }
            }
        }

        let _ = self.run_signal(graphics.clone(), ui.ctx());

        if let Some(e) = self.previously_selected_entity
            && let Ok(entity) = self.world.query_one::<&mut MeshRenderer>(e).get()
        {
            entity.is_selected = false
        }

        if let Some(e) = self.selected_entity
            && let Ok(entity) = self.world.query_one::<&mut MeshRenderer>(e).get()
        {
            entity.is_selected = true
        }

        let current_size = graphics.viewport_texture.size;
        self.size = current_size;

        let new_aspect = current_size.width as f64 / current_size.height as f64;

        {
            let active_camera = *self.active_camera.lock();
            let world = self.world.as_mut();

            if let Some(active_camera) = active_camera
                && let Ok(camera) = world.query_one::<&mut Camera>(active_camera).get()
            {
                camera.aspect = new_aspect;
            }
        }

        if let Some(l) = &mut self.light_cube_pipeline {
            l.update(graphics.clone(), &self.world);
        }

        {
            self.nerd_stats
                .write()
                .record_stats(dt, self.world.len() as u32);
        }

        let open_ui_editor = ui.ctx().data_mut(|d: &mut egui::util::IdTypeMap| {
            d.get_temp::<Option<Entity>>(egui::Id::new("open_ui_editor"))
                .flatten()
                .inspect(|_| d.remove::<Option<Entity>>(egui::Id::new("open_ui_editor")))
        });

        if let Some(entity) = open_ui_editor {
            self.current_page = EditorTabVisibility::UIEditor;
            self.ui_editor.active_entity = Some(entity);
        }

        let (overlay_billboard, overlay_hud) = if let Some(scene_name) = &self.current_scene_name {
            let scenes = SCENES.read();
            if let Some(scene) = scenes.iter().find(|s| s.scene_name == *scene_name) {
                (scene.settings.overlay_billboard, scene.settings.overlay_hud)
            } else {
                (true, false)
            }
        } else {
            (true, false)
        };

        if let Some(kino) = &mut self.kino {
            if overlay_billboard {
                let billboard_trees: Vec<(u64, kino_ui::WidgetTree)> = self
                    .world
                    .query::<(Entity, &BillboardComponent)>()
                    .iter()
                    .filter_map(|(entity, billboard)| {
                        if billboard.enabled {
                            Some((entity.to_bits().get(), billboard.ui_tree.clone()))
                        } else {
                            None
                        }
                    })
                    .collect();

                for (entity_id, tree) in billboard_trees {
                    kino.begin(KinoRenderTargetId::Billboard(entity_id));
                    tree.submit(kino);
                    kino.flush();
                }
            }

            if overlay_hud {
                let hud_trees: Vec<kino_ui::WidgetTree> = self
                    .world
                    .query::<&HUDComponent>()
                    .iter()
                    .map(|hud| hud.tree().clone())
                    .collect();

                if !hud_trees.is_empty() {
                    kino.begin(KinoRenderTargetId::HUD);
                    for tree in hud_trees {
                        tree.submit(kino);
                    }
                    kino.flush();
                }
            }
        }

        self.input_state.window = self.window.clone();
        self.previously_selected_entity = self.selected_entity;
        self.input_state.mouse_delta = None;
    }

    fn render(&mut self, graphics: Arc<SharedGraphicsContext>, ui: &mut Ui) {
        self.editor_specific_render(&graphics, ui);

        let hdr = graphics.hdr.read();
        let mut encoder = CommandEncoder::new(graphics.clone(), Some("runtime viewport encoder"));

        let Some(active_camera) = self.active_camera.lock().as_ref().cloned() else { return };
        log_once::debug_once!("Active camera found: {:?}", active_camera);
        let Some(camera) = self.world.query_one::<&Camera>(active_camera).get().ok().cloned() else { return };
        log_once::debug_once!("Camera ready: {}", camera.label);

        RendererCommon::clear_viewport(&graphics, &mut encoder, &hdr);

        if let Some(p) = &mut self.light_cube_pipeline {
            p.update(graphics.clone(), &self.world);
        }
        let (lights, enabled_light_count) = RendererCommon::collect_lights(&self.world);

        {
            let Some(globals) = &mut self.shader_globals else { return };
            globals.data.num_lights = enabled_light_count;
            if let Some(scene_name) = &self.current_scene_name {
                let scenes = SCENES.read();
                if let Some(scene) = scenes.iter().find(|s| s.scene_name == *scene_name) {
                    globals.data.ambient_strength = scene.settings.ambient_strength;
                }
            }
            globals.write(&graphics.queue);
        }

        let mut batches = HashMap::new();
        let default_skinning = self.animation_pipeline.as_ref().map(|p| p.skinning_buffer.buffer().clone());
        RendererCommon::locate_renderers(&self.world, &mut batches, graphics.clone(), &default_skinning);

        let (_, model_cache) = RendererCommon::prepare_models(&graphics, &batches, &mut self.instance_buffer_cache);

        if self.last_active_camera_for_per_frame != Some(active_camera) {
            self.last_active_camera_for_per_frame = Some(active_camera);
            if let (Some(pipeline), Some(globals), Some(light_pipeline)) = (
                self.main_render_pipeline.as_mut(),
                self.shader_globals.as_ref(),
                self.light_cube_pipeline.as_ref(),
            ) {
                pipeline.per_frame = None;
                pipeline.per_frame_bind_group(
                    graphics.clone(),
                    globals.buffer.buffer(),
                    camera.buffer(),
                    light_pipeline.light_buffer(),
                );
            }
        }

        let sky = self.sky_pipeline.as_ref().expect("Sky pipeline must be initialised");
        let environment_bind_group = &sky.environment_bind_group;

        let Some(pipeline) = self.main_render_pipeline.as_ref() else {
            log_once::warn_once!("Render pipeline not ready");
            return;
        };
        let Some(animation_defaults) = self.animation_pipeline.as_ref() else {
            log_once::warn_once!("Animation pipeline not ready");
            return;
        };

        let per_frame_bind_group = pipeline.per_frame.as_ref()
            .expect("Per-frame bind group not initialised")
            .clone();

        RendererCommon::render_light_cubes(&graphics, &mut encoder, &hdr, &lights, &camera, self.light_cube_pipeline.as_ref());

        RendererCommon::render_models(
            &graphics, &mut encoder, &hdr,
            &self.world, &batches, &model_cache,
            &per_frame_bind_group, environment_bind_group,
            &pipeline, animation_defaults,
            &self.instance_buffer_cache,
            &mut self.animated_instance_buffers,
            &mut self.animated_bind_group_cache,
            &mut self.static_bind_group_cache,
            &mut self.last_morph_info_per_mesh,
        );

        RendererCommon::render_sky(&graphics, &mut encoder, &hdr, sky);

        RendererCommon::render_collider_debug(
            &graphics,
            &self.world,
            self.current_scene_name.as_deref(),
        );

        RendererCommon::render_billboards(
            &graphics, &mut encoder, &hdr, &camera,
            &self.world,
            self.kino.as_mut(),
            self.billboard_pipeline.as_ref(),
        );

        if let Some(debug_draw) = graphics.debug_draw.lock().as_mut() {
            let view_proj = Mat4::from_cols_array_2d(&camera.uniform.view_proj);
            debug_draw.flush(graphics.clone(), &mut encoder, view_proj);
        }

        hdr.process(&mut encoder, &graphics.viewport_texture.view);
        if let Err(e) = encoder.submit() { log_once::error_once!("{}", e); }

        {
            let Some(kino) = &mut self.kino else { return };
            let mut encoder = CommandEncoder::new(graphics.clone(), Some("kino encoder"));
            kino.render(&graphics.device, &graphics.queue, &mut encoder, hdr.view());
            if let Err(e) = encoder.submit() {
                log_once::error_once!("Unable to submit kino: {}", e);
            }
        }
    }

    fn exit(&mut self, _event_loop: &ActiveEventLoop) {}

    fn handle_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::DroppedFile(path) => {
                log::debug!("Dropped file: {}", path.display());
                self.handle_file_drop(path);
            }
            WindowEvent::HoveredFile(path_buf) => {
                log_once::debug_once!("Hovering file: {}", path_buf.display());
            }
            WindowEvent::HoveredFileCancelled => {
                log_once::debug_once!("Hover cancelled");
            }
            WindowEvent::MouseWheel { delta, phase, .. } => {
                if matches!(self.viewport_mode, ViewportMode::CameraMove)
                    && !matches!(phase, TouchPhase::Cancelled)
                {
                    let active = { self.active_camera.lock().clone() };
                    if let Some(e) = active {
                        if let Ok(mut cam) = self.world.get::<&mut Camera>(e) {
                            let speed_delta = match delta {
                                MouseScrollDelta::LineDelta(_, y) => *y as f64 * 2.0,
                                MouseScrollDelta::PixelDelta(pos) => pos.y * 0.015,
                            };

                            cam.settings.speed = (cam.settings.speed + speed_delta).max(0.1);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn run_command(&mut self) -> SceneCommand {
        std::mem::replace(&mut self.scene_command, SceneCommand::None)
    }
}

impl Editor {
    fn handle_file_drop(&mut self, path: &PathBuf) {
        let project_root = { PROJECT.read().project_path.clone() };
        if project_root.as_os_str().is_empty() {
            log::warn!("Drop ignored: no project loaded");
            return;
        }

        if !path.exists() {
            log::warn!("Drop ignored: '{}' does not exist", path.display());
            return;
        }

        if path.is_dir() {
            log::warn!("Drop ignored: '{}' is a directory", path.display());
            return;
        }

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase());

        let target_dir = match extension.as_deref() {
            Some("kt") => Self::script_drop_dir(&project_root),
            Some("eucp") | Some("eucs") => project_root.join("resources").join("scenes"),
            _ => project_root.join("resources"),
        };

        if let Err(err) = fs::create_dir_all(&target_dir) {
            log::warn!(
                "Drop failed: unable to create '{}' ({})",
                target_dir.display(),
                err
            );
            return;
        }

        let Some(file_name) = path.file_name() else {
            log::warn!("Drop ignored: invalid file name for '{}'", path.display());
            return;
        };

        let target_path = Self::unique_drop_target(&target_dir, file_name);
        if let Err(err) = fs::copy(path, &target_path) {
            log::warn!(
                "Drop failed: unable to copy '{}' to '{}' ({})",
                path.display(),
                target_path.display(),
                err
            );
        } else {
            log::info!("Dropped asset copied to '{}'", target_path.display());

            // Generate a .eucmeta sidecar for resource files (not scenes, not scripts).
            let is_resource = !matches!(
                extension.as_deref(),
                Some("kt") | Some("eucp") | Some("eucs")
            );
            if is_resource {
                if let Err(e) =
                    eucalyptus_core::metadata::generate_eucmeta(&target_path, &project_root)
                {
                    log::warn!(
                        "Failed to generate .eucmeta for '{}': {}",
                        target_path.display(),
                        e
                    );
                }
            }
        }
    }

    fn script_drop_dir(project_root: &Path) -> PathBuf {
        let src_root = project_root.join("src");
        let main_kotlin = src_root.join("main").join("kotlin");
        if main_kotlin.exists() {
            return main_kotlin;
        }

        if src_root.exists() {
            return main_kotlin;
        }

        main_kotlin
    }

    fn unique_drop_target(base_dir: &Path, file_name: &std::ffi::OsStr) -> PathBuf {
        let mut candidate = base_dir.join(file_name);
        if !candidate.exists() {
            return candidate;
        }

        let stem = Path::new(file_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
        let ext = Path::new(file_name).extension().and_then(|e| e.to_str());

        let mut index = 1;
        loop {
            let file_name = match ext {
                Some(ext) => format!("{}-{}.{}", stem, index, ext),
                None => format!("{}-{}", stem, index),
            };
            candidate = base_dir.join(file_name);
            if !candidate.exists() {
                return candidate;
            }
            index += 1;
        }
    }

    fn editor_specific_render(&mut self, graphics: &Arc<SharedGraphicsContext>, ui: &mut Ui) {
        self.size = graphics.viewport_texture.size;
        self.texture_id = Some(*graphics.texture_id.clone());
        self.window = Some(graphics.window.clone());

        self.show_ui(ui, graphics.clone());
        eucalyptus_core::logging::render(ui);
    }
}
