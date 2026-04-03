//! Module that relates to the [Kinematic Character Controller](https://rapier.rs/docs/user_guides/rust/character_controller)
//! (or kcc for short) in the [rapier3d] physics engine.

use crate::component::{
    Component, ComponentDescriptor, DisabilityFlags, InspectableComponent, SerializedComponent,
};
use crate::physics::PhysicsState;
use crate::ptr::WorldPtr;
use crate::scripting::native::DropbearNativeError;
use crate::scripting::result::DropbearNativeResult;
use crate::states::Label;
use dropbear_engine::graphics::SharedGraphicsContext;
use egui::{ComboBox, DragValue, Ui};
use hecs::{Entity, World};
use jni::objects::{JObject, JValue};
use jni::{Env, jni_sig, jni_str};
use rapier3d::control::{
    CharacterAutostep, CharacterCollision, CharacterLength, KinematicCharacterController,
};
use rapier3d::dynamics::RigidBodyType;
use rapier3d::math::Rotation;
use rapier3d::na::{UnitVector3, Vector3};
use rapier3d::prelude::QueryFilter;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use glam::Vec3;
use rapier3d::data::Index;

/// The kinematic character controller (kcc) component.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct KCC {
    pub entity: Label,
    pub controller: KinematicCharacterController,
    #[serde(skip)]
    pub collisions: Vec<CharacterCollision>,
    #[serde(skip)]
    pub movement: Option<CharacterMovementResult>,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CharacterMovementResult {
    pub translation: Vec3,
    pub grounded: bool,
    pub is_sliding_down_slope: bool,
}

#[typetag::serde]
impl SerializedComponent for KCC {}

impl Component for KCC {
    type SerializedForm = Self;
    type RequiredComponentTypes = (Self,);

    fn descriptor() -> ComponentDescriptor {
        ComponentDescriptor {
            disabled_flags: DisabilityFlags::Disabled,
            internal: false,
            fqtn: "eucalyptus_core::physics::kcc::KCC".to_string(),
            type_name: "KinematicCharacterController".to_string(),
            category: Some("Physics".to_string()),
            description: Some("A kinematic character controller".to_string()),
        }
    }

    fn init(
        ser: &'_ Self::SerializedForm,
        _: Arc<SharedGraphicsContext>,
    ) -> crate::component::ComponentInitFuture<'_, Self> {
        Box::pin(async move { Ok((ser.clone(),)) })
    }

    fn update_component(
        &mut self,
        _world: &World,
        _physics: &mut PhysicsState,
        _entity: Entity,
        _dt: f32,
        _graphics: Arc<SharedGraphicsContext>,
    ) {
    }

    fn save(&self, _world: &World, _entity: Entity) -> Box<dyn SerializedComponent> {
        Box::new(self.clone())
    }
}

impl InspectableComponent for KCC {
    fn inspect(
        &mut self,
        _world: &World,
        entity: Entity,
        ui: &mut Ui,
        _graphics: Arc<SharedGraphicsContext>,
    ) {
        egui::CollapsingHeader::new("Kinematic Character Controller")
            .default_open(true)
            .id_salt(format!(
                "Kinematic Character Controller {}",
                entity.to_bits()
            ))
            .show(ui, |ui| {
                fn edit_character_length(
                    ui: &mut Ui,
                    id_salt: impl std::hash::Hash,
                    value: &mut CharacterLength,
                    text: &str,
                ) {
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
                    let up = self.controller.up;
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
                        self.controller.up = UnitVector3::new_normalize(up_v).into();
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
                        ui.add(
                            egui::Slider::new(
                                &mut self.controller.normal_nudge_factor,
                                0.001..=0.5,
                            )
                            .logarithmic(true)
                            .smallest_positive(0.001)
                            .largest_finite(0.5)
                            .suffix(" m"),
                        );
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
                        edit_character_length(
                            ui,
                            "kcc_autostep_max_height",
                            &mut step.max_height,
                            "Max height:",
                        );
                        edit_character_length(
                            ui,
                            "kcc_autostep_min_width",
                            &mut step.min_width,
                            "Min width:",
                        );
                        ui.checkbox(&mut step.include_dynamic_bodies, "Include dynamic bodies");
                    }
                });
            });
    }
}

impl KCC {
    pub fn new(label: &Label) -> Self {
        KCC {
            entity: label.clone(),
            controller: KinematicCharacterController::default(),
            collisions: vec![],
            movement: None,
        }
    }
}

#[repr(C)]
struct CharacterCollisionArray {
    entity_id: u64,
    collisions: Vec<Index>,
}
