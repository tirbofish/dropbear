//! The scene for a window that opens up settings related to the project, "Play Mode" runtime and redback-runtime.  

use egui::{CentralPanel, Color32, Id, RichText, Slider, SliderClamping};
use egui_ltreeview::{Action, NodeBuilder};
use gilrs::{Button, GamepadId};
use semver::Version;
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::{WindowId};
use dropbear_engine::graphics::RenderContext;
use dropbear_engine::input::{Controller, Keyboard, Mouse};
use dropbear_engine::scene::{Scene, SceneCommand};
use eucalyptus_core::input::InputState;
use eucalyptus_core::states::PROJECT;
use eucalyptus_core::warn;

#[derive(Default)]
pub enum ProjectSettingsLeaf {
    #[default]
    None,

    Versioning,
    Authoring,
    Runtime,
}

pub struct ProjectSettingsWindow {
    scene_command: SceneCommand,
    input_state: InputState,
    window: Option<WindowId>,
    current_leaf: ProjectSettingsLeaf,
}

impl ProjectSettingsWindow {
    pub fn new() -> Self {
        Self {
            scene_command: SceneCommand::None,
            input_state: Default::default(),
            window: None,
            current_leaf: Default::default(),
        }
    }
}

impl Scene for ProjectSettingsWindow {
    fn load(&mut self, graphics: &mut RenderContext) {
        self.window = Some(graphics.shared.window.id());
    }

    fn physics_update(&mut self, _dt: f32, _graphics: &mut RenderContext) {}

    fn update(&mut self, _dt: f32, graphics: &mut RenderContext) {
        CentralPanel::default().show(&graphics.shared.get_egui_context(), |ui| {
            let mut project = PROJECT.write();

            egui::SidePanel::left("project_settings_tree_panel")
                .resizable(true)
                .default_width(200.0)
                .width_range(150.0..=400.0)
                .show_inside(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            let (_resp, action) = egui_ltreeview::TreeView::new(Id::from("project_settings"))
                                .show(ui, |builder| {
                                    builder.node(
                                        NodeBuilder::dir("Publishing")
                                            .label("Publishing")
                                    );

                                    {
                                        builder.node(
                                            NodeBuilder::leaf("Versioning")
                                                .label("Versioning")
                                        );
                                        builder.node(
                                            NodeBuilder::leaf("Authoring")
                                                .label("Authoring")
                                        );
                                    }

                                    builder.close_dir();

                                    builder.node(
                                        NodeBuilder::leaf("Runtime")
                                            .label("Runtime Settings")
                                    );
                                });

                            for a in action {
                                match a {
                                    Action::SetSelected(selected) => {
                                        let selected = selected.first().cloned();
                                        if let Some(s) = selected {
                                            match s {
                                                "Versioning" => {
                                                    self.current_leaf = ProjectSettingsLeaf::Versioning;
                                                }
                                                "Authoring" => {
                                                    self.current_leaf = ProjectSettingsLeaf::Authoring;
                                                }
                                                "Runtime" => {
                                                    self.current_leaf = ProjectSettingsLeaf::Runtime;
                                                }
                                                _ => {
                                                    self.current_leaf = ProjectSettingsLeaf::None;
                                                },
                                            }
                                        }
                                    }
                                    Action::Move(_) => {}
                                    Action::Drag(_) => {}
                                    Action::Activate(_) => {}
                                    Action::DragExternal(_) => {}
                                    Action::MoveExternal(_) => {}
                                }
                            }
                        });
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        match self.current_leaf {
                            ProjectSettingsLeaf::Versioning => {
                                ui.heading("Versioning Settings");
                                ui.separator();

                                let version = &mut project.project_version;

                                let old = version.clone();
                                ui.label("Version (in semantic form)");
                                let resp = ui.text_edit_singleline(version);

                                if resp.lost_focus() {
                                    if let Err(e) = Version::parse(version.as_str()) {
                                        ui.label(RichText::new(format!("Semver validation for text [{}] failed: {}", version, e)).color(Color32::from_rgb(255, 0, 0)));
                                        *version = old;
                                    } else {
                                        log::debug!("Semver parsing was fine: {}", version);
                                    }
                                }
                            }
                            ProjectSettingsLeaf::Authoring => {
                                ui.heading("Authoring Settings");
                                ui.separator();

                                ui.label("Authors");
                                ui.text_edit_singleline(&mut project.authors.developer);
                            }
                            ProjectSettingsLeaf::Runtime => {
                                ui.heading("Runtime Settings");
                                ui.separator();

                                ui.label("Target FPS:");
                                ui.horizontal(|ui| {
                                    let mut local_set_max_fps = project.runtime_settings.target_fps.is_some();

                                    if ui.checkbox(&mut local_set_max_fps, "Set max frames-per-second (FPS)").changed() {
                                        if local_set_max_fps {
                                            project.runtime_settings.target_fps.enable_or(120); 
                                        } else {
                                            project.runtime_settings.target_fps.disable(); 
                                        }
                                    }

                                    if let Some(v) = project.runtime_settings.target_fps.get_mut() {
                                        ui.add(
                                            Slider::new(v, 1..=1000)
                                            .clamping(SliderClamping::Never)
                                        );
                                    }
                                });
                            }
                            _ => {}
                        }
                    });
            });
        });

        self.window = Some(graphics.shared.window.id());
    }

    fn render(&mut self, _graphics: &mut RenderContext) {
    }

    fn exit(&mut self, _event_loop: &ActiveEventLoop) {
        {
            if let Err(e) = PROJECT.write().write_project_only() {
                warn!("Failed to write project: {:?}", e);
            }
        }
    }

    fn run_command(&mut self) -> SceneCommand {
        std::mem::replace(&mut self.scene_command, SceneCommand::None)
    }
}

impl Keyboard for ProjectSettingsWindow {
    fn key_down(&mut self, key: KeyCode, _event_loop: &ActiveEventLoop) {
        match key {
            KeyCode::Escape => {
                if let Some(id) = self.window {
                    self.scene_command = SceneCommand::CloseWindow(id);
                }
            }
            _ => {
                self.input_state.pressed_keys.insert(key);
            }
        }
        self.input_state.pressed_keys.insert(key);
    }

    fn key_up(&mut self, key: KeyCode, _event_loop: &ActiveEventLoop) {
        self.input_state.pressed_keys.remove(&key);
    }
}

impl Mouse for ProjectSettingsWindow {
    fn mouse_move(&mut self, position: PhysicalPosition<f64>, delta: Option<(f64, f64)>) {
        self.input_state.last_mouse_pos = Some(<(f64, f64)>::from(position));
        self.input_state.mouse_delta = delta;
        self.input_state.mouse_pos = (position.x, position.y);
    }

    fn mouse_down(&mut self, button: MouseButton) {
        self.input_state.mouse_button.insert(button);
    }

    fn mouse_up(&mut self, button: MouseButton) {
        self.input_state.mouse_button.remove(&button);
    }
}

impl Controller for ProjectSettingsWindow {
    fn button_down(&mut self, button: Button, id: GamepadId) {
        self.input_state
            .pressed_buttons
            .entry(id)
            .or_default()
            .insert(button);
    }

    fn button_up(&mut self, button: Button, id: GamepadId) {
        if let Some(buttons) = self.input_state.pressed_buttons.get_mut(&id) {
            buttons.remove(&button);
        }
    }

    fn left_stick_changed(&mut self, x: f32, y: f32, id: GamepadId) {
        self.input_state.left_stick_position.insert(id, (x, y));
    }

    fn right_stick_changed(&mut self, x: f32, y: f32, id: GamepadId) {
        self.input_state.right_stick_position.insert(id, (x, y));
    }

    fn on_connect(&mut self, id: GamepadId) {
        self.input_state.connected_gamepads.insert(id);
    }

    fn on_disconnect(&mut self, id: GamepadId) {
        self.input_state.connected_gamepads.remove(&id);
        self.input_state.pressed_buttons.remove(&id);
        self.input_state.left_stick_position.remove(&id);
        self.input_state.right_stick_position.remove(&id);
    }
}