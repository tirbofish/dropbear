use std::fmt::{Display, Formatter};
use crate::component::{Component, ComponentDescriptor, ComponentInitFuture, DisabilityFlags, InspectableComponent, SerializedComponent};
use crate::hierarchy::EntityTransformExt;
use crate::ptr::WorldPtr;
use crate::scripting::jni::utils::{FromJObject, ToJObject};
use crate::scripting::native::DropbearNativeError;
use crate::scripting::result::DropbearNativeResult;
use crate::types::{NTransform, NVector3};
use ::jni::JNIEnv;
use ::jni::objects::{JObject, JValue};
use dropbear_engine::entity::{EntityTransform, Transform};
use dropbear_engine::graphics::SharedGraphicsContext;
use egui::{CollapsingHeader, ComboBox, Ui};
use glam::{DQuat, DVec3, Vec3};
use hecs::{Entity, World};
use splines::{Interpolation, Key, Spline};
use std::sync::Arc;
use crate::physics::PhysicsState;

/// Allows for the entity to have constricted movement, such as a Camera that follows a player
/// on a set path.
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OnRails {
    /// Whether the component actively moves the entity along the path each frame.
    pub enabled: bool,
    pub path: Vec<DVec3>,
    /// Progress of the entity on the spline between 0.0 to 1.0.
    pub progress: f32, // between 0.0 to 1.0
    pub drive: RailDrive,
    /// Allows for showing the debug path of the rail.
    #[serde(skip)]
    pub debug: bool,
}

/// Returns the progress `t` (0.0–1.0) of the path point closest to `pos`.
fn project_to_path(path: &[DVec3], pos: DVec3) -> f32 {
    let n = path.len();
    if n < 2 {
        return 0.0;
    }
    let mut best_t = 0.0f32;
    let mut best_dist_sq = f64::MAX;
    for i in 0..(n - 1) {
        let a = path[i];
        let b = path[i + 1];
        let t_a = i as f32 / (n - 1) as f32;
        let t_b = (i + 1) as f32 / (n - 1) as f32;
        let ab = b - a;
        let ab_len_sq = ab.length_squared();
        let (seg_t, proj_pos) = if ab_len_sq < 1e-10 {
            (t_a, a)
        } else {
            let frac = ((pos - a).dot(ab) / ab_len_sq).clamp(0.0, 1.0);
            (t_a + frac as f32 * (t_b - t_a), a + ab * frac)
        };
        let dist_sq = pos.distance_squared(proj_pos);
        if dist_sq < best_dist_sq {
            best_dist_sq = dist_sq;
            best_t = seg_t;
        }
    }
    best_t
}

/// Linearly samples the path at `t` in [0, 1]. Matches the `Interpolation::Linear` splines used
/// in `update_component`, so the editor gizmo stays consistent with runtime behaviour.
fn sample_path_linear(path: &[DVec3], t: f32) -> DVec3 {
    let n = path.len();
    if n == 0 {
        return DVec3::ZERO;
    }
    if n == 1 {
        return path[0];
    }
    let t = t.clamp(0.0, 1.0) as f64;
    let seg_f = t * (n - 1) as f64;
    let i = (seg_f as usize).min(n - 2);
    let frac = seg_f - i as f64;
    path[i].lerp(path[i + 1], frac)
}

/// How the progression of the [`OnRails`] component is handled.
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RailDrive {
    /// Progress advances automatically at a fixed speed.
    ///
    /// Use for cutscenes, intros, or automated camera pans.
    Automatic {
        speed: f32,
        looping: bool,
    },

    /// Progress is tied to the closest point on the rail to a target entity.
    ///
    /// Use for third-person or side-scroller cameras following a player.
    FollowEntity {
        target: Entity,
        /// If true, progress can never decrease and the camera won't scroll backwards.
        monotonic: bool
    },

    /// Progress is driven by a specific axis of the target entity's world position.
    ///
    /// e.g. AxisDriven { target: player, axis: Vec3::X } scrolls as the player moves right.
    ///
    /// Use for side-scrollers or top-down cameras with a dominant movement axis.
    AxisDriven {
        target: Entity,
        axis: Vec3,
        /// Maps world units along the axis to 0.0..1.0 progress.
        range: (f32, f32),
    },

    /// Progress is set entirely by external code, such as scripting
    ///
    /// The system won't touch progress at all.
    #[default]
    Manual,
}

impl Display for RailDrive {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            RailDrive::Automatic { .. } => "Automatic",
            RailDrive::FollowEntity { .. } => "FollowEntity",
            RailDrive::AxisDriven { .. } => "AxisDriven",
            RailDrive::Manual => "Manual",
        })
    }
}

#[typetag::serde]
impl SerializedComponent for OnRails {}

impl Component for OnRails {
    type SerializedForm = Self;
    type RequiredComponentTypes = (Self, EntityTransform);

    fn descriptor() -> ComponentDescriptor {
        ComponentDescriptor {
            fqtn: "eucalyptus_core::transform::OnRails".to_string(),
            type_name: "OnRails".to_string(),
            category: Some("Transform".to_string()),
            description: Some("Moves the entity along a fixed path".to_string()),
            disabled_flags: DisabilityFlags::Disabled,
            internal: false,
        }
    }

    fn init(ser: &'_ Self::SerializedForm, _graphics: Arc<SharedGraphicsContext>) -> ComponentInitFuture<'_, Self> {
        Box::pin(async move { Ok((ser.clone(), EntityTransform::default())) })
    }

    fn update_component(&mut self, world: &World, _physics: &mut PhysicsState, entity: Entity, dt: f32, _graphics: Arc<SharedGraphicsContext>) {
        if !self.enabled {
            return;
        }
        let n = self.path.len();
        if n < 2 {
            return;
        }

        let make_spline = |get: fn(&DVec3) -> f64| -> Spline<f64, f64> {
            Spline::from_vec(
                self.path
                    .iter()
                    .enumerate()
                    .map(|(i, p)| Key::new(i as f64 / (n - 1) as f64, get(p), Interpolation::Linear))
                    .collect(),
            )
        };
        let sx = make_spline(|p| p.x);
        let sy = make_spline(|p| p.y);
        let sz = make_spline(|p| p.z);

        match &self.drive {
            RailDrive::Automatic { speed, looping } => {
                let (speed, looping) = (*speed, *looping);
                self.progress += speed * dt;
                if looping {
                    self.progress = self.progress.rem_euclid(1.0);
                } else {
                    self.progress = self.progress.clamp(0.0, 1.0);
                }
            }
            RailDrive::FollowEntity { target, monotonic } => {
                let (target, monotonic) = (*target, *monotonic);
                let Ok(target_et) = world.get::<&EntityTransform>(target) else {
                    return;
                };
                let target_pos = target_et.world().position;
                drop(target_et);

                let mut best_t = self.progress;
                let mut best_dist_sq = f64::MAX;
                for i in 0..(n - 1) {
                    let a = self.path[i];
                    let b = self.path[i + 1];
                    let t_a = i as f32 / (n - 1) as f32;
                    let t_b = (i + 1) as f32 / (n - 1) as f32;
                    let ab = b - a;
                    let ab_len_sq = ab.length_squared();
                    let (seg_t, proj_pos) = if ab_len_sq < 1e-10 {
                        (t_a, a)
                    } else {
                        let frac = ((target_pos - a).dot(ab) / ab_len_sq).clamp(0.0, 1.0);
                        (t_a + frac as f32 * (t_b - t_a), a + ab * frac)
                    };
                    let dist_sq = target_pos.distance_squared(proj_pos);
                    if dist_sq < best_dist_sq {
                        best_dist_sq = dist_sq;
                        best_t = seg_t;
                    }
                }
                self.progress = if monotonic {
                    self.progress.max(best_t)
                } else {
                    best_t
                };
            }
            RailDrive::AxisDriven { target, axis, range } => {
                let (target, axis, range) = (*target, *axis, *range);
                let Ok(target_et) = world.get::<&EntityTransform>(target) else {
                    return;
                };
                let target_pos = target_et.world().position.as_vec3();
                drop(target_et);

                let proj = target_pos.dot(axis);
                self.progress = ((proj - range.0) / (range.1 - range.0)).clamp(0.0, 1.0);
            }
            RailDrive::Manual => {}
        }

        let t = self.progress as f64;
        if let (Some(x), Some(y), Some(z)) = (
            sx.clamped_sample(t),
            sy.clamped_sample(t),
            sz.clamped_sample(t),
        ) {
            if let Ok(mut et) = world.get::<&mut EntityTransform>(entity) {
                et.world_mut().position = DVec3::new(x, y, z);
            }
        }
    }

    fn save(&self, _world: &World, _entity: Entity) -> Box<dyn SerializedComponent> {
        Box::new(self.clone())
    }
}

impl InspectableComponent for OnRails {
    fn inspect(
        &mut self,
        world: &World,
        entity: Entity,
        ui: &mut Ui,
        _graphics: Arc<SharedGraphicsContext>,
    ) {
        let n = self.path.len();

        // if the entity is dragged off the path, it will 
        if self.enabled && n >= 2 {
            let expected = sample_path_linear(&self.path, self.progress);
            let current_pos = world
                .get::<&EntityTransform>(entity)
                .ok()
                .map(|et| et.world().position);
            if let Some(pos) = current_pos {
                if pos.distance_squared(expected) > 1e-8 {
                    self.progress = project_to_path(&self.path, pos);
                    let snapped = sample_path_linear(&self.path, self.progress);
                    if let Ok(mut et) = world.get::<&mut EntityTransform>(entity) {
                        et.world_mut().position = snapped;
                    }
                }
            }
        }

        CollapsingHeader::new("OnRails")
            .default_open(true)
            .id_salt(format!("OnRails {}", entity.to_bits()))
            .show(ui, |ui| {
                ui.checkbox(&mut self.enabled, "Enabled");
                ui.add_space(4.0);

                let slider = ui.add(
                    egui::Slider::new(&mut self.progress, 0.0..=1.0)
                        .text("Progress")
                        .min_decimals(2)
                        .max_decimals(4),
                );
                if slider.changed() && n >= 2 {
                    let pos = sample_path_linear(&self.path, self.progress);
                    if let Ok(mut et) = world.get::<&mut EntityTransform>(entity) {
                        et.world_mut().position = pos;
                    }
                }

                ui.add_space(4.0);

                let mut drive_tag: i32 = match &self.drive {
                    RailDrive::Automatic { .. } => 0,
                    RailDrive::FollowEntity { .. } => 1,
                    RailDrive::AxisDriven { .. } => 2,
                    RailDrive::Manual => 3,
                };
                let prev_tag = drive_tag;
                ComboBox::new(format!("OnRails ComboBox {}", entity.to_bits()), "Drive")
                    .selected_text(format!("{}", self.drive))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut drive_tag, 0, "Automatic");
                        ui.selectable_value(&mut drive_tag, 1, "FollowEntity");
                        ui.selectable_value(&mut drive_tag, 2, "AxisDriven");
                        ui.selectable_value(&mut drive_tag, 3, "Manual");
                    });
                if drive_tag != prev_tag {
                    self.drive = match drive_tag {
                        0 => RailDrive::Automatic { speed: 0.1, looping: false },
                        1 => RailDrive::FollowEntity { target: entity, monotonic: false },
                        2 => RailDrive::AxisDriven {
                            target: entity,
                            axis: Vec3::X,
                            range: (0.0, 100.0),
                        },
                        _ => RailDrive::Manual,
                    };
                }

                ui.add_space(4.0);

                match &mut self.drive {
                    RailDrive::Automatic { speed, looping } => {
                        ui.horizontal(|ui| {
                            ui.label("Speed:");
                            ui.add(egui::DragValue::new(speed).speed(0.001).range(0.0_f32..=f32::MAX));
                        });
                        ui.checkbox(looping, "Looping");
                    }
                    RailDrive::FollowEntity { target, monotonic } => {
                        ui.label(format!("Target entity: {}", target.to_bits()));
                        ui.checkbox(monotonic, "Monotonic");
                    }
                    RailDrive::AxisDriven { target, axis, range } => {
                        ui.label(format!("Target entity: {}", target.to_bits()));
                        ui.horizontal(|ui| {
                            ui.label("Axis:");
                            ui.add(egui::DragValue::new(&mut axis.x).speed(0.01).prefix("X:"));
                            ui.add(egui::DragValue::new(&mut axis.y).speed(0.01).prefix("Y:"));
                            ui.add(egui::DragValue::new(&mut axis.z).speed(0.01).prefix("Z:"));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Range:");
                            ui.add(egui::DragValue::new(&mut range.0).speed(0.1).prefix("Min:"));
                            ui.add(egui::DragValue::new(&mut range.1).speed(0.1).prefix("Max:"));
                        });
                    }
                    RailDrive::Manual => {
                        ui.label("Progress is set externally.");
                    }
                }

                ui.add_space(4.0);

                CollapsingHeader::new(format!("Path ({} points)", n))
                    .default_open(false)
                    .id_salt(format!("OnRails Path {}", entity.to_bits()))
                    .show(ui, |ui| {
                        let mut remove_idx: Option<usize> = None;
                        for (i, point) in self.path.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("[{}]", i));
                                ui.add(egui::DragValue::new(&mut point.x).speed(0.1).prefix("X:"));
                                ui.add(egui::DragValue::new(&mut point.y).speed(0.1).prefix("Y:"));
                                ui.add(egui::DragValue::new(&mut point.z).speed(0.1).prefix("Z:"));
                                if ui.small_button("-").clicked() {
                                    remove_idx = Some(i);
                                }
                            });
                        }
                        if let Some(idx) = remove_idx {
                            self.path.remove(idx);
                        }
                        if ui.button("+ Add Point").clicked() {
                            let last = self.path.last().copied().unwrap_or(DVec3::ZERO);
                            self.path.push(last + DVec3::new(1.0, 0.0, 0.0));
                        }
                    });

                ui.add_space(4.0);
                ui.checkbox(&mut self.debug, "Debug path");
            });
    }
}

#[typetag::serde]
impl SerializedComponent for EntityTransform {}

impl Component for EntityTransform {
    type SerializedForm = Self;
    type RequiredComponentTypes = (Self,);

    fn descriptor() -> ComponentDescriptor {
        ComponentDescriptor {
            disabled_flags: DisabilityFlags::Disabled,
            internal: false,
            fqtn: "dropbear_engine::entity::EntityTransform".to_string(),
            type_name: "EntityTransform".to_string(),
            category: Some("Transform".to_string()),
            description: Some("Allows the entity to have a space within the world".to_string()),
        }
    }

    fn init(
        ser: &Self::SerializedForm,
        _: Arc<SharedGraphicsContext>,
    ) -> crate::component::ComponentInitFuture<Self> {
        Box::pin(async move { Ok((ser.clone(),)) })
    }

    fn update_component(
        &mut self,
        _: &World,
        _physics: &mut crate::physics::PhysicsState,
        _: Entity,
        _: f32,
        _: Arc<SharedGraphicsContext>,
    ) {
    }

    fn save(&self, _: &World, _: Entity) -> Box<dyn SerializedComponent> {
        Box::new(self.clone())
    }
}

impl InspectableComponent for EntityTransform {
    fn inspect(
        &mut self,
        _world: &World,
        entity: Entity,
        ui: &mut Ui,
        _graphics: Arc<SharedGraphicsContext>,
    ) {
        CollapsingHeader::new("Entity Transform")
            .default_open(true)
            .id_salt(format!("Entity Transform {}", entity.to_bits()))
            .show(ui, |ui| {
            CollapsingHeader::new("Local").default_open(true).id_salt(format!("Local {}", entity.to_bits())).show(ui, |ui| {
                self.local_mut().inspect(ui);
            });
            ui.add_space(4.0);
            CollapsingHeader::new("World").default_open(true).id_salt(format!("World {}", entity.to_bits())).show(ui, |ui| {
                self.world_mut().inspect(ui);
            });
        });
    }
}

impl FromJObject for Transform {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self> {
        let pos_val = env
            .get_field(obj, "position", "Lcom/dropbear/math/Vector3d;")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?;

        let pos_obj = pos_val
            .l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let rot_val = env
            .get_field(obj, "rotation", "Lcom/dropbear/math/Quaterniond;")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?;

        let rot_obj = rot_val
            .l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let scale_val = env
            .get_field(obj, "scale", "Lcom/dropbear/math/Vector3d;")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?;

        let scale_obj = scale_val
            .l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let position: DVec3 = NVector3::from_jobject(env, &pos_obj)?.into();
        let scale: DVec3 = NVector3::from_jobject(env, &scale_obj)?.into();

        let mut get_double = |field: &str| -> DropbearNativeResult<f64> {
            env.get_field(&rot_obj, field, "D")
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
                .d()
                .map_err(|_| DropbearNativeError::JNIUnwrapFailed)
        };

        let rx = get_double("x")?;
        let ry = get_double("y")?;
        let rz = get_double("z")?;
        let rw = get_double("w")?;

        let rotation = DQuat::from_xyzw(rx, ry, rz, rw);

        Ok(Transform {
            position,
            rotation,
            scale,
        })
    }
}

impl ToJObject for Transform {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let cls = env.find_class("com/dropbear/math/Transform").map_err(|e| {
            eprintln!("Could not find Transform class: {:?}", e);
            DropbearNativeError::JNIClassNotFound
        })?;

        let p = self.position;
        let r = self.rotation;
        let s = self.scale;

        let args = [
            JValue::Double(p.x),
            JValue::Double(p.y),
            JValue::Double(p.z),
            JValue::Double(r.x),
            JValue::Double(r.y),
            JValue::Double(r.z),
            JValue::Double(r.w),
            JValue::Double(s.x),
            JValue::Double(s.y),
            JValue::Double(s.z),
        ];

        let obj = env.new_object(cls, "(DDDDDDDDDD)V", &args).map_err(|e| {
            eprintln!("Failed to create Transform object: {:?}", e);
            DropbearNativeError::JNIFailedToCreateObject
        })?;

        Ok(obj)
    }
}

impl FromJObject for EntityTransform {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self> {
        let local_val = env
            .get_field(obj, "local", "Lcom/dropbear/math/Transform;")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?;

        let local_obj = local_val
            .l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let world_val = env
            .get_field(obj, "world", "Lcom/dropbear/math/Transform;")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?;

        let world_obj = world_val
            .l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let local = Transform::from_jobject(env, &local_obj)?;
        let world = Transform::from_jobject(env, &world_obj)?;

        Ok(EntityTransform::new(local, world))
    }
}

impl ToJObject for EntityTransform {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let cls = env
            .find_class("com/dropbear/components/EntityTransform")
            .map_err(|e| {
                eprintln!("Could not find EntityTransform class: {:?}", e);
                DropbearNativeError::JNIClassNotFound
            })?;

        let local_obj = self.local().to_jobject(env)?;
        let world_obj = self.world().to_jobject(env)?;

        let args = [JValue::Object(&local_obj), JValue::Object(&world_obj)];

        let obj = env
            .new_object(
                cls,
                "(Lcom/dropbear/math/Transform;Lcom/dropbear/math/Transform;)V",
                &args,
            )
            .map_err(|e| {
                eprintln!("Failed to create EntityTransform object: {:?}", e);
                DropbearNativeError::JNIFailedToCreateObject
            })?;

        Ok(obj)
    }
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.EntityTransformNative",
        func = "entityTransformExistsForEntity"
    ),
    c
)]
fn exists_for_entity(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<bool> {
    Ok(world.get::<&EntityTransform>(entity).is_ok())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.EntityTransformNative",
        func = "getLocalTransform"
    ),
    c
)]
fn get_local_transform(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<NTransform> {
    if let Ok(et) = world.get::<&EntityTransform>(entity) {
        Ok((*et.local()).into())
    } else {
        Err(DropbearNativeError::MissingComponent)
    }
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.EntityTransformNative",
        func = "setLocalTransform"
    ),
    c
)]
fn set_local_transform(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    transform: &NTransform,
) -> DropbearNativeResult<()> {
    if let Ok(mut et) = world.get::<&mut EntityTransform>(entity) {
        *et.local_mut() = (*transform).into();

        Ok(())
    } else {
        Err(DropbearNativeError::MissingComponent)
    }
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.EntityTransformNative",
        func = "getWorldTransform"
    ),
    c
)]
fn get_world_transform(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<NTransform> {
    if let Ok(et) = world.get::<&EntityTransform>(entity) {
        Ok((*et.world()).into())
    } else {
        Err(DropbearNativeError::MissingComponent)
    }
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.EntityTransformNative",
        func = "setWorldTransform"
    ),
    c
)]
fn set_world_transform(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    transform: &NTransform,
) -> DropbearNativeResult<()> {
    if let Ok(mut et) = world.get::<&mut EntityTransform>(entity) {
        *et.world_mut() = (*transform).into();

        Ok(())
    } else {
        Err(DropbearNativeError::MissingComponent)
    }
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.EntityTransformNative",
        func = "propogateTransform"
    ),
    c
)]
fn propagate_transform(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<NTransform> {
    if let Ok(et) = world.get::<&mut EntityTransform>(entity) {
        let result = et.propagate(world, entity);
        Ok(result.into())
    } else {
        Err(DropbearNativeError::MissingComponent)
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "getEnabled"),
    c
)]
fn on_rails_get_enabled(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<bool> {
    let rails = world.get::<&OnRails>(entity).map_err(|_| DropbearNativeError::MissingComponent)?;
    Ok(rails.enabled)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "setEnabled"),
    c
)]
fn on_rails_set_enabled(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    enabled: bool,
) -> DropbearNativeResult<()> {
    let mut rails = world.get::<&mut OnRails>(entity).map_err(|_| DropbearNativeError::MissingComponent)?;
    rails.enabled = enabled;
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "existsForEntity"),
    c
)]
fn on_rails_exists_for_entity(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<bool> {
    Ok(world.get::<&OnRails>(entity).is_ok())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "getProgress"),
    c
)]
fn on_rails_get_progress(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<f32> {
    let rails = world.get::<&OnRails>(entity).map_err(|_| DropbearNativeError::MissingComponent)?;
    Ok(rails.progress)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "setProgress"),
    c
)]
fn on_rails_set_progress(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    progress: f32,
) -> DropbearNativeResult<()> {
    let mut rails = world.get::<&mut OnRails>(entity).map_err(|_| DropbearNativeError::MissingComponent)?;
    rails.progress = progress.clamp(0.0, 1.0);
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "getPathLen"),
    c
)]
fn on_rails_get_path_len(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<i32> {
    let rails = world.get::<&OnRails>(entity).map_err(|_| DropbearNativeError::MissingComponent)?;
    Ok(rails.path.len() as i32)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "getPathPoint"),
    c
)]
fn on_rails_get_path_point(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    index: i32,
) -> DropbearNativeResult<NVector3> {
    let rails = world.get::<&OnRails>(entity).map_err(|_| DropbearNativeError::MissingComponent)?;
    let point = rails.path.get(index as usize).ok_or(DropbearNativeError::InvalidArgument)?;
    Ok(NVector3::from(point))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "clearPath"),
    c
)]
fn on_rails_clear_path(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<()> {
    let mut rails = world.get::<&mut OnRails>(entity).map_err(|_| DropbearNativeError::MissingComponent)?;
    rails.path.clear();
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "pushPathPoint"),
    c
)]
fn on_rails_push_path_point(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    point: &NVector3,
) -> DropbearNativeResult<()> {
    let mut rails = world.get::<&mut OnRails>(entity).map_err(|_| DropbearNativeError::MissingComponent)?;
    rails.path.push(DVec3::from(point));
    Ok(())
}

// `0` = Automatic, `1` = FollowEntity, `2` = AxisDriven, `3` = Manual.
#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "getDriveType"),
    c
)]
fn on_rails_get_drive_type(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<i32> {
    let rails = world.get::<&OnRails>(entity).map_err(|_| DropbearNativeError::MissingComponent)?;
    let tag = match &rails.drive {
        RailDrive::Automatic { .. }    => 0,
        RailDrive::FollowEntity { .. } => 1,
        RailDrive::AxisDriven { .. }   => 2,
        RailDrive::Manual              => 3,
    };
    Ok(tag)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "setDriveAutomatic"),
    c
)]
fn on_rails_set_drive_automatic(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    speed: f32,
    looping: bool,
) -> DropbearNativeResult<()> {
    let mut rails = world.get::<&mut OnRails>(entity).map_err(|_| DropbearNativeError::MissingComponent)?;
    rails.drive = RailDrive::Automatic { speed, looping };
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "setDriveFollowEntity"),
    c
)]
fn on_rails_set_drive_follow_entity(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    target: u64,
    monotonic: bool,
) -> DropbearNativeResult<()> {
    let target_entity = hecs::Entity::from_bits(target).ok_or(DropbearNativeError::InvalidEntity)?;
    let mut rails = world.get::<&mut OnRails>(entity).map_err(|_| DropbearNativeError::MissingComponent)?;
    rails.drive = RailDrive::FollowEntity { target: target_entity, monotonic };
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "setDriveAxisDriven"),
    c
)]
fn on_rails_set_drive_axis_driven(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
    target: u64,
    axis: &NVector3,
    range_min: f32,
    range_max: f32,
) -> DropbearNativeResult<()> {
    let target_entity = hecs::Entity::from_bits(target).ok_or(DropbearNativeError::InvalidEntity)?;
    let mut rails = world.get::<&mut OnRails>(entity).map_err(|_| DropbearNativeError::MissingComponent)?;
    rails.drive = RailDrive::AxisDriven {
        target: target_entity,
        axis: Vec3::new(axis.x as f32, axis.y as f32, axis.z as f32),
        range: (range_min, range_max),
    };
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "setDriveManual"),
    c
)]
fn on_rails_set_drive_manual(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<()> {
    let mut rails = world.get::<&mut OnRails>(entity).map_err(|_| DropbearNativeError::MissingComponent)?;
    rails.drive = RailDrive::Manual;
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "getDriveAutomaticSpeed"),
    c
)]
fn on_rails_get_drive_automatic_speed(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<f32> {
    let rails = world.get::<&OnRails>(entity).map_err(|_| DropbearNativeError::MissingComponent)?;
    if let RailDrive::Automatic { speed, .. } = &rails.drive { Ok(*speed) } else { Err(DropbearNativeError::InvalidArgument) }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "getDriveAutomaticLooping"),
    c
)]
fn on_rails_get_drive_automatic_looping(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<bool> {
    let rails = world.get::<&OnRails>(entity).map_err(|_| DropbearNativeError::MissingComponent)?;
    if let RailDrive::Automatic { looping, .. } = &rails.drive { Ok(*looping) } else { Err(DropbearNativeError::InvalidArgument) }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "getDriveFollowEntityTarget"),
    c
)]
fn on_rails_get_drive_follow_entity_target(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<u64> {
    let rails = world.get::<&OnRails>(entity).map_err(|_| DropbearNativeError::MissingComponent)?;
    if let RailDrive::FollowEntity { target, .. } = &rails.drive { Ok(target.to_bits().get()) } else { Err(DropbearNativeError::InvalidArgument) }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "getDriveFollowEntityMonotonic"),
    c
)]
fn on_rails_get_drive_follow_entity_monotonic(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<bool> {
    let rails = world.get::<&OnRails>(entity).map_err(|_| DropbearNativeError::MissingComponent)?;
    if let RailDrive::FollowEntity { monotonic, .. } = &rails.drive { Ok(*monotonic) } else { Err(DropbearNativeError::InvalidArgument) }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "getDriveAxisDrivenTarget"),
    c
)]
fn on_rails_get_drive_axis_driven_target(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<u64> {
    let rails = world.get::<&OnRails>(entity).map_err(|_| DropbearNativeError::MissingComponent)?;
    if let RailDrive::AxisDriven { target, .. } = &rails.drive { Ok(target.to_bits().get()) } else { Err(DropbearNativeError::InvalidArgument) }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "getDriveAxisDrivenAxis"),
    c
)]
fn on_rails_get_drive_axis_driven_axis(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<NVector3> {
    let rails = world.get::<&OnRails>(entity).map_err(|_| DropbearNativeError::MissingComponent)?;
    if let RailDrive::AxisDriven { axis, .. } = &rails.drive { Ok(NVector3::from(*axis)) } else { Err(DropbearNativeError::InvalidArgument) }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "getDriveAxisDrivenRangeMin"),
    c
)]
fn on_rails_get_drive_axis_driven_range_min(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<f32> {
    let rails = world.get::<&OnRails>(entity).map_err(|_| DropbearNativeError::MissingComponent)?;
    if let RailDrive::AxisDriven { range, .. } = &rails.drive { Ok(range.0) } else { Err(DropbearNativeError::InvalidArgument) }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.components.camera.OnRailsNative", func = "getDriveAxisDrivenRangeMax"),
    c
)]
fn on_rails_get_drive_axis_driven_range_max(
    #[dropbear_macro::define(WorldPtr)] world: &hecs::World,
    #[dropbear_macro::entity] entity: hecs::Entity,
) -> DropbearNativeResult<f32> {
    let rails = world.get::<&OnRails>(entity).map_err(|_| DropbearNativeError::MissingComponent)?;
    if let RailDrive::AxisDriven { range, .. } = &rails.drive { Ok(range.1) } else { Err(DropbearNativeError::InvalidArgument) }
}
