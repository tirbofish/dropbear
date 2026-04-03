//! Colliders and shapes that allow for entities to collide with other entities.
//!
//! ### TODO: Implement collision type detection:
//! - Unreal
//!     - `UCX_` = Convex collision (most common)
//!     - `UBX_` = Box collision
//!     - `USP_` = Sphere collision
//!     - `UCP_` = Capsule collision
//!     - `MCDCX_` = Complex collision as simple
//! - Unity
//!     - Suffix or contains: `_collider`, `_collision`, `_col`
//! - Godot
//!     - `col-` or `-col`
//!     - `-colonly` (invisible collision mesh)

use crate::component::{
    Component, ComponentDescriptor, DisabilityFlags, InspectableComponent, SerializedComponent,
};
use crate::physics::PhysicsState;
use crate::states::Label;
use dropbear_engine::asset::ASSET_REGISTRY;
use dropbear_engine::entity::{MeshRenderer, inspect_rotation_dquat};
use dropbear_engine::graphics::SharedGraphicsContext;
use dropbear_engine::wgpu::util::{BufferInitDescriptor, DeviceExt};
use dropbear_engine::wgpu::{Buffer, BufferUsages};
use egui::{CollapsingHeader, ComboBox, Ui};
use glam::{DQuat, Vec3};
use hecs::{Entity, World};
use rapier3d::prelude::ColliderBuilder;
use rapier3d::prelude::{Rotation, SharedShape, TypedShape, Vector};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ColliderGroup {
    #[serde(default)]
    pub colliders: Vec<Collider>,
}

impl ColliderGroup {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, collider: Collider) {
        self.colliders.push(collider);
    }
}

#[typetag::serde]
impl SerializedComponent for ColliderGroup {}

impl Component for ColliderGroup {
    type SerializedForm = Self;
    type RequiredComponentTypes = (Self,);

    fn descriptor() -> ComponentDescriptor {
        ComponentDescriptor {
            fqtn: "eucalyptus_core::physics::collider::ColliderGroup".to_string(),
            type_name: "ColliderGroup".to_string(),
            category: Some("Physics".to_string()),
            description: Some("A group of colliders".to_string()),
            disabled_flags: DisabilityFlags::Disabled,
            internal: false,
        }
    }

    fn init<'a>(
        ser: &'a Self::SerializedForm,
        _graphics: Arc<SharedGraphicsContext>,
    ) -> crate::component::ComponentInitFuture<'a, Self> {
        Box::pin(async move { Ok((ser.clone(),)) })
    }

    fn update_component(
        &mut self,
        world: &World,
        _physics: &mut PhysicsState,
        entity: Entity,
        _dt: f32,
        _graphics: Arc<SharedGraphicsContext>,
    ) {
        let Ok(label) = world.get::<&Label>(entity) else {
            return;
        };

        for collider in &mut self.colliders {
            if collider.entity != *label {
                collider.entity = Label::new(label.as_str());
            }
        }
    }

    fn save(&self, _world: &World, _entity: Entity) -> Box<dyn SerializedComponent> {
        Box::new(self.clone())
    }
}

impl InspectableComponent for ColliderGroup {
    fn inspect(
        &mut self,
        world: &World,
        entity: Entity,
        ui: &mut Ui,
        _graphics: Arc<SharedGraphicsContext>,
    ) {
        CollapsingHeader::new("Colliders")
            .default_open(true)
            .id_salt(format!("Colliders {}", entity.to_bits()))
            .show(ui, |ui| {
                let mut remove_index: Option<usize> = None;

                for (index, collider) in self.colliders.iter_mut().enumerate() {
                    ui.push_id(index, |ui| {
                        CollapsingHeader::new(format!("Collider {}", index + 1))
                            .default_open(true)
                            .id_salt(format!("Collider {} {}", index + 1, entity.to_bits()))
                            .show(ui, |ui| {
                                collider.inspect(ui);

                                ui.add_space(6.0);
                                if ui.button("Remove Collider").clicked() {
                                    remove_index = Some(index);
                                }
                            });
                    });

                    ui.add_space(6.0);
                }

                if let Some(index) = remove_index {
                    self.colliders.remove(index);
                }

                if ui.button("Add Collider").clicked() {
                    self.colliders.push(Collider::new());
                }

                if ui.button("Derive from Mesh").clicked() {
                    let model_handle = world.get::<&MeshRenderer>(entity).ok().map(|r| r.model());

                    if let Some(handle) = model_handle {
                        let registry = ASSET_REGISTRY.read();
                        if let Some(model) = registry.get_model(handle) {
                            let mut min = [f32::INFINITY; 3];
                            let mut max = [f32::NEG_INFINITY; 3];

                            for mesh in &model.meshes {
                                for vertex in &mesh.vertices {
                                    let p = vertex.position;
                                    for i in 0..3 {
                                        min[i] = min[i].min(p[i]);
                                        max[i] = max[i].max(p[i]);
                                    }
                                }
                            }

                            if min.iter().all(|v| v.is_finite())
                                && max.iter().all(|v| v.is_finite())
                            {
                                let half_extents = [
                                    (max[0] - min[0]) * 0.5,
                                    (max[1] - min[1]) * 0.5,
                                    (max[2] - min[2]) * 0.5,
                                ];
                                let center = [
                                    (max[0] + min[0]) * 0.5,
                                    (max[1] + min[1]) * 0.5,
                                    (max[2] + min[2]) * 0.5,
                                ];
                                let mut collider = Collider::box_collider(half_extents);
                                collider.translation = center;
                                self.colliders.push(collider);
                            }
                        }
                    }
                }
            });
    }
}

#[repr(C)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Collider {
    /// A unique identifier index for this collider.
    #[serde(default)]
    pub id: u32,

    /// The entity this component is attached to.
    #[serde(default)]
    pub entity: Label,

    /// The collision shape.
    pub shape: ColliderShape,

    /// Density of the collider (used to calculate mass).
    #[serde(default = "Collider::default_density")]
    pub density: f32,

    /// Friction coefficient (0.0 = no friction, 1.0 = high friction).
    #[serde(default = "Collider::default_friction")]
    pub friction: f32,

    /// Restitution/bounciness (0.0 = no bounce, 1.0 = perfect bounce).
    #[serde(default)]
    pub restitution: f32,

    /// Whether this is a sensor (triggers collision events but no physical response).
    #[serde(default)]
    pub is_sensor: bool,

    /// Local translation offset from the rigid body.
    #[serde(default)]
    pub translation: [f32; 3],

    /// Local rotation offset from the rigid body (Euler angles in radians).
    #[serde(default)]
    pub rotation: [f32; 3],
}

impl Collider {
    pub fn inspect(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.label("Shape:");
            let current_shape = self.shape_type_name();
            ComboBox::from_id_salt("collider_shape")
                .selected_text(current_shape)
                .show_ui(ui, |ui| {
                    if ui.selectable_label(current_shape == "Box", "Box").clicked() {
                        if current_shape != "Box" {
                            self.shape = ColliderShape::Box {
                                half_extents: [0.5, 0.5, 0.5].into(),
                            };
                        }
                    }
                    if ui
                        .selectable_label(current_shape == "Sphere", "Sphere")
                        .clicked()
                    {
                        if current_shape != "Sphere" {
                            self.shape = ColliderShape::Sphere { radius: 0.5 };
                        }
                    }
                    if ui
                        .selectable_label(current_shape == "Capsule", "Capsule")
                        .clicked()
                    {
                        if current_shape != "Capsule" {
                            self.shape = ColliderShape::Capsule {
                                half_height: 0.5,
                                radius: 0.25,
                            };
                        }
                    }
                    if ui
                        .selectable_label(current_shape == "Cylinder", "Cylinder")
                        .clicked()
                    {
                        if current_shape != "Cylinder" {
                            self.shape = ColliderShape::Cylinder {
                                half_height: 0.5,
                                radius: 0.25,
                            };
                        }
                    }
                    if ui
                        .selectable_label(current_shape == "Cone", "Cone")
                        .clicked()
                    {
                        if current_shape != "Cone" {
                            self.shape = ColliderShape::Cone {
                                half_height: 0.5,
                                radius: 0.25,
                            };
                        }
                    }
                });

            ui.add_space(8.0);

            match &mut self.shape {
                ColliderShape::Box { half_extents } => {
                    ui.label("Half Extents:");
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        ui.add(egui::DragValue::new(&mut half_extents.x).speed(0.01));
                        ui.label("Y:");
                        ui.add(egui::DragValue::new(&mut half_extents.y).speed(0.01));
                        ui.label("Z:");
                        ui.add(egui::DragValue::new(&mut half_extents.z).speed(0.01));
                    });
                }
                ColliderShape::Sphere { radius } => {
                    ui.horizontal(|ui| {
                        ui.label("Radius:");
                        ui.add(egui::DragValue::new(radius).speed(0.01));
                    });
                }
                ColliderShape::Capsule {
                    half_height,
                    radius,
                } => {
                    ui.horizontal(|ui| {
                        ui.label("Half Height:");
                        ui.add(egui::DragValue::new(half_height).speed(0.01));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Radius:");
                        ui.add(egui::DragValue::new(radius).speed(0.01));
                    });
                }
                ColliderShape::Cylinder {
                    half_height,
                    radius,
                } => {
                    ui.horizontal(|ui| {
                        ui.label("Half Height:");
                        ui.add(egui::DragValue::new(half_height).speed(0.01));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Radius:");
                        ui.add(egui::DragValue::new(radius).speed(0.01));
                    });
                }
                ColliderShape::Cone {
                    half_height,
                    radius,
                } => {
                    ui.horizontal(|ui| {
                        ui.label("Half Height:");
                        ui.add(egui::DragValue::new(half_height).speed(0.01));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Radius:");
                        ui.add(egui::DragValue::new(radius).speed(0.01));
                    });
                }
            }

            ui.add_space(8.0);

            ui.separator();
            ui.label("Physical Properties:");

            ui.horizontal(|ui| {
                ui.label("Density:");
                ui.add(egui::DragValue::new(&mut self.density).speed(0.01));
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

            ui.label("Rotation:");
            let mut rotation = DQuat::from_euler(
                glam::EulerRot::XYZ,
                self.rotation[0] as f64,
                self.rotation[1] as f64,
                self.rotation[2] as f64,
            );
            if inspect_rotation_dquat(ui, "collider_local_rotation", &mut rotation) {
                let hint_id = ui
                    .make_persistent_id(("rotation_mode", "collider_local_rotation"))
                    .with("euler_hint_rad");
                if let Some([x, y, z]) = ui.ctx().data(|d| d.get_temp::<[f64; 3]>(hint_id)) {
                    self.rotation = [x as f32, y as f32, z as f32];
                } else {
                    let (x, y, z) = rotation.to_euler(glam::EulerRot::XYZ);
                    self.rotation = [x as f32, y as f32, z as f32];
                }
            }
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColliderShapeType {
    Box,
    Sphere,
    Capsule,
    Cylinder,
    Cone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColliderShapeKey {
    Box {
        half_extents_bits: [u32; 3],
    },
    Sphere {
        radius_bits: u32,
    },
    Capsule {
        half_height_bits: u32,
        radius_bits: u32,
    },
    Cylinder {
        half_height_bits: u32,
        radius_bits: u32,
    },
    Cone {
        half_height_bits: u32,
        radius_bits: u32,
    },
}

impl From<&ColliderShape> for ColliderShapeKey {
    fn from(shape: &ColliderShape) -> Self {
        match *shape {
            ColliderShape::Box { half_extents } => Self::Box {
                half_extents_bits: [
                    (half_extents.x as f32).to_bits(),
                    (half_extents.y as f32).to_bits(),
                    (half_extents.z as f32).to_bits(),
                ],
            },
            ColliderShape::Sphere { radius } => Self::Sphere {
                radius_bits: radius.to_bits(),
            },
            ColliderShape::Capsule {
                half_height,
                radius,
            } => Self::Capsule {
                half_height_bits: half_height.to_bits(),
                radius_bits: radius.to_bits(),
            },
            ColliderShape::Cylinder {
                half_height,
                radius,
            } => Self::Cylinder {
                half_height_bits: half_height.to_bits(),
                radius_bits: radius.to_bits(),
            },
            ColliderShape::Cone {
                half_height,
                radius,
            } => Self::Cone {
                half_height_bits: half_height.to_bits(),
                radius_bits: radius.to_bits(),
            },
        }
    }
}

#[repr(C)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[dropbear_macro::repr_c_enum]
pub enum ColliderShape {
    /// Box shape with half-extents (half-width, half-height, half-depth).
    Box { half_extents: Vec3 },

    /// Sphere shape with radius.
    Sphere { radius: f32 },

    /// Capsule shape along Y-axis.
    Capsule { half_height: f32, radius: f32 },

    /// Cylinder shape along Y-axis.
    Cylinder { half_height: f32, radius: f32 },

    /// Cone shape along Y-axis.
    Cone { half_height: f32, radius: f32 },
}

impl Default for ColliderShape {
    fn default() -> Self {
        ColliderShape::Box {
            half_extents: Vec3::from([0.5, 0.5, 0.5]),
        }
    }
}

impl Collider {
    fn default_density() -> f32 {
        1.0
    }
    fn default_friction() -> f32 {
        0.5
    }

    pub fn new() -> Self {
        Self {
            id: 0 as u32,
            entity: Label::default(),
            shape: ColliderShape::default(),
            density: Self::default_density(),
            friction: Self::default_friction(),
            restitution: 0.0,
            is_sensor: false,
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
        }
    }

    /// Create a box collider
    pub fn box_collider(half_extents: [f32; 3]) -> Self {
        Self {
            shape: ColliderShape::Box {
                half_extents: Vec3::from(half_extents),
            },
            ..Self::new()
        }
    }

    /// Create a sphere collider
    pub fn sphere(radius: f32) -> Self {
        Self {
            shape: ColliderShape::Sphere { radius },
            ..Self::new()
        }
    }

    /// Create a capsule collider
    pub fn capsule(half_height: f32, radius: f32) -> Self {
        Self {
            shape: ColliderShape::Capsule {
                half_height,
                radius,
            },
            ..Self::new()
        }
    }

    /// Create a cylinder collider
    pub fn cylinder(half_height: f32, radius: f32) -> Self {
        Self {
            shape: ColliderShape::Cylinder {
                half_height,
                radius,
            },
            ..Self::new()
        }
    }

    /// Set the collider as a sensor (no physical response, only triggers events)
    pub fn with_sensor(mut self) -> Self {
        self.is_sensor = true;
        self
    }

    /// Set the translation offset
    pub fn with_translation(mut self, translation: [f32; 3]) -> Self {
        self.translation = translation;
        self
    }

    /// Set the rotation offset (in radians)
    pub fn with_rotation(mut self, rotation: [f32; 3]) -> Self {
        self.rotation = rotation;
        self
    }

    /// Set the friction coefficient
    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction;
        self
    }

    /// Set the restitution (bounciness)
    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution;
        self
    }

    /// Set the density
    pub fn with_density(mut self, density: f32) -> Self {
        self.density = density;
        self
    }

    pub fn to_rapier(&self) -> rapier3d::prelude::Collider {
        let shape: ColliderBuilder = match &self.shape {
            ColliderShape::Box { half_extents } => ColliderBuilder::cuboid(
                half_extents.x as f32,
                half_extents.y as f32,
                half_extents.z as f32,
            ),
            ColliderShape::Sphere { radius } => ColliderBuilder::ball(*radius),
            ColliderShape::Capsule {
                half_height,
                radius,
            } => ColliderBuilder::capsule_y(*half_height, *radius),
            ColliderShape::Cylinder {
                half_height,
                radius,
            } => ColliderBuilder::cylinder(*half_height, *radius),
            ColliderShape::Cone {
                half_height,
                radius,
            } => ColliderBuilder::cone(*half_height, *radius),
        };

        shape
            .density(self.density)
            .friction(self.friction)
            .restitution(self.restitution)
            .sensor(self.is_sensor)
            .translation(Vector::from_array(self.translation))
            .rotation(Vector::from_array(self.rotation))
            .build()
    }

    pub fn shape_type_name(&self) -> &'static str {
        match self.shape {
            ColliderShape::Box { .. } => "Box",
            ColliderShape::Sphere { .. } => "Sphere",
            ColliderShape::Capsule { .. } => "Capsule",
            ColliderShape::Cylinder { .. } => "Cylinder",
            ColliderShape::Cone { .. } => "Cone",
            // ColliderShape::ConvexHull { .. } => "ConvexHull",
            // ColliderShape::TriMesh { .. } => "TriMesh",
            // ColliderShape::HeightField { .. } => "HeightField",
            // ColliderShape::Compound { .. } => "Compound",
        }
    }
}