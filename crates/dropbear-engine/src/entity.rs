use glam::{DMat4, DQuat, DVec3, Mat4, Quat, Vec3};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, hash_map::Entry},
    path::Path,
    sync::{Arc, LazyLock},
};

use crate::{
    asset::{ASSET_REGISTRY, AssetRegistry},
    graphics::{Instance, SharedGraphicsContext},
    model::Model,
    texture::Texture,
    utils::{ResourceReference, ResourceReferenceType, EUCA_SCHEME},
};
use anyhow::anyhow;
use egui::{CollapsingHeader, Ui};
use dropbear_traits::{Component, ComponentDescriptor, ComponentInitContext, ComponentInitFuture, InsertBundle, SerializableComponent};
use std::any::Any;
use crate::asset::Handle;

/// A type of transform that is attached to all entities. It contains the local and world transforms.
#[derive(Default, Debug, Deserialize, Serialize, Copy, PartialEq, Clone)]
pub struct EntityTransform {
    local: Transform,
    world: Transform,
}

impl EntityTransform {
    /// Creates a new [EntityTransform] from a local and world [Transform]
    pub fn new(local: Transform, world: Transform) -> Self {
        Self { local, world }
    }

    /// Creates a new [EntityTransform] from a world [Transform] and a default local transform.
    ///
    /// This is best for situations where a local transform is not required.
    pub fn new_from_world(world: Transform) -> Self {
        Self {
            world,
            local: Transform::default(),
        }
    }

    /// Gets a reference to the local transform
    pub fn local(&self) -> &Transform {
        &self.local
    }

    /// Gets a reference to the world transform
    pub fn world(&self) -> &Transform {
        &self.world
    }

    /// Gets a mutable reference to the local transform
    pub fn local_mut(&mut self) -> &mut Transform {
        &mut self.local
    }

    /// Gets a mutable reference to the world transform
    pub fn world_mut(&mut self) -> &mut Transform {
        &mut self.world
    }

    /// Combines both transforms into one, propagating the local transform
    /// to the world transform and returning a uniform [Transform]
    pub fn sync(&self) -> Transform {
        let scaled_pos = self.local.position * self.world.scale;
        let rotated_pos = self.world.rotation * scaled_pos;
        let position = self.world.position + rotated_pos;

        Transform {
            position,
            rotation: self.world.rotation * self.local.rotation,
            scale: self.world.scale * self.local.scale,
        }
    }
}

impl Component for EntityTransform {
    type Serialized = EntityTransform;

    fn static_descriptor() -> ComponentDescriptor {
        ComponentDescriptor {
            fqtn: "dropbear_engine::entity::EntityTransform".to_string(),
            type_name: "EntityTransform".to_string(),
            category: Some("Transform".to_string()),
            description: Some("A component that allows the entity to be transformed both locally and globally".to_string()),
        }
    }

    fn deserialize(serialized: &Self::Serialized) -> Self {
        serialized.clone()
    }

    fn serialize(&self) -> Self::Serialized {
        self.clone()
    }

    fn inspect(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Entity Transform")
            .default_open(true)
            .show(ui, |ui| {
                ui.set_min_width(300.0);

                ui.group(|ui| {
                    ui.strong("Local Transform");
                    ui.add_space(4.0);

                    self.local.inspect(ui);
                });

                ui.add_space(8.0);

                ui.group(|ui| {
                    ui.strong("World Transform");
                    ui.add_space(4.0);

                    self.world.inspect(ui);
                });
            });
    }
}

#[typetag::serde]
impl SerializableComponent for EntityTransform {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn SerializableComponent> {
        Box::new(*self)
    }

    fn init(&self, _ctx: ComponentInitContext) -> ComponentInitFuture {
        let value = *self;
        Box::pin(async move {
            let insert: Box<dyn dropbear_traits::ComponentInsert> =
                Box::new(InsertBundle((value,)));
            Ok(insert)
        })
    }
}

/// A type that represents a position, rotation and scale of an entity
///
/// This type is the most primitive model, as it implements most traits.
#[repr(C)]
#[derive(Debug, Clone, Deserialize, Serialize, Copy, PartialEq)]
pub struct Transform {
    /// The position of the entity as [`DVec3`]
    pub position: DVec3,
    /// The rotation of the entity as [`DQuat`]
    pub rotation: DQuat,
    /// The scale of the entity as [`DVec3`]
    pub scale: DVec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: DVec3::ZERO,
            rotation: DQuat::IDENTITY,
            scale: DVec3::ONE,
        }
    }
}

impl Transform {
    /// Creates a new default instance of Transform
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies an offset, typically used for physics based calculations where [self.scale] 
    /// is not required. 
    pub fn with_offset(&self, translation: [f32; 3], rotation: [f32; 3]) -> Self {
        let offset_pos = Vec3::from(translation).as_dvec3();
        let offset_rot = Quat::from_euler(
            glam::EulerRot::XYZ,
            rotation[0],
            rotation[1],
            rotation[2]
        ).as_dquat();

        Transform {
            position: self.position + self.rotation * offset_pos,
            rotation: self.rotation * offset_rot,
            scale: self.scale,
        }
    }

    /// Returns the matrix of the model
    pub fn matrix(&self) -> DMat4 {
        DMat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    /// Rotates the model on its X axis by a certain angle
    pub fn rotate_x(&mut self, angle_rad: f64) {
        self.rotation *= DQuat::from_euler(glam::EulerRot::XYZ, angle_rad, 0.0, 0.0);
    }

    /// Rotates the model on its Y axis by a certain value
    pub fn rotate_y(&mut self, angle_rad: f64) {
        self.rotation *= DQuat::from_euler(glam::EulerRot::XYZ, 0.0, angle_rad, 0.0);
    }

    /// Rotates the model on its Z axis by a certain value
    pub fn rotate_z(&mut self, angle_rad: f64) {
        self.rotation *= DQuat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, angle_rad);
    }

    /// Translates (moves) the model by a translation [`DVec3`].
    ///
    /// Doesn't replace the position value,
    /// it adds the value.
    pub fn translate(&mut self, translation: DVec3) {
        self.position += translation;
    }

    /// Scales the model by a scale value.
    ///
    /// Doesn't replace the scale value, just multiplies.
    pub fn scale(&mut self, scale: DVec3) {
        self.scale *= scale;
    }

    fn inspect(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Position:");
        });
        ui.horizontal(|ui| {
            ui.colored_label(egui::Color32::from_rgb(200, 80, 80), "X:");
            ui.add(egui::DragValue::new(&mut self.position.x)
                .speed(0.1)
                .fixed_decimals(2));

            ui.colored_label(egui::Color32::from_rgb(80, 200, 80), "Y:");
            ui.add(egui::DragValue::new(&mut self.position.y)
                .speed(0.1)
                .fixed_decimals(2));

            ui.colored_label(egui::Color32::from_rgb(80, 120, 220), "Z:");
            ui.add(egui::DragValue::new(&mut self.position.z)
                .speed(0.1)
                .fixed_decimals(2));
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label("Rotation:");
        });

        let (mut x, mut y, mut z) = self.rotation.to_euler(glam::EulerRot::XYZ);
        x = x.to_degrees();
        y = y.to_degrees();
        z = z.to_degrees();

        let changed = ui.horizontal(|ui| {
            ui.colored_label(egui::Color32::from_rgb(200, 80, 80), "X:");
            let cx = ui.add(egui::DragValue::new(&mut x)
                .speed(1.0)
                .suffix("°")
                .fixed_decimals(1)).changed();

            ui.colored_label(egui::Color32::from_rgb(80, 200, 80), "Y:");
            let cy = ui.add(egui::DragValue::new(&mut y)
                .speed(1.0)
                .suffix("°")
                .fixed_decimals(1)).changed();

            ui.colored_label(egui::Color32::from_rgb(80, 120, 220), "Z:");
            let cz = ui.add(egui::DragValue::new(&mut z)
                .speed(1.0)
                .suffix("°")
                .fixed_decimals(1)).changed();

            cx || cy || cz
        }).inner;

        if changed {
            self.rotation = DQuat::from_euler(
                glam::EulerRot::XYZ,
                x.to_radians(),
                y.to_radians(),
                z.to_radians()
            );
        }

        ui.add_space(4.0);

        // Scale
        ui.horizontal(|ui| {
            ui.label("Scale:");
        });

        let mut uniform_scale = self.scale.x == self.scale.y
            && self.scale.y == self.scale.z;

        ui.horizontal(|ui| {
            if ui.checkbox(&mut uniform_scale, "Uniform").changed() {
                if uniform_scale {
                    let avg = (self.scale.x + self.scale.y + self.scale.z) / 3.0;
                    self.scale = DVec3::splat(avg);
                }
            }
        });

        if uniform_scale {
            ui.horizontal(|ui| {
                ui.label("XYZ:");
                if ui.add(egui::DragValue::new(&mut self.scale.x)
                    .speed(0.01)
                    .fixed_decimals(3)).changed()
                {
                    self.scale = DVec3::splat(self.scale.x);
                }
            });
        } else {
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::from_rgb(200, 80, 80), "X:");
                ui.add(egui::DragValue::new(&mut self.scale.x)
                    .speed(0.01)
                    .fixed_decimals(3));

                ui.colored_label(egui::Color32::from_rgb(80, 200, 80), "Y:");
                ui.add(egui::DragValue::new(&mut self.scale.y)
                    .speed(0.01)
                    .fixed_decimals(3));

                ui.colored_label(egui::Color32::from_rgb(80, 120, 220), "Z:");
                ui.add(egui::DragValue::new(&mut self.scale.z)
                    .speed(0.01)
                    .fixed_decimals(3));
            });
        }
    }
}

#[derive(Clone)]
/// A renderer for meshes and materials related to a model.
///
/// It includes the instances as well as a handle. The reason for a handle is so the model being rendered can be swapped
/// to something else without deleting the entire renderer. Also saves memory by rendering anything that has been loaded.
pub struct MeshRenderer {
    import_scale: f32,

    pub is_selected: bool,
    handle: Handle<Model>,
    pub instance: Instance,
    previous_matrix: DMat4,
    texture_override: Option<Handle<Texture>>,
}

impl MeshRenderer {
    pub fn from_handle(model: Handle<Model>) -> Self {
        Self {
            handle: model,
            instance: Instance::default(),
            previous_matrix: DMat4::IDENTITY,
            import_scale: 1.0,
            texture_override: None,
            is_selected: false,
        }
    }
    
    pub async fn from_path(
        graphics: Arc<SharedGraphicsContext>,
        path: impl AsRef<Path>,
        label: Option<&str>,
    ) -> anyhow::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let handle = Model::load_from_memory_raw(
            graphics.clone(),
            std::fs::read(path)?,
            label,
            ASSET_REGISTRY.clone(),
        ).await?;
        Ok(Self {
            handle,
            instance: Instance::default(),
            import_scale: 1.0,
            previous_matrix: DMat4::IDENTITY,
            texture_override: None,
            is_selected: false,
        })
    }

    pub fn update(&mut self, transform: &Transform) {
        puffin::profile_function!();
        let scale = transform.scale * glam::DVec3::splat(self.import_scale as f64);
        let current_matrix = DMat4::from_scale_rotation_translation(
            scale,
            transform.rotation,
            transform.position,
        );
        if self.previous_matrix != current_matrix {
            self.instance = Instance::from_matrix(current_matrix);
            self.previous_matrix = current_matrix;
        }
    }

    pub fn set_import_scale(&mut self, scale: f32) {
        self.import_scale = scale;
    }

    pub fn import_scale(&self) -> f32 {
        self.import_scale
    }

    pub fn set_model(&mut self, model: Handle<Model>) {
        self.handle = model;
    }

    pub fn model(&self) -> Handle<Model> {
        self.handle
    }

    pub fn set_texture_override(&mut self, texture: Handle<Texture>) {
        self.texture_override = Some(texture);
    }

    pub fn texture_override(&self) -> Option<Handle<Texture>> {
        self.texture_override
    }

    pub fn is_texture_attached(&self, texture: Handle<Texture>) -> bool {
        let registry = ASSET_REGISTRY.read();
        
        if let Some(model) = registry.get_model(self.handle) {
            for material in &model.materials {
                if material.diffuse_texture.hash == Some(texture.id) {
                    return true;
                }
                if material.normal_texture.hash == Some(texture.id) {
                    return true;
                }
                if let Some(emissive) = &material.emissive_texture {
                    if emissive.hash == Some(texture.id) {
                        return true;
                    }
                }
                if let Some(mr) = &material.metallic_roughness_texture {
                    if mr.hash == Some(texture.id) {
                        return true;
                    }
                }
                if let Some(occ) = &material.occlusion_texture {
                    if occ.hash == Some(texture.id) {
                        return true;
                    }
                }
            }
        }
        
        false
    }

    pub fn reset_texture_override(&mut self) {
        self.texture_override = None;
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SerializedMeshRenderer {
    pub handle: ResourceReference,
    pub import_scale: Option<f32>,
    pub texture_override: Option<ResourceReference>,
}

impl Component for MeshRenderer {
    type Serialized = SerializedMeshRenderer;

    fn static_descriptor() -> ComponentDescriptor {
        ComponentDescriptor {
            fqtn: "dropbear_engine::entity::MeshRenderer".to_string(),
            type_name: "MeshRenderer".to_string(),
            category: Some("Meshes".to_string()),
            description: Some("Renders a 3D model".to_string()),
        }
    }

    fn deserialize(serialized: &Self::Serialized) -> Self {
        let handle = match serialized.handle.ref_type {
            ResourceReferenceType::Unassigned { id } => Handle::new(id),
            _ => Handle::NULL,
        };

        let mut renderer = MeshRenderer::from_handle(handle);
        if let Some(scale) = serialized.import_scale {
            renderer.set_import_scale(scale);
        }

        renderer
    }

    fn serialize(&self) -> Self::Serialized {
        let handle = self.model();
        let handle_ref = if handle.is_null() {
            ResourceReference::from_reference(ResourceReferenceType::Unassigned { id: handle.id })
        } else {
            let registry = ASSET_REGISTRY.read();
            registry
                .get_model(handle)
                .map(|model| model.path.clone())
                .unwrap_or_else(|| {
                    ResourceReference::from_reference(ResourceReferenceType::Unassigned { id: handle.id })
                })
        };

        let texture_override = self.texture_override().map(|handle| {
            let registry = ASSET_REGISTRY.read();
            let label = registry.get_label_from_texture_handle(handle);
            let reference = label.and_then(|value| {
                if value.starts_with(EUCA_SCHEME) {
                    Some(ResourceReference::from_reference(ResourceReferenceType::File(value)))
                } else {
                    None
                }
            });

            reference.unwrap_or_else(|| {
                ResourceReference::from_reference(ResourceReferenceType::Unassigned { id: handle.id })
            })
        });

        SerializedMeshRenderer {
            handle: handle_ref,
            import_scale: Some(self.import_scale()),
            texture_override,
        }
    }

    fn inspect(&mut self, ui: &mut Ui) {
        let _ = ui;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelUniform {
    model: [[f32; 4]; 4],
}

impl ModelUniform {
    pub fn new() -> Self {
        Self {
            model: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }
}
