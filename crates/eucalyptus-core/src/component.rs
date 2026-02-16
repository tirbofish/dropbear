use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;
use egui::{CollapsingHeader, Ui};
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};
use dropbear_engine::asset::{Handle, ASSET_REGISTRY};
use dropbear_engine::entity::{EntityTransform, MeshRenderer};
use dropbear_engine::graphics::SharedGraphicsContext;
use dropbear_engine::model::{Model};
use dropbear_engine::procedural::ProcedurallyGeneratedObject;
use dropbear_engine::texture::Texture;
use dropbear_engine::utils::{ResourceReference, ResourceReferenceType};
use crate::hierarchy::EntityTransformExt;
use crate::states::{SerializedMaterialCustomisation, SerializedMeshRenderer};
use crate::utils::ResolveReference;

pub use typetag::*;

pub struct ComponentRegistry {
    /// Maps TypeId to ComponentDescriptor for quick lookups
    descriptors: HashMap<TypeId, ComponentDescriptor>,
    /// Maps fully qualified type name to TypeId for lookups by string
    fqtn_to_type: HashMap<String, TypeId>,
    /// Maps category name to list of TypeIds in that category
    categories: HashMap<String, Vec<TypeId>>,
    /// Functions that extract and serialize components from entities
    extractors: HashMap<TypeId, ExtractorFn>,
}

type ExtractorFn = Box<dyn Fn(&hecs::World, hecs::Entity) -> Option<Box<dyn SerializedComponent>> + Send + Sync>;

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            descriptors: HashMap::new(),
            fqtn_to_type: HashMap::new(),
            categories: HashMap::new(),
            extractors: Default::default(),
        }
    }

    /// Register a component type with the registry
    pub fn register<T: Component + 'static + Sync + Send>(&mut self) {
        let type_id = TypeId::of::<T>();
        let descriptor = T::descriptor();

        self.fqtn_to_type.insert(descriptor.fqtn.clone(), type_id);

        if let Some(ref category) = descriptor.category {
            self.categories
                .entry(category.clone())
                .or_insert_with(Vec::new)
                .push(type_id);
        }

        self.extractors.insert(
            type_id,
            Box::new(|world: &hecs::World, entity: hecs::Entity| {
                world.get::<&T>(entity).ok().map(|component| {
                    component.save(world, entity)
                })
            })
        );

        self.descriptors.insert(type_id, descriptor);
    }

    /// Get descriptor for a specific component type
    pub fn get_descriptor<T: Component + 'static>(&self) -> Option<&ComponentDescriptor> {
        self.descriptors.get(&TypeId::of::<T>())
    }

    /// Get descriptor by fully qualified type name
    pub fn get_descriptor_by_fqtn(&self, fqtn: &str) -> Option<&ComponentDescriptor> {
        self.fqtn_to_type
            .get(fqtn)
            .and_then(|type_id| self.descriptors.get(type_id))
    }

    /// Get all registered component descriptors
    pub fn all_descriptors(&self) -> impl Iterator<Item = &ComponentDescriptor> {
        self.descriptors.values()
    }

    /// Get all component descriptors in a specific category
    pub fn descriptors_in_category(&self, category: &str) -> Vec<&ComponentDescriptor> {
        self.categories
            .get(category)
            .map(|type_ids| {
                type_ids
                    .iter()
                    .filter_map(|type_id| self.descriptors.get(type_id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all category names
    pub fn categories(&self) -> impl Iterator<Item = &String> {
        self.categories.keys()
    }

    /// Check if a component type is registered
    pub fn is_registered<T: Component + 'static>(&self) -> bool {
        self.descriptors.contains_key(&TypeId::of::<T>())
    }

    /// Get the TypeId for a component by its fully qualified type name
    pub fn get_type_id(&self, fqtn: &str) -> Option<TypeId> {
        self.fqtn_to_type.get(fqtn).copied()
    }

    /// Get count of registered components
    pub fn count(&self) -> usize {
        self.descriptors.len()
    }

    /// Extract all registered components from an entity
    pub fn extract_all_components(
        &self,
        world: &hecs::World,
        entity: hecs::Entity,
    ) -> Vec<Box<dyn SerializedComponent>> {
        self.extractors
            .values()
            .filter_map(|extractor| extractor(world, entity))
            .collect()
    }

    /// Extract a specific component by type
    pub fn extract_component<T: Component + 'static>(
        &self,
        world: &hecs::World,
        entity: hecs::Entity,
    ) -> Option<Box<dyn SerializedComponent>> {
        let type_id = TypeId::of::<T>();
        self.extractors
            .get(&type_id)
            .and_then(|extractor| extractor(world, entity))
    }

    /// Extract components by category
    pub fn extract_components_in_category(
        &self,
        world: &hecs::World,
        entity: hecs::Entity,
        category: &str,
    ) -> Vec<Box<dyn SerializedComponent>> {
        self.categories
            .get(category)
            .map(|type_ids| {
                type_ids
                    .iter()
                    .filter_map(|type_id| {
                        self.extractors
                            .get(type_id)
                            .and_then(|extractor| extractor(world, entity))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// A blanket trait for types that can be serialized as a component.
#[typetag::serde(tag = "type")]
pub trait SerializedComponent: dyn_clone::DynClone + Send + Sync {}

dyn_clone::clone_trait_object!(SerializedComponent);

#[derive(Clone, Debug)]
pub struct ComponentDescriptor {
    /// Fully qualified type name of the component, such as `eucalyptus_core::components::MeshRenderer`.
    pub fqtn: String,
    /// Short name of the component, such as `MeshRenderer`.
    pub type_name: String,
    /// Category of the component, such as `Rendering`.
    pub category: Option<String>,
    /// Description of the component, such as `Renders a 3D model`.
    pub description: Option<String>,
}

/// Defines a type that can be considered a component of an entity.
pub trait Component: Sized {
    /// A custom format of the component for saving the state of the component to disk.
    ///
    /// To have your type available, you must include this blanket trait:
    /// ```rust no-run
    /// #[typetag::serde]
    /// impl SerializedComponent for T {}
    /// ```
    type SerializedForm: Serialize + for<'de> Deserialize<'de> + SerializedComponent;

    /// Defines all output types for any type of init function or any world query.
    ///
    /// The default is typically `(Self, )`, however you can even define it as `(Self, Transform, ...)`.
    type RequiredComponentTypes: hecs::DynamicBundle;

    fn descriptor() -> ComponentDescriptor;

    /// Creates a new instance of the component for times when there is no existing component to
    /// initialise from.
    async fn first_time(graphics: Arc<SharedGraphicsContext>) -> anyhow::Result<Self::RequiredComponentTypes>;

    /// Converts [`Self::SerializedForm`] into a [`Component`] instance that can be added to
    /// `hecs::EntityBuilder` during scene initialisation.
    async fn init(ser: Self::SerializedForm, graphics: Arc<SharedGraphicsContext>) -> anyhow::Result<Self::RequiredComponentTypes>;

    /// Called every frame to update the component's state.
    fn update_component(&mut self, world: &hecs::World, entity: hecs::Entity, dt: f32, graphics: Arc<SharedGraphicsContext>);

    /// Called when saving the scene to disk. Returns the [`Self::SerializedForm`] of the component that can be
    /// saved to disk.
    fn save(&self, world: &hecs::World, entity: hecs::Entity) -> Box<dyn SerializedComponent>;

    /// In the editor, how the component will be represented in the `Resource Viewer` dock.
    fn inspect(&mut self, ui: &mut egui::Ui);
}

#[typetag::serde]
impl SerializedComponent for SerializedMeshRenderer {}

// sample for MeshRenderer
impl Component for MeshRenderer {
    type SerializedForm = SerializedMeshRenderer;
    type RequiredComponentTypes = (Self, );

    fn descriptor() -> ComponentDescriptor {
        ComponentDescriptor {
            fqtn: "dropbear_engine::entity::MeshRenderer".to_string(),
            type_name: "MeshRenderer".to_string(),
            category: Some("Rendering".to_string()),
            description: Some("Renders a mesh".to_string()),
        }
    }

    async fn first_time(_graphics: Arc<SharedGraphicsContext>) -> anyhow::Result<Self::RequiredComponentTypes>
    where
        Self: Sized
    {
        Ok((MeshRenderer::from_handle(Handle::NULL), ))
    }

    async fn init(ser: Self::SerializedForm, graphics: Arc<SharedGraphicsContext>) -> anyhow::Result<Self::RequiredComponentTypes> {
        let import_scale = ser.import_scale.unwrap_or(1.0);

        let handle = match &ser.handle.ref_type {
            ResourceReferenceType::None => {
                log::debug!("ResourceReferenceType is None, setting to `Handle::NULL`");
                Handle::NULL
            }
            ResourceReferenceType::File(_) => {
                log::debug!("Loading model from file: {:?}", ser.handle);
                let path = ser.handle.resolve()?;
                let buffer = std::fs::read(&path)?;
                Model::load_from_memory_raw(graphics.clone(), buffer, Some(ser.label.as_str()), ASSET_REGISTRY.clone()).await?
            }
            ResourceReferenceType::Bytes(bytes) => {
                log::debug!("Loading model from bytes [Len: {}]", bytes.len());
                Model::load_from_memory_raw(graphics.clone(), bytes, Some(ser.label.as_str()), ASSET_REGISTRY.clone()).await?
            }
            ResourceReferenceType::ProcObj(obj) => match obj {
                dropbear_engine::procedural::ProcObj::Cuboid { size_bits } => {
                    let size = [
                        f32::from_bits(size_bits[0]),
                        f32::from_bits(size_bits[1]),
                        f32::from_bits(size_bits[2]),
                    ];
                    log::debug!("Loading model from cuboid: {:?}", size);

                    let size_vec = glam::DVec3::new(size[0] as f64, size[1] as f64, size[2] as f64);
                    ProcedurallyGeneratedObject::cuboid(size_vec).build_model(
                        graphics.clone(),
                        None,
                        Some(ser.label.as_str()),
                        ASSET_REGISTRY.clone(),
                    )
                }
            },
        };

        let mut renderer = MeshRenderer::from_handle(handle);
        renderer.set_import_scale(import_scale);

        for (label, m) in ser.texture_override {
            if let Some(mat) = renderer.material_snapshot.get_mut(&label) {
                mat.tint = m.tint;
                mat.emissive_factor = m.emissive_factor;
                mat.metallic_factor = m.metallic_factor;
                mat.roughness_factor = m.roughness_factor;
                mat.alpha_mode = m.alpha_mode;
                mat.alpha_cutoff = m.alpha_cutoff;
                mat.double_sided = m.double_sided;
                mat.occlusion_strength = m.occlusion_strength;
                mat.normal_scale = m.normal_scale;
                mat.uv_tiling = m.uv_tiling;
                mat.texture_tag = m.texture_tag;
                mat.wrap_mode = m.wrap_mode;

                let get_tex = async |resource: Option<ResourceReference>| -> Option<anyhow::Result<Texture>> {
                    if let Some(dif) = resource {
                        match dif.ref_type {
                            ResourceReferenceType::None => {
                                None
                            }
                            ResourceReferenceType::File(_) => {
                                let path = dif.resolve().ok()?;
                                let tex = Texture::from_file(graphics.clone(), &path, Some(&label)).await;
                                Some(tex)
                            }
                            ResourceReferenceType::Bytes(bytes) => {
                                let tex = Texture::from_bytes(graphics.clone(), &bytes, Some(&label));
                                Some(Ok(tex))
                            }
                            ResourceReferenceType::ProcObj(_) => {
                                Some(Err(anyhow::anyhow!("Using a ProcObj as a texture is not valid, for texture with label {}", label)))
                            }
                        }
                    } else {
                        None
                    }
                };

                if let Some(tex) = get_tex(m.diffuse_texture).await {
                    mat.diffuse_texture = tex?;
                }

                if let Some(tex) = get_tex(m.emissive_texture).await {
                    mat.emissive_texture = Some(tex?);
                }

                if let Some(tex) = get_tex(m.normal_texture).await {
                    mat.normal_texture = tex?;
                }

                if let Some(tex) = get_tex(m.occlusion_texture).await {
                    mat.occlusion_texture = Some(tex?);
                }

                if let Some(tex) = get_tex(m.metallic_roughness_texture).await {
                    mat.metallic_roughness_texture = Some(tex?);
                }
            }
        }

        Ok((renderer, ))
    }

    fn update_component(&mut self, world: &World, entity: Entity, _dt: f32, _graphics: Arc<SharedGraphicsContext>) {
        if let Ok((mesh, transform)) = world.query_one::<(&mut MeshRenderer, &EntityTransform)>(entity).get() {
            mesh.update(&transform.propagate(&world, entity))
        }
    }

    fn save(&self, _world: &World, _entity: Entity) -> Box<dyn SerializedComponent> {
        let asset = ASSET_REGISTRY.read();
        let model = asset.get_model(self.model());

        let mut texture_override: HashMap<String, SerializedMaterialCustomisation> = HashMap::new();
        for (label, mat) in &self.material_snapshot {
            let diffuse_texture = model
                .and_then(|m| m.materials.iter()
                    .find(|material|
                        material.diffuse_texture.label.as_ref() == Some(&label) &&
                            material.diffuse_texture.reference == mat.diffuse_texture.reference
                    )
                    .and_then(|material| material.diffuse_texture.reference.clone())
                );

            let normal_texture = model
                .and_then(|m| m.materials.iter()
                    .find(|material|
                        material.normal_texture.label.as_ref() == Some(&label) &&
                            material.normal_texture.reference == mat.normal_texture.reference
                    )
                    .and_then(|material| material.normal_texture.reference.clone())
                );

            let emissive_texture = model
                .and_then(|m| {
                    let mat_ref = mat.emissive_texture.as_ref()?.reference.as_ref()?;
                    m.materials.iter()
                        .find_map(|material| {
                            let nt = material.emissive_texture.as_ref()?;
                            if nt.label.as_ref() == Some(&label) && nt.reference.as_ref() == Some(mat_ref) {
                                nt.reference.clone()
                            } else {
                                None
                            }
                        })
                });

            let occlusion_texture = model
                .and_then(|m| {
                    let mat_ref = mat.occlusion_texture.as_ref()?.reference.as_ref()?;
                    m.materials.iter()
                        .find_map(|material| {
                            let nt = material.occlusion_texture.as_ref()?;
                            if nt.label.as_ref() == Some(&label) && nt.reference.as_ref() == Some(mat_ref) {
                                nt.reference.clone()
                            } else {
                                None
                            }
                        })
                });

            let metallic_roughness_texture = model
                .and_then(|m| {
                    let mat_ref = mat.metallic_roughness_texture.as_ref()?.reference.as_ref()?;
                    m.materials.iter()
                        .find_map(|material| {
                            let nt = material.metallic_roughness_texture.as_ref()?;
                            if nt.label.as_ref() == Some(&label) && nt.reference.as_ref() == Some(mat_ref) {
                                nt.reference.clone()
                            } else {
                                None
                            }
                        })
                });


            texture_override.insert(label.to_string(), SerializedMaterialCustomisation {
                label: label.clone(),
                diffuse_texture,
                tint: mat.tint,
                emissive_factor: mat.emissive_factor,
                metallic_factor: mat.metallic_factor,
                roughness_factor: mat.roughness_factor,
                alpha_mode: mat.alpha_mode,
                alpha_cutoff: mat.alpha_cutoff,
                double_sided: mat.double_sided,
                occlusion_strength: mat.occlusion_strength,
                normal_scale: mat.normal_scale,
                uv_tiling: mat.uv_tiling,
                texture_tag: mat.texture_tag.clone(),
                wrap_mode: mat.wrap_mode,
                emissive_texture,
                normal_texture,
                occlusion_texture,
                metallic_roughness_texture,
            });
        }

        Box::new(SerializedMeshRenderer {
            label: model.unwrap().label.clone(),
            handle: Default::default(),
            import_scale: None,
            texture_override,
        })
    }

    fn inspect(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Mesh Renderer").show(ui, |ui| {
            ui.label("Not implemented yet (MeshRenderer)");
        });
    }
}

