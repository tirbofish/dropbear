use crate::component::{
    Component, ComponentDescriptor, ComponentInitFuture, DisabilityFlags, InspectableComponent,
    SerializedComponent,
};
use crate::states::Property;
use crate::warn;
use dropbear_engine::graphics::SharedGraphicsContext;
use egui::{CollapsingHeader, ComboBox, DragValue, Grid, RichText, TextEdit, Ui};
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

/// Properties for an entity, typically queries with `entity.getProperty<Float>` and `entity.setProperty(21)`
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CustomProperties {
    pub custom_properties: Vec<Property>,
    pub next_id: u64,
}

#[typetag::serde]
impl SerializedComponent for CustomProperties {}

impl Component for CustomProperties {
    type SerializedForm = Self;
    type RequiredComponentTypes = (Self,);

    fn descriptor() -> ComponentDescriptor {
        ComponentDescriptor {
            disabled_flags: DisabilityFlags::Disabled,
            internal: false,
            fqtn: "eucalyptus_core::properties::CustomProperties".to_string(),
            type_name: "CustomProperties".to_string(),
            category: Some("Properties".to_string()),
            description: Some("Custom properties for an entity".to_string()),
        }
    }

    fn init(
        ser: &Self::SerializedForm,
        _graphics: Arc<SharedGraphicsContext>,
    ) -> ComponentInitFuture<'_, Self> {
        Box::pin(async move { Ok((ser.clone(),)) })
    }

    fn update_component(
        &mut self,
        _world: &World,
        _physics: &mut crate::physics::PhysicsState,
        _entity: Entity,
        _dt: f32,
        _graphics: Arc<SharedGraphicsContext>,
    ) {
    }

    fn save(&self, _world: &World, _entity: Entity) -> Box<dyn SerializedComponent> {
        Box::new(self.clone())
    }
}

impl InspectableComponent for CustomProperties {
    fn inspect(
        &mut self,
        _world: &World,
        entity: Entity,
        ui: &mut Ui,
        _graphics: Arc<SharedGraphicsContext>,
    ) {
        CollapsingHeader::new("Custom Properties")
            .default_open(true)
            .id_salt(format!("Custom Properties {}", entity.to_bits()))
            .show(ui, |ui| {
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
                                    ui.selectable_value(
                                        &mut selected_type,
                                        ValueType::Float,
                                        "Float",
                                    );
                                    ui.selectable_value(&mut selected_type, ValueType::Int, "Int");
                                    ui.selectable_value(
                                        &mut selected_type,
                                        ValueType::Bool,
                                        "Bool",
                                    );
                                    ui.selectable_value(
                                        &mut selected_type,
                                        ValueType::Vec3,
                                        "Vec3",
                                    );
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
            });
    }
}

impl CustomProperties {
    /// Creates a new [CustomProperties]
    pub fn new() -> Self {
        Self {
            custom_properties: Vec::new(),
            next_id: 0,
        }
    }

    /// Sets the property based on the [Value] (type) and its key.
    ///
    /// If the value does NOT exist, it will be created.
    /// If the value does exist, it will replace the contents of that item.
    pub fn set_property(&mut self, key: String, value: Value) {
        if let Some(prop) = self.custom_properties.iter_mut().find(|p| p.key == key) {
            prop.value = value;
        } else {
            self.custom_properties.push(Property {
                id: self.next_id,
                key,
                value,
            });
            self.next_id += 1;
        }
    }

    /// Fetches the property by its key.
    pub fn get_property(&self, key: &str) -> Option<&Value> {
        self.custom_properties
            .iter()
            .find(|p| p.key == key)
            .map(|p| &p.value)
    }

    /// Fetches the float property
    pub fn get_float(&self, key: &str) -> Option<f64> {
        match self.get_property(key)? {
            Value::Double(f) => Some(*f),
            _ => None,
        }
    }

    /// Fetches the integer property
    pub fn get_int(&self, key: &str) -> Option<i64> {
        match self.get_property(key)? {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// Creates a new property based on a key and a value.
    ///
    /// It will push that value again to the property vector.
    pub fn add_property(&mut self, key: String, value: Value) {
        self.custom_properties.push(Property {
            id: self.next_id,
            key,
            value,
        });
        self.next_id += 1;
    }

    /// Shows a template of the different values when inspected as a component in the editor.
    pub fn show_value_editor(ui: &mut Ui, value: &mut Value) -> bool {
        match value {
            Value::String(s) => ui.text_edit_singleline(s).changed(),
            Value::Int(i) => ui
                .add(egui::Slider::new(i, -1000..=1000).text(""))
                .changed(),
            Value::Double(f) => ui
                .add(egui::Slider::new(f, -100.0..=100.0).text(""))
                .changed(),
            Value::Bool(b) => ui.checkbox(b, "").changed(),
            Value::Vec3(vec) => {
                let mut changed = false;
                ui.horizontal(|ui| {
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut vec[0], -10.0..=10.0)
                                .text("X")
                                .fixed_decimals(2),
                        )
                        .changed();
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut vec[1], -10.0..=10.0)
                                .text("Y")
                                .fixed_decimals(2),
                        )
                        .changed();
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut vec[2], -10.0..=10.0)
                                .text("Z")
                                .fixed_decimals(2),
                        )
                        .changed();
                });
                changed
            }
        }
    }
}

impl Default for CustomProperties {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Value {
    String(String),
    Int(i64),
    Double(f64),
    Bool(bool),
    Vec3([f64; 3]),
}

impl Default for Value {
    fn default() -> Self {
        Self::String(String::new())
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let string: String = match self {
            Value::String(_) => "String".into(),
            Value::Int(_) => "Int".into(),
            Value::Double(_) => "Double".into(),
            Value::Bool(_) => "Bool".into(),
            Value::Vec3(_) => "Vec3".into(),
        };
        write!(f, "{}", string)
    }
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

pub mod shared {
    use crate::properties::CustomProperties;
    use hecs::World;

    pub fn custom_properties_exists_for_entity(world: &World, entity: hecs::Entity) -> bool {
        world.get::<&CustomProperties>(entity).is_ok()
    }
}

