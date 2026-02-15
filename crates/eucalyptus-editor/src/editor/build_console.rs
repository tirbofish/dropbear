impl<'a> EditorTabViewer<'a> {
    pub fn build_console(&mut self, ui: &mut egui::Ui) {
        fn analyse_error(log: &Vec<String>) -> Vec<ConsoleItem> {
                    fn parse_compiler_location(
                        line: &str,
                    ) -> Option<(ErrorLevel, PathBuf, String)> {
                        let trimmed = line.trim_start();
                        let (error_level, rest) =
                            if let Some(r) = trimmed.strip_prefix("e: file:///") {
                                (ErrorLevel::Error, r)
                            } else if let Some(r) = trimmed.strip_prefix("w: file:///") {
                                (ErrorLevel::Warn, r)
                            } else {
                                return None;
                            };

                        let location = rest.split_whitespace().next()?;

                        let mut segments = location.rsplitn(3, ':');
                        let column = segments.next()?;
                        let row = segments.next()?;
                        let path = segments.next()?;

                        Some((error_level, PathBuf::from(path), format!("{row}:{column}")))
                    }

                    let mut list: Vec<ConsoleItem> = Vec::new();
                    for (index, line) in log.iter().enumerate() {
                        if line.contains("The required library") {
                            list.push(ConsoleItem {
                                error_level: ErrorLevel::Error,
                                msg: line.clone(),
                                file_location: None,
                                line_ref: None,
                                id: index as u64,
                            });
                        } else if let Some((error_level, path, loc)) = parse_compiler_location(line) {
                            list.push(ConsoleItem {
                                error_level,
                                msg: line.clone(),
                                file_location: Some(path),
                                line_ref: Some(loc),
                                id: index as u64,
                            });
                        } else {
                            list.push(ConsoleItem {
                                error_level: ErrorLevel::Info,
                                msg: line.clone(),
                                file_location: None,
                                line_ref: None,
                                id: index as u64,
                            });
                        }
                    }
                    list
                }

                let logs = analyse_error(&self.build_logs);

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        if logs.is_empty() {
                            ui.label("Build output will appear here once available.");
                            return;
                        }

                        for item in &logs {
                            let (bg_color, text_color, stroke_color) = match item.error_level {
                                ErrorLevel::Error => (
                                    egui::Color32::from_rgb(60, 20, 20),
                                    egui::Color32::from_rgb(255, 200, 200),
                                    egui::Color32::from_rgb(255, 200, 200),
                                ),
                                ErrorLevel::Warn => (
                                    egui::Color32::from_rgb(40, 40, 10),
                                    egui::Color32::from_rgb(255, 255, 200),
                                    egui::Color32::from_rgb(255, 255, 200),
                                ),
                                ErrorLevel::Info => (
                                    egui::Color32::TRANSPARENT,
                                    ui.style().visuals.text_color(),
                                    egui::Color32::TRANSPARENT,
                                ),
                            };

                            if matches!(item.error_level, ErrorLevel::Info) {
                                ui.label(RichText::new(&item.msg).monospace());
                            } else {
                                let available_width = ui.available_width();
                                let frame = egui::Frame::new()
                                    .inner_margin(Margin::symmetric(8, 6))
                                    .fill(bg_color)
                                    .stroke(egui::Stroke::new(1.0, stroke_color));

                                let response = frame
                                    .show(ui, |ui| {
                                        ui.set_width(available_width - 10.0);
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(&item.msg).color(text_color).monospace());
                                        });
                                    })
                                    .response;

                                if response.clicked() {
                                    log::debug!("Log item clicked: {}", &item.id);
                                    if let (Some(path), Some(loc)) =
                                        (&item.file_location, &item.line_ref)
                                    {
                                        let location_arg = format!("{}:{}", path.display(), loc);

                                        match std::process::Command::new("code")
                                            .args(["-g", &location_arg])
                                            .spawn()
                                            .map(|_| ())
                                        {
                                            Ok(()) => {
                                                log::info!(
                                                    "Launched Visual Studio Code at the error: {}",
                                                    &location_arg
                                                );
                                            }
                                            Err(e) => {
                                                warn!(
                                                    "Failed to open '{}' in VS Code: {}",
                                                    location_arg, e
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    });
    }
}