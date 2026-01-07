use egui::{ComboBox, DragValue, Ui};
use hecs::Entity;
use eucalyptus_core::physics::kcc::KCC;
use eucalyptus_core::rapier3d::control::{CharacterAutostep, CharacterLength};
use eucalyptus_core::rapier3d::na::{UnitVector3, Vector3};
use eucalyptus_core::states::Label;
use crate::editor::component::InspectableComponent;
use crate::editor::{Signal, StaticallyKept, UndoableAction};

impl InspectableComponent for KCC {
    fn inspect(
        &mut self,
        _entity: &mut Entity,
        _cfg: &mut StaticallyKept,
        ui: &mut Ui,
        _undo_stack: &mut Vec<UndoableAction>,
        _signal: &mut Signal,
        label: &mut String
    ) {
        self.entity = Label::new(label.clone());

        fn edit_character_length(ui: &mut Ui, id_salt: impl std::hash::Hash, value: &mut CharacterLength, text: &str) {
            ui.horizontal(|ui| {
                ui.label(text);

                let (mut kind, mut v) = match *value {
                    CharacterLength::Absolute(x) => (0, x),
                    CharacterLength::Relative(x) => (1, x),
                };

                ComboBox::from_id_salt(id_salt)
                    .selected_text(match kind {
                        0 => "Absolute",
                        _ => "Relative",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut kind, 0, "Absolute");
                        ui.selectable_value(&mut kind, 1, "Relative");
                    });

                ui.add(DragValue::new(&mut v).speed(0.01));

                *value = if kind == 0 {
                    CharacterLength::Absolute(v)
                } else {
                    CharacterLength::Relative(v)
                };
            });
        }

        ui.vertical(|ui| {
            ui.label("Up Vector:");
            let up = *self.controller.up;
            let mut up_v = Vector3::new(up.x, up.y, up.z);

            ui.horizontal(|ui| {
                ui.label("X:");
                ui.add(DragValue::new(&mut up_v.x).speed(0.01));
                ui.label("Y:");
                ui.add(DragValue::new(&mut up_v.y).speed(0.01));
                ui.label("Z:");
                ui.add(DragValue::new(&mut up_v.z).speed(0.01));
            });

            if up_v.norm_squared() > 0.0 {
                self.controller.up = UnitVector3::new_normalize(up_v);
            }

            ui.add_space(8.0);

            edit_character_length(ui, "kcc_offset", &mut self.controller.offset, "Offset:");

            ui.add_space(8.0);
            ui.separator();

            ui.checkbox(&mut self.controller.slide, "Slide against floor?");

            ui.label("Slope Angles (degrees):");
            ui.horizontal(|ui| {
                ui.label("Max climb:");
                let mut deg = self.controller.max_slope_climb_angle.to_degrees();
                if ui.add(DragValue::new(&mut deg).speed(1.0)).changed() {
                    self.controller.max_slope_climb_angle = deg.to_radians();
                }
            });
            ui.horizontal(|ui| {
                ui.label("Min slide:");
                let mut deg = self.controller.min_slope_slide_angle.to_degrees();
                if ui.add(DragValue::new(&mut deg).speed(1.0)).changed() {
                    self.controller.min_slope_slide_angle = deg.to_radians();
                }
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Normal nudge:");
                ui.add(DragValue::new(&mut self.controller.normal_nudge_factor).speed(0.001));
            });

            ui.add_space(8.0);
            ui.separator();

            let mut enable_snap = self.controller.snap_to_ground.is_some();
            ui.checkbox(&mut enable_snap, "Snap to ground");
            if enable_snap && self.controller.snap_to_ground.is_none() {
                self.controller.snap_to_ground = Some(CharacterLength::Absolute(0.2));
            } else if !enable_snap {
                self.controller.snap_to_ground = None;
            }

            if let Some(ref mut snap) = self.controller.snap_to_ground {
                edit_character_length(ui, "kcc_snap_to_ground", snap, "Snap distance:");
            }

            ui.add_space(8.0);
            ui.separator();

            let mut enable_autostep = self.controller.autostep.is_some();
            ui.checkbox(&mut enable_autostep, "Enable autostep");
            if enable_autostep && self.controller.autostep.is_none() {
                self.controller.autostep = Some(CharacterAutostep::default());
            } else if !enable_autostep {
                self.controller.autostep = None;
            }

            if let Some(step) = &mut self.controller.autostep {
                edit_character_length(ui, "kcc_autostep_max_height", &mut step.max_height, "Max height:");
                edit_character_length(ui, "kcc_autostep_min_width", &mut step.min_width, "Min width:");
                ui.checkbox(&mut step.include_dynamic_bodies, "Include dynamic bodies");
            }
        });
    }
}