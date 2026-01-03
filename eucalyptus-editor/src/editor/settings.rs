use crate::editor::{EditorTabViewer, StaticallyKept};
use eucalyptus_core::states::SCENES;

impl<'a> EditorTabViewer<'a> {
    pub fn scene_settings(&mut self, _cfg: &mut StaticallyKept, ui: &mut egui::Ui) {
        ui.label("Scene Settings");

        let editor = unsafe { &mut *self.editor };
        let current_scene_name = editor.current_scene_name.clone();
        
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
            } else {
                ui.label("Scene not found");
            }
        } else {
            ui.label("No scene currently loaded");
        }
    }
}