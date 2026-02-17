use crossbeam_channel::unbounded;
use dropbear_engine::animation::AnimationComponent;
use dropbear_engine::buffer::ResizableBuffer;
use glam::{DMat4, Mat4};
use wgpu::util::DeviceExt;
use std::collections::HashMap;
use std::sync::Arc;
use super::*;
use crate::signal::SignalController;
use crate::spawn::PendingSpawnController;
use dropbear_engine::asset::{ASSET_REGISTRY, Handle};
use dropbear_engine::graphics::{CommandEncoder, InstanceRaw};
use dropbear_engine::{
    entity::{EntityTransform, MeshRenderer, Transform},
    lighting::{Light, LightComponent, MAX_LIGHTS},
    model::{DrawLight, DrawModel},
    scene::{Scene, SceneCommand},
};
use eucalyptus_core::states::{Label, WorldLoadingStatus};
use log;
use parking_lot::Mutex;
use winit::{event_loop::ActiveEventLoop, keyboard::KeyCode};
use eucalyptus_core::physics::collider::ColliderGroup;
use eucalyptus_core::physics::collider::ColliderShapeKey;
use eucalyptus_core::physics::collider::shader::{ColliderInstanceRaw, create_wireframe_geometry};
use eucalyptus_core::properties::CustomProperties;

impl Scene for Editor {
    fn load(&mut self, graphics: std::sync::Arc<dropbear_engine::graphics::SharedGraphicsContext>) {
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

        let dock_state_shared = Arc::new(Mutex::new(self.dock_state.clone()));
        let dock_state_for_loading = dock_state_shared.clone();

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

        self.window = Some(graphics.window.clone());
        self.is_world_loaded.mark_scene_loaded();
    }

    fn physics_update(&mut self, _dt: f32, _graphics: std::sync::Arc<dropbear_engine::graphics::SharedGraphicsContext>) {}

    fn update(&mut self, dt: f32, graphics: std::sync::Arc<dropbear_engine::graphics::SharedGraphicsContext>) {
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
            self.show_project_loading_window(&graphics.get_egui_context());
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

        self.component_registry
            .update_components(self.world.as_mut(), dt, graphics.clone());

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
            graphics.clone(),
            graphics.future_queue.clone(),
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
                if let Some(l) = graphics
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
                && let Ok((camera, _)) = self
                    .world
                    .query_one::<(&mut Camera, &CameraComponent)>(active_camera).get()
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

        let _ = self.run_signal(graphics.clone());
        

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
            self.nerd_stats.write().record_stats(dt, self.world.len() as u32);
        }

        self.input_state.window = self.window.clone();
        self.previously_selected_entity = self.selected_entity;
        self.input_state.mouse_delta = None;
    }

    fn render<'a>(&mut self, graphics: Arc<SharedGraphicsContext>) {
        self.editor_specific_render(&graphics);

        let hdr = graphics.hdr.read();

        let mut encoder = CommandEncoder::new(graphics.clone(), Some("runtime viewport encoder"));

        let active_camera = {self.active_camera.lock().as_ref().cloned()};
        let Some(active_camera) = active_camera else {
            return;
        };
        log_once::debug_once!("Active camera found: {:?}", active_camera);

        let q = self.world.query_one::<&Camera>(active_camera).get().ok().cloned();

        let Some(camera) = q else {
            return;
        };
        log_once::debug_once!("Camera ready");
        log_once::debug_once!("Camera currently being viewed: {}", camera.label);

        let Some(pipeline) = &self.main_render_pipeline else {
            log_once::warn_once!("Render pipeline not ready");
            return;
        };
        log_once::debug_once!("Pipeline ready");

        {
            let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("viewport clear pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: hdr.view(),
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 100.0 / 255.0,
                            g: 149.0 / 255.0,
                            b: 237.0 / 255.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &graphics.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }

        let lights = {
            let mut lights = Vec::new();
            let mut query = self.world.query::<(&Light, &LightComponent)>();
            for (light, comp) in query.iter() {
                lights.push((light.clone(), comp.clone()));
            }
            lights
        };

            if let Some(globals) = &mut self.shader_globals {
                let enabled_count = lights
                    .iter()
                    .filter(|(_, comp)| comp.enabled)
                    .take(MAX_LIGHTS)
                    .count() as u32;
                globals.set_num_lights(enabled_count);
                globals.write(&graphics.queue);
            }

        let mut static_batches: HashMap<u64, Vec<InstanceRaw>> = HashMap::new();
        let mut animated_instances: Vec<(u64, InstanceRaw, wgpu::BindGroup)> = Vec::new();

        {
            let mut query = self
                .world
                .query::<(&MeshRenderer, Option<&AnimationComponent>)>();

            for (renderer, animation) in query.iter() {
                let handle = renderer.model();
                if handle.is_null() {
                    continue;
                }

                let instance = renderer.instance.to_raw();
                if let Some(bind_group) = animation.and_then(|anim| anim.bind_group.clone()) {
                    animated_instances.push((handle.id, instance, bind_group));
                } else {
                    static_batches.entry(handle.id).or_default().push(instance);
                }
            }
        }

        let registry = ASSET_REGISTRY.read();
        let mut prepared_models = Vec::new();
        for (handle, instances) in static_batches {
            let Some(model) = registry.get_model(Handle::new(handle)).cloned() else {
                log_once::error_once!("Missing model handle {} in registry", handle);
                continue;
            };

            let instance_buffer = graphics.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Runtime Instance Buffer"),
                    contents: bytemuck::cast_slice(&instances),
                    usage: wgpu::BufferUsages::VERTEX,
                },
            );

            prepared_models.push((model, instance_buffer, instances.len() as u32));
        }

        let registry = ASSET_REGISTRY.read();
        {
            let mut render_pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("light cube render pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: hdr.view(),
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &graphics.depth_texture.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });
            if let Some(light_pipeline) = &self.light_cube_pipeline {
                render_pass.set_pipeline(light_pipeline.pipeline());
                for (light, component) in &lights {
                    render_pass.set_vertex_buffer(1, light.instance_buffer.buffer().slice(..));
                    if !component.visible {
                        continue;
                    }

                    let Some(model) = registry.get_model(light.cube_model) else {
                        log_once::error_once!(
                            "Missing light cube model handle {} in registry",
                            light.cube_model.id
                        );
                        continue;
                    };

                    render_pass.draw_light_model(
                        model,
                        &camera.bind_group,
                        &light.bind_group,
                    );
                }
            }
        }

        if self.default_skinning_bind_group.is_none() {
            let identity = [Mat4::IDENTITY.to_cols_array_2d()];
            let buffer = graphics.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("default skinning buffer"),
                contents: bytemuck::cast_slice(&identity),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

            let bind_group = graphics.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("default skinning bind group"),
                layout: &graphics.layouts.skinning_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });

            self.default_skinning_buffer = Some(buffer);
            self.default_skinning_bind_group = Some(bind_group);
        }

        let default_skinning_bind_group = self
            .default_skinning_bind_group
            .as_ref()
            .expect("Default skinning bind group not initialized");

        // model rendering
        if let Some(lcp) = &self.light_cube_pipeline {
            for (model, instance_buffer, instance_count) in prepared_models {
                let globals_bind_group = &self
                    .shader_globals
                    .as_ref()
                    .expect("Shader globals not initialised")
                    .bind_group;

                let mut render_pass = encoder
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("model render pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: hdr.view(),
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &graphics.depth_texture.view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });
                render_pass.set_pipeline(pipeline.pipeline());
                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                render_pass.set_bind_group(3, globals_bind_group, &[]);

                for mesh in &model.meshes {
                    let material = &model.materials[mesh.material];
                    render_pass.draw_mesh_instanced(
                        mesh,
                        material,
                        0..instance_count,
                        &camera.bind_group,
                        lcp.bind_group(),
                        Some(default_skinning_bind_group),
                    );
                }
            }
        }

        if let Some(lcp) = &self.light_cube_pipeline {
            let globals_bind_group = &self
                .shader_globals
                .as_ref()
                .expect("Shader globals not initialised")
                .bind_group;

            for (handle, instance, skin_bind_group) in animated_instances {
                let Some(model) = registry.get_model(Handle::new(handle)).cloned() else {
                    log_once::error_once!("Missing model handle {} in registry", handle);
                    continue;
                };

                let instance_buffer = graphics.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Runtime Animated Instance Buffer"),
                        contents: bytemuck::cast_slice(&[instance]),
                        usage: wgpu::BufferUsages::VERTEX,
                    },
                );

                let mut render_pass = encoder
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("model render pass (animated)"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: hdr.view(),
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &graphics.depth_texture.view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });

                render_pass.set_pipeline(pipeline.pipeline());
                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                render_pass.set_bind_group(3, globals_bind_group, &[]);

                for mesh in &model.meshes {
                    let material = &model.materials[mesh.material];
                    render_pass.draw_mesh_instanced(
                        mesh,
                        material,
                        0..1,
                        &camera.bind_group,
                        lcp.bind_group(),
                        Some(&skin_bind_group),
                    );
                }
            }
        }

        // collider pipeline
        {
            let show_hitboxes = self
                .current_scene_name
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
                    let mut render_pass = encoder
                        .begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("model render pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: hdr.view(),
                                depth_slice: None,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                                view: &graphics.depth_texture.view,
                                depth_ops: Some(wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: wgpu::StoreOp::Store,
                                }),
                                stencil_ops: None,
                            }),
                            occlusion_query_set: None,
                            timestamp_writes: None,
                        });
                    
                    render_pass.set_pipeline(&collider_pipeline.pipeline);
                    render_pass.set_bind_group(0, &camera.bind_group, &[]);

                    let mut instances_by_shape: HashMap<ColliderShapeKey, Vec<ColliderInstanceRaw>> =
                        HashMap::new();

                    let mut q = self.world.query::<(&EntityTransform, &ColliderGroup)>();
                    for (entity_transform, group) in q.iter() {
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
                                create_wireframe_geometry(
                                    graphics.clone(),
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
                                &graphics.device,
                                all_instances.len().max(10),
                                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                                "Collider Instance Buffer",
                            )
                        });
                        instance_buffer.write(
                            &graphics.device,
                            &graphics.queue,
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

        if let Some(sky) = &self.sky_pipeline {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("sky render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: hdr.view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &graphics.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&sky.pipeline);
            render_pass.set_bind_group(0, &camera.bind_group, &[]);
            render_pass.set_bind_group(1, &sky.bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        hdr.process(&mut encoder, &graphics.viewport_texture.view);

        if let Err(e) = encoder.submit() {
            log_once::error_once!("{}", e);
        }
    }

    fn exit(&mut self, _event_loop: &ActiveEventLoop) {}

    fn run_command(&mut self) -> SceneCommand {
        std::mem::replace(&mut self.scene_command, SceneCommand::None)
    }
}

impl Editor {
    fn editor_specific_render(&mut self, graphics: &Arc<SharedGraphicsContext>) {
        self.size = graphics.viewport_texture.size;
        self.texture_id = Some(*graphics.texture_id.clone());
        self.window = Some(graphics.window.clone());
        
        self.show_ui(&graphics.get_egui_context());
        eucalyptus_core::logging::render(&graphics.get_egui_context());
    }
}