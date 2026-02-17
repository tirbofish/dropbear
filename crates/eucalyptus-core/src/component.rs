use std::any::TypeId;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
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
use downcast_rs::{Downcast, impl_downcast};

pub use typetag::*;

pub struct ComponentRegistry {
    /// Maps TypeId to ComponentDescriptor for quick lookups
    descriptors: HashMap<TypeId, ComponentDescriptor>,
    /// Maps fully qualified type name to TypeId for lookups by string
    fqtn_to_type: HashMap<String, TypeId>,
    /// Maps category name to list of TypeIds in that category
    categories: HashMap<String, Vec<TypeId>>,
    /// Maps serialized TypeId to component TypeId
    serialized_to_component: HashMap<TypeId, TypeId>,
    /// Functions that extract and serialize components from entities
    extractors: HashMap<TypeId, ExtractorFn>,
    /// Functions that allow for the entity to load.
    loaders: HashMap<TypeId, LoaderFn>,
    /// Functions that update the contents of the component. 
    updaters: HashMap<TypeId, UpdateFn>,
    /// Functions that create default serialized components.
    defaults: HashMap<TypeId, DefaultFn>,
    /// Functions that remove components by type.
    removers: HashMap<TypeId, RemoveFn>,
    /// Functions that find entities with a component.
    finders: HashMap<TypeId, FindFn>,
}

/// Describes a handy little future for [`Component::init`], which deals with initialising a component from its serialized form. 
/// 
/// Typically thrown in as a return parameter as `-> ComponentInitFuture<'a, Self>`
pub type ComponentInitFuture<'a, T: Component> = std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<T::RequiredComponentTypes>> + Send + Sync + 'a>>;

type LoaderFuture<'a> = Pin<Box<
    dyn Future<Output = anyhow::Result<Box<dyn for<'b> FnOnce(&'b mut hecs::EntityBuilder) + Send + Sync>>> + Send + Sync + 'a
>>;
type LoaderFn = Box<
    dyn for<'a> Fn(&'a dyn SerializedComponent, Arc<SharedGraphicsContext>) -> LoaderFuture<'a>
        + Send
        + Sync
>;
type ExtractorFn = Box<dyn Fn(&hecs::World, hecs::Entity) -> Option<Box<dyn SerializedComponent>> + Send + Sync>;
type UpdateFn = Box<dyn Fn(&mut hecs::World, f32, Arc<SharedGraphicsContext>) + Send + Sync>;
type DefaultFn = Box<dyn Fn() -> Box<dyn SerializedComponent> + Send + Sync>;
type RemoveFn = Box<dyn Fn(&mut hecs::World, hecs::Entity) + Send + Sync>;
type FindFn = Box<dyn Fn(&hecs::World) -> Vec<hecs::Entity> + Send + Sync>;

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            descriptors: HashMap::new(),
            fqtn_to_type: HashMap::new(),
            categories: HashMap::new(),
            serialized_to_component: HashMap::new(),
            extractors: HashMap::new(),
            loaders: HashMap::new(),
            updaters: HashMap::new(),
            defaults: HashMap::new(),
            removers: HashMap::new(),
            finders: HashMap::new(),
        }
    }

    /// Register a component type with the registry
    pub fn register<T>(&mut self)
    where
        T: Component + Send + Sync + 'static,
        T::SerializedForm: 'static + Default,
        T::RequiredComponentTypes: Send + Sync,
    {
        let type_id = TypeId::of::<T>();
        let serialized_type_id = TypeId::of::<T::SerializedForm>();
        let desc = T::descriptor();

        self.fqtn_to_type.insert(desc.fqtn.clone(), type_id);
        if let Some(ref cat) = desc.category {
            self.categories.entry(cat.clone()).or_default().push(type_id);
        }
        self.descriptors.insert(type_id, desc);
        self.serialized_to_component
            .insert(serialized_type_id, type_id);

        self.extractors.insert(type_id, Box::new(|world, entity| {
            let Ok(c) = world.get::<&T>(entity) else { return None };
            Some(c.save(world, entity))
        }));

        self.loaders.insert(serialized_type_id, Box::new(|serialized, graphics| {
            let serialized = serialized
                .as_any()
                .downcast_ref::<T::SerializedForm>()
                .expect("type mismatch in loader — registry bug");

            Box::pin(async move {
                let bundle = T::init(serialized, graphics).await?;
                let applier: Box<dyn FnOnce(&mut hecs::EntityBuilder) + Send + Sync> =
                    Box::new(move |builder: &mut hecs::EntityBuilder| {
                        builder.add_bundle(bundle);
                    });
                Ok(applier)
            })
        }));

        self.defaults.insert(type_id, Box::new(|| {
            Box::new(T::SerializedForm::default())
        }));

        self.removers.insert(type_id, Box::new(|world, entity| {
            let _ = world.remove_one::<T>(entity);
        }));

        self.finders.insert(type_id, Box::new(|world| {
            world
                .query::<(hecs::Entity, &T)>()
                .iter()
                .map(|(entity, _)| entity)
                .collect()
        }));

        self.updaters.insert(type_id, Box::new(|world, dt, graphics| {
            let world_ptr = world as *mut hecs::World; // safe assuming world is kept at the DropbearAppBuilder application level (lifetime)
            let mut query = world.query::<(hecs::Entity, &mut T)>();
            for (entity, component) in query.iter() {
                let world_ref = unsafe { &*world_ptr };
                component.update_component(world_ref, entity, dt, graphics.clone());
            }
        }));
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

    /// Iterates available component descriptors alongside their numeric ids.
    pub fn iter_available_components(&self) -> impl Iterator<Item = (u64, &ComponentDescriptor)> {
        self.descriptors
            .iter()
            .map(|(type_id, desc)| (Self::numeric_id(*type_id), desc))
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

    /// Gets the numeric id for a serialized component instance.
    pub fn id_for_component(&self, component: &dyn SerializedComponent) -> Option<u64> {
        let serialized_type_id = component.as_any().type_id();
        self.serialized_to_component
            .get(&serialized_type_id)
            .copied()
            .map(Self::numeric_id)
    }

    /// Creates a default serialized component by numeric id.
    pub fn create_default_component(&self, id: u64) -> Option<Box<dyn SerializedComponent>> {
        self.type_id_from_numeric_id(id)
            .and_then(|type_id| self.defaults.get(&type_id))
            .map(|create| create())
    }

    /// Removes a component by numeric id from an entity.
    pub fn remove_component_by_id(
        &self,
        world: &mut hecs::World,
        entity: hecs::Entity,
        id: u64,
    ) {
        if let Some(type_id) = self.type_id_from_numeric_id(id) {
            if let Some(remover) = self.removers.get(&type_id) {
                remover(world, entity);
            }
        }
    }

    /// Finds entities with the component matching a numeric id.
    pub fn find_entities_by_numeric_id(
        &self,
        world: &hecs::World,
        id: u64,
    ) -> Vec<hecs::Entity> {
        self.type_id_from_numeric_id(id)
            .and_then(|type_id| self.finders.get(&type_id))
            .map(|finder| finder(world))
            .unwrap_or_default()
    }

    /// Gets a component descriptor by numeric id.
    pub fn get_descriptor_by_numeric_id(&self, id: u64) -> Option<&ComponentDescriptor> {
        self.type_id_from_numeric_id(id)
            .and_then(|type_id| self.descriptors.get(&type_id))
    }

    /// Create a component applier from a serialized component using the registry loader.
    pub fn load_component<'a>(
        &'a self,
        serialized: &'a dyn SerializedComponent,
        graphics: Arc<SharedGraphicsContext>,
    ) -> Option<LoaderFuture<'a>> {
        let serialized_type_id = serialized.as_any().type_id();
        self.loaders
            .get(&serialized_type_id)
            .map(|loader| loader(serialized, graphics))
    }

    /// Updates all registered components that exist in the world.
    pub fn update_components(
        &self,
        world: &mut hecs::World,
        dt: f32,
        graphics: Arc<SharedGraphicsContext>,
    ) {
        for updater in self.updaters.values() {
            updater(world, dt, graphics.clone());
        }
    }

    /// Inspects all registered components attached to an entity.
    pub fn inspect_components(
        &self,
        world: &mut hecs::World,
        entity: hecs::Entity,
        ui: &mut egui::Ui,
    ) {
        if let Ok(mut label) = world.get::<&mut crate::states::Label>(entity) {
            ui.horizontal(|ui| {
                ui.label("Label");
                ui.add(egui::TextEdit::singleline(label.as_mut_string()));
            });
            ui.separator();
        }

        let type_ids = world
            .entity(entity)
            .map(|e| e.component_types().collect::<Vec<_>>())
            .unwrap_or_default();

        for type_id in type_ids {
            let Some(desc) = self.descriptors.get(&type_id) else {
                continue;
            };

            match desc.fqtn.as_str() {
                "dropbear_engine::entity::EntityTransform" => {
                    if let Ok(mut comp) = world.get::<&mut EntityTransform>(entity) {
                        comp.inspect(ui);
                    }
                }
                "eucalyptus_core::properties::CustomProperties" => {
                    if let Ok(mut comp) = world.get::<&mut crate::properties::CustomProperties>(entity) {
                        comp.inspect(ui);
                    }
                }
                "dropbear_engine::lighting::Light" => {
                    if let Ok(mut comp) = world.get::<&mut dropbear_engine::lighting::Light>(entity) {
                        comp.inspect(ui);
                    }
                }
                "eucalyptus_core::states::Script" => {
                    if let Ok(mut comp) = world.get::<&mut crate::states::Script>(entity) {
                        comp.inspect(ui);
                    }
                }
                "dropbear_engine::entity::MeshRenderer" => {
                    if let Ok(mut comp) = world.get::<&mut MeshRenderer>(entity) {
                        comp.inspect(ui);
                    }
                }
                "dropbear_engine::camera::Camera" => {
                    if let Ok(mut comp) = world.get::<&mut dropbear_engine::camera::Camera>(entity) {
                        comp.inspect(ui);
                    }
                }
                "eucalyptus_core::physics::rigidbody::RigidBody" => {
                    if let Ok(mut comp) = world.get::<&mut crate::physics::rigidbody::RigidBody>(entity) {
                        comp.inspect(ui);
                    }
                }
                "eucalyptus_core::physics::collider::ColliderGroup" => {
                    if let Ok(mut comp) = world.get::<&mut crate::physics::collider::ColliderGroup>(entity) {
                        comp.inspect(ui);
                    }
                }
                "eucalyptus_core::physics::kcc::KCC" => {
                    if let Ok(mut comp) = world.get::<&mut crate::physics::kcc::KCC>(entity) {
                        comp.inspect(ui);
                    }
                }
                "dropbear_engine::animation::AnimationComponent" => {
                    if let Ok(mut comp) = world.get::<&mut dropbear_engine::animation::AnimationComponent>(entity) {
                        comp.inspect(ui);
                    }
                }
                _ => {
                    ui.label(format!("{} (no inspector)", desc.type_name));
                }
            }
        }
    }

    fn numeric_id(type_id: TypeId) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        type_id.hash(&mut hasher);
        let mut id = hasher.finish();
        if id == 0 {
            id = 1;
        }
        id
    }

    fn type_id_from_numeric_id(&self, id: u64) -> Option<TypeId> {
        self.descriptors
            .keys()
            .copied()
            .find(|type_id| Self::numeric_id(*type_id) == id)
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// A blanket trait for types that can be serialized as a component.
#[typetag::serde(tag = "type")]
pub trait SerializedComponent: Downcast + dyn_clone::DynClone + Send + Sync {}

impl_downcast!(SerializedComponent);
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
pub trait Component: Sized + Sync + Send {
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
    fn init<'a>(
        ser: &'a Self::SerializedForm,
        graphics: Arc<SharedGraphicsContext>,
    ) -> ComponentInitFuture<'a, Self>;

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

    fn init<'a>(
        ser: &'a Self::SerializedForm,
        graphics: Arc<SharedGraphicsContext>,
    ) -> ComponentInitFuture<'a, Self> {
        Box::pin(async move {
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
                    Model::load_from_memory_raw(
                        graphics.clone(),
                        buffer,
                        Some(ser.label.as_str()),
                        ASSET_REGISTRY.clone(),
                    )
                    .await?
                }
                ResourceReferenceType::Bytes(bytes) => {
                    log::debug!("Loading model from bytes [Len: {}]", bytes.len());
                    Model::load_from_memory_raw(
                        graphics.clone(),
                        bytes,
                        Some(ser.label.as_str()),
                        ASSET_REGISTRY.clone(),
                    )
                    .await?
                }
                ResourceReferenceType::ProcObj(obj) => match obj {
                    dropbear_engine::procedural::ProcObj::Cuboid { size_bits } => {
                        let size = [
                            f32::from_bits(size_bits[0]),
                            f32::from_bits(size_bits[1]),
                            f32::from_bits(size_bits[2]),
                        ];
                        log::debug!("Loading model from cuboid: {:?}", size);

                        let size_vec = glam::DVec3::new(
                            size[0] as f64,
                            size[1] as f64,
                            size[2] as f64,
                        );
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

            for (label, m) in &ser.texture_override {
                if let Some(mat) = renderer.material_snapshot.get_mut(&label.clone()) {
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
                    mat.texture_tag = m.texture_tag.clone();
                    mat.wrap_mode = m.wrap_mode;

                    let get_tex = async |resource: &Option<ResourceReference>| -> Option<anyhow::Result<Texture>> {
                        if let Some(dif) = resource {
                            match &dif.ref_type {
                                ResourceReferenceType::None => {
                                    None
                                }
                                ResourceReferenceType::File(_) => {
                                    let path = dif.resolve().ok()?;
                                    let tex = Texture::from_file(graphics.clone(), &path, Some(&label)).await;
                                    Some(tex)
                                }
                                ResourceReferenceType::Bytes(bytes) => {
                                    let tex = Texture::from_bytes(graphics.clone(), bytes, Some(&label));
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

                    if let Some(tex) = get_tex(&m.diffuse_texture).await {
                        mat.diffuse_texture = tex?;
                    }

                    if let Some(tex) = get_tex(&m.emissive_texture).await {
                        mat.emissive_texture = Some(tex?);
                    }

                    if let Some(tex) = get_tex(&m.normal_texture).await {
                        mat.normal_texture = tex?;
                    }

                    if let Some(tex) = get_tex(&m.occlusion_texture).await {
                        mat.occlusion_texture = Some(tex?);
                    }

                    if let Some(tex) = get_tex(&m.metallic_roughness_texture).await {
                        mat.metallic_roughness_texture = Some(tex?);
                    }
                }
            }

            Ok((renderer, ))
        })
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

