//! Additional information and context for cameras from the [`dropbear_engine::camera`]
use crate::component::{
    Component, ComponentDescriptor, ComponentInitFuture, DisabilityFlags, InspectableComponent,
    SerializedComponent,
};
use crate::ptr::WorldPtr;
use crate::scripting::result::DropbearNativeResult;
use crate::states::SerializableCamera;
use crate::types::NVector3;
use dropbear_engine::camera::{Camera, CameraBuilder, CameraSettings};
use dropbear_engine::graphics::SharedGraphicsContext;
use egui::{CollapsingHeader, Ui};
use glam::DVec3;
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraComponent {
    pub camera_type: CameraType,
    pub starting_camera: bool,
}

#[typetag::serde]
impl SerializedComponent for SerializableCamera {}

impl Component for Camera {
    type SerializedForm = SerializableCamera;
    type RequiredComponentTypes = (Self, CameraComponent);

    fn descriptor() -> ComponentDescriptor {
        ComponentDescriptor {
            disabled_flags: DisabilityFlags::Disabled,
            internal: false,
            fqtn: "dropbear_engine::camera::Camera".to_string(),
            type_name: "Camera3D".to_string(),
            category: Some("Camera".to_string()),
            description: Some(
                "Allows you to view the scene through the eyes of the component".to_string(),
            ),
        }
    }

    fn init(
        ser: &'_ Self::SerializedForm,
        graphics: Arc<SharedGraphicsContext>,
    ) -> ComponentInitFuture<'_, Self> {
        Box::pin(async move {
            let label = ser.label.clone();
            let builder = CameraBuilder::from(ser.clone());
            Ok((
                Camera::new(graphics.clone(), builder, Some(label.as_str())),
                CameraComponent::from(ser.clone()),
            ))
        })
    }

    fn update_component(
        &mut self,
        _world: &World,
        _physics: &mut crate::physics::PhysicsState,
        _entity: Entity,
        _dt: f32,
        graphics: Arc<SharedGraphicsContext>,
    ) {
        self.update(graphics.clone())
    }

    fn save(&self, world: &World, entity: Entity) -> Box<dyn SerializedComponent> {
        if let Ok((cam, comp)) = world.query_one::<(&Camera, &CameraComponent)>(entity).get() {
            Box::new(SerializableCamera::from_ecs_camera(cam, comp))
        } else {
            crate::warn!("Unable to save Camera3D's properties: Not found within world");
            Box::new(SerializableCamera::default())
        }
    }
}

impl InspectableComponent for Camera {
    fn inspect(
        &mut self,
        _world: &World,
        entity: Entity,
        ui: &mut Ui,
        _graphics: Arc<SharedGraphicsContext>,
    ) {
        CollapsingHeader::new("Camera3D")
            .default_open(true)
            .id_salt(format!("Camera3D {}", entity.to_bits()))
            .show(ui, |ui| {
                let mut changed = false;

                ui.horizontal(|ui| {
                    ui.label("Eye");
                    changed |= ui
                        .add(egui::DragValue::new(&mut self.eye.x).speed(0.1))
                        .changed();
                    changed |= ui
                        .add(egui::DragValue::new(&mut self.eye.y).speed(0.1))
                        .changed();
                    changed |= ui
                        .add(egui::DragValue::new(&mut self.eye.z).speed(0.1))
                        .changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Target");
                    changed |= ui
                        .add(egui::DragValue::new(&mut self.target.x).speed(0.1))
                        .changed();
                    changed |= ui
                        .add(egui::DragValue::new(&mut self.target.y).speed(0.1))
                        .changed();
                    changed |= ui
                        .add(egui::DragValue::new(&mut self.target.z).speed(0.1))
                        .changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Up");
                    changed |= ui
                        .add(egui::DragValue::new(&mut self.up.x).speed(0.1))
                        .changed();
                    changed |= ui
                        .add(egui::DragValue::new(&mut self.up.y).speed(0.1))
                        .changed();
                    changed |= ui
                        .add(egui::DragValue::new(&mut self.up.z).speed(0.1))
                        .changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Aspect");
                    changed |= ui
                        .add(
                            egui::DragValue::new(&mut self.aspect)
                                .speed(0.01)
                                .range(0.1..=10.0),
                        )
                        .changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Near Plane");
                    changed |= ui
                        .add(
                            egui::DragValue::new(&mut self.znear)
                                .speed(0.01)
                                .range(0.01..=1000.0),
                        )
                        .changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Far Plane");
                    changed |= ui
                        .add(
                            egui::DragValue::new(&mut self.zfar)
                                .speed(1.0)
                                .range(0.1..=10000.0),
                        )
                        .changed();
                });

                ui.horizontal(|ui| {
                    ui.label("FOV");
                    changed |= ui
                        .add(egui::Slider::new(&mut self.settings.fov_y, 1.0..=179.0).suffix("°"))
                        .changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Speed");
                    changed |= ui
                        .add(egui::DragValue::new(&mut self.settings.speed).speed(0.1))
                        .changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Sensitivity");
                    changed |= ui
                        .add(egui::DragValue::new(&mut self.settings.sensitivity).speed(0.001))
                        .changed();
                });

                if changed {
                    self.update_view_proj();
                }
            });
    }
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl CameraComponent {
    pub fn new() -> Self {
        Self {
            camera_type: CameraType::Normal,
            starting_camera: false,
        }
    }
}

impl From<SerializableCamera> for CameraBuilder {
    fn from(value: SerializableCamera) -> Self {
        let forward = value.transform.rotation * DVec3::Z;
        let up = if matches!(value.camera_type, CameraType::Debug | CameraType::Normal) {
            DVec3::Y
        } else {
            value.transform.rotation * DVec3::Y
        };

        Self {
            eye: value.transform.position,
            target: value.transform.position + forward,
            up,
            aspect: value.aspect,
            znear: value.near as f64,
            zfar: value.far as f64,
            settings: CameraSettings {
                speed: value.speed as f64,
                sensitivity: value.sensitivity as f64,
                fov_y: value.fov as f64,
            },
        }
    }
}

impl From<SerializableCamera> for CameraComponent {
    fn from(value: SerializableCamera) -> Self {
        Self {
            camera_type: value.camera_type,
            starting_camera: value.starting_camera,
        }
    }
}

pub struct PlayerCamera;

impl PlayerCamera {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> CameraComponent {
        CameraComponent {
            camera_type: CameraType::Player,
            ..CameraComponent::new()
        }
    }
}

pub struct DebugCamera;

impl DebugCamera {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> CameraComponent {
        CameraComponent {
            camera_type: CameraType::Debug,
            ..CameraComponent::new()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CameraType {
    Normal,
    Debug,
    Player,
}

impl Default for CameraType {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone)]
pub enum CameraAction {
    SetPlayerTarget { entity: hecs::Entity, offset: DVec3 },
    ClearPlayerTarget,
    SetCurrentPositionAsOffset(hecs::Entity),
}

pub mod shared {
    pub fn camera_exists_for_entity(world: &hecs::World, entity: hecs::Entity) -> bool {
        world
            .get::<&dropbear_engine::camera::Camera>(entity)
            .is_ok()
    }
}

