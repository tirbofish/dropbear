use crate::editor::{EditorTabViewer, TABS_GLOBAL};

impl<'a> EditorTabViewer<'a> {
    pub(crate) fn resource_inspector(&mut self, ui: &mut egui::Ui) {
        let mut cfg = TABS_GLOBAL.lock();

        let local_scene_settings = cfg.root_node_selected;

        if let Some(entity) = self.selected_entity {
            let inspect_entity = *entity;

            if !cfg.root_node_selected {
                ui.label(format!("Entity ID: {}", inspect_entity.id()));
                ui.separator();

                self.component_registry
                    .inspect_components(self.world, inspect_entity, ui, self.graphics.clone());
            }
        } else if !local_scene_settings {
            ui.label("No entity selected, therefore no info to provide. Go on, what are you waiting for? Click an entity!");
        }

        if local_scene_settings {
            log_once::debug_once!("Rendering scene settings");
            self.scene_settings(&mut cfg, ui);
        }
    }
}