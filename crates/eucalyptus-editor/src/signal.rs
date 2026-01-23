use crate::editor::{Editor, EditorState, Signal};
use dropbear_engine::camera::Camera;
use dropbear_engine::entity::{MeshRenderer, Transform};
use dropbear_engine::graphics::{SharedGraphicsContext};
use dropbear_engine::lighting::{Light as EngineLight, LightComponent};
use dropbear_engine::model::{LoadedModel, Material, Model, ModelId, MODEL_CACHE};
use dropbear_engine::procedural::{ProcedurallyGeneratedObject, ProcObj};
use dropbear_engine::texture::{Texture, TextureWrapMode};
use dropbear_engine::utils::{relative_path_from_euca, EUCA_SCHEME, ResourceReference, ResourceReferenceType};
use egui::Align2;
use eucalyptus_core::camera::{CameraComponent, CameraType};
use eucalyptus_core::scripting::{build_jvm, BuildStatus};
use eucalyptus_core::spawn::{push_pending_spawn, PendingSpawn};
use eucalyptus_core::states::{
    EditorTab, Label, Light, Script, PROJECT,
};
use eucalyptus_core::{fatal, info, success, success_without_console, warn, warn_without_console};
use std::any::TypeId;
use std::path::PathBuf;
use std::sync::Arc;
use winit::keyboard::KeyCode;
use eucalyptus_core::properties::CustomProperties;

fn resolve_editor_path(uri: &str) -> PathBuf {
    if uri.starts_with(EUCA_SCHEME) {
        let relative = relative_path_from_euca(uri)
            .unwrap_or_else(|_| uri.trim_start_matches(EUCA_SCHEME));
        let project_path = PROJECT.read().project_path.clone();
        project_path.join("resources").join(relative)
    } else {
        PathBuf::from(uri)
    }
}

pub trait SignalController {
    fn run_signal(&mut self, graphics: Arc<SharedGraphicsContext>) -> anyhow::Result<()>;
}

impl SignalController for Editor {
    fn run_signal(&mut self, graphics: Arc<SharedGraphicsContext>) -> anyhow::Result<()> {
        fn is_legacy_internal_cube_uri(uri: &str) -> bool {
            let uri = uri.replace('\\', "/");
            uri.ends_with("internal/dropbear/models/cube")
        }

        let local_signal: Option<Signal> = None;
        let show = true;

        match &self.signal {
            Signal::None => {
                // returns absolutely nothing because no signal is set.
                Ok::<(), anyhow::Error>(())
            }
            Signal::Copy(_) => Ok(()),
            Signal::Paste(scene_entity) => {
                let spawn = PendingSpawn {
                    scene_entity: scene_entity.clone(),
                    handle: None,
                };
                push_pending_spawn(spawn);
                self.signal = Signal::Copy(scene_entity.clone());
                Ok(())
            }

            Signal::SetModelImportScale(entity, scale) => {
                if let Ok(mut renderer) = self.world.get::<&mut MeshRenderer>(*entity) {
                    renderer.set_import_scale(*scale);
                }
                self.signal = Signal::None;
                Ok(())
            }
            Signal::Delete => {
                if let Some(sel_e) = &self.selected_entity {
                    let is_viewport_cam =
                        if let Ok(c) = self.world.query_one::<&CameraComponent>(*sel_e).get() {
                            matches!(c.camera_type, CameraType::Debug)
                        } else {
                            false
                        };
                    if is_viewport_cam {
                        warn!("You can't delete the viewport camera");
                        self.signal = Signal::None;
                        Ok(())
                    } else {
                        match self.world.despawn(*sel_e) {
                            Ok(_) => {
                                info!("Decimated entity");
                                self.signal = Signal::None;
                                Ok(())
                            }
                            Err(e) => {
                                self.signal = Signal::None;
                                fatal!("Failed to delete entity: {}", e);
                                Err(anyhow::anyhow!(e))
                            }
                        }
                    }
                } else {
                    // no entity has been selected, so all good
                    Ok(())
                }
            }
            Signal::Undo => {
                if let Some(action) = self.undo_stack.pop() {
                    match action.undo(&mut self.world) {
                        Ok(_) => {
                            info!("Undid action");
                        }
                        Err(e) => {
                            warn!("Failed to undo action: {}", e);
                        }
                    }
                } else {
                    warn_without_console!("Nothing to undo");
                    log::debug!("No undoable actions in stack");
                }
                self.signal = Signal::None;
                Ok(())
            }
            Signal::Play => {
                if matches!(self.editor_state, EditorState::Playing) {
                    log::warn!("Unable to play: already in playing mode");
                    self.signal = Signal::None;
                    return Err(anyhow::anyhow!("Unable to play: already in playing mode"));
                }

                if matches!(self.editor_state, EditorState::Editing) {
                    log::debug!("Project save");
                    match self.save_project_config() {
                        Ok(_) => {}
                        Err(e) => {
                            fatal!("Error saving project: {}", e);
                        }
                    }

                    log::debug!("Starting build process");
                    let (tx, rx) = crossbeam_channel::unbounded();
                    self.progress_rx = Some(rx);

                    self.build_logs.clear();
                    self.build_progress = 0.0;
                    self.show_build_window = true;
                    self.last_build_error = None;

                    let project_root = {
                        let cfg = PROJECT.read();
                        cfg.project_path.clone()
                    };

                    let project_root = project_root.to_path_buf();
                    let status_tx = tx.clone();

                    let handle = graphics
                        .future_queue
                        .push(async move { build_jvm(project_root, status_tx).await });

                    log::debug!(
                        "Pushed future to future_queue, received handle: {:?}",
                        handle
                    );

                    self.handle_created = Some(handle);

                    self.editor_state = EditorState::Building;
                    log::debug!("Set editor state to EditorState::Building");
                }

                if matches!(self.editor_state, EditorState::Building) {
                    #[cfg(not(target_os = "macos"))]
                    let ctrl_pressed = self
                        .input_state
                        .pressed_keys
                        .contains(&KeyCode::ControlLeft)
                        || self
                            .input_state
                            .pressed_keys
                            .contains(&KeyCode::ControlRight);
                    #[cfg(target_os = "macos")]
                    let ctrl_pressed = self.input_state.pressed_keys.contains(&KeyCode::SuperLeft)
                        || self.input_state.pressed_keys.contains(&KeyCode::SuperRight);

                    let alt_pressed = self.input_state.pressed_keys.contains(&KeyCode::AltLeft)
                        || self.input_state.pressed_keys.contains(&KeyCode::AltRight);

                    // Ctrl+Alt+P skips build process and starts running, such as if using cached jar
                    if ctrl_pressed
                        && alt_pressed
                        && self.input_state.pressed_keys.contains(&KeyCode::KeyP)
                    {
                        if let Some(handle) = self.handle_created {
                            log::debug!("Cancelling build task due to manual intervention");
                            graphics.future_queue.cancel(&handle);
                        } else {
                            log::warn!("No handle was created during this time. Weird...")
                        }

                        let project_root = {
                            let cfg = PROJECT.read();
                            cfg.project_path.clone()
                        };
                        let libs_dir = project_root.join("build").join("libs");
                        if !libs_dir.exists() {
                            let err =
                                "Build succeeded but 'build/libs' directory is missing".to_string();
                            return Err(anyhow::anyhow!(err));
                        }

                        let jar_files: Vec<PathBuf> = std::fs::read_dir(&libs_dir)?
                            .filter_map(|entry| entry.ok().map(|e| e.path()))
                            .filter(|path| {
                                path.extension()
                                    .map_or(false, |ext| ext.eq_ignore_ascii_case("jar"))
                                    && !path
                                        .file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                        .contains("-sources")
                                    && !path
                                        .file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                        .contains("-javadoc")
                            })
                            .collect();

                        if jar_files.is_empty() {
                            let err = "No JAR artifact found in 'build/libs'".to_string();
                            return Err(anyhow::anyhow!(err));
                        }

                        let fat_jar = jar_files.iter().find(|path| {
                            path.file_name()
                                .and_then(|n| n.to_str())
                                .map_or(false, |name| name.contains("-all"))
                        });

                        let jar_path = if let Some(fat) = fat_jar {
                            fat.clone()
                        } else {
                            jar_files
                                .into_iter()
                                .max_by_key(|path| {
                                    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
                                })
                                .unwrap()
                        };

                        info!("Using cached JAR: {}", jar_path.display());

                        self.show_build_window = false;

                        self.load_play_mode()?;
                        return Ok(());
                    }

                    let mut local_handle_exchanged: Option<anyhow::Result<PathBuf>> = None;
                    if let Some(rx) = &self.progress_rx {
                        while let Ok(status) = rx.try_recv() {
                            match status {
                                BuildStatus::Started => {
                                    self.build_logs.push("Build started...".to_string());
                                    self.build_progress = 0.1;
                                    log::info!("Build started");
                                }
                                BuildStatus::Building(msg) => {
                                    log::info!("[BUILD] {}", msg);
                                    self.build_logs.push(msg.clone());
                                    self.build_progress = (self.build_progress + 0.01).min(0.9);
                                }
                                BuildStatus::Completed => {
                                    self.build_logs
                                        .push("Build completed successfully!".to_string());
                                    self.build_progress = 1.0;
                                    success_without_console!("Build completed");
                                    log::info!("Build completed successfully!");

                                    if let Some(handle) = self.handle_created {
                                        if let Some(result) = graphics
                                            .future_queue
                                            .exchange_owned_as::<anyhow::Result<PathBuf>>(&handle)
                                        {
                                            local_handle_exchanged = Some(result);
                                        }
                                    } else {
                                        self.signal = Signal::None;
                                        self.show_build_window = false;
                                        self.editor_state = EditorState::Editing;
                                    }
                                }
                                BuildStatus::Failed(_e) => {
                                    let error_msg = format!("Build failed, check logs");
                                    self.build_logs.push(error_msg.clone());

                                    self.build_progress = 0.0;
                                    fatal!("Failed to build gradle, check logs");

                                    self.signal = Signal::None;
                                    self.show_build_window = false;
                                    self.editor_state = EditorState::Editing;
                                    self.dock_state
                                        .push_to_focused_leaf(EditorTab::ErrorConsole);
                                }
                            }
                        }
                    }

                    if self.show_build_window {
                        let mut window_open = true;
                        egui::Window::new("Building Project")
                            .collapsible(false)
                            .resizable(false)
                            .fixed_size([500.0, 400.0])
                            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                            .open(&mut window_open)
                            .show(&graphics.get_egui_context(), |ui| {
                                ui.vertical_centered(|ui| {
                                    ui.heading("Gradle Build Progress");
                                    ui.add_space(10.0);

                                    let progress_bar = egui::ProgressBar::new(self.build_progress)
                                        .show_percentage()
                                        .animate(true);
                                    ui.add(progress_bar);

                                    ui.add_space(15.0);
                                    ui.separator();
                                    ui.add_space(5.0);

                                    ui.heading("Build Log");
                                    ui.add_space(5.0);

                                    egui::ScrollArea::vertical()
                                        .stick_to_bottom(true)
                                        .max_height(200.0)
                                        .auto_shrink([false, false])
                                        .show(ui, |ui| {
                                            for log_line in &self.build_logs {
                                                ui.label(
                                                    egui::RichText::new(log_line)
                                                        .family(egui::FontFamily::Monospace)
                                                        .size(12.0),
                                                );
                                            }

                                            if !self.build_logs.is_empty() {
                                                ui.add_space(10.0);
                                                ui.label(
                                                    egui::RichText::new(format!(
                                                        "Total log entries: {}",
                                                        self.build_logs.len()
                                                    ))
                                                    .italics()
                                                    .color(egui::Color32::GRAY),
                                                );
                                                ui.label(
                                                    "Tip: Press Ctrl+Alt+P to skip build and start running",
                                                );
                                            }
                                        });

                                    ui.add_space(10.0);
                                });
                            });

                        if !window_open {
                            if let Some(handle) = self.handle_created {
                                log::warn!("Cancelling build task due to window close");
                                graphics.future_queue.cancel(&handle);
                            }

                            self.show_build_window = false;
                            self.handle_created = None;
                            self.progress_rx = None;
                            self.editor_state = EditorState::Editing;
                            self.signal = Signal::None;
                        }
                    }

                    if let Some(result) = local_handle_exchanged {
                        log::debug!("Build future completed, processing result");
                        self.handle_created = None;
                        self.progress_rx = None;

                        match result {
                            Ok(path) => {
                                log::debug!("Path is valid, JAR location as {}", path.display());
                                success!("Build completed successfully!");
                                self.show_build_window = false;

                                self.load_play_mode()?;
                            }
                            Err(e) => {
                                let error_msg = format!("Build process error: {}", e);
                                self.build_logs.push(error_msg.clone());
                                self.last_build_error = Some(self.build_logs.join("\n"));

                                fatal!("Failed to ready script manager interface because {}", e);
                                self.signal = Signal::None;
                                self.show_build_window = false;
                                self.show_build_error_window = true;
                                self.editor_state = EditorState::Editing;
                            }
                        }
                    }
                }

                if self.show_build_error_window {
                    if let Some(error_log) = &self.last_build_error {
                        let mut window_open = true;
                        let mut close_clicked = false;

                        egui::Window::new("Build Error")
                            .collapsible(true)
                            .resizable(false)
                            .fixed_size([700.0, 500.0])
                            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                            .open(&mut window_open)
                            .show(&graphics.get_egui_context(), |ui| {
                                ui.vertical(|ui| {
                                    ui.heading("Build Failed");
                                    ui.add_space(5.0);
                                    ui.label("The Gradle build failed. See the error log below:");
                                    ui.add_space(10.0);
                                    ui.separator();
                                    ui.add_space(10.0);

                                    egui::ScrollArea::both()
                                        .auto_shrink([false, false])
                                        .max_height(300.0)
                                        .show(ui, |ui| {
                                            ui.add(
                                                egui::TextEdit::multiline(&mut error_log.as_str())
                                                    .font(egui::TextStyle::Monospace)
                                                    .desired_width(f32::INFINITY)
                                                    .desired_rows(20),
                                            );
                                        });

                                    ui.add_space(10.0);

                                    if ui.button("Close").clicked() {
                                        close_clicked = true;
                                    }
                                });
                            });

                        if !window_open || close_clicked {
                            self.show_build_error_window = false;
                        }
                    } else {
                        self.show_build_error_window = false;
                    }
                }
                Ok(())
            }
            Signal::StopPlaying => {
                self.editor_state = EditorState::Editing;
                
                if let Some(pid) = self.play_mode_pid {
                    log::debug!("Play mode requested to be exited, killing processes [{}]", pid);
                    let _ = crate::process::kill_process(pid);
                }
                
                self.play_mode_pid = None;
                self.play_mode_exit_rx = None;
                
                success!("Exited play mode");
                log::info!("Back to the editor you go...");

                self.signal = Signal::None;
                Ok(())
            }
            Signal::LogEntities => {
                log::debug!("====================");
                log::info!("world total items: {}", self.world.len());
                log::info!("typeid of Label: {:?}", TypeId::of::<Label>());
                log::info!("typeid of MeshRenderer: {:?}", TypeId::of::<MeshRenderer>());
                log::info!("typeid of Transform: {:?}", TypeId::of::<Transform>());
                log::info!(
                    "typeid of ModelProperties: {:?}",
                    TypeId::of::<CustomProperties>()
                );
                log::info!("typeid of Camera: {:?}", TypeId::of::<Camera>());
                log::info!(
                    "typeid of CameraComponent: {:?}",
                    TypeId::of::<CameraComponent>()
                );
                log::info!("typeid of Script: {:?}", TypeId::of::<Script>());
                log::info!("typeid of EngineLight: {:?}", TypeId::of::<EngineLight>());
                log::info!(
                    "typeid of LightComponent: {:?}",
                    TypeId::of::<LightComponent>()
                );
                for i in self.world.iter() {
                    log::info!("entity id: {:?}", i.entity().id());
                    log::info!("entity bytes: {:?}", i.entity().to_bits().get());
                    log::info!(
                        "components [{}]: ",
                        i.component_types().collect::<Vec<_>>().len()
                    );
                    let mut comp_builder = String::new();
                    for j in i.component_types() {
                        comp_builder.push_str(format!("{:?} ", j).as_str());
                        if TypeId::of::<Label>() == j {
                            log::info!(" |- Label");
                        }

                        if TypeId::of::<MeshRenderer>() == j {
                            log::info!(" |- MeshRenderer");
                        }

                        if TypeId::of::<Transform>() == j {
                            log::info!(" |- Transform");
                        }

                        if TypeId::of::<CustomProperties>() == j {
                            log::info!(" |- ModelProperties");
                        }

                        if TypeId::of::<Camera>() == j {
                            log::info!(" |- Camera");
                        }

                        if TypeId::of::<CameraComponent>() == j {
                            log::info!(" |- CameraComponent");
                        }

                        if TypeId::of::<Script>() == j {
                            log::info!(" |- Script");
                        }

                        if TypeId::of::<EngineLight>() == j {
                            log::info!(" |- Light");
                        }

                        if TypeId::of::<LightComponent>() == j {
                            log::info!(" |- LightComponent");
                        }
                        log::info!("----------")
                    }
                    log::info!("components (typeid) [{}]: ", comp_builder);
                }
                self.signal = Signal::None;
                Ok(())
            }
            Signal::AddComponent(entity, component_name) => {
                if component_name == "MeshRenderer" {
                    let unassigned_id = (*entity).to_bits().get();
                    let reference = ResourceReference::from_reference(
                        ResourceReferenceType::Unassigned { id: unassigned_id },
                    );

                    let model = std::sync::Arc::new(Model {
                        label: "None".to_string(),
                        path: reference,
                        meshes: Vec::new(),
                        materials: Vec::new(),
                        id: ModelId(unassigned_id),
                    });

                    let loaded_model = LoadedModel::new_raw(
                        &dropbear_engine::asset::ASSET_REGISTRY,
                        model,
                    );

                    let renderer = dropbear_engine::entity::MeshRenderer::from_handle(loaded_model);
                    let _ = self.world.insert_one(*entity, renderer);
                    success!("Added MeshRenderer (unassigned) for entity {:?}", entity);
                } else if component_name == "CameraComponent" {
                    let graphics_clone = graphics.clone();
                    let future = async move {
                        let camera = Camera::predetermined(graphics_clone, Some("New Camera"));
                        let component = CameraComponent::new();
                        Ok::<(Camera, CameraComponent), anyhow::Error>((camera, component))
                    };
                    let handle = graphics.future_queue.push(Box::pin(future));
                    self.pending_components.push((*entity, handle));
                    success!("Queued Camera addition for entity {:?}", entity);
                } else if component_name == "Light" {
                    let graphics_clone = graphics.clone();
                    let future = async move {
                        let light_comp = LightComponent::default();
                        let transform = Transform::default();
                        let engine_light = EngineLight::new(
                            graphics_clone,
                            light_comp.clone(),
                            transform,
                            Some("New Light"),
                        )
                        .await;

                        let light_config = Light {
                            label: "New Light".to_string(),
                            transform,
                            light_component: light_comp.clone(),
                            enabled: true,
                            entity_id: None,
                        };

                        Ok::<(LightComponent, EngineLight, Light, Transform), anyhow::Error>((
                            light_comp,
                            engine_light,
                            light_config,
                            transform,
                        ))
                    };
                    let handle = graphics.future_queue.push(Box::pin(future));
                    self.pending_components.push((*entity, handle));
                    success!("Queued Light addition for entity {:?}", entity);
                } else {
                    warn!(
                        "Unknown component type for AddComponent signal: {}",
                        component_name
                    );
                }
                self.signal = Signal::None;
                Ok(())
            }

            Signal::ClearModel(entity) => {
                let unassigned_id = (*entity).to_bits().get();
                let reference = ResourceReference::from_reference(
                    ResourceReferenceType::Unassigned { id: unassigned_id },
                );

                let model = std::sync::Arc::new(Model {
                    label: "None".to_string(),
                    path: reference,
                    meshes: Vec::new(),
                    materials: Vec::new(),
                    id: ModelId(unassigned_id),
                });
                let loaded_model = LoadedModel::new_raw(&dropbear_engine::asset::ASSET_REGISTRY, model);

                if let Ok(mut renderer) = self.world.get::<&mut MeshRenderer>(*entity) {
                    renderer.set_handle(loaded_model);
                } else {
                    let renderer = MeshRenderer::from_handle(loaded_model);
                    let _ = self.world.insert_one(*entity, renderer);
                }

                self.signal = Signal::None;
                Ok(())
            }
            Signal::ReplaceModel(entity, uri) => {
                let graphics_clone = graphics.clone();
                let uri_clone = uri.clone();
                let future = async move {
                    let mut loaded_model = if is_legacy_internal_cube_uri(&uri_clone) {
                        let size = glam::DVec3::new(1.0, 1.0, 1.0);
                        let size_bits = [1.0f32.to_bits(), 1.0f32.to_bits(), 1.0f32.to_bits()];
                        let mut loaded = ProcedurallyGeneratedObject::cuboid(size)
                            .build_model(graphics_clone.clone(), None, Some("Cuboid"));

                        let model = loaded.make_mut();
                        model.path = ResourceReference::from_reference(ResourceReferenceType::ProcObj(ProcObj::Cuboid { size_bits }));
                        loaded.refresh_registry();
                        loaded
                    } else {
                        let path = resolve_editor_path(&uri_clone);
                        Model::load(graphics_clone.clone(), &path, Some(&uri_clone), None).await?
                    };

                    // Ensure imports start as pure white; users can tint later.
                    {
                        let model = loaded_model.make_mut();
                        for material in &mut model.materials {
                            material.set_tint(graphics_clone.as_ref(), [1.0, 1.0, 1.0, 1.0]);
                        }
                    }

                    loaded_model.refresh_registry();
                    Ok::<LoadedModel, anyhow::Error>(loaded_model)
                };

                let handle = graphics.future_queue.push(Box::pin(future));
                self.pending_model_swaps.push((*entity, handle));
                success!("Queued model swap for entity {:?} from '{}'", entity, uri);
                self.signal = Signal::None;
                Ok(())
            }

            Signal::LoadModel(entity, uri) => {
                let graphics_clone = graphics.clone();
                let uri_clone = uri.clone();
                let future = async move {
                    if is_legacy_internal_cube_uri(&uri_clone) {
                        let size = glam::DVec3::new(1.0, 1.0, 1.0);
                        let size_bits = [1.0f32.to_bits(), 1.0f32.to_bits(), 1.0f32.to_bits()];
                        let mut loaded_model = ProcedurallyGeneratedObject::cuboid(size)
                            .build_model(graphics_clone.clone(), None, Some("Cuboid"));

                        {
                            let model = loaded_model.make_mut();
                            model.path = ResourceReference::from_reference(ResourceReferenceType::ProcObj(ProcObj::Cuboid { size_bits }));
                            for material in &mut model.materials {
                                material.set_tint(graphics_clone.as_ref(), [1.0, 1.0, 1.0, 1.0]);
                            }
                        }
                        loaded_model.refresh_registry();

                        Ok::<MeshRenderer, anyhow::Error>(MeshRenderer::from_handle(loaded_model))
                    } else {
                        let path = resolve_editor_path(&uri_clone);
                        let mut model = dropbear_engine::model::Model::load(
                            graphics_clone.clone(),
                            &path,
                            Some(&uri_clone),
                            None,
                        )
                        .await?;

                        {
                            let model_mut = model.make_mut();
                            for material in &mut model_mut.materials {
                                material.set_tint(graphics_clone.as_ref(), [1.0, 1.0, 1.0, 1.0]);
                            }
                        }

                        model.refresh_registry();

                        Ok::<MeshRenderer, anyhow::Error>(
                            dropbear_engine::entity::MeshRenderer::from_handle(model),
                        )
                    }
                };

                let handle = graphics.future_queue.push(Box::pin(future));
                self.pending_components.push((*entity, handle));
                success!("Queued model load for entity {:?} from '{}'", entity, uri);
                self.signal = Signal::None;
                Ok(())
            }

            Signal::SetProceduralCuboid(entity, size) | Signal::UpdateProceduralCuboid(entity, size) => {
                let previous_customisation: Option<
                    Vec<(String, [f32; 4], Option<String>, TextureWrapMode, [f32; 2])>,
                > =
                    self.world
                        .get::<&MeshRenderer>(*entity)
                        .ok()
                        .map(|renderer| {
                            renderer
                                .model()
                                .materials
                                .iter()
                                .map(|mat| {
                                    (
                                        mat.name.clone(),
                                        mat.tint,
                                        mat.texture_tag.clone(),
                                        mat.wrap_mode,
                                        mat.uv_tiling,
                                    )
                                })
                                .collect()
                        });

                let label = self
                    .world
                    .get::<&eucalyptus_core::states::Label>(*entity)
                    .map(|l| l.to_string())
                    .unwrap_or_else(|_| "Cuboid".to_string());

                {
                    let mut cache_guard = MODEL_CACHE.lock();
                    cache_guard.remove(&label);
                }

                let size_bits = [size[0].to_bits(), size[1].to_bits(), size[2].to_bits()];
                let size_vec = glam::DVec3::new(size[0] as f64, size[1] as f64, size[2] as f64);

                let mut loaded_model = ProcedurallyGeneratedObject::cuboid(size_vec)
                    .build_model(graphics.clone(), None, Some(&label));

                {
                    let model = loaded_model.make_mut();
                    model.path = ResourceReference::from_reference(ResourceReferenceType::ProcObj(ProcObj::Cuboid { size_bits }));
                }
                loaded_model.refresh_registry();

                if let Ok(mut renderer) = self.world.get::<&mut MeshRenderer>(*entity) {
                    renderer.set_handle(loaded_model);

                    if let Some(previous) = previous_customisation {
                        let model = renderer.make_model_mut();
                        for (mat_name, tint, texture_tag, wrap_mode, uv_tiling) in previous {
                            if let Some(material) = model.materials.iter_mut().find(|m| m.name == mat_name) {
                                material.wrap_mode = wrap_mode;
                                material.set_tint(graphics.as_ref(), tint);
                                material.set_uv_tiling(graphics.as_ref(), uv_tiling);

                                if let Some(uri) = texture_tag {
                                    if uri.to_ascii_lowercase().ends_with(".png")
                                        || uri.to_ascii_lowercase().ends_with(".jpg")
                                        || uri.to_ascii_lowercase().ends_with(".jpeg")
                                        || uri.to_ascii_lowercase().ends_with(".tga")
                                        || uri.to_ascii_lowercase().ends_with(".bmp")
                                    {
                                        let path = resolve_editor_path(&uri);

                                        if let Ok(bytes) = std::fs::read(&path) {
                                            let diffuse = Texture::from_bytes_verbose_mipmapped(
                                                graphics.clone(),
                                                &bytes,
                                                None,
                                                None,
                                                Some(Texture::sampler_from_wrap(wrap_mode)),
                                                Some(mat_name.as_str())
                                            );
                                            let flat_normal = (*dropbear_engine::asset::ASSET_REGISTRY
                                                .solid_texture_rgba8(
                                                    graphics.clone(),
                                                    [128, 128, 255, 255],
                                                ))
                                            .clone();

                                            material.diffuse_texture = diffuse;
                                            material.normal_texture = flat_normal;
                                            material.bind_group = dropbear_engine::model::Material::create_bind_group(
                                                graphics.as_ref(),
                                                &material.diffuse_texture,
                                                &material.normal_texture,
                                                &material.name,
                                            );
                                            material.texture_tag = Some(uri);
                                        }
                                    } else {
                                        material.texture_tag = Some(uri);
                                    }
                                }
                            }
                        }

                        renderer.sync_asset_registry();
                    }
                } else {
                    let renderer = MeshRenderer::from_handle(loaded_model);
                    let _ = self.world.insert_one(*entity, renderer);
                }

                self.signal = Signal::None;
                Ok(())
            }

            Signal::SetMaterialTexture(entity, target_material, uri, wrap_mode) => {
                let path = resolve_editor_path(uri);

                let bytes = match std::fs::read(&path) {
                    Ok(bytes) => bytes,
                    Err(err) => {
                        warn!("Failed to read texture '{}': {}", path.display(), err);
                        self.signal = Signal::None;
                        return Ok(());
                    }
                };

                let diffuse = Texture::from_bytes_verbose_mipmapped(
                    graphics.clone(),
                    &bytes,
                    None,
                    None,
                    Some(Texture::sampler_from_wrap(wrap_mode.clone())),
                    Some(target_material)
                );
                let flat_normal = (*dropbear_engine::asset::ASSET_REGISTRY
                    .solid_texture_rgba8(graphics.clone(), [128, 128, 255, 255]))
                    .clone();

                if let Ok(mut renderer) = self.world.get::<&mut MeshRenderer>(*entity) {
                    let model = renderer.make_model_mut();
                    if let Some(material) = model
                        .materials
                        .iter_mut()
                        .find(|mat| mat.name == *target_material)
                    {
                        material.diffuse_texture = diffuse;
                        material.normal_texture = flat_normal;
                        material.bind_group = Material::create_bind_group(
                            graphics.as_ref(),
                            &material.diffuse_texture,
                            &material.normal_texture,
                            &material.name,
                        );
                        material.texture_tag = Some(uri.clone());
                        material.wrap_mode = *wrap_mode;
                    } else {
                        warn!("Material '{}' not found on renderer", target_material);
                    }

                    renderer.sync_asset_registry();
                }

                self.signal = Signal::None;
                Ok(())
            }

            Signal::SetMaterialWrapMode(entity, target_material, wrap_mode) => {
                if let Ok(mut renderer) = self.world.get::<&mut MeshRenderer>(*entity) {
                    let model = renderer.make_model_mut();
                    if let Some(material) = model
                        .materials
                        .iter_mut()
                        .find(|mat| mat.name == *target_material)
                    {
                        material.wrap_mode = *wrap_mode;

                        if let Some(uri) = material.texture_tag.clone() {
                            let path = resolve_editor_path(&uri);

                            if let Ok(bytes) = std::fs::read(&path) {
                                let diffuse = Texture::from_bytes_verbose_mipmapped(
                                    graphics.clone(),
                                    &bytes,
                                    None,
                                    None,
                                    Some(Texture::sampler_from_wrap(wrap_mode.clone())),
                                    Some(target_material)
                                );
                                material.diffuse_texture = diffuse;
                                material.bind_group = Material::create_bind_group(
                                    graphics.as_ref(),
                                    &material.diffuse_texture,
                                    &material.normal_texture,
                                    &material.name,
                                );
                            } else {
                                warn!(
                                    "Failed to read texture '{}' to apply wrap mode",
                                    path.display()
                                );
                            }
                        }
                    } else {
                        warn!("Material '{}' not found on renderer", target_material);
                    }

                    renderer.sync_asset_registry();
                }

                self.signal = Signal::None;
                Ok(())
            }

            Signal::SetMaterialUvTiling(entity, target_material, uv_tiling) => {
                if let Ok(mut renderer) = self.world.get::<&mut MeshRenderer>(*entity) {
                    let model = renderer.make_model_mut();
                    if let Some(material) = model
                        .materials
                        .iter_mut()
                        .find(|mat| mat.name == *target_material)
                    {
                        material.set_uv_tiling(graphics.as_ref(), *uv_tiling);
                    } else {
                        warn!("Material '{}' not found on renderer", target_material);
                    }

                    renderer.sync_asset_registry();
                }

                self.signal = Signal::None;
                Ok(())
            }

            Signal::ClearMaterialTexture(entity, target_material) => {
                let diffuse = (*dropbear_engine::asset::ASSET_REGISTRY.grey_texture(graphics.clone())).clone();
                let flat_normal = (*dropbear_engine::asset::ASSET_REGISTRY
                    .solid_texture_rgba8(graphics.clone(), [128, 128, 255, 255]))
                    .clone();

                if let Ok(mut renderer) = self.world.get::<&mut MeshRenderer>(*entity) {
                    let model = renderer.make_model_mut();
                    if let Some(material) = model
                        .materials
                        .iter_mut()
                        .find(|mat| mat.name == *target_material)
                    {
                        material.diffuse_texture = diffuse;
                        material.normal_texture = flat_normal;
                        material.bind_group = Material::create_bind_group(
                            graphics.as_ref(),
                            &material.diffuse_texture,
                            &material.normal_texture,
                            &material.name,
                        );
                        material.texture_tag = None;
                    } else {
                        warn!("Material '{}' not found on renderer", target_material);
                    }

                    renderer.sync_asset_registry();
                }

                self.signal = Signal::None;
                Ok(())
            }

            Signal::SetMaterialTint(entity, target_material, tint) => {
                if let Ok(mut renderer) = self.world.get::<&mut MeshRenderer>(*entity) {
                    let model = renderer.make_model_mut();
                    if let Some(material) = model
                        .materials
                        .iter_mut()
                        .find(|mat| mat.name == *target_material)
                    {
                        material.set_tint(graphics.as_ref(), *tint);
                    } else {
                        warn!("Material '{}' not found on renderer", target_material);
                    }
                }

                self.signal = Signal::None;
                Ok(())
            }
            Signal::RequestNewWindow(window_data) => {
                use dropbear_engine::scene::SceneCommand;
                self.scene_command = SceneCommand::RequestWindow(window_data.clone());
                self.signal = Signal::None;
                Ok(())
            }
        }?;
        if !show {
            self.signal = Signal::None;
        }
        if let Some(signal) = local_signal {
            self.signal = signal;
        }
        Ok(())
    }
}
