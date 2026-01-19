//! The scene for a window that opens up settings related to the eucalyptus-editor.

use app_dirs2::AppDataType;
use egui::{CentralPanel, Id, Slider, SliderClamping};
use egui_dock::DockState;
use egui_ltreeview::{Action, NodeBuilder};
use eucalyptus_core::utils::option::HistoricalOption;
use gilrs::{Button, GamepadId};
use hecs::spin::Lazy;
use parking_lot::RwLock;
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::{WindowId};
use dropbear_engine::input::{Controller, Keyboard, Mouse};
use dropbear_engine::scene::{Scene, SceneCommand};
use eucalyptus_core::input::InputState;
use serde::{Deserialize, Serialize};
use eucalyptus_core::{warn, APP_INFO};
use eucalyptus_core::states::{EditorTab};

pub static EDITOR_SETTINGS: Lazy<RwLock<EditorSettings>> =
    Lazy::new(|| RwLock::new(EditorSettings::new()));

/// Settings related to the eucalyptus-editor.
///
/// This is not related to a project, and is for each user who uses the editor.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditorSettings {
    /// The layout of the dock.
    #[serde(default)]
    pub dock_layout: Option<DockState<EditorTab>>,

    #[serde(default)]
    pub target_fps: HistoricalOption<u32>,

    /// Is the debug menu shown?
    ///
    /// Primarily used internally for testing out features of the editor, however
    /// could be useful for certain people.
    ///
    /// This option will not be shown in the editor settings, and can only be edited by file.
    #[serde(default)]
    pub is_debug_menu_shown: bool,
}

impl EditorSettings {
    /// Creates a new instance of [EditorSettings]
    pub fn new() -> Self {
        Self {
            dock_layout: None,
            is_debug_menu_shown: false,
            target_fps: HistoricalOption::none(),
        }
    }

    /// Saves the current EditorSettings configuration (as shown in [EDITOR_SETTINGS]) into `{app_dir}/editor.eucc`.
    pub fn save(&self) -> anyhow::Result<()> {
        let app_data = app_dirs2::app_root(AppDataType::UserData, &APP_INFO)?;
        let serialized = ron::ser::to_string_pretty(&self, ron::ser::PrettyConfig::default())?;
        std::fs::write(app_data.join("editor.eucc"), serialized)?;
        log::debug!("Saved editor config to {}", app_data.join("editor.eucc").display());
        Ok(())
    }

    /// Reads the current EditorSettings configuration from `{app_dir}/editor.eucc` and saves into [EDITOR_SETTINGS]
    /// as well as returns the value.
    ///
    /// If the configuration file does not exist, it will create a new configuration and then attempt to read from that.
    pub fn read() -> anyhow::Result<Self> {
        let app_data = app_dirs2::app_root(AppDataType::UserData, &APP_INFO)?;
        let real = match std::fs::read(app_data.join("editor.eucc")) {
            Ok(v) => v,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                log::warn!("Unable to read the configuration, overwriting");
                {
                    EDITOR_SETTINGS.read().save()?;
                }
                std::fs::read(app_data.join("editor.eucc"))?
            }
            Err(e) => return Err(e.into()),
        };
        let content: EditorSettings = ron::de::from_reader(real.as_slice())?;
        {
            let mut temp = EDITOR_SETTINGS.write();
            *temp = content.clone();
        }
        Ok(content)
    }
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self::new()
    }
}

pub struct EditorSettingsWindow {
    scene_command: SceneCommand,
    input_state: InputState,
    window: Option<WindowId>,

    current_leaf: EditorSettingsCurrentLeaf,
}

#[derive(Default)]
enum EditorSettingsCurrentLeaf {
    #[default]
    None,

    Performance,
}

impl EditorSettingsWindow {
    pub fn new() -> Self {
        Self {
            scene_command: SceneCommand::None,
            input_state: Default::default(),
            window: None,
            current_leaf: Default::default(),
        }
    }
}

impl Scene for EditorSettingsWindow {
    fn load(&mut self, graphics: std::sync::Arc<dropbear_engine::graphics::SharedGraphicsContext>) {
        self.window = Some(graphics.window.id());
    }

    fn physics_update(&mut self, _dt: f32, _graphics: std::sync::Arc<dropbear_engine::graphics::SharedGraphicsContext>) {}

    fn update(&mut self, _dt: f32, graphics: std::sync::Arc<dropbear_engine::graphics::SharedGraphicsContext>) {
        CentralPanel::default().show(&graphics.get_egui_context(), |ui| {
            let mut editor = EDITOR_SETTINGS.write();

            egui::SidePanel::left("editor_settings_tree_panel")
                .resizable(true)
                .default_width(200.0)
                .width_range(150.0..=400.0)
                .show_inside(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            let (_resp, action) = egui_ltreeview::TreeView::new(Id::from("editor_settings"))
                                .show(ui, |builder| {
                                    builder.node(
                                        NodeBuilder::leaf("Performance")
                                            .label("Performance")
                                    );
                                });

                            for a in action {
                                match a {
                                    Action::SetSelected(selected) => {
                                        let selected = selected.first().cloned();
                                        if let Some(s) = selected {
                                            match s {
                                                "Performance" => {
                                                    self.current_leaf = EditorSettingsCurrentLeaf::Performance;
                                                }
                                                _ => {
                                                    self.current_leaf = EditorSettingsCurrentLeaf::None;
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
                            EditorSettingsCurrentLeaf::Performance => {
                                ui.heading("Performance Settings");
                                ui.separator();

                                ui.label("Target FPS:");
                                ui.horizontal(|ui| {
                                    let mut local_set_max_fps = editor.target_fps.is_some();

                                    if ui.checkbox(&mut local_set_max_fps, "Set max frames-per-second (FPS)").changed() {
                                        if local_set_max_fps {
                                            editor.target_fps.enable_or(120); 
                                        } else {
                                            editor.target_fps.disable(); 
                                        }
                                    }

                                    if let Some(v) = editor.target_fps.get_mut() {
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

        self.window = Some(graphics.window.id());
    }

    fn render<'a>(&mut self, _graphics: std::sync::Arc<dropbear_engine::graphics::SharedGraphicsContext>) {
    }

    fn exit(&mut self, _event_loop: &ActiveEventLoop) {
        if let Err(e) = EDITOR_SETTINGS.read().save() {
            warn!("Failed to save editor settings: {:?}", e);
        }
    }

    fn run_command(&mut self) -> SceneCommand {
        std::mem::replace(&mut self.scene_command, SceneCommand::None)
    }
}

impl Keyboard for EditorSettingsWindow {
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

impl Mouse for EditorSettingsWindow {
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

impl Controller for EditorSettingsWindow {
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