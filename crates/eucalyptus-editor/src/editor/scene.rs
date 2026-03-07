use super::*;
use crate::signal::SignalController;
use crate::spawn::PendingSpawnController;
use crossbeam_channel::unbounded;
use dropbear_engine::animation::{AnimationComponent, MorphTargetInfo};
use dropbear_engine::asset::{ASSET_REGISTRY, Handle};
use dropbear_engine::buffer::ResizableBuffer;
use dropbear_engine::graphics::{CommandEncoder, InstanceRaw};
use dropbear_engine::{
    entity::{EntityTransform, MeshRenderer, Transform},
    lighting::Light,
    model::{DrawLight, DrawModel},
    scene::{Scene, SceneCommand},
};
use eucalyptus_core::billboard::BillboardComponent;
use eucalyptus_core::physics::collider::ColliderGroup;
use eucalyptus_core::physics::collider::ColliderShapeKey;
use eucalyptus_core::physics::collider::shader::{ColliderInstanceRaw, create_wireframe_geometry};
use eucalyptus_core::properties::CustomProperties;
use eucalyptus_core::states::{Label, WorldLoadingStatus, SCENES};
use eucalyptus_core::ui::HUDComponent;
use hecs::Entity;
use log;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::{
    fs,
    path::{Path, PathBuf},
};
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, keyboard::KeyCode};
use winit::event::{MouseScrollDelta, TouchPhase};
use kino_ui::rendering::KinoRenderTargetId;

impl Scene for Editor {
    fn load(&mut self, graphics: Arc<SharedGraphicsContext>) {
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
    ) {
    }

    fn update(
        &mut self,
        dt: f32,
        graphics: std::sync::Arc<dropbear_engine::graphics::SharedGraphicsContext>,
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
            self.show_project_loading_window(&graphics.get_egui_context());
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
            } else if self.pending_aa_reload.is_some() && matches!(self.signal, Signal::None) {
                log::debug!("Anti aliasing mode applied, reloading WGPU data");
                self.signal = Signal::ReloadWGPUData {
                    skybox_texture: None,
                };
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
            self.nerd_stats
                .write()
                .record_stats(dt, self.world.len() as u32);
        }

        let open_ui_editor = graphics.get_egui_context().data_mut(|d| {
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

    fn render<'a>(&mut self, graphics: Arc<SharedGraphicsContext>) {
        self.editor_specific_render(&graphics);

        let hdr = graphics.hdr.read();

        let mut encoder = CommandEncoder::new(graphics.clone(), Some("runtime viewport encoder"));

        let active_camera = { self.active_camera.lock().as_ref().cloned() };
        let Some(active_camera) = active_camera else {
            return;
        };
        log_once::debug_once!("Active camera found: {:?}", active_camera);

        let q = self
            .world
            .query_one::<&Camera>(active_camera)
            .get()
            .ok()
            .cloned();

        let Some(camera) = q else {
            return;
        };
        log_once::debug_once!("Camera ready");
        log_once::debug_once!("Camera currently being viewed: {}", camera.label);

        let Some(camera_bind_group) = self.camera_bind_group.as_ref() else {
            log_once::warn_once!("Camera bind group not ready");
            return;
        };

        {
            puffin::profile_scope!("Clearing viewport");
            let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("viewport clear pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: hdr.render_view(),
                    depth_slice: None,
                    resolve_target: hdr.resolve_target(),
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

        if let Some(light_pipeline) = &mut self.light_cube_pipeline {
            light_pipeline.update(graphics.clone(), &self.world);
        }

        let lights = {
            puffin::profile_scope!("Locating lights");
            let mut lights = Vec::new();
            let mut query = self.world.query::<&Light>();
            for light in query.iter() {
                lights.push(light.clone());
            }
            lights
        };

        if let Some(globals) = &mut self.shader_globals {
            puffin::profile_scope!("Fetching globals");
            let enabled_count = lights
                .iter()
                .filter(|light| light.component.enabled)
                .count() as u32;
            globals.set_num_lights(enabled_count);
            globals.write(&graphics.queue);
        }

        let mut static_batches: HashMap<u64, Vec<InstanceRaw>> = HashMap::new();
        let mut animated_instances: Vec<
            (Entity, u64, InstanceRaw, wgpu::Buffer, wgpu::Buffer, wgpu::Buffer, u32),
        > = Vec::new();

        {
            puffin::profile_scope!("finding all renderers and animation components");
            let mut query = self
                .world
                .query::<(Entity, &MeshRenderer, Option<&mut AnimationComponent>)>();

            for (entity, renderer, animation) in query.iter() {
                let handle = renderer.model();
                if handle.is_null() {
                    continue;
                }

                let instance = renderer.instance.to_raw();
                if let Some(animation) = animation {
                    let has_skinning = !animation.skinning_matrices.is_empty();
                    let has_morph_weights = !animation.morph_weights.is_empty();
                    if !has_skinning && !has_morph_weights {
                        static_batches.entry(handle.id).or_default().push(instance);
                        continue;
                    }

                    animation.prepare_gpu_resources(graphics.clone());

                    let skinning_buffer = if let Some(buffer) = animation
                        .skinning_buffer
                        .as_ref()
                        .map(|buffer| buffer.buffer().clone())
                    {
                        buffer
                    } else if !has_skinning {
                        let Some(default_skinning_buffer) = self.default_skinning_buffer.as_ref()
                        else {
                            static_batches.entry(handle.id).or_default().push(instance);
                            continue;
                        };
                        default_skinning_buffer.clone()
                    } else {
                        static_batches.entry(handle.id).or_default().push(instance);
                        continue;
                    };
                    let Some(morph_weights_buffer) = animation
                        .morph_weights_buffer
                        .as_ref()
                        .map(|buffer| buffer.buffer().clone())
                    else {
                        static_batches.entry(handle.id).or_default().push(instance);
                        continue;
                    };
                    let Some(morph_info_buffer) = animation
                        .morph_info_buffer
                        .as_ref()
                        .map(|buffer| buffer.buffer().clone())
                    else {
                        static_batches.entry(handle.id).or_default().push(instance);
                        continue;
                    };

                    animated_instances.push((
                        entity,
                        handle.id,
                        instance,
                        skinning_buffer,
                        morph_weights_buffer,
                        morph_info_buffer,
                        animation.morph_weight_count,
                    ));
                } else {
                    static_batches.entry(handle.id).or_default().push(instance);
                }
            }
        }

        let registry = ASSET_REGISTRY.read();
        let mut prepared_models = Vec::new();
        for (handle, instances) in static_batches {
            puffin::profile_scope!("preparing models");
            let Some(model) = registry.get_model(Handle::new(handle)) else {
                log_once::error_once!("Missing model handle {} in registry", handle);
                continue;
            };

            let instance_buffer = self.instance_buffer_cache.entry(handle).or_insert_with(|| {
                ResizableBuffer::new(
                    &graphics.device,
                    instances.len().max(1),
                    wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    "Runtime Instance Buffer",
                )
            });
            instance_buffer.write(&graphics.device, &graphics.queue, &instances);

            prepared_models.push((model, handle, instances.len() as u32));
        }

        if let Some(light_pipeline) = &self.light_cube_pipeline {
            if let Some(l) = lights.first()
                && let Some(model) = registry.get_model(l.cube_model)
            {
                {
                    puffin::profile_scope!("light cube pass");
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("light cube render pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: hdr.render_view(),
                            depth_slice: None,
                            resolve_target: hdr.resolve_target(),
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

                    render_pass.set_pipeline(light_pipeline.pipeline());
                    for light in &lights {
                        render_pass.set_vertex_buffer(1, light.instance_buffer.buffer().slice(..));
                        if !light.component.visible {
                            continue;
                        }
                        render_pass.draw_light_model(&model, camera_bind_group, &light.bind_group);
                    }
                }
            } else {
                log_once::error_once!("Missing light cube model handle in registry",);
            }
        }

        // model rendering
        let sky = self
            .sky_pipeline
            .as_ref()
            .expect("Sky pipeline must be initialised before rendering models");
        let environment_bind_group = &sky.environment_bind_group;

        let Some(pipeline) = self.main_render_pipeline.as_mut() else {
            log_once::warn_once!("Render pipeline not ready");
            return;
        };
        log_once::debug_once!("Pipeline ready");

        // static models
        if let Some(_) = &self.light_cube_pipeline {
            puffin::profile_scope!("model render pass");
            for (model, handle, instance_count) in prepared_models {
                let default_skinning_buffer = self
                    .default_skinning_buffer
                    .as_ref()
                    .expect("Default skinning buffer not initialised");
                let default_morph_weights_buffer = self
                    .default_morph_weights_buffer
                    .as_ref()
                    .expect("Default morph weights buffer not initialised");
                let default_morph_info_buffer = self
                    .default_morph_info_buffer
                    .as_ref()
                    .expect("Default morph info buffer not initialised");
                let per_frame_bind_group = pipeline
                    .per_frame
                    .as_ref()
                    .expect("Per-frame bind group not initialised")
                    .clone();

                let morph_deltas_buffer = model
                    .morph_deltas_buffer
                    .as_ref()
                    .or(self.default_morph_deltas_buffer.as_ref());
                let Some(morph_deltas_buffer) = morph_deltas_buffer else {
                    log_once::error_once!("Missing morph deltas buffer for model {}", handle);
                    continue;
                };

                let animation_bind_group = pipeline.animation_bind_group(
                    graphics.clone(),
                    default_skinning_buffer,
                    morph_deltas_buffer,
                    default_morph_weights_buffer,
                    default_morph_info_buffer,
                );

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("model render pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: hdr.render_view(),
                        depth_slice: None,
                        resolve_target: hdr.resolve_target(),
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
                if let Some(instance_buffer) = self.instance_buffer_cache.get(&handle) {
                    render_pass
                        .set_vertex_buffer(1, instance_buffer.slice(instance_count as usize));
                } else {
                    continue;
                }

                for mesh in &model.meshes {
                    let mut weights = mesh.morph_default_weights.clone();
                    let target_count = mesh.morph_target_count as usize;
                    if weights.len() < target_count {
                        weights.resize(target_count, 0.0);
                    }
                    if weights.is_empty() {
                        weights.push(0.0);
                    }

                    graphics.queue.write_buffer(
                        default_morph_weights_buffer,
                        0,
                        bytemuck::cast_slice(&weights),
                    );

                    let info = MorphTargetInfo {
                        num_vertices: mesh.morph_vertex_count,
                        num_targets: mesh.morph_target_count,
                        base_offset: mesh.morph_deltas_offset,
                        weight_offset: 0,
                        uses_morph: if mesh.morph_target_count > 0 && !weights.is_empty() {
                            1
                        } else {
                            0
                        },
                        _padding: Default::default(),
                    };

                    graphics
                        .queue
                        .write_buffer(default_morph_info_buffer, 0, bytemuck::bytes_of(&info));

                    let material = &model.materials[mesh.material];
                    render_pass.draw_mesh_instanced(
                        mesh,
                        material,
                        0..instance_count,
                        &per_frame_bind_group,
                        &animation_bind_group,
                        environment_bind_group,
                    );
                }
            }
        }

        // animated models
        if let Some(_) = &self.light_cube_pipeline {
            puffin::profile_scope!("animated model render pass");
            for (
                entity,
                handle,
                instance,
                skinning_buffer,
                morph_weights_buffer,
                morph_info_buffer,
                morph_weight_count,
            ) in animated_instances
            {
                let Some(model) = registry.get_model(Handle::new(handle)) else {
                    log_once::error_once!("Missing model handle {} in registry", handle);
                    continue;
                };

                let morph_deltas_buffer = model
                    .morph_deltas_buffer
                    .as_ref()
                    .or(self.default_morph_deltas_buffer.as_ref());
                let Some(morph_deltas_buffer) = morph_deltas_buffer else {
                    log_once::error_once!("Missing morph deltas buffer for model {}", handle);
                    continue;
                };

                let animation_bind_group = pipeline.animation_bind_group(
                    graphics.clone(),
                    &skinning_buffer,
                    &morph_deltas_buffer,
                    &morph_weights_buffer,
                    &morph_info_buffer,
                );
                let per_frame_bind_group = pipeline
                    .per_frame
                    .as_ref()
                    .expect("Per-frame bind group not initialised")
                    .clone();

                let instance_buffer = self
                    .animated_instance_buffers
                    .entry(entity)
                    .or_insert_with(|| {
                        ResizableBuffer::new(
                            &graphics.device,
                            1,
                            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                            "animated instance buffer",
                        )
                    });
                instance_buffer.write(&graphics.device, &graphics.queue, &[instance]);

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("model render pass (animated)"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: hdr.render_view(),
                        depth_slice: None,
                        resolve_target: hdr.resolve_target(),
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
                render_pass.set_vertex_buffer(1, instance_buffer.slice(1));

                for mesh in &model.meshes {
                    let mesh_target_count = mesh.morph_target_count.min(morph_weight_count);
                    let info = MorphTargetInfo {
                        num_vertices: mesh.morph_vertex_count,
                        num_targets: mesh_target_count,
                        base_offset: mesh.morph_deltas_offset,
                        weight_offset: 0,
                        uses_morph: if mesh_target_count > 0 { 1 } else { 0 },
                        _padding: Default::default(),
                    };

                    graphics
                        .queue
                        .write_buffer(&morph_info_buffer, 0, bytemuck::bytes_of(&info));

                    let material = &model.materials[mesh.material];
                    render_pass.draw_mesh_instanced(
                        mesh,
                        material,
                        0..1,
                        &per_frame_bind_group,
                        &animation_bind_group,
                        environment_bind_group,
                    );
                }
            }
        }

        // skybox rendering
        if let Some(sky) = &self.sky_pipeline {
            puffin::profile_scope!("sky render pass");
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("sky render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: hdr.render_view(),
                    depth_slice: None,
                    resolve_target: hdr.resolve_target(),
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
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&sky.pipeline);
            render_pass.set_bind_group(0, &sky.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &sky.environment_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
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
                puffin::profile_scope!("collider wireframe pipeline");
                if let Some(collider_pipeline) = &self.collider_wireframe_pipeline {
                    log_once::debug_once!("Found collider wireframe pipeline");
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("collider wireframe render pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: hdr.render_view(),
                            depth_slice: None,
                            resolve_target: hdr.resolve_target(),
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
                    render_pass.set_bind_group(0, camera_bind_group, &[]);

                    let mut instances_by_shape: HashMap<
                        ColliderShapeKey,
                        Vec<ColliderInstanceRaw>,
                    > = HashMap::new();

                    let mut q = self.world.query::<(&EntityTransform, &ColliderGroup)>();
                    for (entity_transform, group) in q.iter() {
                        for collider in &group.colliders {
                            let world_tf = entity_transform.sync();

                            let entity_matrix = world_tf.matrix().as_mat4();

                            let offset_transform = Transform::new()
                                .with_offset(collider.translation, collider.rotation);
                            let offset_matrix = offset_transform.matrix().as_mat4();

                            let final_matrix = entity_matrix * offset_matrix;

                            let color = [0.0, 1.0, 0.0, 1.0];
                            let instance = ColliderInstanceRaw::from_matrix(final_matrix, color);

                            let key = ColliderShapeKey::from(&collider.shape);
                            instances_by_shape.entry(key).or_default().push(instance);

                            self.collider_wireframe_geometry_cache
                                .entry(key)
                                .or_insert_with(|| {
                                    create_wireframe_geometry(graphics.clone(), &collider.shape)
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

                        let instance_buffer =
                            self.collider_instance_buffer.get_or_insert_with(|| {
                                ResizableBuffer::new(
                                    &graphics.device,
                                    all_instances.len().max(10),
                                    wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                                    "Collider Instance Buffer",
                                )
                            });
                        instance_buffer.write(&graphics.device, &graphics.queue, &all_instances);

                        for (key, start, count) in draws {
                            let Some(geometry) = self.collider_wireframe_geometry_cache.get(&key)
                            else {
                                continue;
                            };

                            let start_bytes = (start * std::mem::size_of::<ColliderInstanceRaw>())
                                as wgpu::BufferAddress;
                            let end_bytes = ((start + count)
                                * std::mem::size_of::<ColliderInstanceRaw>())
                                as wgpu::BufferAddress;

                            render_pass.set_vertex_buffer(
                                1,
                                instance_buffer.buffer().slice(start_bytes..end_bytes),
                            );
                            render_pass.set_vertex_buffer(0, geometry.vertex_buffer.slice(..));
                            render_pass.set_index_buffer(
                                geometry.index_buffer.slice(..),
                                wgpu::IndexFormat::Uint16,
                            );
                            render_pass.draw_indexed(0..geometry.index_count, 0, 0..count as u32);
                        }
                    }
                } else {
                    log_once::warn_once!("No collider pipeline found");
                }
            }
        }

        // kino billboard renderer (late stage, runtime parity)
        {
            if let Some(kino) = &mut self.kino {
                let mut kino_encoder = CommandEncoder::new(graphics.clone(), Some("kino billboard encoder"));
                kino.render_billboard_targets(
                    &graphics.device,
                    &graphics.queue,
                    &mut kino_encoder,
                );

                if let Err(e) = kino_encoder.submit() {
                    log_once::error_once!("Unable to submit billboard kino pass: {}", e);
                }
            }

            if let Some(billboard_pipeline) = &self.billboard_pipeline {
                let camera_position = camera.position().as_vec3();
                let camera_projection = Mat4::from_cols_array_2d(&camera.uniform.view_proj);

                let mut kino_views = HashMap::<u64, wgpu::TextureView>::new();
                if let Some(kino) = &mut self.kino {
                    kino_views.extend(kino.billboard_render_target_views());
                }

                let single_fallback_view = if kino_views.len() == 1 {
                    kino_views.values().next().cloned()
                } else {
                    None
                };

                let mut billboards: Vec<(Mat4, wgpu::TextureView)> = Vec::new();
                let mut query = self
                    .world
                    .query::<(Entity, &BillboardComponent, Option<&EntityTransform>)>();

                for (entity, billboard, entity_transform) in query.iter() {
                    if !billboard.enabled {
                        continue;
                    }

                    let entity_id = entity.to_bits().get();
                    let texture_view = kino_views
                        .get(&entity_id)
                        .cloned()
                        .or_else(|| single_fallback_view.clone());

                    let Some(texture_view) = texture_view else {
                        continue;
                    };

                    let position = entity_transform
                        .map(|transform| transform.sync().position.as_vec3())
                        .unwrap_or(glam::Vec3::ZERO)
                        + billboard.offset;
                    let world_size = billboard.world_size;
                    let scale = glam::Vec3::new(world_size.x, world_size.y, 1.0);

                    let rotation = if let Some(rotation) = billboard.rotation {
                        rotation
                    } else {
                        let to_camera = (camera_position - position).normalize_or_zero();
                        if to_camera.length_squared() > 0.0 {
                            let mut world_up = glam::Vec3::Y;
                            if to_camera.dot(world_up).abs() > 0.999 {
                                world_up = glam::Vec3::X;
                            }

                            let right = world_up.cross(to_camera).normalize_or_zero();
                            let up = to_camera.cross(right).normalize_or_zero();
                            let basis = glam::Mat3::from_cols(right, up, to_camera);
                            glam::Quat::from_mat3(&basis)
                        } else {
                            glam::Quat::IDENTITY
                        }
                    };

                    let transform = Mat4::from_scale_rotation_translation(scale, rotation, position);
                    billboards.push((transform, texture_view));
                }

                if !billboards.is_empty() {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("editor billboard render pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: hdr.render_view(),
                            depth_slice: None,
                            resolve_target: hdr.resolve_target(),
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
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    for (transform, texture_view) in billboards {
                        billboard_pipeline.draw(
                            graphics.clone(),
                            &mut render_pass,
                            transform,
                            camera_projection,
                            &texture_view,
                        );
                    }
                }
            }
        }

        hdr.process(&mut encoder, &graphics.viewport_texture.view);

        if let Err(e) = encoder.submit() {
            log_once::error_once!("{}", e);
        }

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
            Some("eucp") | Some("eucs") => project_root.join("scenes"),
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

    fn editor_specific_render(&mut self, graphics: &Arc<SharedGraphicsContext>) {
        self.size = graphics.viewport_texture.size;
        self.texture_id = Some(*graphics.texture_id.clone());
        self.window = Some(graphics.window.clone());

        self.show_ui(&graphics.get_egui_context(), graphics.clone());
        eucalyptus_core::logging::render(&graphics.get_egui_context());
    }
}
