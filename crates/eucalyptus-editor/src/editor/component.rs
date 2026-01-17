//! This module should describe the different components that are editable in the resource inspector.

use crate::editor::{Signal, StaticallyKept, UndoableAction};
use dropbear_engine::asset::{AssetHandle, ASSET_REGISTRY};
use dropbear_engine::attenuation::ATTENUATION_PRESETS;
use dropbear_engine::entity::{EntityTransform, MeshRenderer, Transform};
use dropbear_engine::lighting::LightType;
use dropbear_engine::utils::ResourceReferenceType;
use egui::{CollapsingHeader, ComboBox, DragValue, Grid, RichText, TextEdit, Ui, UiBuilder};
use eucalyptus_core::camera::CameraType;
use eucalyptus_core::states::{Camera3D, Light, Property, Script};
use eucalyptus_core::{fatal, warn};
use glam::{DVec3, Vec3};
use hecs::Entity;
use std::time::Instant;
use eucalyptus_core::properties::{CustomProperties, Value};

/// A trait that can added to any component that allows you to inspect the value in the editor.
pub trait InspectableComponent {
    fn inspect(
        &mut self,
        entity: &mut Entity,
        cfg: &mut StaticallyKept,
        ui: &mut Ui,
        undo_stack: &mut Vec<UndoableAction>,
        signal: &mut Signal,
        _label: &mut String,
    );
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ValueType {
    String,
    Float,
    Int,
    Bool,
    Vec3,
}

impl From<Value> for ValueType {
    fn from(value: Value) -> Self {
        match value {
            Value::String(_) => ValueType::String,
            Value::Int(_) => ValueType::Int,
            Value::Double(_) => ValueType::Float,
            Value::Bool(_) => ValueType::Bool,
            Value::Vec3(_) => ValueType::Vec3,
        }
    }
}

impl From<&mut Value> for ValueType {
    fn from(value: &mut Value) -> Self {
        match value {
            Value::String(_) => ValueType::String,
            Value::Int(_) => ValueType::Int,
            Value::Double(_) => ValueType::Float,
            Value::Bool(_) => ValueType::Bool,
            Value::Vec3(_) => ValueType::Vec3,
        }
    }
}

fn wrap_angle_degrees(angle: f64) -> f64 {
    (angle + 180.0).rem_euclid(360.0) - 180.0
}

fn reconcile_angle(angle: f64, reference: f64) -> f64 {
    let delta = wrap_angle_degrees(angle - reference);
    wrap_angle_degrees(reference + delta)
}

impl InspectableComponent for CustomProperties {
    fn inspect(
        &mut self,
        _entity: &mut Entity,
        _cfg: &mut StaticallyKept,
        ui: &mut Ui,
        _undo_stack: &mut Vec<UndoableAction>,
        _signal: &mut Signal,
        _label: &mut String,
    ) {
        ui.vertical(|ui| {
            Grid::new("properties").striped(true).show(ui, |ui| {
                ui.label(RichText::new("Key"));
                ui.label(RichText::new("Type"));
                ui.label(RichText::new("Value"));
                ui.label(RichText::new("Action"));
                ui.end_row();

                let mut to_delete: Option<u64> = None;
                let mut to_rename: Option<(u64, String)> = None;

                for (_i, property) in self.custom_properties.iter_mut().enumerate() {
                    let mut edited_key = property.key.clone();
                    ui.add_sized([100.0, 20.0], TextEdit::singleline(&mut edited_key));

                    if edited_key != property.key {
                        to_rename = Some((property.id, edited_key));
                    }

                    let current_type = ValueType::from(&mut property.value);
                    let mut selected_type = current_type;

                    ComboBox::from_id_salt(format!("type_{}", property.id))
                        .selected_text(format!("{:?}", selected_type))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut selected_type,
                                ValueType::String,
                                "String",
                            );
                            ui.selectable_value(&mut selected_type, ValueType::Float, "Float");
                            ui.selectable_value(&mut selected_type, ValueType::Int, "Int");
                            ui.selectable_value(&mut selected_type, ValueType::Bool, "Bool");
                            ui.selectable_value(&mut selected_type, ValueType::Vec3, "Vec3");
                        });

                    if selected_type != current_type {
                        property.value = match selected_type {
                            ValueType::String => Value::String(String::new()),
                            ValueType::Float => Value::Double(0.0),
                            ValueType::Int => Value::Int(0),
                            ValueType::Bool => Value::Bool(false),
                            ValueType::Vec3 => Value::Vec3([0.0, 0.0, 0.0]),
                        };
                    }

                    let speed = {
                        let input = ui.input(|i| i.modifiers);
                        if input.shift {
                            0.01
                        } else if cfg!(target_os = "macos") && input.mac_cmd
                            || !cfg!(target_os = "macos") && input.ctrl
                        {
                            1.0
                        } else {
                            0.1
                        }
                    };

                    match &mut property.value {
                        Value::String(s) => {
                            ui.add_sized([100.0, 20.0], egui::TextEdit::singleline(s));
                        }
                        Value::Int(n) => {
                            ui.add(DragValue::new(n).speed(1.0));
                        }
                        Value::Double(f) => {
                            ui.add(DragValue::new(f).speed(speed));
                        }
                        Value::Bool(b) => {
                            if ui.button(if *b { "✅" } else { "❌" }).clicked() {
                                *b = !*b;
                            }
                        }
                        Value::Vec3(v) => {
                            ui.horizontal(|ui| {
                                ui.add(DragValue::new(&mut v[0]).speed(speed));
                                ui.add(DragValue::new(&mut v[1]).speed(speed));
                                ui.add(DragValue::new(&mut v[2]).speed(speed));
                            });
                        }
                    }

                    if ui.button("🗑️").clicked() {
                        log::debug!("Trashing {}", property.key);
                        to_delete = Some(property.id);
                    }

                    ui.end_row();
                }

                if let Some(id) = to_delete {
                    self.custom_properties.retain(|p| p.id != id);
                }

                if let Some((id, new_key)) = to_rename {
                    if let Some(property) =
                        self.custom_properties.iter_mut().find(|p| p.id == id)
                    {
                        property.key = new_key;
                    } else {
                        warn!("Failed to rename property: id not found");
                    }
                }

                if ui.button("Add").clicked() {
                    log::debug!("Inserting new default value");
                    let mut new_key = String::from("new_property");
                    let mut counter = 1;
                    while self.custom_properties.iter().any(|p| p.key == new_key) {
                        new_key = format!("new_property_{}", counter);
                        counter += 1;
                    }
                    self.custom_properties.push(Property {
                        id: self.next_id,
                        key: new_key,
                        value: Value::default(),
                    });
                    self.next_id += 1;
                }
            });
        });
    }
}

impl InspectableComponent for EntityTransform {
    fn inspect(
        &mut self,
        entity: &mut Entity,
        cfg: &mut StaticallyKept,
        ui: &mut Ui,
        undo_stack: &mut Vec<UndoableAction>,
        signal: &mut Signal,
        _label: &mut String,
    ) {
        self.local_mut().inspect(
            entity,
            cfg,
            ui,
            undo_stack,
            signal,
            &mut "Local Transform".to_string(),
        );
        self.world_mut().inspect(
            entity,
            cfg,
            ui,
            undo_stack,
            signal,
            &mut "World Transform".to_string(),
        );
    }
}

fn inspect_light_transform(
    transform: &mut Transform,
    entity: &mut Entity,
    cfg: &mut StaticallyKept,
    ui: &mut Ui,
    undo_stack: &mut Vec<UndoableAction>,
    light_type: &LightType,
) {
    let show_position = matches!(light_type, LightType::Point | LightType::Spot);
    let show_rotation = !matches!(light_type, LightType::Point);

    if show_position {
        ui.label(RichText::new("Position").strong());

        ui.horizontal_wrapped(|ui| {
            ui.horizontal(|ui| {
                ui.label("X:");
                let response = ui.add(
                    egui::DragValue::new(&mut transform.position.x)
                        .speed(0.1)
                        .fixed_decimals(3),
                );

                if response.drag_started() {
                    cfg.transform_old_entity = Some(*entity);
                    cfg.transform_original_transform = Some(*transform);
                    cfg.transform_in_progress = true;
                }

                if response.drag_stopped() && cfg.transform_in_progress {
                    if let Some(ent) = cfg.transform_old_entity.take()
                        && let Some(orig) = cfg.transform_original_transform.take()
                    {
                        UndoableAction::push_to_undo(
                            undo_stack,
                            UndoableAction::Transform(ent, orig),
                        );
                    }
                    cfg.transform_in_progress = false;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Y:");
                let response = ui.add(
                    egui::DragValue::new(&mut transform.position.y)
                        .speed(0.1)
                        .fixed_decimals(3),
                );

                if response.drag_started() {
                    cfg.transform_old_entity = Some(*entity);
                    cfg.transform_original_transform = Some(*transform);
                    cfg.transform_in_progress = true;
                }

                if response.drag_stopped() && cfg.transform_in_progress {
                    if let Some(ent) = cfg.transform_old_entity.take()
                        && let Some(orig) = cfg.transform_original_transform.take()
                    {
                        UndoableAction::push_to_undo(
                            undo_stack,
                            UndoableAction::Transform(ent, orig),
                        );
                    }
                    cfg.transform_in_progress = false;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Z:");
                let response = ui.add(
                    egui::DragValue::new(&mut transform.position.z)
                        .speed(0.1)
                        .fixed_decimals(3),
                );

                if response.drag_started() {
                    cfg.transform_old_entity = Some(*entity);
                    cfg.transform_original_transform = Some(*transform);
                    cfg.transform_in_progress = true;
                }

                if response.drag_stopped() && cfg.transform_in_progress {
                    if let Some(ent) = cfg.transform_old_entity.take()
                        && let Some(orig) = cfg.transform_original_transform.take()
                    {
                        UndoableAction::push_to_undo(
                            undo_stack,
                            UndoableAction::Transform(ent, orig),
                        );
                    }
                    cfg.transform_in_progress = false;
                }
            });
        });

        if ui.button("Reset Position").clicked() {
            transform.position = DVec3::ZERO;
        }
        ui.add_space(5.0);
    }

    if show_rotation {
        ui.label(RichText::new("Rotation").strong());

        let mut rotation_deg = *cfg
            .transform_rotation_cache
            .entry(*entity)
            .or_insert_with(|| {
                let (x, y, z) = transform.rotation.to_euler(glam::EulerRot::XYZ);
                DVec3::new(x.to_degrees(), y.to_degrees(), z.to_degrees())
            });

        if let Some(prev) = cfg.transform_rotation_cache.get(entity) {
            let mut degrees = rotation_deg;
            let (x, y, z) = transform.rotation.to_euler(glam::EulerRot::XYZ);
            degrees.x = x.to_degrees();
            degrees.y = y.to_degrees();
            degrees.z = z.to_degrees();

            degrees.x = reconcile_angle(degrees.x, prev.x);
            degrees.y = reconcile_angle(degrees.y, prev.y);
            degrees.z = reconcile_angle(degrees.z, prev.z);

            degrees.x = wrap_angle_degrees(degrees.x);
            degrees.y = wrap_angle_degrees(degrees.y);
            degrees.z = wrap_angle_degrees(degrees.z);

            cfg.transform_rotation_cache.insert(*entity, degrees);
            rotation_deg = degrees;
        };

        let mut rotation_changed = false;

        ui.horizontal(|ui| {
            ui.label("Pitch (X):");
            let response = ui.add(
                egui::DragValue::new(&mut rotation_deg.x)
                    .speed(0.5)
                    .suffix("°")
                    .range(-180.0..=180.0)
                    .fixed_decimals(2),
            );

            if response.drag_started() {
                cfg.transform_old_entity = Some(*entity);
                cfg.transform_original_transform = Some(*transform);
                cfg.transform_in_progress = true;
            }

            if response.changed() {
                rotation_changed = true;
            }

            if response.drag_stopped() && cfg.transform_in_progress {
                if let Some(ent) = cfg.transform_old_entity.take()
                    && let Some(orig) = cfg.transform_original_transform.take()
                {
                    UndoableAction::push_to_undo(
                        undo_stack,
                        UndoableAction::Transform(ent, orig),
                    );
                }
                cfg.transform_in_progress = false;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Yaw (Y):");
            let response = ui.add(
                egui::DragValue::new(&mut rotation_deg.y)
                    .speed(0.5)
                    .suffix("°")
                    .range(-180.0..=180.0)
                    .fixed_decimals(2),
            );

            if response.drag_started() {
                cfg.transform_old_entity = Some(*entity);
                cfg.transform_original_transform = Some(*transform);
                cfg.transform_in_progress = true;
            }

            if response.changed() {
                rotation_changed = true;
            }

            if response.drag_stopped() && cfg.transform_in_progress {
                if let Some(ent) = cfg.transform_old_entity.take()
                    && let Some(orig) = cfg.transform_original_transform.take()
                {
                    UndoableAction::push_to_undo(
                        undo_stack,
                        UndoableAction::Transform(ent, orig),
                    );
                }
                cfg.transform_in_progress = false;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Roll (Z):");
            let response = ui.add(
                egui::DragValue::new(&mut rotation_deg.z)
                    .speed(0.5)
                    .suffix("°")
                    .range(-180.0..=180.0)
                    .fixed_decimals(2),
            );

            if response.drag_started() {
                cfg.transform_old_entity = Some(*entity);
                cfg.transform_original_transform = Some(*transform);
                cfg.transform_in_progress = true;
            }

            if response.changed() {
                rotation_changed = true;
            }

            if response.drag_stopped() && cfg.transform_in_progress {
                if let Some(ent) = cfg.transform_old_entity.take()
                    && let Some(orig) = cfg.transform_original_transform.take()
                {
                    UndoableAction::push_to_undo(
                        undo_stack,
                        UndoableAction::Transform(ent, orig),
                    );
                }
                cfg.transform_in_progress = false;
            }
        });

        if rotation_changed {
            let rot_x = glam::DQuat::from_rotation_x(rotation_deg.x.to_radians());
            let rot_y = glam::DQuat::from_rotation_y(rotation_deg.y.to_radians());
            let rot_z = glam::DQuat::from_rotation_z(rotation_deg.z.to_radians());
            transform.rotation = rot_y * rot_x * rot_z;
            cfg.transform_rotation_cache.insert(*entity, rotation_deg);
        }

        if ui.button("Reset Rotation").clicked() {
            transform.rotation = glam::DQuat::IDENTITY;
            cfg.transform_rotation_cache.insert(*entity, DVec3::ZERO);
        }
        ui.add_space(5.0);
    }
}

fn inspect_transform(
    transform: &mut Transform,
    entity: &mut Entity,
    cfg: &mut StaticallyKept,
    ui: &mut Ui,
    undo_stack: &mut Vec<UndoableAction>,
    label: &str,
    show_position: bool,
    show_rotation: bool,
    show_scale: bool,
) {
    ui.vertical(|ui| {
        CollapsingHeader::new(label)
            .default_open(true)
            .show(ui, |ui| {
                if show_position {
                    ui.horizontal(|ui| {
                        ui.label("Position:");
                    });

                    ui.horizontal_wrapped(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("X:");
                            let response = ui.add(
                                egui::DragValue::new(&mut transform.position.x)
                                    .speed(0.1)
                                    .fixed_decimals(3),
                            );

                            if response.drag_started() {
                                cfg.transform_old_entity = Some(*entity);
                                cfg.transform_original_transform = Some(*transform);
                                cfg.transform_in_progress = true;
                            }

                            if response.drag_stopped() && cfg.transform_in_progress {
                                if let Some(ent) = cfg.transform_old_entity.take()
                                    && let Some(orig) = cfg.transform_original_transform.take()
                                {
                                    UndoableAction::push_to_undo(
                                        undo_stack,
                                        UndoableAction::Transform(ent, orig),
                                    );
                                    log::debug!("Pushed X transform change to undo stack");
                                }
                                cfg.transform_in_progress = false;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Y:");
                            let response = ui.add(
                                egui::DragValue::new(&mut transform.position.y)
                                    .speed(0.1)
                                    .fixed_decimals(3),
                            );

                            if response.drag_started() {
                                cfg.transform_old_entity = Some(*entity);
                                cfg.transform_original_transform = Some(*transform);
                                cfg.transform_in_progress = true;
                            }

                            if response.drag_stopped() && cfg.transform_in_progress {
                                if let Some(ent) = cfg.transform_old_entity.take()
                                    && let Some(orig) = cfg.transform_original_transform.take()
                                {
                                    UndoableAction::push_to_undo(
                                        undo_stack,
                                        UndoableAction::Transform(ent, orig),
                                    );
                                    log::debug!("Pushed Y transform change to undo stack");
                                }
                                cfg.transform_in_progress = false;
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Z:");
                            let response = ui.add(
                                egui::DragValue::new(&mut transform.position.z)
                                    .speed(0.1)
                                    .fixed_decimals(3),
                            );

                            if response.drag_started() {
                                cfg.transform_old_entity = Some(*entity);
                                cfg.transform_original_transform = Some(*transform);
                                cfg.transform_in_progress = true;
                            }

                            if response.drag_stopped() && cfg.transform_in_progress {
                                if let Some(ent) = cfg.transform_old_entity.take()
                                    && let Some(orig) = cfg.transform_original_transform.take()
                                {
                                    UndoableAction::push_to_undo(
                                        undo_stack,
                                        UndoableAction::Transform(ent, orig),
                                    );
                                    log::debug!("Pushed Z transform change to undo stack");
                                }
                                cfg.transform_in_progress = false;
                            }
                        });
                    });

                    if ui.button("Reset Position").clicked() {
                        transform.position = DVec3::ZERO;
                    }

                    ui.add_space(5.0);
                }

                if show_rotation {
                    ui.label("Rotation:");

                    ui.horizontal_wrapped(|ui| {
                        let cached_rotation = cfg.transform_rotation_cache.get(entity).copied();

                        let mut rotation_deg: DVec3 = if cfg.transform_in_progress {
                            cached_rotation.unwrap_or_else(|| {
                                let (x, y, z) = transform.rotation.to_euler(glam::EulerRot::YXZ);
                                DVec3::new(x.to_degrees(), y.to_degrees(), z.to_degrees())
                            })
                        } else {
                            let (x, y, z) = transform.rotation.to_euler(glam::EulerRot::YXZ);
                            let mut degrees =
                                DVec3::new(x.to_degrees(), y.to_degrees(), z.to_degrees());

                            if let Some(prev) = cached_rotation {
                                degrees.x = reconcile_angle(degrees.x, prev.x);
                                degrees.y = reconcile_angle(degrees.y, prev.y);
                                degrees.z = reconcile_angle(degrees.z, prev.z);
                            }

                            degrees.x = wrap_angle_degrees(degrees.x);
                            degrees.y = wrap_angle_degrees(degrees.y);
                            degrees.z = wrap_angle_degrees(degrees.z);

                            cfg.transform_rotation_cache.insert(*entity, degrees);
                            degrees
                        };

                        let mut rotation_changed = false;

                        ui.horizontal(|ui| {
                            ui.label("Pitch (X):");
                            let response = ui.add(
                                egui::DragValue::new(&mut rotation_deg.x)
                                    .speed(0.5)
                                    .suffix("°")
                                    .range(-180.0..=180.0)
                                    .fixed_decimals(2),
                            );

                            if response.drag_started() {
                                cfg.transform_old_entity = Some(*entity);
                                cfg.transform_original_transform = Some(*transform);
                                cfg.transform_in_progress = true;
                            }

                            if response.changed() {
                                rotation_changed = true;
                            }

                            if response.drag_stopped() && cfg.transform_in_progress {
                                if let Some(ent) = cfg.transform_old_entity.take()
                                    && let Some(orig) = cfg.transform_original_transform.take()
                                {
                                    UndoableAction::push_to_undo(
                                        undo_stack,
                                        UndoableAction::Transform(ent, orig),
                                    );
                                    log::debug!("Pushed X rotation change to undo stack");
                                }
                                cfg.transform_in_progress = false;
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Yaw (Y):");
                            let response = ui.add(
                                egui::DragValue::new(&mut rotation_deg.y)
                                    .speed(0.5)
                                    .suffix("°")
                                    .range(-180.0..=180.0)
                                    .fixed_decimals(2),
                            );

                            if response.drag_started() {
                                cfg.transform_old_entity = Some(*entity);
                                cfg.transform_original_transform = Some(*transform);
                                cfg.transform_in_progress = true;
                            }

                            if response.changed() {
                                rotation_changed = true;
                            }

                            if response.drag_stopped() && cfg.transform_in_progress {
                                if let Some(ent) = cfg.transform_old_entity.take()
                                    && let Some(orig) = cfg.transform_original_transform.take()
                                {
                                    UndoableAction::push_to_undo(
                                        undo_stack,
                                        UndoableAction::Transform(ent, orig),
                                    );
                                    log::debug!("Pushed Y rotation change to undo stack");
                                }
                                cfg.transform_in_progress = false;
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Roll (Z):");
                            let response = ui.add(
                                egui::DragValue::new(&mut rotation_deg.z)
                                    .speed(0.5)
                                    .suffix("°")
                                    .range(-180.0..=180.0)
                                    .fixed_decimals(2),
                            );

                            if response.drag_started() {
                                cfg.transform_old_entity = Some(*entity);
                                cfg.transform_original_transform = Some(*transform);
                                cfg.transform_in_progress = true;
                            }

                            if response.changed() {
                                rotation_changed = true;
                            }

                            if response.drag_stopped() && cfg.transform_in_progress {
                                if let Some(ent) = cfg.transform_old_entity.take()
                                    && let Some(orig) = cfg.transform_original_transform.take()
                                {
                                    UndoableAction::push_to_undo(
                                        undo_stack,
                                        UndoableAction::Transform(ent, orig),
                                    );
                                    log::debug!("Pushed Z rotation change to undo stack");
                                }
                                cfg.transform_in_progress = false;
                            }
                        });

                        if rotation_changed {
                            rotation_deg.x = wrap_angle_degrees(rotation_deg.x);
                            rotation_deg.y = wrap_angle_degrees(rotation_deg.y);
                            rotation_deg.z = wrap_angle_degrees(rotation_deg.z);

                            cfg.transform_rotation_cache.insert(*entity, rotation_deg);
                            transform.rotation = glam::DQuat::from_euler(
                                glam::EulerRot::YXZ,
                                rotation_deg.x.to_radians(),
                                rotation_deg.y.to_radians(),
                                rotation_deg.z.to_radians(),
                            );
                        }
                    });

                    if ui.button("Reset Rotation").clicked() {
                        transform.rotation = glam::DQuat::IDENTITY;
                        cfg.transform_rotation_cache.insert(*entity, DVec3::ZERO);
                    }
                    ui.add_space(5.0);
                }

                if show_scale {
                    ui.horizontal(|ui| {
                        ui.label("Scale:");
                        let lock_icon = if cfg.scale_locked { "🔒" } else { "🔓" };
                        if ui
                            .button(lock_icon)
                            .on_hover_text("Lock uniform scaling")
                            .clicked()
                        {
                            cfg.scale_locked = !cfg.scale_locked;
                        }
                    });

                    let mut scale_changed = false;
                    let mut new_scale = transform.scale;

                    ui.horizontal(|ui| {
                        ui.label("X:");
                        let response = ui.add(
                            DragValue::new(&mut new_scale.x)
                                .speed(0.01)
                                .fixed_decimals(3),
                        );

                        if response.drag_started() {
                            cfg.transform_old_entity = Some(*entity);
                            cfg.transform_original_transform = Some(*transform);
                            cfg.transform_in_progress = true;
                        }

                        if response.changed() {
                            scale_changed = true;
                            if cfg.scale_locked {
                                let scale_factor = new_scale.x / transform.scale.x;
                                new_scale.y = transform.scale.y * scale_factor;
                                new_scale.z = transform.scale.z * scale_factor;
                            }
                        }

                        if response.drag_stopped() && cfg.transform_in_progress {
                            if let Some(ent) = cfg.transform_old_entity.take()
                                && let Some(orig) = cfg.transform_original_transform.take()
                            {
                                UndoableAction::push_to_undo(
                                    undo_stack,
                                    UndoableAction::Transform(ent, orig),
                                );
                                log::debug!("Pushed X scale change to undo stack");
                            }
                            cfg.transform_in_progress = false;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Y:");
                        let y_slider = egui::DragValue::new(&mut new_scale.y)
                            .speed(0.01)
                            .fixed_decimals(3);

                        let response = ui.add_enabled(!cfg.scale_locked, y_slider);

                        if response.drag_started() && !cfg.scale_locked {
                            cfg.transform_old_entity = Some(*entity);
                            cfg.transform_original_transform = Some(*transform);
                            cfg.transform_in_progress = true;
                        }

                        if response.changed() {
                            scale_changed = true;
                        }

                        if response.drag_stopped() && cfg.transform_in_progress {
                            if let Some(ent) = cfg.transform_old_entity.take()
                                && let Some(orig) = cfg.transform_original_transform.take()
                            {
                                UndoableAction::push_to_undo(
                                    undo_stack,
                                    UndoableAction::Transform(ent, orig),
                                );
                                log::debug!("Pushed Y scale change to undo stack");
                            }
                            cfg.transform_in_progress = false;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Z:");
                        let z_slider = egui::DragValue::new(&mut new_scale.z)
                            .speed(0.01)
                            .fixed_decimals(3);

                        let response = ui.add_enabled(!cfg.scale_locked, z_slider);

                        if response.drag_started() && !cfg.scale_locked {
                            cfg.transform_old_entity = Some(*entity);
                            cfg.transform_original_transform = Some(*transform);
                            cfg.transform_in_progress = true;
                        }

                        if response.changed() {
                            scale_changed = true;
                        }

                        if response.drag_stopped() && cfg.transform_in_progress {
                            if let Some(ent) = cfg.transform_old_entity.take()
                                && let Some(orig) = cfg.transform_original_transform.take()
                            {
                                UndoableAction::push_to_undo(
                                    undo_stack,
                                    UndoableAction::Transform(ent, orig),
                                );
                                log::debug!("Pushed Z scale change to undo stack");
                            }
                            cfg.transform_in_progress = false;
                        }
                    });

                    if scale_changed {
                        transform.scale = new_scale;
                    }

                    if ui.button("Reset Scale").clicked() {
                        transform.scale = DVec3::ONE;
                    }
                    ui.add_space(5.0);
                }
            });
    });
    ui.separator();
}

impl InspectableComponent for Transform {
    fn inspect(
        &mut self,
        entity: &mut Entity,
        cfg: &mut StaticallyKept,
        ui: &mut Ui,
        undo_stack: &mut Vec<UndoableAction>,
        _signal: &mut Signal,
        label: &mut String,
    ) {
        inspect_transform(self, entity, cfg, ui, undo_stack, label, true, true, true);
    }
}

impl InspectableComponent for Script {
    fn inspect(
        &mut self,
        _entity: &mut Entity,
        _cfg: &mut StaticallyKept,
        ui: &mut Ui,
        _undo_stack: &mut Vec<UndoableAction>,
        _signal: &mut Signal,
        _label: &mut String,
    ) {
        ui.vertical(|ui| {
            CollapsingHeader::new("Tags")
                .default_open(true)
                .show(ui, |ui| {
                    let mut local_del: Option<usize> = None;
                    for (i, tag) in self.tags.iter_mut().enumerate() {
                        let current_width = ui.available_width();
                        ui.horizontal(|ui| {
                            ui.add_sized(
                                [current_width * 70.0 / 100.0, 20.0],
                                TextEdit::singleline(tag),
                            );
                            if ui.button("🗑️").clicked() {
                                local_del = Some(i);
                            }
                        });
                    }
                    if let Some(i) = local_del {
                        self.tags.remove(i);
                    }
                    if ui.button("➕ Add").clicked() {
                        self.tags.push(String::new())
                    }
                });
        });
    }
}

impl InspectableComponent for eucalyptus_core::states::Label {
    fn inspect(
        &mut self,
        entity: &mut Entity,
        cfg: &mut StaticallyKept,
        ui: &mut Ui,
        undo_stack: &mut Vec<UndoableAction>,
        _signal: &mut Signal,
        _label: &mut String,
    ) {
        ui.horizontal(|ui| {
            ui.label("Name: ");

            let resp = ui.text_edit_singleline(self.as_mut_string());

            if resp.changed() {
                if cfg.old_label_entity.is_none() {
                    cfg.old_label_entity = Some(*entity);
                    cfg.label_original = Some(self.to_string());
                }
                cfg.label_last_edit = Some(Instant::now());
            }

            if resp.lost_focus() {
                if let Some(ent) = cfg.old_label_entity.take() {
                    if ent == *entity {
                        if let Some(orig) = cfg.label_original.take() {
                            UndoableAction::push_to_undo(
                                undo_stack,
                                UndoableAction::Label(ent, orig),
                            );
                            log::debug!("Pushed label change to undo stack (immediate)");
                        }
                    } else {
                        cfg.label_original = None;
                    }
                }
                cfg.label_last_edit = None;
            }
        });
    }
}

impl InspectableComponent for MeshRenderer {
    fn inspect(
        &mut self,
        entity: &mut Entity,
        cfg: &mut StaticallyKept,
        ui: &mut Ui,
        _undo_stack: &mut Vec<UndoableAction>,
        signal: &mut Signal,
        _label: &mut String,
    ) {
        fn is_probably_model_uri(uri: &str) -> bool {
            let uri = uri.to_ascii_lowercase();
            uri.ends_with(".glb")
                || uri.ends_with(".gltf")
                || uri.ends_with(".obj")
                || uri.ends_with(".fbx")
        }

        fn is_probably_texture_uri(uri: &str) -> bool {
            let uri = uri.to_ascii_lowercase();
            uri.ends_with(".png")
                || uri.ends_with(".jpg")
                || uri.ends_with(".jpeg")
                || uri.ends_with(".tga")
                || uri.ends_with(".bmp")
        }

        let model_reference = self.model().path.clone();
        let model_title = match &model_reference.ref_type {
            ResourceReferenceType::Cuboid { .. } => "Cuboid".to_string(),
            ResourceReferenceType::Unassigned { .. } => "None".to_string(),
            _ => self.handle().label.clone(),
        };

        ui.vertical(|ui| {
            let expand_id = ui.make_persistent_id(format!("mesh_renderer_expand_{:?}", entity));
            let mut expanded = ui.data_mut(|d| d.get_temp::<bool>(expand_id).unwrap_or(false));

            let mut selected_model: Option<AssetHandle> = None;
            let mut choose_proc_cuboid = false;
            let mut choose_none = false;

            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), 72.0),
                egui::Sense::click(),
            );

            let fill = if response.hovered() {
                ui.visuals().widgets.hovered.bg_fill
            } else {
                ui.visuals().widgets.inactive.bg_fill
            };

            ui.painter()
                .rect_filled(rect, 4.0, fill);
            ui.painter()
                .rect_stroke(
                    rect,
                    4.0,
                    ui.visuals().widgets.inactive.bg_stroke,
                    egui::StrokeKind::Inside,
                );

            let mut card_ui = ui.new_child(
                UiBuilder::new()
                    .layout(egui::Layout::top_down(egui::Align::Min))
                    .max_rect(rect),
            );

            card_ui.horizontal(|ui| {
                let arrow = if expanded { "🔽" } else { "▶️" };
                if ui.button(arrow).clicked() {
                    expanded = !expanded;
                }

                ui.vertical(|ui| {
                    ui.label(RichText::new(&model_title).strong());
                    ui.label(
                        RichText::new("Drop a model from the Asset Viewer")
                            .small()
                            .color(ui.visuals().weak_text_color()),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ComboBox::from_id_salt("mesh_renderer_model_picker")
                        .selected_text(&model_title)
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_label(
                                    matches!(
                                        model_reference.ref_type,
                                        ResourceReferenceType::Unassigned { .. }
                                    ),
                                    "None",
                                )
                                .clicked()
                            {
                                choose_none = true;
                            }

                            ui.separator();

                            if ui
                                .selectable_label(
                                    matches!(model_reference.ref_type, ResourceReferenceType::Cuboid { .. }),
                                    "Cuboid",
                                )
                                .clicked()
                            {
                                choose_proc_cuboid = true;
                            }

                            ui.separator();

                            for i in ASSET_REGISTRY.iter_model() {
                                if i.path.as_uri().is_none() {
                                    continue;
                                }

                                let is_selected = self.asset_handle() == *i.key();
                                if ui
                                    .selectable_label(is_selected, i.label.clone())
                                    .clicked()
                                {
                                    selected_model = Some(*i.key());
                                }
                            }
                        });
                });
            });

            let pointer_released = ui.input(|i| i.pointer.any_released());
            if pointer_released && response.hovered() {
                if let Some(asset) = cfg.dragged_asset.clone() {
                    if let Some(uri) = asset.path.as_uri() {
                        if is_probably_model_uri(uri) {
                            *signal = Signal::ReplaceModel(*entity, uri.to_string());
                            cfg.dragged_asset = None;
                        }
                    }
                }
            }

            ui.data_mut(|d| d.insert_temp(expand_id, expanded));

            if choose_proc_cuboid {
                let default_size = match &model_reference.ref_type {
                    ResourceReferenceType::Cuboid { size_bits } => [
                        f32::from_bits(size_bits[0]),
                        f32::from_bits(size_bits[1]),
                        f32::from_bits(size_bits[2]),
                    ],
                    _ => [1.0, 1.0, 1.0],
                };
                *signal = Signal::SetProceduralCuboid(*entity, default_size);
            } else if choose_none {
                *signal = Signal::ClearModel(*entity);
            } else if let Some(model) = selected_model {
                if let Err(e) = self.set_asset_handle(model) {
                    fatal!("Unable to swap model: {}", e);
                }
            }

            if expanded {
                ui.add_space(6.0);

                if let ResourceReferenceType::File(uri) = &model_reference.ref_type {
                    if is_probably_model_uri(uri) {
                        let mut import_scale = self.import_scale();
                        ui.horizontal(|ui| {
                            ui.label("Import Scale");
                            let resp = ui.add(
                                egui::DragValue::new(&mut import_scale)
                                    .speed(0.01)
                                    .range(0.0001..=10_000.0),
                            );

                            if resp.changed() {
                                self.set_import_scale(import_scale);
                            }

                            if ui.button("Reset").clicked() {
                                self.set_import_scale(1.0);
                            }
                        });
                        ui.add_space(6.0);
                    }
                }

                if let ResourceReferenceType::Cuboid { size_bits } = &self.model().path.ref_type {
                    let mut size = [
                        f32::from_bits(size_bits[0]),
                        f32::from_bits(size_bits[1]),
                        f32::from_bits(size_bits[2]),
                    ];

                    ui.label(RichText::new("Cuboid").strong());
                    ui.horizontal(|ui| {
                        ui.label("Extents:");
                        let mut changed = false;
                        ui.label("X");
                        changed |= ui
                            .add(DragValue::new(&mut size[0]).speed(0.05).range(0.01..=10_000.0))
                            .changed();
                        ui.label("Y");
                        changed |= ui
                            .add(DragValue::new(&mut size[1]).speed(0.05).range(0.01..=10_000.0))
                            .changed();
                        ui.label("Z");
                        changed |= ui
                            .add(DragValue::new(&mut size[2]).speed(0.05).range(0.01..=10_000.0))
                            .changed();

                        if changed {
                            *signal = Signal::UpdateProceduralCuboid(*entity, size);
                        }
                    });

                    ui.separator();
                }

                ui.label(RichText::new("Textures").strong());

                for material in self.model().materials.iter() {
                    let material_name = material.name.clone();
                    let mut tint_rgb = [material.tint[0], material.tint[1], material.tint[2]];
                    let mut wrap_mode = material.wrap_mode;
                    let mut uv_tiling = material.uv_tiling;
                    let texture_label = material
                        .texture_tag
                        .clone()
                        .and_then(|tag| if is_probably_texture_uri(&tag) { Some(tag) } else { None })
                        .unwrap_or_else(|| "(embedded)".to_string());

                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&material_name).strong());

                        let (slot_rect, slot_resp) = ui.allocate_exact_size(
                            egui::vec2(160.0, 22.0),
                            egui::Sense::click(),
                        );

                        let slot_fill = if slot_resp.hovered() {
                            ui.visuals().widgets.hovered.bg_fill
                        } else {
                            ui.visuals().widgets.inactive.bg_fill
                        };
                        ui.painter().rect_filled(slot_rect, 3.0, slot_fill);
                        ui.painter().rect_stroke(
                            slot_rect,
                            3.0,
                            ui.visuals().widgets.inactive.bg_stroke,
                            egui::StrokeKind::Inside,
                        );

                        let mut slot_ui = ui.new_child(
                            UiBuilder::new()
                                .layout(egui::Layout::left_to_right(egui::Align::Center))
                                .max_rect(slot_rect),
                        );
                        slot_ui.add_space(4.0);
                        slot_ui.label(
                            RichText::new(texture_label)
                                .small()
                                .color(slot_ui.visuals().weak_text_color()),
                        );

                        if ui.button("Remove").clicked() {
                            *signal = Signal::ClearMaterialTexture(*entity, material_name.clone());
                        }

                        let mut wrap_changed = false;
                        egui::ComboBox::from_id_salt(format!(
                            "mesh_renderer_wrap_{:?}_{}",
                            entity,
                            material_name
                        ))
                        .selected_text(match wrap_mode {
                            dropbear_engine::utils::TextureWrapMode::Repeat => "Repeat",
                            dropbear_engine::utils::TextureWrapMode::Clamp => "Clamp",
                        })
                        .show_ui(ui, |ui| {
                            wrap_changed |= ui
                                .selectable_value(
                                &mut wrap_mode,
                                dropbear_engine::utils::TextureWrapMode::Repeat,
                                "Repeat",
                                )
                                .changed();
                            wrap_changed |= ui
                                .selectable_value(
                                &mut wrap_mode,
                                dropbear_engine::utils::TextureWrapMode::Clamp,
                                "Clamp",
                                )
                                .changed();
                        });

                        if wrap_changed {
                            *signal = Signal::SetMaterialWrapMode(
                                *entity,
                                material_name.clone(),
                                wrap_mode,
                            );
                        }

                        if matches!(wrap_mode, dropbear_engine::utils::TextureWrapMode::Repeat) {
                            ui.label("Repeat");
                            let mut tiling_changed = ui
                                .add(DragValue::new(&mut uv_tiling[0]).speed(0.05).range(0.01..=10_000.0))
                                .changed();
                            ui.label("x");
                            tiling_changed |= ui
                                .add(DragValue::new(&mut uv_tiling[1]).speed(0.05).range(0.01..=10_000.0))
                                .changed();

                            if tiling_changed {
                                *signal = Signal::SetMaterialUvTiling(
                                    *entity,
                                    material_name.clone(),
                                    uv_tiling,
                                );
                            }
                        }

                        let colour_changed =
                            egui::color_picker::color_edit_button_rgb(ui, &mut tint_rgb)
                                .changed();
                        if colour_changed {
                            *signal = Signal::SetMaterialTint(
                                *entity,
                                material_name.clone(),
                                [tint_rgb[0], tint_rgb[1], tint_rgb[2], 1.0],
                            );
                        }

                        let pointer_released = ui.input(|i| i.pointer.any_released());
                        if pointer_released && slot_resp.hovered() {
                            if let Some(asset) = cfg.dragged_asset.clone() {
                                if let Some(uri) = asset.path.as_uri() {
                                    if is_probably_texture_uri(uri) {
                                        *signal = Signal::SetMaterialTexture(
                                            *entity,
                                            material_name.clone(),
                                            uri.to_string(),
                                            wrap_mode,
                                        );
                                        cfg.dragged_asset = None;
                                    }
                                }
                            }
                        }
                    });
                }
            }
        });

        ui.separator();
    }
}

impl InspectableComponent for Light {
    fn inspect(
        &mut self,
        entity: &mut Entity,
        cfg: &mut StaticallyKept,
        ui: &mut Ui,
        undo_stack: &mut Vec<UndoableAction>,
        _signal: &mut Signal,
        _label: &mut String,
    ) {
        CollapsingHeader::new("Light").default_open(true).show(ui, |ui| {
            ui.horizontal(|ui| {
                ComboBox::new("light_type", "Light Type")
                    .selected_text(self.light_component.light_type.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.light_component.light_type,
                            LightType::Directional,
                            "Directional",
                        );
                        ui.selectable_value(&mut self.light_component.light_type, LightType::Point, "Point");
                        ui.selectable_value(&mut self.light_component.light_type, LightType::Spot, "Spot");
                    });
            });
            
            ui.separator();
            
            inspect_light_transform(
                &mut self.transform,
                entity,
                cfg,
                ui,
                undo_stack,
                &self.light_component.light_type,
            );
            
            let is_point = matches!(self.light_component.light_type, LightType::Point);
            let is_spot = matches!(self.light_component.light_type, LightType::Spot);

            ui.separator();
            let mut colour = self.light_component.colour.clone().as_vec3().to_array();
            ui.horizontal(|ui| {
                ui.label("Colour");
                egui::color_picker::color_edit_button_rgb(ui, &mut colour)
            });
            self.light_component.colour = Vec3::from_array(colour).as_dvec3();

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Intensity");
                ui.add(egui::Slider::new(&mut self.light_component.intensity, 0.0..=10.0));
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.light_component.enabled, "Enabled");
                ui.checkbox(&mut self.light_component.visible, "Visible");
            });

            if is_spot || is_point {
                ui.separator();
                ui.horizontal(|ui| {
                    ComboBox::new("attenuation_range", "Range")
                        .selected_text(format!("Range {}", self.light_component.attenuation.range))
                        .show_ui(ui, |ui| {
                            for (preset, label) in ATTENUATION_PRESETS {
                                ui.selectable_value(&mut self.light_component.attenuation, *preset, *label);
                            }
                        });
                });
            }

            if is_spot {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Slider::new(&mut self.light_component.cutoff_angle, 1.0..=89.0)
                            .text("Inner")
                            .suffix("°")
                            .step_by(0.1),
                    );
                });

                ui.horizontal(|ui| {
                    ui.add(
                        egui::Slider::new(&mut self.light_component.outer_cutoff_angle, 1.0..=90.0)
                            .text("Outer")
                            .suffix("°")
                            .step_by(0.1),
                    );
                });

                if self.light_component.outer_cutoff_angle <= self.light_component.cutoff_angle {
                    self.light_component.outer_cutoff_angle = self.light_component.cutoff_angle + 1.0;
                }

                let cone_softness = self.light_component.outer_cutoff_angle - self.light_component.cutoff_angle;
                ui.label(format!("Soft edge: {:.1}°", cone_softness));
            }

            ui.separator();

            ui.label("Shadows");
            ui.checkbox(&mut self.light_component.cast_shadows, "Cast Shadows");
            ui.horizontal(|ui| {
                ui.label("Depth");
                ui.add(egui::DragValue::new(&mut self.light_component.depth.start).speed(0.1));
                ui.label("..");
                ui.add(egui::DragValue::new(&mut self.light_component.depth.end).speed(0.1));
            });

            if self.light_component.depth.end < self.light_component.depth.start {
                self.light_component.depth.end = self.light_component.depth.start;
            }

            ui.separator();
        });
    }
}

impl InspectableComponent for Camera3D {
    fn inspect(
        &mut self,
        entity: &mut Entity,
        cfg: &mut StaticallyKept,
        ui: &mut Ui,
        undo_stack: &mut Vec<UndoableAction>,
        signal: &mut Signal,
        label: &mut String,
    ) {
        self.transform
            .inspect(entity, cfg, ui, undo_stack, signal, label);

        ui.vertical(|ui| {
            CollapsingHeader::new("Camera Settings")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Type");
                        ComboBox::from_id_salt("camera_type")
                            .selected_text(format!("{:?}", self.camera_type))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.camera_type,
                                    CameraType::Normal,
                                    "Normal",
                                );
                                ui.selectable_value(
                                    &mut self.camera_type,
                                    CameraType::Debug,
                                    "Debug",
                                );
                                ui.selectable_value(
                                    &mut self.camera_type,
                                    CameraType::Player,
                                    "Player",
                                );
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label("FOV");
                        ui.add(egui::Slider::new(&mut self.fov, 1.0..=179.0).suffix("°"));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Near Plane");
                        ui.add(
                            egui::DragValue::new(&mut self.near)
                                .speed(0.1)
                                .range(0.01..=1000.0),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Far Plane");
                        ui.add(
                            egui::DragValue::new(&mut self.far)
                                .speed(1.0)
                                .range(0.1..=10000.0),
                        );
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Speed");
                        ui.add(egui::DragValue::new(&mut self.speed).speed(0.1));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Sensitivity");
                        ui.add(egui::DragValue::new(&mut self.sensitivity).speed(0.01));
                    });

                    ui.separator();

                    ui.checkbox(&mut self.starting_camera, "Starting Camera");
                });
        });
        ui.separator();
    }
}
