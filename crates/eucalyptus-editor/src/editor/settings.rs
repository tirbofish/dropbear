pub mod editor;
pub mod project;

use crate::editor::{EditorTabViewer, StaticallyKept};
use eucalyptus_core::states::SCENES;

impl<'a> EditorTabViewer<'a> {
    pub fn scene_settings(&mut self, _cfg: &mut StaticallyKept, ui: &mut egui::Ui) {
        ui.label("Scene Settings");

        let current_scene_name = self.current_scene_name.clone();

        if let Some(scene_name) = current_scene_name {
            let mut scenes = SCENES.write();
            if let Some(scene) = scenes.iter_mut().find(|s| s.scene_name == scene_name) {
                ui.label(format!("Scene: {}", scene.scene_name));
                ui.separator();

                let mut preloaded = scene.settings.preloaded;
                if ui.checkbox(&mut preloaded, "Preload Assets").changed() {
                    scene.settings.preloaded = preloaded;
                }
                ui.label("Ensures scene assets are preloaded at game start");

                let mut show_hitboxes = scene.settings.show_hitboxes;
                if ui.checkbox(&mut show_hitboxes, "Render Hitboxes").changed() {
                    scene.settings.show_hitboxes = show_hitboxes;
                }
                ui.label("Renders collider wireframes for debugging");

                let mut overlay_billboard = scene.settings.overlay_billboard;
                if ui.checkbox(&mut overlay_billboard, "Overlay Billboard UI").changed() {
                    scene.settings.overlay_billboard = overlay_billboard;
                }
                ui.label("Renders billboard UI widgets for all entities");

                let mut overlay_hud = scene.settings.overlay_hud;
                if ui.checkbox(&mut overlay_hud, "Overlay HUD").changed() {
                    scene.settings.overlay_hud = overlay_hud;
                }
                ui.label("Renders the HUD UI overlay on top of the viewport");

                ui.separator();
                ui.label("Ambient Strength");
                let mut ambient = scene.settings.ambient_strength;
                if ui.add(egui::Slider::new(&mut ambient, 0.0..=2.0).step_by(0.01)).changed() {
                    scene.settings.ambient_strength = ambient;
                }
                ui.label("Controls the intensity of ambient/IBL lighting");
            } else {
                ui.label("Scene not found");
            }
        } else {
            ui.label("No scene currently loaded");
        }
    }
}
