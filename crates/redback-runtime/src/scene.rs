use std::sync::Arc;

use crate::PlayMode;
use dropbear_engine::camera::Camera;
use dropbear_engine::entity::EntityTransform;
use dropbear_engine::graphics::CommandEncoder;
use dropbear_engine::graphics::SharedGraphicsContext;
use dropbear_engine::scene::{Scene, SceneCommand};
use eucalyptus_core::billboard::BillboardComponent;
use eucalyptus_core::command::CommandBufferPoller;
use eucalyptus_core::egui::CentralPanel;
use eucalyptus_core::entity_status::EntityStatus;
use eucalyptus_core::hierarchy::{EntityTransformExt, Parent};
use eucalyptus_core::physics::kcc::KCC;
use eucalyptus_core::rapier3d::geometry::SharedShape;
use eucalyptus_core::rapier3d::prelude::QueryFilter;
use eucalyptus_core::rendering::RendererCommon;
use eucalyptus_core::scene::loading::{IsSceneLoaded, SCENE_LOADER, SceneLoadResult};
use eucalyptus_core::states::SCENES;
use eucalyptus_core::states::{Label, PROJECT};
use eucalyptus_core::ui::HUDComponent;
use glam::{DVec3, Mat4, Quat, Vec2};
use hecs::Entity;
use kino_ui::WidgetTree;
use kino_ui::rendering::KinoRenderTargetId;
use std::collections::HashMap;
use egui::Ui;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;

impl Scene for PlayMode {
    fn load(&mut self, graphics: Arc<SharedGraphicsContext>, _ui: &mut Ui,) {
        if self.current_scene.is_none() {
            let initial_scene = if let Some(s) = &self.initial_scene {
                s.clone()
            } else {
                let proj = PROJECT.read();
                proj.runtime_settings
                    .initial_scene
                    .clone()
                    .expect("No initial scene set in project settings")
            };

            log::debug!("Loading initial scene: {}", initial_scene);

            let first_time = IsSceneLoaded::new_first_time(initial_scene);

            self.request_async_scene_load(graphics, first_time);
        }
    }

    fn physics_update(&mut self, dt: f32, _graphics: Arc<SharedGraphicsContext>, _ui: &mut Ui,) {
        if self.scripts_ready {
            let _ = self
                .script_manager
                .physics_update_script(self.world.as_mut(), dt as f64);
        }

        for kcc in self.world.query::<&mut KCC>().iter() {
            kcc.collisions.clear();
        }

        for (e, l, _) in self.world.query::<(Entity, &Label, &KCC)>().iter() {
            log_once::debug_once!(
                "This entity [{:?}, label = {}] has the KCC (KinematicCharacterController) component attached",
                e,
                l
            );
        }

        if !self.physics_state.collision_events_to_deal_with.is_empty() {
            let entities_with_collisions: Vec<Entity> = self
                .physics_state
                .collision_events_to_deal_with
                .keys()
                .copied()
                .collect();

            for entity in entities_with_collisions {
                let Some(collisions) = self
                    .physics_state
                    .collision_events_to_deal_with
                    .remove(&entity)
                else {
                    continue;
                };
                if collisions.is_empty() {
                    continue;
                }

                if let Ok(mut kcc) = self.world.get::<&mut KCC>(entity) {
                    kcc.collisions = collisions.clone();
                }

                let (label, kcc_controller) =
                    match self.world.query_one::<(&Label, &KCC)>(entity).get() {
                        Ok(v) => (v.0.clone(), v.1.clone()),
                        Err(e) => {
                            log_once::warn_once!("Unable to query {:?}: {}", entity, e);
                            continue;
                        }
                    };

                let Some(rigid_body_handle) =
                    self.physics_state.bodies_entity_map.get(&label).copied()
                else {
                    continue;
                };

                let Some((_, collider_handle)) = self
                    .physics_state
                    .colliders_entity_map
                    .get(&label)
                    .and_then(|handles| handles.first())
                    .copied()
                else {
                    continue;
                };

                let (character_shape, character_mass): (SharedShape, f32) = {
                    let Some(collider) = self.physics_state.colliders.get(collider_handle) else {
                        continue;
                    };

                    (collider.shared_shape().clone(), collider.mass())
                };

                let character_mass = if character_mass > 0.0 {
                    character_mass
                } else {
                    1.0
                };

                let filter = QueryFilter::default()
                    .exclude_rigid_body(rigid_body_handle)
                    .exclude_sensors();
                let dispatcher = self.physics_state.narrow_phase.query_dispatcher();

                let broad_phase = &mut self.physics_state.broad_phase;
                let bodies = &mut self.physics_state.bodies;
                let colliders = &mut self.physics_state.colliders;

                let mut query_pipeline_mut =
                    broad_phase.as_query_pipeline_mut(dispatcher, bodies, colliders, filter);

                kcc_controller
                    .controller
                    .solve_character_collision_impulses(
                        dt,
                        &mut query_pipeline_mut,
                        character_shape.as_ref(),
                        character_mass,
                        &collisions,
                    );
            }
        }

        let mut entity_label_map = HashMap::new();
        {
            let all: Vec<(Entity, Label)> = self
                .world
                .query::<(Entity, &Label)>()
                .iter()
                .map(|(e, l)| (e, l.clone()))
                .collect();
            for (entity, label) in all {
                // disabled physics cannot partake
                let disabled = self
                    .world
                    .get::<&EntityStatus>(entity)
                    .map(|s| s.disabled)
                    .unwrap_or(false);
                if !disabled {
                    entity_label_map.insert(entity, label);
                }
            }
        }

        self.physics_state.step(
            entity_label_map,
            &mut self.physics_pipeline,
            &(),
            &self.event_collector,
        );

        if self.scripts_ready {
            if let (Some(ce_r), Some(cfe_r)) = (
                &self.collision_event_receiver,
                &self.collision_force_event_receiver,
            ) {
                // both are not crucial, so no need to panic
                while let Ok(event) = ce_r.try_recv() {
                    log_once::debug_once!("Collision event received");
                    if let Some(evt) = eucalyptus_core::types::CollisionEvent::from_rapier3d(
                        &self.physics_state,
                        event,
                    ) {
                        if let Err(err) = self
                            .script_manager
                            .collision_event_script(self.world.as_mut(), &evt)
                        {
                            log::error!("Script collision event error: {}", err);
                        }
                    }
                }

                while let Ok(event) = cfe_r.try_recv() {
                    log_once::debug_once!("Contact force event received");
                    if let Some(evt) = eucalyptus_core::types::ContactForceEvent::from_rapier3d(
                        &self.physics_state,
                        event,
                    ) {
                        if let Err(err) = self
                            .script_manager
                            .contact_force_event_script(self.world.as_mut(), &evt)
                        {
                            log::error!("Script contact force event error: {}", err);
                        }
                    }
                }
            }
        }

        let mut sync_updates = Vec::new();

        for (entity, label, _) in self
            .world
            .query::<(Entity, &Label, &EntityTransform)>()
            .iter()
        {
            if let Some(handle) = self.physics_state.bodies_entity_map.get(label) {
                if let Some(body) = self.physics_state.bodies.get(*handle) {
                    let p = body.translation();
                    let r = body.rotation();

                    sync_updates.push((
                        entity,
                        DVec3::new(p.x as f64, p.y as f64, p.z as f64),
                        Quat::from(r.clone()).as_dquat(),
                    ));
                }
            }
        }

        for (entity, new_world_pos, new_world_rot) in sync_updates {
            let parent_world = if let Ok(parent_comp) = self.world.get::<&Parent>(entity) {
                let parent_entity = parent_comp.parent();
                if let Ok(p_transform) = self.world.get::<&EntityTransform>(parent_entity) {
                    Some(p_transform.propagate(&self.world, parent_entity))
                } else {
                    None
                }
            } else {
                None
            };

            if let Ok(mut entity_transform) = self.world.get::<&mut EntityTransform>(entity) {
                if let Some(p_world) = parent_world {
                    let inv_p_rot = p_world.rotation.inverse();

                    let relative_pos = new_world_pos - p_world.position;
                    let new_local_pos = (inv_p_rot * relative_pos) / p_world.scale;
                    let new_local_rot = inv_p_rot * new_world_rot;

                    let base = entity_transform.world_mut();
                    base.position = new_local_pos;
                    base.rotation = new_local_rot;
                } else {
                    let base = entity_transform.world_mut();
                    base.position = new_world_pos;
                    base.rotation = new_world_rot;
                }
            }
        }
    }

    fn update(&mut self, dt: f32, graphics: Arc<SharedGraphicsContext>, ui: &mut Ui,) {
        graphics.future_queue.poll();
        self.poll(graphics.clone());

        self.display_settings.update(graphics.clone());

        if matches!(self.scene_command, SceneCommand::None) {
            let window_size = graphics.window.inner_size();
            if window_size.width > 0 && window_size.height > 0 {
                let current = graphics.viewport_texture.size;
                if current.width != window_size.width || current.height != window_size.height {
                    self.scene_command =
                        SceneCommand::ResizeViewport((window_size.width, window_size.height));
                }
            }
        }

        {
            let window_size = graphics.window.inner_size();
            let size_changed = window_size.width != self.display_settings.last_size.0
                || window_size.height != self.display_settings.last_size.1;
            if size_changed && window_size.height > 0 {
                if let Some(active_camera) = self.active_camera {
                    if let Ok(cam) = self.world.query_one_mut::<&mut Camera>(active_camera) {
                        cam.aspect = window_size.width as f64 / window_size.height as f64;
                    }
                }
            }
        }

        {
            if let Some(fps) = PROJECT.read().runtime_settings.target_fps.get() {
                log_once::debug_once!("setting new fps for play mode session: {}", fps);
                if matches!(self.scene_command, SceneCommand::None) {
                    self.scene_command = SceneCommand::SetFPS(*fps);
                }
            }
        }

        if let Some(ref progress) = self.scene_progress {
            if !progress.scene_handle_requested
                && self.world_receiver.is_none()
                && self.scene_loading_handle.is_none()
            {
                log::debug!(
                    "Starting async load for scene: {}",
                    progress.requested_scene
                );
                let scene_to_load = IsSceneLoaded::new(progress.requested_scene.clone());
                self.request_async_scene_load(graphics.clone(), scene_to_load);
            }
        }

        if let Some(mut receiver) = self.world_receiver.take() {
            if let Ok(loaded_world) = receiver.try_recv() {
                self.pending_world = Some(Box::new(loaded_world));
                log::debug!("World received");
                if let Some(ref mut progress) = self.scene_progress {
                    progress.world_loaded = true;

                    if progress.camera_received {
                        if let Some(id) = progress.id {
                            let mut loader = SCENE_LOADER.lock();
                            if let Some(entry) = loader.get_entry_mut(id) {
                                entry.result = SceneLoadResult::Success;
                            }
                        }
                    }
                }
            } else {
                self.world_receiver = Some(receiver);
            }
        }

        if let Some(mut receiver) = self.physics_receiver.take() {
            if let Ok(loaded_physics) = receiver.try_recv() {
                self.pending_physics_state = Some(Box::new(loaded_physics));
                log::debug!("PhysicsState received");
            } else {
                self.physics_receiver = Some(receiver);
            }
        }

        if let Some(handle) = self.scene_loading_handle.take() {
            if let Some(cam) = graphics.future_queue.exchange_owned_as::<Entity>(&handle) {
                self.pending_camera = Some(cam);
                log::debug!("Camera entity received: {:?}", cam);
                if let Some(ref mut progress) = self.scene_progress {
                    progress.camera_received = true;

                    if progress.world_loaded {
                        if let Some(id) = progress.id {
                            let mut loader = SCENE_LOADER.lock();
                            if let Some(entry) = loader.get_entry_mut(id) {
                                entry.result = SceneLoadResult::Success;
                            }
                        }
                    }
                }
            } else {
                self.scene_loading_handle = Some(handle)
            }
        }

        if let Some(ref progress) = self.scene_progress {
            if progress.is_everything_loaded() {
                if self.current_scene.as_ref() != Some(&progress.requested_scene) {
                    self.switch_to(progress.clone(), graphics.clone());
                }
            }
        }

        if self.scripts_ready {
            if let Err(e) = self
                .script_manager
                .update_script(self.world.as_mut(), dt as f64)
            {
                panic!("Script update error: {}", e);
            }
        }

        self.component_registry.update_components(
            self.world.as_mut(),
            &mut self.physics_state,
            dt,
            graphics.clone(),
        );

        #[cfg(feature = "debug")]
        egui::Panel::top("menu_bar").show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                use crate::WindowMode;
                ui.menu_button("Window", |ui| {
                    ui.menu_button("Window Mode", |ui| {
                        let is_windowed =
                            matches!(self.display_settings.window_mode, WindowMode::Windowed);
                        if ui.selectable_label(is_windowed, "Windowed").clicked() {
                            self.display_settings.window_mode = WindowMode::Windowed;
                            ui.close();
                        }

                        let is_maximized =
                            matches!(self.display_settings.window_mode, WindowMode::Maximized);
                        if ui.selectable_label(is_maximized, "Maximized").clicked() {
                            self.display_settings.window_mode = WindowMode::Maximized;
                            ui.close();
                        }

                        let is_fullscreen =
                            matches!(self.display_settings.window_mode, WindowMode::Fullscreen);
                        if ui.selectable_label(is_fullscreen, "Fullscreen").clicked() {
                            self.display_settings.window_mode = WindowMode::Fullscreen;
                            ui.close();
                        }

                        let is_borderless = matches!(
                            self.display_settings.window_mode,
                            WindowMode::BorderlessFullscreen
                        );
                        if ui
                            .selectable_label(is_borderless, "Borderless Fullscreen")
                            .clicked()
                        {
                            self.display_settings.window_mode = WindowMode::BorderlessFullscreen;
                            ui.close();
                        }
                    });

                    ui.separator();

                    ui.checkbox(
                        &mut self.display_settings.maintain_aspect_ratio,
                        "Maintain aspect ratio",
                    );
                    ui.checkbox(&mut self.display_settings.vsync, "VSync")
                        .clicked();
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.group(|ui| {
                        ui.add_enabled_ui(true, |ui| {
                            if ui.button("⏹").clicked() {
                                log::debug!("Menu button Stop button pressed");
                                self.scene_command =
                                    SceneCommand::CloseWindow(graphics.window.id());
                            }
                        });

                        ui.add_enabled_ui(false, |ui| {
                            if ui.button("▶").clicked() {
                                log::debug!("how tf can you press this???");
                            }
                        });
                    });
                });
            });
        });

        CentralPanel::default().show_inside(ui, |ui| {
            if let Some(p) = &self.scene_progress {
                if !p.is_everything_loaded() && p.is_first_scene {
                    ui.centered_and_justified(|ui| {
                        ui.add(
                            egui::Image::new(egui::include_image!(
                                "../../../resources/eucalyptus-editor.png"
                            ))
                            .max_width(128.0),
                        )
                    });
                    return;
                }
            }

            let texture_id = *graphics.texture_id;

            let available_size = ui.available_size();
            let available_rect = ui.available_rect_before_wrap();

            if let Some(active_camera) = self.active_camera {
                if let Ok(cam) = self.world.query_one_mut::<&mut Camera>(active_camera) {
                    if !self.has_initial_resize_done {
                        cam.aspect = (available_size.x / available_size.y) as f64;

                        self.has_initial_resize_done = true;
                    }

                    if !self.display_settings.maintain_aspect_ratio {
                        cam.aspect = (available_size.x / available_size.y) as f64;
                    }
                    cam.update_view_proj();
                    cam.update(graphics.clone());

                    let (display_width, display_height) =
                        if self.display_settings.maintain_aspect_ratio {
                            let width = available_size.x;
                            let height = width / cam.aspect as f32;
                            (width, height)
                        } else {
                            (available_size.x, available_size.y)
                        };

                    let center_x = available_rect.center().x;
                    let center_y = available_rect.center().y;

                    let image_rect = egui::Rect::from_center_size(
                        egui::pos2(center_x, center_y),
                        egui::vec2(display_width, display_height),
                    );

                    self.viewport_offset = (image_rect.min.x, image_rect.min.y);
                    if let Some(kino) = &mut self.kino {
                        let scale_x = if display_width > 0.0 {
                            graphics.viewport_texture.size.width as f32 / display_width
                        } else {
                            1.0
                        };
                        let scale_y = if display_height > 0.0 {
                            graphics.viewport_texture.size.height as f32 / display_height
                        } else {
                            1.0
                        };
                        kino.set_viewport_transform(
                            Vec2::new(image_rect.min.x, image_rect.min.y),
                            Vec2::new(scale_x, scale_y),
                        );
                    }

                    ui.allocate_exact_size(available_size, egui::Sense::hover());

                    ui.scope_builder(egui::UiBuilder::new().max_rect(image_rect), |ui| {
                        ui.add(egui::Image::new(egui::load::SizedTexture {
                            id: texture_id,
                            size: egui::vec2(display_width, display_height),
                        }));
                    });

                    let billboard_trees: Vec<(u64, WidgetTree)> = self
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

                    let hud_trees: Vec<WidgetTree> = self
                        .world
                        .query::<&HUDComponent>()
                        .iter()
                        .map(|hud| hud.tree().clone())
                        .collect();

                    if let Some(kino) = &mut self.kino {
                        for (entity_id, tree) in billboard_trees {
                            kino.begin(KinoRenderTargetId::Billboard(entity_id));
                            tree.submit(kino);
                            kino.flush();
                        }

                        if !hud_trees.is_empty() {
                            kino.begin(KinoRenderTargetId::HUD);
                            for tree in hud_trees {
                                // there can only be one.
                                tree.submit(kino);
                            }
                            kino.flush();
                        }
                    }
                } else {
                    log::warn!("No such camera exists in the world");
                }
            } else {
                log::warn!("No active camera available");
            }
        });

        self.input_state.mouse_delta = None;
    }

    fn render<'a>(&mut self, graphics: Arc<SharedGraphicsContext>, _ui: &mut Ui,) {
        let hdr = graphics.hdr.read();
        let mut encoder = CommandEncoder::new(graphics.clone(), Some("runtime viewport encoder"));

        let Some(active_camera) = self.active_camera.as_ref().cloned() else { return };
        log_once::debug_once!("Active camera found: {:?}", active_camera);
        let Some(camera) = self.world.query_one::<&Camera>(active_camera).get().ok().cloned() else { return };
        log_once::debug_once!("Camera ready: {}", camera.label);

        RendererCommon::clear_viewport(&graphics, &mut encoder, &hdr);

        if let Some(light_pipeline) = &mut self.light_cube_pipeline {
            light_pipeline.update(graphics.clone(), &self.world);
        }
        let (lights, enabled_light_count) = RendererCommon::collect_lights(&self.world);

        if let Some(globals) = &mut self.shader_globals {
            globals.set_num_lights(enabled_light_count);
            if let Some(scene_name) = &self.current_scene {
                let scenes = SCENES.read();
                if let Some(scene) = scenes.iter().find(|s| &s.scene_name == scene_name) {
                    globals.set_ambient_strength(scene.settings.ambient_strength);
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
                self.main_pipeline.as_mut(),
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

        let sky = self.sky_pipeline.as_ref().expect("Sky pipeline must be initialised before rendering models");
        let environment_bind_group = &sky.environment_bind_group;

        let Some(pipeline) = self.main_pipeline.as_ref() else {
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
            pipeline, animation_defaults,
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
            self.current_scene.as_deref(),
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

        if let Some(kino) = &mut self.kino {
            let mut encoder = CommandEncoder::new(graphics.clone(), Some("kino encoder"));
            kino.render(&graphics.device, &graphics.queue, &mut encoder, hdr.view());
            if let Err(e) = encoder.submit() {
                log_once::error_once!("Unable to submit kino: {}", e);
            }
        }
    }

    fn exit(&mut self, _event_loop: &ActiveEventLoop) {}

    fn handle_event(&mut self, event: &WindowEvent) {
        if let Some(kino) = &mut self.kino {
            kino.handle_event(event);
        }
    }

    fn run_command(&mut self) -> SceneCommand {
        std::mem::replace(&mut self.scene_command, SceneCommand::None)
    }
}
