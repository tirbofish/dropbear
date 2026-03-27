use crate::editor::{AssetClipboard, Editor, EditorState, Signal};
use crate::spawn::{PendingSpawn, push_pending_spawn};
use dropbear_engine::asset::ASSET_REGISTRY;
use dropbear_engine::entity::MeshRenderer;
use dropbear_engine::graphics::SharedGraphicsContext;
use egui::Align2;
use eucalyptus_core::camera::{CameraComponent, CameraType};
use eucalyptus_core::scene::SceneEntity;
use eucalyptus_core::scripting::{BuildStatus, build_jvm};
use eucalyptus_core::scripting::types::KotlinComponents;
use eucalyptus_core::states::{Label, PROJECT};
use eucalyptus_core::{fatal, info, success, success_without_console, warn, warn_without_console};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use winit::keyboard::KeyCode;

pub trait SignalController {
    fn run_signal(&mut self, graphics: Arc<SharedGraphicsContext>) -> anyhow::Result<()>;
}

impl SignalController for Editor {
    fn run_signal(&mut self, graphics: Arc<SharedGraphicsContext>) -> anyhow::Result<()> {
        let mut requeue = vec![];
        while let Some(signal) = self.signal.pop_front() {
            let local_signal: Option<Signal> = None;
            let show = true;

            match signal {
                Signal::None => {
                    // returns absolutely nothing because no signal is set.
                    Ok::<(), anyhow::Error>(())
                }
                Signal::Copy(entities, parent_map) => {
                    requeue.push(Signal::Copy(entities, parent_map));
                    Ok(())
                },
                Signal::AssetCopy { source, division } => {
                    self.asset_clipboard = Some(AssetClipboard {
                        source: source.clone(),
                        division: division,
                    });
                    
                    Ok(())
                }
                Signal::AssetPaste {
                    target_dir,
                    division,
                } => {
                    let clipboard = self.asset_clipboard.clone();
                    if clipboard.is_none() {
                        warn!("Nothing copied to paste");
                        
                        return Ok(());
                    }

                    let clipboard = clipboard.unwrap();
                    if clipboard.division != division {
                        warn!("Cannot paste across different asset divisions");
                        
                        return Ok(());
                    }

                    if !clipboard.source.is_file() {
                        warn!("Copied asset is not a file");
                        
                        return Ok(());
                    }

                    if !target_dir.exists() {
                        warn!("Target directory does not exist");
                        
                        return Ok(());
                    }

                    let Some(file_name) = clipboard.source.file_name() else {
                        warn!("Unable to paste: invalid file name");
                        
                        return Ok(());
                    };

                    let target_path = target_dir.join(file_name);
                    if target_path.exists() {
                        warn!("Target already exists: {}", target_path.display());
                        
                        return Ok(());
                    }

                    if let Err(err) = fs::copy(&clipboard.source, &target_path) {
                        warn!("Unable to paste file: {}", err);
                        
                        return Ok(());
                    }

                    info!("Pasted asset to {}", target_path.display());
                    
                    Ok(())
                }
                Signal::Paste(entities, parent_map) => {
                    log::debug!("Paste requested for {} entity(ies)", entities.len());

                    // Rename all entities to ensure uniqueness, tracking old→new label mapping.
                    let mut label_remap: HashMap<Label, Label> = HashMap::new();
                    let mut renamed: Vec<SceneEntity> = Vec::new();
                    for mut scene_entity in entities {
                        let new_label = Editor::unique_label_for_world(
                            self.world.as_ref(),
                            scene_entity.label.as_str(),
                        );
                        label_remap.insert(scene_entity.label.clone(), new_label.clone());
                        scene_entity.label = new_label;
                        renamed.push(scene_entity);
                    }

                    // Rebuild parent map with renamed labels.
                    let renamed_parent_map: HashMap<Label, Label> = parent_map
                        .into_iter()
                        .filter_map(|(child, parent)| {
                            let new_child = label_remap.get(&child)?.clone();
                            let new_parent = label_remap.get(&parent)?.clone();
                            Some((new_child, new_parent))
                        })
                        .collect();

                    // Keep clipboard alive so the user can paste again.
                    self.signal.push_back(Signal::Copy(renamed.clone(), renamed_parent_map.clone()));

                    // Queue a PendingSpawn for each entity, with parent_label set as needed.
                    for scene_entity in renamed {
                        let parent_label = renamed_parent_map.get(&scene_entity.label).cloned();
                        push_pending_spawn(PendingSpawn {
                            scene_entity,
                            handle: None,
                            parent_label,
                        });
                    }
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
                            
                            Ok(())
                        } else {
                            match self.world.despawn(*sel_e) {
                                Ok(_) => {
                                    info!("Decimated entity");
                                    
                                    Ok(())
                                }
                                Err(e) => {
                                    
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
                    
                    Ok(())
                }
                Signal::Play => {
                    if matches!(self.editor_state, EditorState::Playing) {
                        log::warn!("Unable to play: already in playing mode");
                        
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
                                            
                                            self.show_build_window = false;
                                            self.editor_state = EditorState::Editing;
                                        }
                                    }
                                    BuildStatus::Failed(_e) => {
                                        let error_msg = format!("Build failed, check logs");
                                        self.build_logs.push(error_msg.clone());

                                        self.build_progress = 0.0;
                                        fatal!("Failed to build gradle, check logs");

                                        
                                        self.show_build_window = false;
                                        self.editor_state = EditorState::Editing;
                                        // self.dock_state
                                        //     .push_to_focused_leaf(EditorTab::ErrorConsole); // getting too problematic
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

                    if matches!(self.editor_state, EditorState::Building) || self.show_build_error_window {
                        requeue.push(Signal::Play);
                    }
                    Ok(())
                }
                Signal::StopPlaying => {
                    self.editor_state = EditorState::Editing;

                    if let Some(pid) = self.play_mode_pid {
                        log::debug!(
                            "Play mode requested to be exited, killing processes [{}]",
                            pid
                        );
                        let _ = crate::process::kill_process(pid);
                    }

                    self.play_mode_pid = None;
                    self.play_mode_exit_rx = None;

                    success!("Exited play mode");
                    log::info!("Back to the editor you go...");

                    
                    Ok(())
                }
                Signal::AddComponent(entity, component) => {
                    if let Some(kc_box) = component.as_any().downcast_ref::<KotlinComponents>() {
                        for fqcn in kc_box.fqcns.clone() {
                            if let Ok(mut existing) = self.world.get::<&mut KotlinComponents>(entity) {
                                if existing.has(&fqcn) {
                                    warn!("Entity {:?} already has Kotlin component '{}'", entity, fqcn);
                                } else {
                                    existing.attach(&fqcn);
                                    success!("Added Kotlin component '{}' to entity {:?}", fqcn, entity);
                                }
                            } else {
                                let mut new_kc = KotlinComponents::default();
                                new_kc.attach(&fqcn);
                                if let Err(e) = self.world.insert_one(entity, new_kc) {
                                    warn!("Failed to insert KotlinComponents for '{}': {}", fqcn, e);
                                } else {
                                    success!("Added Kotlin component '{}' to entity {:?}", fqcn, entity);
                                }
                            }
                        }
                        return Ok(());
                    }

                    let registry = self.component_registry.clone();

                    let Some(component_id) = registry.id_for_component(component.as_ref()) else {
                        warn!(
                            "Failed to resolve component type for add request on entity {:?}",
                            entity
                        );
                        
                        return Ok(());
                    };

                    if registry
                        .find_entities_by_numeric_id(&self.world, component_id)
                        .contains(&entity)
                    {
                        let component_name = registry
                            .get_descriptor_by_numeric_id(component_id)
                            .map(|desc| desc.type_name.as_str())
                            .unwrap_or("Unknown");

                        warn!(
                            "Entity {:?} already has component '{}'",
                            entity,
                            component_name
                        );
                        
                        return Ok(());
                    }

                    let graphics_clone = graphics.clone();
                    let init_future = async move {
                        let Some(loader_future) =
                            registry.load_component(component.as_ref(), graphics_clone.clone())
                        else {
                            return Err(anyhow::anyhow!(
                                "Component type is not registered in ComponentRegistry"
                            ));
                        };

                        loader_future.await
                    };
                    let handle = graphics.future_queue.push(init_future);
                    self.pending_components.push((entity, handle));

                    success!("Queued component addition for entity {:?}", entity);
                    
                    Ok(())
                }
                Signal::RequestNewWindow(window_data) => {
                    use dropbear_engine::scene::SceneCommand;
                    self.scene_command = SceneCommand::RequestWindow(window_data.clone());
                    
                    Ok(())
                }
                Signal::UpdateViewportSize((x, y)) => {
                    let width = x.max(1.0).round() as u32;
                    let height = y.max(1.0).round() as u32;
                    let current_size = graphics.viewport_texture.size;

                    if current_size.width != width || current_size.height != height {
                        self.scene_command =
                            dropbear_engine::scene::SceneCommand::ResizeViewport((width, height));
                    }
                    
                    Ok(())
                }
                Signal::FlushUnusedAssets => {
                    let live_model_ids: HashSet<u64> = self
                        .world
                        .query::<&MeshRenderer>()
                        .iter()
                        .map(|mr| mr.model().id)
                        .collect();

                    let mut asset = ASSET_REGISTRY.write();
                    let count = asset.flush_unused_with_live_ids(&live_model_ids);
                    success!("Flushed {} unused assets", count);

                    Ok(())
                }
                Signal::ReloadWGPUData { skybox_texture } => {
                    self.main_render_pipeline = None;
                    self.light_cube_pipeline = None;
                    self.shader_globals = None;
                    self.collider_wireframe_pipeline = None;
                    self.mipmapper = None;
                    self.texture_id = None;
                    self.window = None;
                    self.sky_pipeline = None;
                    self.load_wgpu_nerdy_stuff(graphics.clone(), skybox_texture.as_ref());

                    Ok(())
                }
            }?;
            if !show {
                
            }
            if let Some(signal) = local_signal {
                self.signal.push_back(signal);
            }
        }

        for s in requeue {
            self.signal.push_front(s);
        }

        Ok(())
    }
}
