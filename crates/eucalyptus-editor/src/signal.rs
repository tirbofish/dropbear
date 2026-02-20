use crate::editor::{AssetClipboard, Editor, EditorState, Signal};
use crate::spawn::{PendingSpawn, push_pending_spawn};
use dropbear_engine::graphics::SharedGraphicsContext;
use egui::Align2;
use eucalyptus_core::camera::{CameraComponent, CameraType};
use eucalyptus_core::scripting::{BuildStatus, build_jvm};
use eucalyptus_core::states::{PROJECT};
use eucalyptus_core::{fatal, info, success, success_without_console, warn, warn_without_console};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use winit::keyboard::KeyCode;

pub trait SignalController {
    fn run_signal(&mut self, graphics: Arc<SharedGraphicsContext>) -> anyhow::Result<()>;
}

impl SignalController for Editor {
    fn run_signal(&mut self, graphics: Arc<SharedGraphicsContext>) -> anyhow::Result<()> {
        let local_signal: Option<Signal> = None;
        let show = true;

        match &self.signal {
            Signal::None => {
                // returns absolutely nothing because no signal is set.
                Ok::<(), anyhow::Error>(())
            }
            Signal::Copy(_) => Ok(()),
            Signal::AssetCopy { source, division } => {
                self.asset_clipboard = Some(AssetClipboard {
                    source: source.clone(),
                    division: *division,
                });
                self.signal = Signal::None;
                Ok(())
            }
            Signal::AssetPaste {
                target_dir,
                division,
            } => {
                let clipboard = self.asset_clipboard.clone();
                if clipboard.is_none() {
                    warn!("Nothing copied to paste");
                    self.signal = Signal::None;
                    return Ok(());
                }

                let clipboard = clipboard.unwrap();
                if clipboard.division != *division {
                    warn!("Cannot paste across different asset divisions");
                    self.signal = Signal::None;
                    return Ok(());
                }

                if !clipboard.source.is_file() {
                    warn!("Copied asset is not a file");
                    self.signal = Signal::None;
                    return Ok(());
                }

                if !target_dir.exists() {
                    warn!("Target directory does not exist");
                    self.signal = Signal::None;
                    return Ok(());
                }

                let Some(file_name) = clipboard.source.file_name() else {
                    warn!("Unable to paste: invalid file name");
                    self.signal = Signal::None;
                    return Ok(());
                };

                let target_path = target_dir.join(file_name);
                if target_path.exists() {
                    warn!("Target already exists: {}", target_path.display());
                    self.signal = Signal::None;
                    return Ok(());
                }

                if let Err(err) = fs::copy(&clipboard.source, &target_path) {
                    warn!("Unable to paste file: {}", err);
                    self.signal = Signal::None;
                    return Ok(());
                }

                info!("Pasted asset to {}", target_path.display());
                self.signal = Signal::None;
                Ok(())
            }
            Signal::Paste(scene_entity) => {
                let mut scene_entity = scene_entity.clone();
                scene_entity.label = Editor::unique_label_for_world(
                    self.world.as_ref(),
                    scene_entity.label.as_str(),
                );
                let spawn = PendingSpawn {
                    scene_entity: scene_entity.clone(),
                    handle: None,
                };
                push_pending_spawn(spawn);
                self.signal = Signal::Copy(scene_entity.clone());
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

                self.signal = Signal::None;
                Ok(())
            }
            Signal::AddComponent(entity, component) => {
                let component = component.clone();
                let registry = self.component_registry.clone();
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
                self.pending_components.push((*entity, handle));

                success!("Queued component addition for entity {:?}", entity);
                self.signal = Signal::None;
                Ok(())
            }
            Signal::RequestNewWindow(window_data) => {
                use dropbear_engine::scene::SceneCommand;
                self.scene_command = SceneCommand::RequestWindow(window_data.clone());
                self.signal = Signal::None;
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
