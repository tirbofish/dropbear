use std::collections::HashMap;
use std::path::PathBuf;
use crossbeam_channel::unbounded;
use egui::{CentralPanel, MenuBar, TopBottomPanel};
use hecs::{Entity, World};
use wgpu::Color;
use wgpu::util::DeviceExt;
use winit::event_loop::ActiveEventLoop;
use dropbear_engine::camera::Camera;
use dropbear_engine::entity::MeshRenderer;
use dropbear_engine::graphics::{InstanceRaw, RenderContext};
use dropbear_engine::lighting::{Light, LightComponent};
use dropbear_engine::model::{DrawLight, DrawModel, ModelId, MODEL_CACHE};
use dropbear_engine::scene::{Scene, SceneCommand};
use eucalyptus_core::command::{COMMAND_BUFFER, CommandBufferPoller};
use eucalyptus_core::input::InputState;
use eucalyptus_core::logging;
use eucalyptus_core::scripting::ScriptTarget;
use eucalyptus_core::states::{Script, WorldLoadingStatus, PROJECT, SCENES};
use crate::editor::Signal;
use crate::runtime::{PlayMode, WindowMode};

impl Scene for PlayMode {
    fn load(&mut self, graphics: &mut RenderContext) {
        if self.current_scene.is_none() {
            let initial_scene = if let Some(s) = &self.initial_scene {
                s.clone()
            } else {
                let proj = PROJECT.read();
                proj.runtime_settings.initial_scene.clone().unwrap()
            };

            let first_time = IsSceneLoaded::new_first_time(initial_scene);

            self.request_async_scene_load(graphics, first_time);
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
                self.world = Box::new(loaded_world);
                log::debug!("World received");
                if let Some(ref mut progress) = self.scene_progress {
                    progress.world_loaded = true;
                    self.current_scene = Some(progress.requested_scene.clone());
                }
            } else {
                self.world_receiver = Some(receiver);
                return;
            }
        }

        if let Some(handle) = self.scene_loading_handle.take() {
            if let Some(cam) = graphics.shared.future_queue.exchange_owned_as::<Entity>(&handle) {
                self.active_camera = Some(cam);
                log::debug!("Camera entity received: {:?}", cam);
                if let Some(ref mut progress) = self.scene_progress {
                    progress.camera_received = true;
                }
                self.load_wgpu_nerdy_stuff(graphics);
            } else {
                self.scene_loading_handle = Some(handle)
            }
        }
        
        if let Some(ref progress) = self.scene_progress {
            if progress.is_everything_loaded() {
                if progress.is_first_scene() && !self.scripts_ready {
                    log::debug!("Initialising scripts for first scene load");

                    let mut entity_tag_map: HashMap<String, Vec<Entity>> = HashMap::new();
                    for (entity_id, script) in self.world.query::<&Script>().iter() {
                        for tag in &script.tags {
                            entity_tag_map.entry(tag.clone()).or_default().push(entity_id);
                        }
                    }

                    fn find_jvm_library_path() -> PathBuf {
                        let proj = PROJECT.read();
                        let project_path = if !proj.project_path.is_dir() {
                            proj.project_path.parent().unwrap().to_path_buf()
                        } else {
                            proj.project_path.clone()
                        }.join("build/libs");

                        let mut latest_jar: Option<(PathBuf, std::time::SystemTime)> = None;

                        for entry in std::fs::read_dir(&project_path).unwrap() {
                            let entry = entry.unwrap();
                            let path = entry.path();

                            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                                if filename.ends_with("-all.jar") {
                                    let metadata = entry.metadata().unwrap();
                                    let modified = metadata.modified().unwrap();

                                    match latest_jar {
                                        None => latest_jar = Some((path.clone(), modified)),
                                        Some((_, latest_time)) if modified > latest_time => {
                                            latest_jar = Some((path.clone(), modified));
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }

                        latest_jar.map(|(path, _)| path).expect("No suitable candidate for a JVM targeted play mode session available")
                    }

                    let target = ScriptTarget::JVM { library_path: find_jvm_library_path() };

                    if let Err(e) = self.script_manager.init_script(
                        None,
                        entity_tag_map.clone(),
                        target.clone(),
                    ) {
                        log::error!("Failed to initialise scripts: {}", e);
                    } else {
                        let world_ptr = self.world.as_mut() as *mut World;
                        let input_ptr = &mut self.input_state as *mut InputState;
                        let graphics_ptr = &COMMAND_BUFFER.0 as *const _ as *const _;

                        if let Err(e) = self.script_manager.load_script(world_ptr, input_ptr, graphics_ptr) {
                            log::error!("Failed to load scripts: {}", e);
                        } else {
                            self.scripts_ready = true;
                            log::debug!("Scripts initialised successfully");
                        }
                    }
                }

                if self.scripts_ready {
                    if let Err(e) = unsafe { self.script_manager.update_script(&self.world, dt) } {
                        log::error!("Script update error: {}", e);
                    }
                }
            }
        }

        TopBottomPanel::top("menu_bar").show(&graphics.shared.get_egui_context(), |ui| {
            MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Window", |ui| {
                    ui.menu_button("Resolution", |ui| {
                        let resolutions = [
                            (1280, 720, "1280×720 (HD)"),
                            (1600, 900, "1600×900"),
                            (1920, 1080, "1920×1080 (Full HD)"),
                            (2560, 1440, "2560×1440 (QHD)"),
                            (3840, 2160, "3840×2160 (4K)"),
                        ];

                        for (width, height, label) in resolutions {
                            let is_current = self.display_settings.render_resolution == (width, height);
                            if ui.selectable_label(is_current, label).clicked() {
                                self.display_settings.render_resolution = (width, height);
                                ui.close();
                            }
                        }
                    });

                    ui.separator();

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

                    if ui.checkbox(&mut self.display_settings.maintain_aspect_ratio, "Maintain aspect ratio").clicked() {
                        self.display_settings.maintain_aspect_ratio = !self.display_settings.maintain_aspect_ratio;
                    }

                    if ui.checkbox(&mut self.display_settings.vsync, "VSync").clicked() {
                        self.display_settings.vsync = !self.display_settings.vsync;
                    }
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
                if !p.is_everything_loaded() {
                    // todo: change from label to "splashscreen"
                    ui.centered_and_justified(|ui| {
                        ui.label("Loading scene...");
                    });
                    return;
                }
            }

            let texture_id = *graphics.shared.texture_id;
            
            let (render_width, render_height) = self.display_settings.render_resolution;
            let render_aspect = render_width as f32 / render_height as f32;

            let available_size = ui.available_size();
            let available_rect = ui.available_rect_before_wrap();

            let (display_width, display_height) = if self.display_settings.maintain_aspect_ratio {
                if available_size.x / available_size.y > render_aspect {
                    let height = available_size.y;
                    let width = height * render_aspect;
                    (width, height)
                } else {
                    let width = available_size.x;
                    let height = width / render_aspect;
                    (width, height)
                }
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
                ui.add_sized(
                    [display_width, display_height],
                    egui::Image::new((texture_id, [display_width, display_height].into()))
                        .fit_to_exact_size([display_width, display_height].into()),
                )
            });
        });
    }

    fn render(&mut self, graphics: &mut RenderContext) {
        // cornflower blue
        let color = Color {
            r: 100.0 / 255.0,
            g: 149.0 / 255.0,
            b: 237.0 / 255.0,
            a: 1.0,
        };

        if let Some(pipeline) = &self.render_pipeline {
            log_once::debug_once!("Found render pipeline");

            if let Some(active_camera) = self.active_camera {
                let cam = {
                    if let Ok(mut query) = self.world.query_one::<&Camera>(active_camera) {
                        query.get().cloned()
                    } else {
                        None
                    }
                };

                if let Some(camera) = cam {
                    let lights = {
                        let mut lights = Vec::new();
                        let mut light_query = self.world.query::<(&Light, &LightComponent)>();
                        for (_, (light, comp)) in light_query.iter() {
                            lights.push((light.clone(), comp.clone()));
                        }
                        lights
                    };

                    let entities = {
                        let mut entities = Vec::new();
                        let mut entity_query = self.world.query::<&MeshRenderer>();
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
            if let Some(p) = &self.scene_progress {
                if p.is_everything_loaded() {
                    log_once::warn_once!("No render pipeline exists");
                } else {
                    log_once::debug_once!("No render pipeline exists, but world not loaded yet");
                }
            } else {
                log_once::debug_once!("No render pipeline exists");
            }
        }
    }

    fn exit(&mut self, _event_loop: &ActiveEventLoop) {}

    fn run_command(&mut self) -> SceneCommand {
        std::mem::replace(&mut self.scene_command, SceneCommand::None)
    }
}

pub struct IsSceneLoaded {
    pub(crate) requested_scene: String,
    is_first_scene: bool,
    pub(crate) scene_handle_requested: bool,
    pub(crate) world_loaded: bool,
    pub(crate) camera_received: bool,
}

impl IsSceneLoaded {
    pub fn new(requested_scene: String) -> Self {
        Self {
            requested_scene,
            is_first_scene: false,
            scene_handle_requested: false,
            world_loaded: false,
            camera_received: false,
        }
    }

    pub fn new_first_time(requested_scene: String) -> Self {
        Self {
            requested_scene,
            is_first_scene: true,
            scene_handle_requested: false,
            world_loaded: false,
            camera_received: false,
        }
    }

    pub fn is_everything_loaded(&self) -> bool {
        self.scene_handle_requested && self.world_loaded && self.camera_received
    }

    pub fn is_first_scene(&self) -> bool {
        self.is_first_scene
    }
}