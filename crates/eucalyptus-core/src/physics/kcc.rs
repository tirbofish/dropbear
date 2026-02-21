//! Module that relates to the [Kinematic Character Controller](https://rapier.rs/docs/user_guides/rust/character_controller)
//! (or kcc for short) in the [rapier3d] physics engine.

pub mod character_collision;

use crate::component::{Component, ComponentDescriptor, InspectableComponent, SerializedComponent};
use crate::physics::PhysicsState;
use crate::ptr::{WorldPtr};
use crate::scripting::jni::utils::ToJObject;
use crate::scripting::native::DropbearNativeError;
use crate::scripting::result::DropbearNativeResult;
use crate::states::Label;
use crate::types::{IndexNative, NQuaternion, NVector3};
use dropbear_engine::graphics::SharedGraphicsContext;
use egui::{ComboBox, DragValue, Ui};
use hecs::{Entity, World};
use jni::JNIEnv;
use jni::objects::{JObject, JValue};
use rapier3d::control::{CharacterAutostep, CharacterCollision, CharacterLength, KinematicCharacterController};
use rapier3d::dynamics::RigidBodyType;
use rapier3d::math::Rotation;
use rapier3d::na::{UnitVector3, Vector3};
use rapier3d::prelude::QueryFilter;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
    pub translation: NVector3,
    pub grounded: bool,
    pub is_sliding_down_slope: bool,
}

impl ToJObject for CharacterMovementResult {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .find_class("com/dropbear/physics/CharacterMovementResult")
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let translation_obj = self.translation.to_jobject(env)?;

        let args = [
            JValue::Object(&translation_obj),
            JValue::Bool(self.grounded as u8),
            JValue::Bool(self.is_sliding_down_slope as u8),
        ];

        let obj = env
            .new_object(
                &class,
                "(Lcom/dropbear/math/Vector3d;ZZ)V",
                &args,
            )
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        Ok(obj)
    }
}

#[typetag::serde]
impl SerializedComponent for KCC {}

impl Component for KCC {
    type SerializedForm = Self;
    type RequiredComponentTypes = (Self,);

    fn descriptor() -> ComponentDescriptor {
        ComponentDescriptor {
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
        _entity: Entity,
        ui: &mut Ui,
        _graphics: Arc<SharedGraphicsContext>,
    ) {
        egui::CollapsingHeader::new("Kinematic Character Controller")
            .default_open(true)
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
    collisions: Vec<IndexNative>,
}

impl ToJObject for CharacterCollisionArray {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let collision_cls = env
            .find_class("com/dropbear/physics/CharacterCollision")
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let entity_cls = env
            .find_class("com/dropbear/EntityId")
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let entity_obj = env
            .new_object(&entity_cls, "(J)V", &[JValue::Long(self.entity_id as i64)])
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        let out = env
            .new_object_array(
                self.collisions.len() as i32,
                &collision_cls,
                JObject::null(),
            )
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        for (i, handle) in self.collisions.iter().enumerate() {
            let index_obj = handle.to_jobject(env)?;
            let collision_obj = env
                .new_object(
                    &collision_cls,
                    "(Lcom/dropbear/EntityId;Lcom/dropbear/physics/Index;)V",
                    &[JValue::Object(&entity_obj), JValue::Object(&index_obj)],
                )
                .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

            env.set_object_array_element(&out, i as i32, collision_obj)
                .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;
        }

        Ok(JObject::from(out))
    }
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.KinematicCharacterControllerNative",
        func = "existsForEntity"
    ),
    c
)]
fn kcc_exists_for_entity(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<bool> {
    Ok(world.get::<&KCC>(entity).is_ok())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.KinematicCharacterControllerNative",
        func = "moveCharacter"
    ),
    c
)]
fn move_character(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::define(crate::ptr::PhysicsStatePtr)] physics_state: &mut PhysicsState,
    #[dropbear_macro::entity] entity: hecs::Entity,
    translation: &NVector3,
    delta_time: f64,
) -> DropbearNativeResult<()> {
    if let Ok((label, kcc)) = world.query_one::<(&Label, &mut KCC)>(entity).get() {
        let rigid_body_handle = physics_state
            .bodies_entity_map
            .get(label)
            .ok_or(DropbearNativeError::NoSuchHandle)?;

        let (body_type, body_pos) = {
            let body = physics_state
                .bodies
                .get(*rigid_body_handle)
                .ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
            (body.body_type(), *body.position())
        };

        if body_type != RigidBodyType::KinematicPositionBased {
            return Ok(()); // soft error, just tell the user
        }

        let collider_handles = physics_state
            .colliders_entity_map
            .get(label)
            .ok_or(DropbearNativeError::NoSuchHandle)?;
        let (_, collider_handle) = collider_handles
            .first()
            .ok_or(DropbearNativeError::NoSuchHandle)?;
        let collider = physics_state
            .colliders
            .get(*collider_handle)
            .ok_or(DropbearNativeError::PhysicsObjectNotFound)?;

        let character_pos = if let Some(pos_wrt_parent) = collider.position_wrt_parent() {
            body_pos * (*pos_wrt_parent)
        } else {
            *collider.position()
        };

        let filter = QueryFilter::default().exclude_rigid_body(*rigid_body_handle);
        let query_pipeline = physics_state.broad_phase.as_query_pipeline(
            physics_state.narrow_phase.query_dispatcher(),
            &physics_state.bodies,
            &physics_state.colliders,
            filter,
        );

        let movement = kcc.controller.move_shape(
            delta_time as f32,
            &query_pipeline,
            collider.shape(),
            &character_pos,
            rapier3d::prelude::Vector::new(
                translation.x as f32,
                translation.y as f32,
                translation.z as f32,
            ),
            |collision| {
                if let Some(collisions) =
                    physics_state.collision_events_to_deal_with.get_mut(&entity)
                {
                    collisions.push(collision)
                } else {
                    physics_state
                        .collision_events_to_deal_with
                        .insert(entity, vec![collision]);
                }
            },
        );

        if let Some(body) = physics_state.bodies.get_mut(*rigid_body_handle) {
            let current_pos = body.translation();
            let new_pos = current_pos + movement.translation;
            body.set_next_kinematic_translation(new_pos);
        }

        kcc.movement = Some(CharacterMovementResult {
            translation: movement.translation.into(),
            grounded: movement.grounded,
            is_sliding_down_slope: movement.is_sliding_down_slope,
        });

        Ok(())
    } else {
        Err(DropbearNativeError::MissingComponent)
    }
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.KinematicCharacterControllerNative",
        func = "setRotation"
    ),
    c
)]
fn set_rotation(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::define(crate::ptr::PhysicsStatePtr)] physics_state: &mut PhysicsState,
    #[dropbear_macro::entity] entity: hecs::Entity,
    rotation: &NQuaternion,
) -> DropbearNativeResult<()> {
    if let Ok((label, _)) = world.query_one::<(&Label, &KCC)>(entity).get() {
        let rigid_body_handle = physics_state
            .bodies_entity_map
            .get(label)
            .ok_or(DropbearNativeError::NoSuchHandle)?;

        let body_type = {
            let body = physics_state
                .bodies
                .get(*rigid_body_handle)
                .ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
            body.body_type()
        };

        if body_type != RigidBodyType::KinematicPositionBased {
            return Err(DropbearNativeError::InvalidArgument);
        }

        let len = (rotation.x * rotation.x
            + rotation.y * rotation.y
            + rotation.z * rotation.z
            + rotation.w * rotation.w)
            .sqrt();
        let (x, y, z, w) = if len > 0.0 {
            (
                rotation.x / len,
                rotation.y / len,
                rotation.z / len,
                rotation.w / len,
            )
        } else {
            (0.0, 0.0, 0.0, 1.0)
        };

        if let Some(body) = physics_state.bodies.get_mut(*rigid_body_handle) {
            let rot = Rotation::from_xyzw(x as f32, y as f32, z as f32, w as f32);
            body.set_next_kinematic_rotation(rot);
        }

        Ok(())
    } else {
        Err(DropbearNativeError::MissingComponent)
    }
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.KinematicCharacterControllerNative",
        func = "getHit"
    ),
    c
)]
fn get_hit(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<CharacterCollisionArray> {
    let kcc = world
        .get::<&KCC>(entity)
        .map_err(|_| DropbearNativeError::NoSuchComponent)?;

    let mut collisions = Vec::with_capacity(kcc.collisions.len());
    for collision in &kcc.collisions {
        let (idx, generation) = collision.handle.into_raw_parts();
        collisions.push(IndexNative {
            index: idx,
            generation,
        });
    }

    Ok(CharacterCollisionArray {
        entity_id: entity.to_bits().get(),
        collisions,
    })
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.physics.KinematicCharacterControllerNative",
        func = "getMovementResult"
    ),
    c
)]
fn get_movement_result(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<Option<CharacterMovementResult>> {
    world.get::<&KCC>(entity).map(|kcc| kcc.movement.clone()).map(Ok).unwrap_or(Ok(None))
}