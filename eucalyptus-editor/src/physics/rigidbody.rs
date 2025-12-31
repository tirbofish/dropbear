use egui::{ComboBox, Ui};
use hecs::Entity;
use eucalyptus_core::physics::rigidbody::{RigidBody, RigidBodyMode};
use eucalyptus_core::states::Label;
use crate::editor::component::InspectableComponent;
use crate::editor::{Signal, StaticallyKept, UndoableAction};

impl InspectableComponent for RigidBody {
    fn inspect(
        &mut self,
        _entity: &mut Entity,
        _cfg: &mut StaticallyKept,
        ui: &mut Ui,
        _undo_stack: &mut Vec<UndoableAction>,
        _signal: &mut Signal,
        label: &mut String
    ) {
        ui.vertical(|ui| {
            self.entity = Label::new(label.clone());

            let mut selected = self.mode.clone();
            ComboBox::from_id_salt("rigidbody")
                .selected_text(format!("{:?}", self.mode))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut selected, RigidBodyMode::Dynamic, "Dynamic");
                    ui.selectable_value(&mut selected, RigidBodyMode::Fixed, "Fixed");
                    ui.selectable_value(&mut selected, RigidBodyMode::KinematicPosition, "Kinematic Position");
                    ui.selectable_value(&mut selected, RigidBodyMode::KinematicVelocity, "Kinematic Velocity");
                });

            if selected != self.mode {
                self.mode = selected;
            }

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Gravity Scale:");
                ui.add(egui::DragValue::new(&mut self.gravity_scale)
                    .speed(0.1)
                    .range(0.0..=10.0));
            });


            ui.checkbox(&mut self.can_sleep, "Can Sleep");

            ui.checkbox(&mut self.sleeping, "Initially sleeping?");

            ui.checkbox(&mut self.ccd_enabled, "CCD Enabled");

            ui.add_space(8.0);

            ui.label("Linear Velocity:");
            ui.horizontal(|ui| {
                ui.label("X:");
                ui.add(egui::DragValue::new(&mut self.linvel[0]).speed(0.1));
                ui.label("Y:");
                ui.add(egui::DragValue::new(&mut self.linvel[1]).speed(0.1));
                ui.label("Z:");
                ui.add(egui::DragValue::new(&mut self.linvel[2]).speed(0.1));
            });

            ui.label("Angular Velocity:");
            ui.horizontal(|ui| {
                ui.label("X:");
                ui.add(egui::DragValue::new(&mut self.angvel[0]).speed(0.1));
                ui.label("Y:");
                ui.add(egui::DragValue::new(&mut self.angvel[1]).speed(0.1));
                ui.label("Z:");
                ui.add(egui::DragValue::new(&mut self.angvel[2]).speed(0.1));
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Linear Damping:");
                ui.add(egui::DragValue::new(&mut self.linear_damping)
                    .speed(0.01)
                    .range(0.0..=10.0));
            });

            ui.horizontal(|ui| {
                ui.label("Angular Damping:");
                ui.add(egui::DragValue::new(&mut self.angular_damping)
                    .speed(0.01)
                    .range(0.0..=10.0))
            });

            ui.add_space(8.0);

            ui.label("Lock Translation:");
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.lock_translation.x, "X");
                ui.checkbox(&mut self.lock_translation.y, "Y");
                ui.checkbox(&mut self.lock_translation.z, "Z");
            });

            ui.label("Lock Rotation:");
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.lock_rotation.x, "X");
                ui.checkbox(&mut self.lock_rotation.y, "Y");
                ui.checkbox(&mut self.lock_rotation.z, "Z");
            });
        });
    }
}