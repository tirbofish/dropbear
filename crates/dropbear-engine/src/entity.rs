use glam::{DMat4, DQuat, DVec3, Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path, sync::Arc};

use crate::asset::Handle;
use crate::model::{Material, NodeTransform};
use crate::{
    asset::ASSET_REGISTRY,
    graphics::{Instance, SharedGraphicsContext},
    model::Model,
    texture::Texture,
    utils::ResourceReference,
};
use egui::Ui;

/// A type of transform that is attached to all entities. It contains the local and world transforms.
#[derive(Default, Debug, Deserialize, Serialize, Copy, PartialEq, Clone)]
pub struct EntityTransform {
    local: Transform,
    world: Transform,
    #[serde(default)]
    animation: Transform,
}

impl EntityTransform {
    /// Creates a new [EntityTransform] from a local and world [Transform]
    pub fn new(local: Transform, world: Transform) -> Self {
        Self {
            local,
            world,
            animation: Transform::default(),
        }
    }

    /// Creates a new [EntityTransform] from a world [Transform] and a default local transform.
    ///
    /// This is best for situations where a local transform is not required.
    pub fn new_from_world(world: Transform) -> Self {
        Self {
            world,
            local: Transform::default(),
            animation: Transform::default(),
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
        let combined = self.world.matrix() * self.local.matrix() * self.animation.matrix();
        let (scale, rotation, position) = combined.to_scale_rotation_translation();

        Transform {
            position,
            rotation,
            scale,
        }
    }

    /// Applies a node transform for TRS animation as an absolute local transform.
    pub fn apply_animation(&mut self, node_transform: &NodeTransform) {
        self.animation.position = node_transform.translation.as_dvec3();
        self.animation.rotation = node_transform.rotation.as_dquat();
        self.animation.scale = node_transform.scale.as_dvec3();
    }

    /// Clears the animation contribution to the local transform.
    pub fn clear_animation(&mut self) {
        self.animation = Transform::default();
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
        let offset_rot =
            Quat::from_euler(glam::EulerRot::XYZ, rotation[0], rotation[1], rotation[2]).as_dquat();

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

    pub fn inspect(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Position:");
        });
        ui.horizontal(|ui| {
            ui.colored_label(egui::Color32::from_rgb(200, 80, 80), "X:");
            ui.add(
                egui::DragValue::new(&mut self.position.x)
                    .speed(0.1)
                    .fixed_decimals(2),
            );

            ui.colored_label(egui::Color32::from_rgb(80, 200, 80), "Y:");
            ui.add(
                egui::DragValue::new(&mut self.position.y)
                    .speed(0.1)
                    .fixed_decimals(2),
            );

            ui.colored_label(egui::Color32::from_rgb(80, 120, 220), "Z:");
            ui.add(
                egui::DragValue::new(&mut self.position.z)
                    .speed(0.1)
                    .fixed_decimals(2),
            );
        });
        if ui.button("Reset Position").clicked() {
            self.position = DVec3::ZERO;
        }

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label("Rotation:");
        });

        let (mut x, mut y, mut z) = self.rotation.to_euler(glam::EulerRot::XYZ);
        x = x.to_degrees();
        y = y.to_degrees();
        z = z.to_degrees();

        let changed = ui
            .horizontal(|ui| {
                ui.colored_label(egui::Color32::from_rgb(200, 80, 80), "X:");
                let cx = ui
                    .add(
                        egui::DragValue::new(&mut x)
                            .speed(1.0)
                            .suffix("°")
                            .fixed_decimals(1),
                    )
                    .changed();

                ui.colored_label(egui::Color32::from_rgb(80, 200, 80), "Y:");
                let cy = ui
                    .add(
                        egui::DragValue::new(&mut y)
                            .speed(1.0)
                            .suffix("°")
                            .fixed_decimals(1),
                    )
                    .changed();

                ui.colored_label(egui::Color32::from_rgb(80, 120, 220), "Z:");
                let cz = ui
                    .add(
                        egui::DragValue::new(&mut z)
                            .speed(1.0)
                            .suffix("°")
                            .fixed_decimals(1),
                    )
                    .changed();

                cx || cy || cz
            })
            .inner;

        if ui.button("Reset Rotation").clicked() {
            self.rotation = DQuat::IDENTITY;
        }

        if changed {
            self.rotation = DQuat::from_euler(
                glam::EulerRot::XYZ,
                x.to_radians(),
                y.to_radians(),
                z.to_radians(),
            );
        }

        ui.add_space(4.0);

        // Scale
        ui.horizontal(|ui| {
            ui.label("Scale:");
        });

        ui.horizontal(|ui| {
            ui.colored_label(egui::Color32::from_rgb(200, 80, 80), "X:");
            ui.add(
                egui::DragValue::new(&mut self.scale.x)
                    .speed(0.01)
                    .fixed_decimals(3),
            );

            ui.colored_label(egui::Color32::from_rgb(80, 200, 80), "Y:");
            ui.add(
                egui::DragValue::new(&mut self.scale.y)
                    .speed(0.01)
                    .fixed_decimals(3),
            );

            ui.colored_label(egui::Color32::from_rgb(80, 120, 220), "Z:");
            ui.add(
                egui::DragValue::new(&mut self.scale.z)
                    .speed(0.01)
                    .fixed_decimals(3),
            );
        });

        if ui.button("Reset Scale").clicked() {
            self.scale = DVec3::ONE;
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
    pub material_snapshot: HashMap<String, Material>,
}

impl MeshRenderer {
    pub fn from_handle(model: Handle<Model>) -> Self {
        let mut hm = HashMap::new();
        let material_snapshot = ASSET_REGISTRY
            .read()
            .get_model(model)
            .map(|m| m.materials.clone())
            .unwrap_or_default();
        for m in material_snapshot {
            hm.insert(m.name.clone(), m);
        }
        Self {
            handle: model,
            instance: Instance::default(),
            previous_matrix: DMat4::IDENTITY,
            import_scale: 1.0,
            is_selected: false,
            material_snapshot: hm,
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
            std::fs::read(&path)?,
            Some(ResourceReference::from_path(&path)?),
            label,
            ASSET_REGISTRY.clone(),
        )
        .await?;
        Ok(Self {
            handle,
            instance: Instance::default(),
            import_scale: 1.0,
            previous_matrix: DMat4::IDENTITY,
            is_selected: false,
            material_snapshot: Default::default(),
        })
    }

    pub fn update(&mut self, transform: &Transform) {
        puffin::profile_function!();
        let scale = transform.scale * glam::DVec3::splat(self.import_scale as f64);
        let current_matrix =
            DMat4::from_scale_rotation_translation(scale, transform.rotation, transform.position);
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

    pub fn mutate_material(&mut self, material_name: &str, f: impl FnOnce(&mut Material)) {
        self.material_snapshot
            .entry(material_name.to_string())
            .and_modify(f);
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
        let mut hm = HashMap::new();
        let material_snapshot = ASSET_REGISTRY
            .read()
            .get_model(self.handle)
            .map(|m| m.materials.clone())
            .unwrap_or_default();
        for m in material_snapshot {
            hm.insert(m.name.clone(), m);
        }
        self.material_snapshot = hm;
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
