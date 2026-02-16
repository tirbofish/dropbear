use egui::{ComboBox, Ui};
use hecs::Entity;
use eucalyptus_core::physics::collider::{Collider, ColliderShape};
use eucalyptus_core::states::Label;
use crate::editor::{Signal, StaticallyKept, UndoableAction};

impl InspectableComponent for Collider {
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

            ui.label("Shape:");
            let current_shape = self.shape_type_name();
            ComboBox::from_id_salt("collider_shape")
                .selected_text(current_shape)
                .show_ui(ui, |ui| {
                    if ui.selectable_label(current_shape == "Box", "Box").clicked() {
                        if current_shape != "Box" {
                            self.shape = ColliderShape::Box { half_extents: [0.5, 0.5, 0.5] };
                        }
                    }
                    if ui.selectable_label(current_shape == "Sphere", "Sphere").clicked() {
                        if current_shape != "Sphere" {
                            self.shape = ColliderShape::Sphere { radius: 0.5 };
                        }
                    }
                    if ui.selectable_label(current_shape == "Capsule", "Capsule").clicked() {
                        if current_shape != "Capsule" {
                            self.shape = ColliderShape::Capsule { half_height: 0.5, radius: 0.25 };
                        }
                    }
                    if ui.selectable_label(current_shape == "Cylinder", "Cylinder").clicked() {
                        if current_shape != "Cylinder" {
                            self.shape = ColliderShape::Cylinder { half_height: 0.5, radius: 0.25 };
                        }
                    }
                    if ui.selectable_label(current_shape == "Cone", "Cone").clicked() {
                        if current_shape != "Cone" {
                            self.shape = ColliderShape::Cone { half_height: 0.5, radius: 0.25 };
                        }
                    }
                });

            ui.add_space(8.0);

            match &mut self.shape {
                ColliderShape::Box { half_extents } => {
                    ui.label("Half Extents:");
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        ui.add(egui::DragValue::new(&mut half_extents[0])
                            .speed(0.01));
                        ui.label("Y:");
                        ui.add(egui::DragValue::new(&mut half_extents[1])
                            .speed(0.01));
                        ui.label("Z:");
                        ui.add(egui::DragValue::new(&mut half_extents[2])
                            .speed(0.01));
                    });
                }
                ColliderShape::Sphere { radius } => {
                    ui.horizontal(|ui| {
                        ui.label("Radius:");
                        ui.add(egui::DragValue::new(radius)
                            .speed(0.01));
                    });
                }
                ColliderShape::Capsule { half_height, radius } => {
                    ui.horizontal(|ui| {
                        ui.label("Half Height:");
                        ui.add(egui::DragValue::new(half_height)
                            .speed(0.01));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Radius:");
                        ui.add(egui::DragValue::new(radius)
                            .speed(0.01));
                    });
                }
                ColliderShape::Cylinder { half_height, radius } => {
                    ui.horizontal(|ui| {
                        ui.label("Half Height:");
                        ui.add(egui::DragValue::new(half_height)
                            .speed(0.01));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Radius:");
                        ui.add(egui::DragValue::new(radius)
                            .speed(0.01));
                    });
                }
                ColliderShape::Cone { half_height, radius } => {
                    ui.horizontal(|ui| {
                        ui.label("Half Height:");
                        ui.add(egui::DragValue::new(half_height)
                            .speed(0.01));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Radius:");
                        ui.add(egui::DragValue::new(radius)
                            .speed(0.01));
                    });
                }
            }

            ui.add_space(8.0);

            ui.separator();
            ui.label("Physical Properties:");

            ui.horizontal(|ui| {
                ui.label("Density:");
                ui.add(egui::DragValue::new(&mut self.density)
                    .speed(0.01));
            });

            ui.horizontal(|ui| {
                ui.label("Friction:");
                ui.add(egui::Slider::new(&mut self.friction, 0.0..=2.0));
            });

            ui.horizontal(|ui| {
                ui.label("Restitution:");
                ui.add(egui::Slider::new(&mut self.restitution, 0.0..=1.0));
            });

            ui.checkbox(&mut self.is_sensor, "Is Sensor (No physical response)");

            ui.add_space(8.0);

            ui.separator();
            ui.label("Local Offset:");

            ui.label("Translation:");
            ui.horizontal(|ui| {
                ui.label("X:");
                ui.add(egui::DragValue::new(&mut self.translation[0]).speed(0.01));
                ui.label("Y:");
                ui.add(egui::DragValue::new(&mut self.translation[1]).speed(0.01));
                ui.label("Z:");
                ui.add(egui::DragValue::new(&mut self.translation[2]).speed(0.01));
            });

            ui.label("Rotation (degrees):");
            ui.horizontal(|ui| {
                ui.label("X:");
                let mut deg_x = self.rotation[0].to_degrees();
                if ui.add(egui::DragValue::new(&mut deg_x).speed(1.0)).changed() {
                    self.rotation[0] = deg_x.to_radians();
                }
                ui.label("Y:");
                let mut deg_y = self.rotation[1].to_degrees();
                if ui.add(egui::DragValue::new(&mut deg_y).speed(1.0)).changed() {
                    self.rotation[1] = deg_y.to_radians();
                }
                ui.label("Z:");
                let mut deg_z = self.rotation[2].to_degrees();
                if ui.add(egui::DragValue::new(&mut deg_z).speed(1.0)).changed() {
                    self.rotation[2] = deg_z.to_radians();
                }
            });
        });
    }
}