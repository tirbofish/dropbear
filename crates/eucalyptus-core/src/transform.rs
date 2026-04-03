use crate::camera::{CameraComponent, CameraType};
use crate::component::{
    Component, ComponentDescriptor, ComponentInitFuture, DisabilityFlags, InspectableComponent,
    SerializedComponent,
};
use crate::physics::PhysicsState;
use dropbear_engine::camera::Camera;
use dropbear_engine::entity::{EntityTransform, inspect_rotation_dquat};
use dropbear_engine::graphics::SharedGraphicsContext;
use egui::{CollapsingHeader, ComboBox, Ui};
use glam::{DMat3, DQuat, DVec3, Vec3};
use hecs::{Entity, World};
use splines::{Interpolation, Key, Spline};
use std::fmt::{Display, Formatter};
use std::sync::Arc;

/// A single point on an [`OnRails`] path, capturing both position and an optional explicit
/// rotation. When `rotation` is `None`, the sampler derives orientation from the path tangent.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RailPoint {
    pub position: DVec3,
    /// Explicit look-at rotation captured from the camera. `None` means tangent-derived.
    pub rotation: Option<DQuat>,
}

impl Default for RailPoint {
    fn default() -> Self {
        Self {
            position: DVec3::ZERO,
            rotation: None,
        }
    }
}

/// Allows for the entity to have constricted movement, such as a Camera that follows a player
/// on a set path.
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OnRails {
    /// Whether the component actively moves the entity along the path each frame.
    pub enabled: bool,
    pub path: Vec<RailPoint>,
    /// Progress of the entity on the spline between 0.0 to 1.0.
    pub progress: f32, // between 0.0 to 1.0
    pub drive: RailDrive,
    /// Allows for showing the debug path of the rail.
    #[serde(skip)]
    pub debug: bool,
    /// Stores the desired position and rotation to be applied to [`EntityTransform`] after the
    /// component update loop, avoiding a double-borrow of the same archetype in hecs.
    #[serde(skip)]
    pub pending_transform: Option<(DVec3, DQuat)>,
}

/// Returns the progress `t` (0.0–1.0) of the path point closest to `pos`.
fn project_to_path(path: &[RailPoint], pos: DVec3) -> f32 {
    let n = path.len();
    if n < 2 {
        return 0.0;
    }
    let mut best_t = 0.0f32;
    let mut best_dist_sq = f64::MAX;
    for i in 0..(n - 1) {
        let a = path[i].position;
        let b = path[i + 1].position;
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

fn path_tangent_rotation(path: &[RailPoint], t: f32) -> DQuat {
    let n = path.len();
    if n < 2 {
        return DQuat::IDENTITY;
    }
    let t_clamped = (t as f64 * (n - 1) as f64).clamp(0.0, (n - 2) as f64);
    let i = t_clamped as usize;
    let forward = (path[i + 1].position - path[i].position).normalize_or_zero();
    if forward.length_squared() < 1e-10 {
        return DQuat::IDENTITY;
    }
    let world_up = if forward.dot(DVec3::Y).abs() > 0.99 {
        DVec3::Z
    } else {
        DVec3::Y
    };
    let right = world_up.cross(forward).normalize();
    let up = forward.cross(right).normalize();
    DQuat::from_mat3(&DMat3::from_cols(right, up, -forward))
}


fn sample_path_rotation(path: &[RailPoint], t: f32) -> DQuat {
    let n = path.len();
    if n < 2 {
        return DQuat::IDENTITY;
    }
    let t_clamped = t.clamp(0.0, 1.0);
    let seg_f = t_clamped as f64 * (n - 1) as f64;
    let i = (seg_f as usize).min(n - 2);
    let frac = (seg_f - i as f64) as f32;

    match (path[i].rotation, path[i + 1].rotation) {
        (Some(a), Some(b)) => a.slerp(b, frac as f64),
        (Some(a), None) => a,
        (None, Some(b)) => b,
        (None, None) => path_tangent_rotation(path, t),
    }
}

/// Build a look-rotation quaternion (same -Z-forward convention as [`path_tangent_rotation`])
/// from an arbitrary forward direction vector.
fn look_rot_from_forward(forward: DVec3) -> DQuat {
    let forward = forward.normalize_or_zero();
    if forward.length_squared() < 1e-10 {
        return DQuat::IDENTITY;
    }
    let world_up = if forward.dot(DVec3::Y).abs() > 0.99 {
        DVec3::Z
    } else {
        DVec3::Y
    };
    let right = world_up.cross(forward).normalize();
    let up = forward.cross(right).normalize();
    DQuat::from_mat3(&DMat3::from_cols(right, up, -forward))
}

/// Apply a sampled OnRails rotation to a [`Camera`], keeping `yaw`/`pitch` in sync so
/// user mouse-drag doesn't snap after the rails set the orientation.
fn apply_rot_to_camera(camera: &mut Camera, new_eye: DVec3, rot: DQuat) {
    let forward = rot * DVec3::NEG_Z;
    camera.eye = new_eye;
    camera.target = new_eye + forward;
    let dir = forward.normalize();
    camera.pitch = dir.y.clamp(-1.0, 1.0).asin();
    camera.yaw = dir.z.atan2(dir.x);
}

fn sample_path_linear(path: &[RailPoint], t: f32) -> DVec3 {
    let n = path.len();
    if n == 0 {
        return DVec3::ZERO;
    }
    if n == 1 {
        return path[0].position;
    }
    let t = t.clamp(0.0, 1.0) as f64;
    let seg_f = t * (n - 1) as f64;
    let i = (seg_f as usize).min(n - 2);
    let frac = seg_f - i as f64;
    path[i].position.lerp(path[i + 1].position, frac)
}

/// How the progression of the [`OnRails`] component is handled.
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RailDrive {
    /// Progress advances automatically at a fixed speed.
    ///
    /// Use for cutscenes, intros, or automated camera pans.
    Automatic { speed: f32, looping: bool },

    /// Progress is tied to the closest point on the rail to a target entity.
    ///
    /// Use for third-person or side-scroller cameras following a player.
    FollowEntity {
        target: Entity,
        /// If true, progress can never decrease and the camera won't scroll backwards.
        monotonic: bool,
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
        write!(
            f,
            "{}",
            match self {
                RailDrive::Automatic { .. } => "Automatic",
                RailDrive::FollowEntity { .. } => "FollowEntity",
                RailDrive::AxisDriven { .. } => "AxisDriven",
                RailDrive::Manual => "Manual",
            }
        )
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

    fn init(
        ser: &'_ Self::SerializedForm,
        _graphics: Arc<SharedGraphicsContext>,
    ) -> ComponentInitFuture<'_, Self> {
        Box::pin(async move { Ok((ser.clone(), EntityTransform::default())) })
    }

    fn update_component(
        &mut self,
        world: &World,
        _physics: &mut PhysicsState,
        entity: Entity,
        dt: f32,
        _graphics: Arc<SharedGraphicsContext>,
    ) {
        // this might be bad practice, idk
        for (rails, et) in
            world.query::<(&mut OnRails, &mut EntityTransform)>().iter()
        {
            if let Some((pos, rot)) = rails.pending_transform.take() {
                et.world_mut().position = pos;
                et.world_mut().rotation = rot;
            }
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
                    .map(|(i, p)| {
                        Key::new(i as f64 / (n - 1) as f64, get(&p.position), Interpolation::Linear)
                    })
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
                    let a = self.path[i].position;
                    let b = self.path[i + 1].position;
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
            RailDrive::AxisDriven {
                target,
                axis,
                range,
            } => {
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
            let new_pos = DVec3::new(x, y, z);
            let rot = sample_path_rotation(&self.path, self.progress);

            self.pending_transform = Some((new_pos, rot));
            if let Ok(mut camera) = world.get::<&mut Camera>(entity) {
                apply_rot_to_camera(&mut camera, new_pos, rot);
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
        graphics: Arc<SharedGraphicsContext>,
    ) {
        let n = self.path.len();

        if n >= 2 {
            let expected = sample_path_linear(&self.path, self.progress);
            let current_pos = world
                .get::<&EntityTransform>(entity)
                .ok()
                .map(|et| et.world().position);
            if let Some(pos) = current_pos {
                if pos.distance_squared(expected) > 1e-8 {
                    self.progress = project_to_path(&self.path, pos);
                    let snapped = sample_path_linear(&self.path, self.progress);
                    let rot = sample_path_rotation(&self.path, self.progress);
                    if let Ok(mut et) = world.get::<&mut EntityTransform>(entity) {
                        et.world_mut().position = snapped;
                        et.world_mut().rotation = rot;
                    }
                    if let Ok(mut camera) = world.get::<&mut Camera>(entity) {
                        apply_rot_to_camera(&mut camera, snapped, rot);
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
                    let rot = sample_path_rotation(&self.path, self.progress);
                    if let Ok(mut et) = world.get::<&mut EntityTransform>(entity) {
                        et.world_mut().position = pos;
                        et.world_mut().rotation = rot;
                    }
                    if let Ok(mut camera) = world.get::<&mut Camera>(entity) {
                        apply_rot_to_camera(&mut camera, pos, rot);
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
                        0 => RailDrive::Automatic {
                            speed: 0.1,
                            looping: false,
                        },
                        1 => RailDrive::FollowEntity {
                            target: entity,
                            monotonic: false,
                        },
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
                            ui.add(
                                egui::DragValue::new(speed)
                                    .speed(0.001)
                                    .range(0.0_f32..=f32::MAX),
                            );
                        });
                        ui.checkbox(looping, "Looping");
                    }
                    RailDrive::FollowEntity { target, monotonic } => {
                        ui.label(format!("Target entity: {}", target.to_bits()));
                        ui.checkbox(monotonic, "Monotonic");
                    }
                    RailDrive::AxisDriven {
                        target,
                        axis,
                        range,
                    } => {
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
                        ui.style_mut().interaction.selectable_labels = false;
                        let mut remove_idx: Option<usize> = None;
                        for (i, point) in self.path.iter_mut().enumerate() {
                            ui.push_id(i, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(format!("[{}]", i));
                                    ui.add(egui::DragValue::new(&mut point.position.x).speed(0.1).prefix("X:"));
                                    ui.add(egui::DragValue::new(&mut point.position.y).speed(0.1).prefix("Y:"));
                                    ui.add(egui::DragValue::new(&mut point.position.z).speed(0.1).prefix("Z:"));
                                    let has_rot = point.rotation.is_some();
                                    if ui
                                        .small_button(if has_rot { "R\u{2713}" } else { "R" })
                                        .on_hover_text(if has_rot {
                                            "Explicit rotation set \u{2014} click to clear (use tangent)"
                                        } else {
                                            "No explicit rotation (tangent-derived) \u{2014} click to pin"
                                        })
                                        .clicked()
                                    {
                                        point.rotation = if has_rot { None } else { Some(DQuat::IDENTITY) };
                                    }
                                    if ui.small_button("-").clicked() {
                                        remove_idx = Some(i);
                                    }
                                });
                                if let Some(ref mut rot) = point.rotation {
                                    ui.indent(format!("rail_rot_{}", i), |ui| {
                                        inspect_rotation_dquat(
                                            ui,
                                            ("rail_point_rot", entity.to_bits(), i),
                                            rot,
                                        );
                                    });
                                }
                            });
                        }
                        if let Some(idx) = remove_idx {
                            self.path.remove(idx);
                        }
                        if ui.button("+ Add Point").clicked() {
                            let last = self.path.last().map(|p| p.position).unwrap_or(DVec3::ZERO);
                            self.path.push(RailPoint {
                                position: last + DVec3::new(1.0, 0.0, 0.0),
                                rotation: None,
                            });
                        }
                        if ui.button("+ Add Point from Camera").clicked() {
                            let cam_data = world
                                .query::<(&Camera, &CameraComponent)>()
                                .iter()
                                .find_map(|(cam, comp)| {
                                    matches!(comp.camera_type, CameraType::Debug)
                                        .then_some((cam.eye, (cam.target - cam.eye).normalize_or_zero()))
                                })
                                .or_else(|| {
                                    world
                                        .query::<&Camera>()
                                        .iter()
                                        .next()
                                        .map(|cam| (cam.eye, (cam.target - cam.eye).normalize_or_zero()))
                                });
                            if let Some((pos, fwd)) = cam_data {
                                self.path.push(RailPoint {
                                    position: pos,
                                    rotation: Some(look_rot_from_forward(fwd)),
                                });
                            }
                        }
                    });

                ui.add_space(4.0);
                ui.checkbox(&mut self.debug, "Debug path");

                if self.debug && self.path.len() >= 2 {
                    if let Some(dd) = graphics.debug_draw.lock().as_mut() {
                        let red = [1.0, 0.0, 0.0, 1.0];
                        dd.draw_point(self.path[0].position.as_vec3(), 0.1, red);
                        for i in 0..(self.path.len() - 1) {
                            let a = self.path[i].position.as_vec3();
                            let b = self.path[i + 1].position.as_vec3();
                            dd.draw_line(a, b, red);
                            dd.draw_point(b, 0.1, red);
                        }
                    }
                }
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
        ser: &'_ Self::SerializedForm,
        _: Arc<SharedGraphicsContext>,
    ) -> crate::component::ComponentInitFuture<'_, Self> {
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
                CollapsingHeader::new("Local")
                    .default_open(true)
                    .id_salt(format!("Local {}", entity.to_bits()))
                    .show(ui, |ui| {
                        self.local_mut().inspect(ui);
                    });
                ui.add_space(4.0);
                CollapsingHeader::new("World")
                    .default_open(true)
                    .id_salt(format!("World {}", entity.to_bits()))
                    .show(ui, |ui| {
                        self.world_mut().inspect(ui);
                    });
            });
    }
}

