use crate::component::{
    Component, ComponentDescriptor, ComponentInitFuture, DisabilityFlags, InspectableComponent,
    SerializedComponent,
};
use crate::ptr::WorldPtr;
use crate::states::SerializedLight;
use dropbear_engine::attenuation::ATTENUATION_PRESETS;
use dropbear_engine::entity::{EntityTransform, Transform, inspect_rotation_dquat};
use crate::hierarchy::EntityTransformExt;
use dropbear_engine::graphics::SharedGraphicsContext;
use dropbear_engine::lighting::{Light, LightType};
use egui::{CollapsingHeader, ComboBox, DragValue, Ui};
use glam::{DQuat, DVec3, Vec3};
use hecs::{Entity, World};
use std::sync::Arc;

const LIGHT_FORWARD_AXIS: DVec3 = DVec3::new(0.0, -1.0, 0.0);

#[typetag::serde]
impl SerializedComponent for SerializedLight {}

impl Component for Light {
    type SerializedForm = SerializedLight;
    type RequiredComponentTypes = (Self, Transform);

    fn descriptor() -> ComponentDescriptor {
        ComponentDescriptor {
            disabled_flags: DisabilityFlags::Disabled,
            internal: false,
            fqtn: "dropbear_engine::lighting::Light".to_string(),
            type_name: "Light".to_string(),
            category: Some("Lighting".to_string()),
            description: Some("An object that emits light".to_string()),
        }
    }

    fn init<'a>(
        ser: &'a Self::SerializedForm,
        graphics: Arc<SharedGraphicsContext>,
    ) -> ComponentInitFuture<'a, Self> {
        Box::pin(async move {
            let light_component = ser.light_component.clone();
            let light = Light::new(
                graphics.clone(),
                light_component.clone(),
                Some(ser.label.as_str()),
            )
            .await;
            let transform = light_component.to_transform();

            Ok((light, transform))
        })
    }

    fn update_component(
        &mut self,
        world: &World,
        _physics: &mut crate::physics::PhysicsState,
        entity: Entity,
        _dt: f32,
        graphics: Arc<SharedGraphicsContext>,
    ) {
        let synced = &mut self.component;
        if let Ok(entity_transform) = world.query_one::<&EntityTransform>(entity).get() {
            let transform = entity_transform.propagate(world, entity);
            synced.position = transform.position;
            synced.direction = (transform.rotation * LIGHT_FORWARD_AXIS).normalize_or_zero();
        } else if let Ok(transform) = world.query_one::<&Transform>(entity).get() {
            synced.position = transform.position;
            synced.direction = (transform.rotation * LIGHT_FORWARD_AXIS).normalize_or_zero();
        }

        self.update(&graphics);
    }

    fn save(&self, _: &World, entity: Entity) -> Box<dyn SerializedComponent> {
        Box::new(SerializedLight {
            label: self.label.clone(),
            light_component: self.component.clone(),
            entity_id: Some(entity),
        })
    }
}

impl InspectableComponent for Light {
    fn inspect(
        &mut self,
        world: &World,
        entity: Entity,
        ui: &mut Ui,
        _graphics: Arc<SharedGraphicsContext>,
    ) {
        CollapsingHeader::new("Light")
            .default_open(true)
            .id_salt(format!("Light {}", entity.to_bits()))
            .show(ui, |ui| {
                ui.add_space(6.0);
                ui.label("Uniform");

                ui.label("Light Type");
                ComboBox::from_id_salt("Light Type")
                    .selected_text(format!("{}", self.component.light_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.component.light_type,
                            LightType::Directional,
                            "Directional",
                        );
                        ui.selectable_value(
                            &mut self.component.light_type,
                            LightType::Point,
                            "Point",
                        );
                        ui.selectable_value(
                            &mut self.component.light_type,
                            LightType::Spot,
                            "Spot",
                        );
                    });

                let mut display_pos = |yueye: &mut Ui| {
                    let pos_id = yueye.make_persistent_id(("light_pos", entity.to_bits()));
                    let stored = yueye.ctx().data(|d| d.get_temp::<[f64; 3]>(pos_id));
                    let [mut px, mut py, mut pz] = stored.unwrap_or([
                        self.component.position.x,
                        self.component.position.y,
                        self.component.position.z,
                    ]);

                    let mut changed = false;
                    let mut any_dragging = false;
                    let mut reset = false;

                    yueye.horizontal(|yueye| {
                        yueye.label("Position");
                        let rx = yueye.add(DragValue::new(&mut px).speed(0.01));
                        changed |= rx.changed();
                        any_dragging |= rx.dragged();

                        let ry = yueye.add(DragValue::new(&mut py).speed(0.01));
                        changed |= ry.changed();
                        any_dragging |= ry.dragged();

                        let rz = yueye.add(DragValue::new(&mut pz).speed(0.01));
                        changed |= rz.changed();
                        any_dragging |= rz.dragged();

                        if yueye.button("Reset").clicked() {
                            px = 0.0;
                            py = 0.0;
                            pz = 0.0;
                            changed = true;
                            reset = true;
                        }
                    });

                    if any_dragging || changed || reset {
                        yueye
                            .ctx()
                            .data_mut(|d| d.insert_temp(pos_id, [px, py, pz]));
                        self.component.position = DVec3::new(px, py, pz);
                    } else {
                        yueye.ctx().data_mut(|d| {
                            d.insert_temp(
                                pos_id,
                                [
                                    self.component.position.x,
                                    self.component.position.y,
                                    self.component.position.z,
                                ],
                            )
                        });
                    }

                    changed
                };

                let mut display_rot = |yueye: &mut Ui| {
                    yueye.label("Rotation");
                    let mut direction = self.component.direction.normalize_or_zero();
                    if direction.length_squared() < 1e-12 {
                        direction = LIGHT_FORWARD_AXIS;
                    }

                    let light_rot_cache_id = egui::Id::new(("light_quat_cache", entity.to_bits()));
                    let mut rotation = {
                        let cached = yueye
                            .ctx()
                            .data(|d| d.get_temp::<DQuat>(light_rot_cache_id));
                        if let Some(cached_quat) = cached {
                            let cached_fwd = (cached_quat * LIGHT_FORWARD_AXIS).normalize_or_zero();
                            if cached_fwd.dot(direction) > 0.999_999 {
                                // Direction unchanged externally; keep the accumulated quat.
                                cached_quat
                            } else {
                                // External change (script/physics moved the light); rebuild.
                                DQuat::from_rotation_arc(LIGHT_FORWARD_AXIS, direction)
                            }
                        } else {
                            DQuat::from_rotation_arc(LIGHT_FORWARD_AXIS, direction)
                        }
                    };

                    let mut changed = inspect_rotation_dquat(
                        yueye,
                        ("light_rotation", entity.to_bits()),
                        &mut rotation,
                    );

                    if changed {
                        yueye
                            .ctx()
                            .data_mut(|d| d.insert_temp(light_rot_cache_id, rotation));
                        self.component.direction =
                            (rotation * LIGHT_FORWARD_AXIS).normalize_or_zero();
                    }

                    if yueye.button("Reset Rotation").clicked() {
                        self.component.direction = LIGHT_FORWARD_AXIS;
                        yueye
                            .ctx()
                            .data_mut(|d| d.insert_temp(light_rot_cache_id, DQuat::IDENTITY));
                        changed = true;
                    }

                    changed
                };

                let mut position_changed = false;
                let mut direction_changed = false;

                match self.component.light_type {
                    LightType::Directional => {
                        direction_changed |= display_rot(ui);
                    }
                    LightType::Point => {
                        position_changed |= display_pos(ui);
                    }
                    LightType::Spot => {
                        position_changed |= display_pos(ui);
                        direction_changed |= display_rot(ui);
                    }
                }

                if position_changed {
                    if let Ok(entity_transform) =
                        world.query_one::<&mut EntityTransform>(entity).get()
                    {
                        entity_transform.local_mut().position = self.component.position;
                    } else if let Ok(transform) = world.query_one::<&mut Transform>(entity).get() {
                        transform.position = self.component.position;
                    }
                }

                if direction_changed {
                    let desired = self.component.direction.normalize_or_zero();
                    if desired.length_squared() >= 1e-12 {
                        self.component.direction = desired;
                        let rotation = DQuat::from_rotation_arc(LIGHT_FORWARD_AXIS, desired);

                        if let Ok(entity_transform) =
                            world.query_one::<&mut EntityTransform>(entity).get()
                        {
                            entity_transform.local_mut().rotation = rotation;
                        } else if let Ok(transform) =
                            world.query_one::<&mut Transform>(entity).get()
                        {
                            transform.rotation = rotation;
                        }
                    }
                }

                let mut colour_rgb = [
                    self.component.colour[0] as f32,
                    self.component.colour[1] as f32,
                    self.component.colour[2] as f32,
                ];
                egui::color_picker::color_edit_button_rgb(ui, &mut colour_rgb);
                self.component.colour = Vec3::from_array(colour_rgb).as_dvec3();

                ui.horizontal(|ui| {
                    ui.label("Intensity");
                    ui.add(
                        DragValue::new(&mut self.component.intensity)
                            .speed(0.05)
                            .range(0.0..=f64::MAX),
                    );
                });

                if matches!(
                    self.component.light_type,
                    LightType::Point | LightType::Spot
                ) {
                    ui.horizontal(|ui| {
                        ComboBox::from_id_salt("Attenuation Range")
                            .selected_text(format!("Range {}", self.component.attenuation.range))
                            .show_ui(ui, |ui| {
                                for (preset, label) in ATTENUATION_PRESETS {
                                    ui.selectable_value(
                                        &mut self.component.attenuation,
                                        *preset,
                                        *label,
                                    );
                                }
                            });
                    });
                }

                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.component.enabled, "Enabled");
                    ui.checkbox(&mut self.component.visible, "Visible");
                });

                if matches!(self.component.light_type, LightType::Spot) {
                    ui.horizontal(|ui| {
                        ui.label("Cutoff");
                        ui.add(
                            DragValue::new(&mut self.component.cutoff_angle)
                                .speed(0.1)
                                .range(0.0..=180.0),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Outer Cutoff");
                        ui.add(
                            DragValue::new(&mut self.component.outer_cutoff_angle)
                                .speed(0.1)
                                .range(0.0..=180.0),
                        );
                    });

                    if self.component.outer_cutoff_angle <= self.component.cutoff_angle {
                        self.component.outer_cutoff_angle = self.component.cutoff_angle + 1.0;
                    }
                }

                ui.separator();
                ui.label("Shadows");
                ui.checkbox(&mut self.component.cast_shadows, "Cast Shadows");
                ui.horizontal(|ui| {
                    ui.label("Depth");
                    ui.add(
                        DragValue::new(&mut self.component.depth.start)
                            .speed(0.1)
                            .range(0.0..=f64::MAX),
                    );
                    ui.label("..");
                    ui.add(
                        DragValue::new(&mut self.component.depth.end)
                            .speed(0.1)
                            .range(0.0..=f64::MAX),
                    );
                });

                if self.component.depth.end < self.component.depth.start {
                    self.component.depth.end = self.component.depth.start;
                }
            });
    }
}

