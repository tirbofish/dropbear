use std::collections::HashMap;
use egui::{CentralPanel, MenuBar, TopBottomPanel};
use eucalyptus_core::physics::collider::ColliderGroup;
use eucalyptus_core::physics::collider::ColliderShapeKey;
use eucalyptus_core::physics::collider::shader::ColliderInstanceRaw;
use glam::{DMat4, DQuat, DVec3};
use hecs::Entity;
use wgpu::Color;
use wgpu::util::DeviceExt;
use winit::event_loop::ActiveEventLoop;
use dropbear_engine::camera::Camera;
use dropbear_engine::buffer::ResizableBuffer;
use dropbear_engine::entity::{EntityTransform, MeshRenderer, Transform};
use dropbear_engine::graphics::{InstanceRaw, RenderContext};
use dropbear_engine::lighting::{Light, LightComponent};
use dropbear_engine::model::{DrawLight, DrawModel, ModelId, MODEL_CACHE};
use dropbear_engine::scene::{Scene, SceneCommand};
use eucalyptus_core::camera::CameraComponent;
use eucalyptus_core::command::CommandBufferPoller;
use eucalyptus_core::hierarchy::{EntityTransformExt, Parent};
use eucalyptus_core::states::{Label, PROJECT};
use eucalyptus_core::states::SCENES;
use eucalyptus_core::scene::loading::{IsSceneLoaded, SceneLoadResult, SCENE_LOADER};
use crate::editor::Editor;
use crate::runtime::{PlayMode, WindowMode};

impl Scene for PlayMode {
    fn load(&mut self, graphics: &mut RenderContext) {
        if self.current_scene.is_none() {
            let initial_scene = if let Some(s) = &self.initial_scene {
                s.clone()
            } else {
                let proj = PROJECT.read();
                proj.runtime_settings.initial_scene.clone().expect("No initial scene set in project settings")
            };

            log::debug!("Loading initial scene: {}", initial_scene);

            let first_time = IsSceneLoaded::new_first_time(initial_scene);

            self.request_async_scene_load(graphics, first_time);
        }
    }

    fn physics_update(&mut self, _dt: f32, _graphics: &mut RenderContext) {
        if self.scripts_ready {
            let _ = self.script_manager.physics_update_script(self.world.as_mut(), _dt);
        }
        
        let mut entity_label_map = HashMap::new();
        for (entity, label) in self.world.query::<&Label>().iter() {
            entity_label_map.insert(entity, label.clone());
        }
        
        self.physics_state.step(entity_label_map, &mut self.physics_pipeline, &(), &self.event_collector);

        if self.scripts_ready {
            if let (Some(ce_r), Some(cfe_r)) = (&self.collision_event_receiver, &self.collision_force_event_receiver) {
                // TODO: implement this
                if let Ok(_event) = ce_r.try_recv() {

                }

                if let Ok(_event) = cfe_r.try_recv() {
                }
            }
        }

        let mut sync_updates = Vec::new();

        for (entity, (label, _)) in self.world.query::<(&Label, &EntityTransform)>().iter() {
            if let Some(handle) = self.physics_state.bodies_entity_map.get(label) {
                if let Some(body) = self.physics_state.bodies.get(*handle) {
                    if !body.is_sleeping() {
                        let p = body.translation();
                        let r = body.rotation();

                        sync_updates.push((
                            entity,
                            DVec3::new(p.x as f64, p.y as f64, p.z as f64),
                            DQuat::from_xyzw(r.i as f64, r.j as f64, r.k as f64, r.w as f64)
                        ));
                    }
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

                    let offset = entity_transform.local_mut();
                    offset.position = DVec3::ZERO;
                    offset.rotation = DQuat::IDENTITY;
                } else {
                    let base = entity_transform.world_mut();
                    base.position = new_world_pos;
                    base.rotation = new_world_rot;

                    let offset = entity_transform.local_mut();
                    offset.position = DVec3::ZERO;
                    offset.rotation = DQuat::IDENTITY;
                }
            }
        }
    }

    fn update(&mut self, dt: f32, graphics: &mut RenderContext) {
        graphics.shared.future_queue.poll();
        self.poll(graphics);

        if let Some(ref progress) = self.scene_progress {
            if !progress.scene_handle_requested && self.world_receiver.is_none() && self.scene_loading_handle.is_none() {
                log::debug!("Starting async load for scene: {}", progress.requested_scene);
                let scene_to_load = IsSceneLoaded::new(progress.requested_scene.clone());
                self.request_async_scene_load(graphics, scene_to_load);
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
            if let Some(cam) = graphics.shared.future_queue.exchange_owned_as::<Entity>(&handle) {
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
                    self.switch_to(progress.clone(), graphics);
                }
            }
        }

        if self.scripts_ready {
            if let Err(e) = self.script_manager.update_script(self.world.as_mut(), dt) {
                panic!("Script update error: {}", e);
            }
        }

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

        TopBottomPanel::top("menu_bar").show(&graphics.shared.get_egui_context(), |ui| {
            MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Window", |ui| {
                    ui.menu_button("Window Mode", |ui| {
                        let is_windowed = matches!(self.display_settings.window_mode, WindowMode::Windowed);
                        if ui.selectable_label(is_windowed, "Windowed").clicked() {
                            self.display_settings.window_mode = WindowMode::Windowed;
                            ui.close();
                        }

                        let is_maximized = matches!(self.display_settings.window_mode, WindowMode::Maximized);
                        if ui.selectable_label(is_maximized, "Maximized").clicked() {
                            self.display_settings.window_mode = WindowMode::Maximized;
                            ui.close();
                        }

                        let is_fullscreen = matches!(self.display_settings.window_mode, WindowMode::Fullscreen);
                        if ui.selectable_label(is_fullscreen, "Fullscreen").clicked() {
                            self.display_settings.window_mode = WindowMode::Fullscreen;
                            ui.close();
                        }

                        let is_borderless = matches!(self.display_settings.window_mode, WindowMode::BorderlessFullscreen);
                        if ui.selectable_label(is_borderless, "Borderless Fullscreen").clicked() {
                            self.display_settings.window_mode = WindowMode::BorderlessFullscreen;
                            ui.close();
                        }
                    });

                    ui.separator();

                    ui.checkbox(&mut self.display_settings.maintain_aspect_ratio, "Maintain aspect ratio");
                    ui.checkbox(&mut self.display_settings.vsync, "VSync").clicked();
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.group(|ui| {
                        ui.add_enabled_ui(true, |ui| {
                            if ui.button("⏹").clicked() {
                                log::debug!("Menu button Stop button pressed");
                                self.scene_command = SceneCommand::CloseWindow(graphics.shared.window.id());
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

        CentralPanel::default().show(&graphics.shared.get_egui_context(), |ui| {
            if let Some(p) = &self.scene_progress {
                if !p.is_everything_loaded() && p.is_first_scene {
                    // todo: change from label to "splashscreen"
                    ui.centered_and_justified(|ui| {
                        ui.label("Loading scene...");
                    });
                    return;
                }
            }

            let texture_id = *graphics.shared.texture_id;

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
                    cam.update(graphics.shared.clone());

                    let (display_width, display_height) = if self.display_settings.maintain_aspect_ratio {
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

                    ui.allocate_exact_size(available_size, egui::Sense::hover());

                    ui.scope_builder(egui::UiBuilder::new().max_rect(image_rect), |ui| {
                        ui.add(egui::Image::new(egui::load::SizedTexture {
                            id: texture_id,
                            size: egui::vec2(display_width, display_height),
                        }));
                    });
                } else {
                    log::warn!("No such camera exists in the world");
                }
            } else {
                log::warn!("No active camera available");
            }
        });

        self.input_state.mouse_delta = None;
    }

    fn render(&mut self, graphics: &mut RenderContext) {
        let Some(active_camera) = self.active_camera else {
            return;
        };
        log_once::debug_once!("Active camera found: {:?}", active_camera);

        let q = if let Ok(mut query) = self.world.query_one::<&Camera>(active_camera) {
            query.get().cloned()
        } else {
            None
        };

        let Some(camera) = q else {
            return;
        };
        log_once::debug_once!("Camera ready");
        log_once::debug_once!("Camera currently being viewed: {}", camera.label);

        let Some(pipeline) = &self.render_pipeline else {
            log_once::warn_once!("Render pipeline not ready");
            return;
        };
        log_once::debug_once!("Pipeline ready");

        let clear_color = Color {
            r: 100.0 / 255.0,
            g: 149.0 / 255.0,
            b: 237.0 / 255.0,
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
            let mut query = self.world.query::<(&mut LightComponent, &dropbear_engine::entity::Transform, &mut Light)>();
            for (_, (light_component, transform, light)) in query.iter() {
                light.update(light_component, transform);
            }
        }

        self.light_manager.update(graphics.shared.clone(), &self.world);

        {
            let mut render_pass = graphics.clear_colour(clear_color);
            if let Some(light_pipeline) = &self.light_manager.pipeline {
                render_pass.set_pipeline(light_pipeline);
                for (light, component) in &lights {
                    render_pass.set_vertex_buffer(1, light.instance_buffer.buffer().slice(..));
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

        {
            let show_hitboxes = self
                .current_scene
                .as_ref()
                .and_then(|scene_name| {
                    let scenes = SCENES.read();
                    scenes
                        .iter()
                        .find(|scene| &scene.scene_name == scene_name)
                        .map(|scene| scene.settings.show_hitboxes)
                })
                .unwrap_or(false);

            if show_hitboxes {
                if let Some(collider_pipeline) = &self.collider_wireframe_pipeline {
                let mut render_pass = graphics.continue_pass();
                render_pass.set_pipeline(&collider_pipeline.pipeline);
                render_pass.set_bind_group(0, camera.bind_group(), &[]);

                    let mut instances_by_shape: HashMap<ColliderShapeKey, Vec<ColliderInstanceRaw>> =
                        HashMap::new();

                    let mut q = self.world.query::<(&EntityTransform, &ColliderGroup)>();
                    for (_entity, (entity_transform, group)) in q.iter() {
                        for collider in &group.colliders {
                            let world_tf = entity_transform.sync();

                            let entity_matrix = DMat4::from_rotation_translation(
                                world_tf.rotation,
                                world_tf.position,
                            )
                            .as_mat4();

                            let offset_transform = Transform::new()
                                .with_offset(collider.translation, collider.rotation);
                            let offset_matrix = offset_transform.matrix().as_mat4();

                            let final_matrix = entity_matrix * offset_matrix;

                            let color = [0.0, 1.0, 0.0, 1.0];
                            let instance = ColliderInstanceRaw::from_matrix(final_matrix, color);

                            let key = ColliderShapeKey::from(&collider.shape);
                            instances_by_shape.entry(key).or_default().push(instance);

                            self.collider_wireframe_geometry_cache.entry(key).or_insert_with(|| {
                                Editor::create_wireframe_geometry(
                                    graphics.shared.clone(),
                                    &collider.shape,
                                )
                            });
                        }
                    }

                    if !instances_by_shape.is_empty() {
                        let total_instances: usize =
                            instances_by_shape.values().map(|v| v.len()).sum();
                        let mut all_instances = Vec::with_capacity(total_instances);
                        let mut draws: Vec<(ColliderShapeKey, usize, usize)> = Vec::new();

                        for (key, instances) in instances_by_shape {
                            let start = all_instances.len();
                            all_instances.extend_from_slice(&instances);
                            let count = instances.len();
                            draws.push((key, start, count));
                        }

                        let instance_buffer = self.collider_instance_buffer.get_or_insert_with(|| {
                            ResizableBuffer::new(
                                &graphics.shared.device,
                                all_instances.len().max(10),
                                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                                "Collider Instance Buffer",
                            )
                        });
                        instance_buffer.write(
                            &graphics.shared.device,
                            &graphics.shared.queue,
                            &all_instances,
                        );

                        for (key, start, count) in draws {
                            let Some(geometry) = self.collider_wireframe_geometry_cache.get(&key) else {
                                continue;
                            };

                            let start_bytes =
                                (start * std::mem::size_of::<ColliderInstanceRaw>()) as wgpu::BufferAddress;
                            let end_bytes =
                                ((start + count) * std::mem::size_of::<ColliderInstanceRaw>()) as wgpu::BufferAddress;

                            render_pass.set_vertex_buffer(
                                1,
                                instance_buffer.buffer().slice(start_bytes..end_bytes),
                            );
                            render_pass.set_vertex_buffer(
                                0,
                                geometry.vertex_buffer.slice(..),
                            );
                            render_pass.set_index_buffer(
                                geometry.index_buffer.slice(..),
                                wgpu::IndexFormat::Uint16,
                            );
                            render_pass.draw_indexed(
                                0..geometry.index_count,
                                0,
                                0..count as u32,
                            );
                        }
                    }
                }
            }
        }
    }

    fn exit(&mut self, _event_loop: &ActiveEventLoop) {}

    fn run_command(&mut self) -> SceneCommand {
        std::mem::replace(&mut self.scene_command, SceneCommand::None)
    }
}

