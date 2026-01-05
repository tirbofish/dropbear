use egui::{DragValue, Slider, Ui, Widget};
use hecs::Entity;
use eucalyptus_core::physics::kcc::KCC;
use eucalyptus_core::rapier3d::control::{CharacterAutostep, CharacterLength};
use crate::editor::component::InspectableComponent;
use crate::editor::{Signal, StaticallyKept, UndoableAction};

impl InspectableComponent for KCC {
    fn inspect(
        &mut self,
        entity: &mut Entity,
        cfg: &mut StaticallyKept,
        ui: &mut Ui,
        undo_stack: &mut Vec<UndoableAction>,
        signal: &mut Signal,
        _label: &mut String
    ) {
        ui.vertical(|ui| {
            let mut local_enable_autostep = self.controller.autostep.is_some();

            ui.checkbox(&mut local_enable_autostep, "Enable Autostep");

            if local_enable_autostep != self.controller.autostep.is_some() {
                if self.controller.autostep.is_some() {
                    self.controller.autostep = None;
                } else if self.controller.autostep.is_none() {
                    self.controller.autostep = Some(CharacterAutostep::default())
                }
            }

            if let Some(step) = &mut self.controller.autostep {
                ui.checkbox(&mut step.include_dynamic_bodies, "Include Dynamic Bodies?");
            }

            ui.separator();

            ui.checkbox(&mut self.controller.slide, "Slide against floor?");
            ui.add(DragValue::new(&mut self.controller.min_slope_slide_angle));
        });
    }
}