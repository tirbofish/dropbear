//! Used to aid with debugging any issues with the editor.

mod window;

use crate::debug::window::DebugWindow;
use crate::editor::Signal;
use dropbear_engine::DropbearWindowBuilder;
use egui::Ui;
use parking_lot::RwLock;
use std::rc::Rc;
use winit::window::WindowAttributes;

pub(crate) fn show_menu_bar(ui: &mut Ui, signal: &mut Signal) {
    ui.menu_button("Debug", |ui_debug| {
        if ui_debug.button("Panic").clicked() {
            log::warn!("Panic caused on purpose from Menu Button Click");
            panic!("Testing out panicking with new panic module, this is a test")
        }

        if ui_debug.button("size_of::<Editor>()").clicked() {
            log::debug!("size_of::<Editor>() is clicked");
            let size = size_of::<crate::editor::Editor>();
            log::info!("size_of::<Editor>() is {}", size);
            log::debug!("I'm so fat - editor")
        }

        if ui_debug.button("Launch new debug test window").clicked() {
            let debug_window = Rc::new(RwLock::new(DebugWindow::new()));

            let window_data = DropbearWindowBuilder::new()
                .with_attributes(
                    WindowAttributes::default().with_title("eucalyptus-editor debug window"),
                )
                .add_scene_with_input(debug_window, "debug_window")
                .set_initial_scene("debug_window")
                .build();

            *signal = Signal::RequestNewWindow(window_data);
        }

        if ui_debug.button("Show TypeID of components").clicked() {
            if crate::features::is_enabled(crate::features::ShowComponentTypeIDInEditor) {
                crate::features::disable(crate::features::ShowComponentTypeIDInEditor);
            } else {
                crate::features::enable(crate::features::ShowComponentTypeIDInEditor);
            }
        }
    });
}
