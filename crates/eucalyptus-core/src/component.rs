use std::any::TypeId;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use egui::{CollapsingHeader, ComboBox, DragValue, RichText, UiBuilder};
use hecs::{Entity, World};
pub use serde::{Deserialize, Serialize};
use dropbear_engine::asset::{Handle, ASSET_REGISTRY};
use dropbear_engine::entity::{EntityTransform, MeshRenderer};
use dropbear_engine::graphics::SharedGraphicsContext;
use dropbear_engine::model::{Model};
use dropbear_engine::procedural::{ProcObjType, ProcedurallyGeneratedObject};
use dropbear_engine::texture::Texture;
use dropbear_engine::utils::{ResourceReference, ResourceReferenceType};
use crate::hierarchy::EntityTransformExt;
use crate::physics::PhysicsState;
use crate::states::{SerializedMaterialCustomisation, SerializedMeshRenderer};
use crate::utils::ResolveReference;
use downcast_rs::{Downcast, impl_downcast};

pub use typetag::*;

pub const DRAGGED_ASSET_ID: &str = "dragged_asset_reference";

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
    /// Allows for inspecting the component in the Resource Inspector dock. 
    inspectors: HashMap<TypeId, InspectFn>,
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
type UpdateFn = Box<dyn Fn(&mut hecs::World, &mut PhysicsState, f32, Arc<SharedGraphicsContext>) + Send + Sync>;
type DefaultFn = Box<dyn Fn() -> Box<dyn SerializedComponent> + Send + Sync>;
type RemoveFn = Box<dyn Fn(&mut hecs::World, hecs::Entity) + Send + Sync>;
type FindFn = Box<dyn Fn(&hecs::World) -> Vec<hecs::Entity> + Send + Sync>;
type InspectFn = Box<dyn Fn(&mut hecs::World, hecs::Entity, &mut egui::Ui, Arc<SharedGraphicsContext>) + Send + Sync>;

// fn inspect(&mut self, ui: &mut egui::Ui);

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
            inspectors: HashMap::new(),
        }
    }

    /// Register a component type with the registry
    pub fn register<T>(&mut self)
    where
        T: Component + InspectableComponent + Send + Sync + 'static,
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

        self.updaters.insert(type_id, Box::new(|world, physics, dt, graphics| {
            let world_ptr = world as *mut hecs::World; // safe assuming world is kept at the DropbearAppBuilder application level (lifetime)
            let mut query = world.query::<(hecs::Entity, &mut T)>();
            for (entity, component) in query.iter() {
                let world_ref = unsafe { &*world_ptr };
                component.update_component(world_ref, physics, entity, dt, graphics.clone());
            }
        }));

        self.inspectors.insert(type_id, Box::new(|world, entity, ui, graphics| {
            if let Ok(mut comp) = world.get::<&mut T>(entity) {
                comp.inspect(ui, graphics);
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
        physics: &mut PhysicsState,
        dt: f32,
        graphics: Arc<SharedGraphicsContext>,
    ) {
        for updater in self.updaters.values() {
            updater(world, physics, dt, graphics.clone());
        }
    }

    /// Inspects all registered components attached to an entity.
    pub fn inspect_components(
        &self,
        world: &mut hecs::World,
        entity: hecs::Entity,
        ui: &mut egui::Ui,
        graphics: Arc<SharedGraphicsContext>,
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

            if let Some(inspector) = self.inspectors.get(&type_id) {
                inspector(world, entity, ui, graphics.clone());
            } else {
                ui.label(format!("{} (no inspector)", desc.type_name));
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
pub trait Component: Sync + Send {
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

    /// Converts [`Self::SerializedForm`] into a [`Component`] instance that can be added to
    /// `hecs::EntityBuilder` during scene initialisation.
    fn init<'a>(
        ser: &'a Self::SerializedForm,
        graphics: Arc<SharedGraphicsContext>,
    ) -> ComponentInitFuture<'a, Self>;

    /// Called every frame to update the component's state.
    fn update_component(&mut self, world: &hecs::World, physics: &mut PhysicsState, entity: hecs::Entity, dt: f32, graphics: Arc<SharedGraphicsContext>);

    /// Called when saving the scene to disk. Returns the [`Self::SerializedForm`] of the component that can be
    /// saved to disk.
    fn save(&self, world: &hecs::World, entity: hecs::Entity) -> Box<dyn SerializedComponent>;
}

pub trait InspectableComponent: Send + Sync {
    /// In the editor, how the component will be represented in the `Resource Viewer` dock.
    fn inspect(&mut self, ui: &mut egui::Ui, graphics: Arc<SharedGraphicsContext>);
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
                ResourceReferenceType::File(reference) => {
                    log::debug!("Loading model from file: {:?}", ser.handle);
                    let path = ser.handle.clone().resolve()?;
                    let buffer = std::fs::read(&path)?;
                    Model::load_from_memory_raw(
                        graphics.clone(),
                        buffer,
                        Some(ser.handle.clone()),
                        Some(reference),
                        ASSET_REGISTRY.clone(),
                    )
                    .await?
                }
                ResourceReferenceType::Bytes(bytes) => {
                    log::debug!("Loading model from bytes [Len: {}]", bytes.len());
                    Model::load_from_memory_raw(
                        graphics.clone(),
                        bytes,
                        Some(ser.handle.clone()),
                        None,
                        ASSET_REGISTRY.clone(),
                    )
                    .await?
                }
                ResourceReferenceType::ProcObj(obj) => {
                    obj.build_model(graphics.clone(), None, None, ASSET_REGISTRY.clone())
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

    fn update_component(&mut self, world: &World, _physics: &mut PhysicsState, entity: Entity, _dt: f32, _graphics: Arc<SharedGraphicsContext>) {
        if let Ok(transform) = world.query_one::<&EntityTransform>(entity).get() {
            self.update(&transform.propagate(&world, entity));
        }
    }

    fn save(&self, _world: &World, _entity: Entity) -> Box<dyn SerializedComponent> {
        let asset = ASSET_REGISTRY.read();
        let model = asset.get_model(self.model());
        let (label, handle) = if let Some(model) = model {
            (model.label.clone(), model.path.clone())
        } else {
            if !self.model().is_null() {
                log::warn!("MeshRenderer save: missing model handle {} in registry", self.model().id);
            }
            ("None".to_string(), ResourceReference::default())
        };

        let mut texture_override: HashMap<String, SerializedMaterialCustomisation> = HashMap::new();
        for (label, mat) in &self.material_snapshot {
            let diffuse_texture = mat.diffuse_texture.reference.clone();
            let normal_texture = mat.normal_texture.reference.clone();
            let emissive_texture = mat.emissive_texture.as_ref().and_then(|t| t.reference.clone());
            let occlusion_texture = mat.occlusion_texture.as_ref().and_then(|t| t.reference.clone());
            let metallic_roughness_texture = mat.metallic_roughness_texture.as_ref().and_then(|t| t.reference.clone());

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
            label,
            handle,
            import_scale: Some(self.import_scale()),
            texture_override,
        })
    }
}

impl InspectableComponent for MeshRenderer {
    fn inspect(&mut self, ui: &mut egui::Ui, graphics: Arc<SharedGraphicsContext>) {
        fn is_probably_model_uri(uri: &str) -> bool {
            let uri = uri.to_ascii_lowercase();
            uri.ends_with(".glb")
                || uri.ends_with(".gltf")
                || uri.ends_with(".obj")
                || uri.ends_with(".fbx")
        }

        fn is_probably_texture_uri(uri: &str) -> bool {
            let uri = uri.to_ascii_lowercase();
            uri.ends_with(".png")
                || uri.ends_with(".jpg")
                || uri.ends_with(".jpeg")
                || uri.ends_with(".tga")
                || uri.ends_with(".bmp")
                || uri.ends_with(".webp")
        }

        fn proc_obj_size(obj: &ProcedurallyGeneratedObject) -> Option<[f32; 3]> {
            if obj.ty != ProcObjType::Cuboid {
                return None;
            }

            let mut min = [f32::INFINITY; 3];
            let mut max = [f32::NEG_INFINITY; 3];
            for v in &obj.vertices {
                let pos = v.position;
                for i in 0..3 {
                    min[i] = min[i].min(pos[i]);
                    max[i] = max[i].max(pos[i]);
                }
            }

            if min.iter().any(|v| !v.is_finite()) || max.iter().any(|v| !v.is_finite()) {
                return None;
            }

            Some([max[0] - min[0], max[1] - min[1], max[2] - min[2]])
        }

        let apply_cuboid = |renderer: &mut MeshRenderer, size: [f32; 3], force_new: bool| {
            let size_vec = glam::DVec3::new(size[0] as f64, size[1] as f64, size[2] as f64);
            let current_model = renderer.model();

            let proc_obj = ProcedurallyGeneratedObject::cuboid(size_vec);
            if force_new {
                let mut model = proc_obj.construct(
                    graphics.clone(),
                    None,
                    None,
                    None,
                    ASSET_REGISTRY.clone(),
                );
                let mut hasher = DefaultHasher::new();
                model.hash.hash(&mut hasher);
                if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) {
                    duration.as_nanos().hash(&mut hasher);
                }
                let new_hash = hasher.finish();
                model.hash = new_hash;
                model.label = format!("Cuboid {:016x}", new_hash);

                let handle = {
                    let mut asset = ASSET_REGISTRY.write();
                    asset.add_model(model)
                };
                renderer.set_model(handle);
            } else if current_model.is_null() {
                let handle = proc_obj.build_model(
                    graphics.clone(),
                    None,
                    None,
                    ASSET_REGISTRY.clone(),
                );
                renderer.set_model(handle);
            } else {
                let existing_label = {
                    let asset = ASSET_REGISTRY.read();
                    asset.get_model(current_model).map(|model| model.label.clone())
                };

                if let Some(existing_label) = existing_label {
                    let mut model = proc_obj.construct(
                        graphics.clone(),
                        None,
                        None,
                        None,
                        ASSET_REGISTRY.clone(),
                    );
                    model.hash = current_model.id;
                    model.label = existing_label;

                    let mut asset = ASSET_REGISTRY.write();
                    asset.update_model(current_model, model);
                } else {
                    let handle = proc_obj.build_model(
                        graphics.clone(),
                        None,
                        None,
                        ASSET_REGISTRY.clone(),
                    );
                    renderer.set_model(handle);
                }
            }
            
            renderer.reset_texture_override();
        };

        CollapsingHeader::new("Mesh Renderer").show(ui, |ui| {
            let registry = ASSET_REGISTRY.read();
            let current_model = registry.get_model(self.model());
            let model_list = registry.list_models();

            let model_reference = current_model
                .as_ref()
                .map(|model| model.path.clone())
                .unwrap_or_default();
            let model_label = current_model
                .as_ref()
                .map(|model| model.label.clone())
                .unwrap_or_else(|| "None".to_string());

            let model_title = match &model_reference.ref_type {
                ResourceReferenceType::None => "None".to_string(),
                ResourceReferenceType::ProcObj(obj) => match obj.ty {
                    ProcObjType::Cuboid => "Cuboid".to_string(),
                },
                _ => model_label,
            };

            ui.vertical(|ui| {
                let expand_id = ui.make_persistent_id("mesh_renderer_expand");
                let mut expanded = ui
                    .ctx()
                    .data_mut(|d| d.get_temp::<bool>(expand_id).unwrap_or(false));

                let mut selected_model: Option<Handle<Model>> = None;
                let mut choose_proc_cuboid = false;
                let mut choose_none = false;

                let drag_id = egui::Id::new(DRAGGED_ASSET_ID);
                let dragging_valid_model = ui.ctx().data_mut(|d| {
                    d.get_temp::<Option<ResourceReference>>(drag_id)
                        .unwrap_or(None)
                        .and_then(|r| r.as_uri().map(|u| is_probably_model_uri(u)))
                        .unwrap_or(false)
                });

                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 72.0),
                    egui::Sense::click(),
                );

                let fill = if dragging_valid_model && response.hovered() {
                    ui.visuals().selection.bg_fill
                } else if response.hovered() {
                    ui.visuals().widgets.hovered.bg_fill
                } else if dragging_valid_model {
                    ui.visuals().widgets.active.bg_fill
                } else {
                    ui.visuals().widgets.inactive.bg_fill
                };

                ui.painter().rect_filled(rect, 4.0, fill);
                ui.painter().rect_stroke(
                    rect,
                    4.0,
                    ui.visuals().widgets.inactive.bg_stroke,
                    egui::StrokeKind::Inside,
                );

                let mut card_ui = ui.new_child(
                    UiBuilder::new()
                        .layout(egui::Layout::top_down(egui::Align::Min))
                        .max_rect(rect),
                );

                card_ui.horizontal(|ui| {
                    let arrow = if expanded { "v" } else { ">" };
                    if ui.button(arrow).clicked() {
                        expanded = !expanded;
                    }

                    ui.vertical(|ui| {
                        ui.label(RichText::new(&model_title).strong());
                        ui.label(
                            RichText::new("Drop a model from the Asset Viewer")
                                .small()
                                .color(ui.visuals().weak_text_color()),
                        );
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ComboBox::from_id_salt("mesh_renderer_model_picker")
                            .selected_text(&model_title)
                            .show_ui(ui, |ui| {
                                if ui
                                    .selectable_label(
                                        matches!(model_reference.ref_type, ResourceReferenceType::None),
                                        "None",
                                    )
                                    .clicked()
                                {
                                    choose_none = true;
                                }

                                ui.separator();

                                if ui
                                    .selectable_label(
                                        matches!(
                                            model_reference.ref_type,
                                            ResourceReferenceType::ProcObj(_)
                                        ),
                                        "Cuboid",
                                    )
                                    .clicked()
                                {
                                    choose_proc_cuboid = true;
                                }

                                ui.separator();

                                for (handle, label, _) in &model_list {
                                    if handle.is_null() {
                                        continue;
                                    }
                                    if label.eq_ignore_ascii_case("light cube") {
                                        continue;
                                    }
                                    let is_selected = self.model() == *handle;
                                    if ui.selectable_label(is_selected, label).clicked() {
                                        selected_model = Some(*handle);
                                    }
                                }
                            });
                    });
                });

                let pointer_released = ui.input(|i| i.pointer.any_released());
                if pointer_released && response.hovered() {
                    let drag_id = egui::Id::new(DRAGGED_ASSET_ID);
                    let dragged_reference = ui
                        .ctx()
                        .data_mut(|d| d.get_temp::<Option<ResourceReference>>(drag_id).unwrap_or(None));
                    if let Some(reference) = dragged_reference {
                        if let Some(uri) = reference.as_uri() {
                            if is_probably_model_uri(uri) {
                                if let Some(handle) =
                                    ASSET_REGISTRY.read().get_model_handle_by_reference(&reference)
                                {
                                    if let Some(model) = ASSET_REGISTRY.read().get_model(handle) {
                                        if model.label.eq_ignore_ascii_case("light cube") {
                                            ui.ctx().data_mut(|d| {
                                                d.insert_temp(drag_id, None::<ResourceReference>)
                                            });
                                            return;
                                        }
                                    }
                                    self.set_model(handle);
                                    self.reset_texture_override();
                                }
                                ui.ctx().data_mut(|d| {
                                    d.insert_temp(drag_id, None::<ResourceReference>)
                                });
                            }
                        }
                    }
                }

                ui.ctx().data_mut(|d| d.insert_temp(expand_id, expanded));

                if choose_proc_cuboid {
                    let default_size = match &model_reference.ref_type {
                        ResourceReferenceType::ProcObj(obj) => {
                            proc_obj_size(obj).unwrap_or([1.0, 1.0, 1.0])
                        }
                        _ => [1.0, 1.0, 1.0],
                    };
                    apply_cuboid(self, default_size, true);
                } else if choose_none {
                    self.set_model(Handle::NULL);
                    self.reset_texture_override();
                } else if let Some(handle) = selected_model {
                    self.set_model(handle);
                    self.reset_texture_override();
                }

                if expanded {
                    ui.add_space(6.0);

                    if let ResourceReferenceType::File(reference) = &model_reference.ref_type {
                        if is_probably_model_uri(reference) {
                            let mut import_scale = self.import_scale();
                            ui.horizontal(|ui| {
                                ui.label("Import Scale");
                                let resp = ui.add(
                                    DragValue::new(&mut import_scale)
                                        .speed(0.01)
                                        .range(0.0001..=10_000.0),
                                );

                                if resp.changed() {
                                    self.set_import_scale(import_scale);
                                }

                                if ui.button("Reset").clicked() {
                                    self.set_import_scale(1.0);
                                }
                            });
                            ui.add_space(6.0);
                        }
                    }

                    if let ResourceReferenceType::ProcObj(obj) = &model_reference.ref_type {
                        if let Some(mut size) = proc_obj_size(obj) {
                            ui.label(RichText::new("Cuboid").strong());
                            ui.horizontal(|ui| {
                            ui.label("Extents:");
                            let mut changed = false;
                            ui.label("X");
                            changed |= ui
                                .add(DragValue::new(&mut size[0]).speed(0.05).range(0.01..=10_000.0))
                                .changed();
                            ui.label("Y");
                            changed |= ui
                                .add(DragValue::new(&mut size[1]).speed(0.05).range(0.01..=10_000.0))
                                .changed();
                            ui.label("Z");
                            changed |= ui
                                .add(DragValue::new(&mut size[2]).speed(0.05).range(0.01..=10_000.0))
                                .changed();

                            if changed {
                                // Preserve material customizations across cuboid size change
                                let saved_materials = self.material_snapshot.clone();
                                apply_cuboid(self, size, false);
                                // Re-apply material customizations to the new model
                                for (name, saved_mat) in saved_materials {
                                    if let Some(mat) = self.material_snapshot.get_mut(&name) {
                                        mat.tint = saved_mat.tint;
                                        mat.emissive_factor = saved_mat.emissive_factor;
                                        mat.metallic_factor = saved_mat.metallic_factor;
                                        mat.roughness_factor = saved_mat.roughness_factor;
                                        mat.alpha_mode = saved_mat.alpha_mode;
                                        mat.alpha_cutoff = saved_mat.alpha_cutoff;
                                        mat.double_sided = saved_mat.double_sided;
                                        mat.occlusion_strength = saved_mat.occlusion_strength;
                                        mat.normal_scale = saved_mat.normal_scale;
                                        mat.uv_tiling = saved_mat.uv_tiling;
                                        mat.wrap_mode = saved_mat.wrap_mode;
                                        mat.texture_tag = saved_mat.texture_tag.clone();
                                    }
                                }
                            }
                        });

                        ui.separator();
                        }
                    }

                    ui.label("Materials");

                    for (material_name, material) in self.material_snapshot.iter_mut() {
                        ui.separator();
                        ui.label(material_name.as_str());

                        let mut tint_rgb = [material.tint[0], material.tint[1], material.tint[2]];
                        if egui::color_picker::color_edit_button_rgb(ui, &mut tint_rgb).changed() {
                            material.tint[0] = tint_rgb[0];
                            material.tint[1] = tint_rgb[1];
                            material.tint[2] = tint_rgb[2];
                        }

                        let mut emissive_rgb = [
                            material.emissive_factor[0],
                            material.emissive_factor[1],
                            material.emissive_factor[2],
                        ];
                        if egui::color_picker::color_edit_button_rgb(ui, &mut emissive_rgb).changed() {
                            material.emissive_factor[0] = emissive_rgb[0];
                            material.emissive_factor[1] = emissive_rgb[1];
                            material.emissive_factor[2] = emissive_rgb[2];
                        }

                        ui.horizontal(|ui| {
                            ui.label("Metallic");
                            ui.add(DragValue::new(&mut material.metallic_factor)
                                .speed(0.01)
                                .range(0.0..=1.0));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Roughness");
                            ui.add(DragValue::new(&mut material.roughness_factor)
                                .speed(0.01)
                                .range(0.0..=1.0));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Occlusion Strength");
                            ui.add(DragValue::new(&mut material.occlusion_strength)
                                .speed(0.01)
                                .range(0.0..=1.0));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Normal Scale");
                            ui.add(DragValue::new(&mut material.normal_scale)
                                .speed(0.01)
                                .range(0.0..=10.0));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Alpha Mode");
                            egui::ComboBox::from_id_salt(format!("alpha_mode_{}", material_name))
                                .selected_text(match material.alpha_mode {
                                    dropbear_engine::model::AlphaMode::Opaque => "Opaque",
                                    dropbear_engine::model::AlphaMode::Mask => "Mask",
                                    dropbear_engine::model::AlphaMode::Blend => "Blend",
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut material.alpha_mode,
                                        dropbear_engine::model::AlphaMode::Opaque,
                                        "Opaque",
                                    );
                                    ui.selectable_value(
                                        &mut material.alpha_mode,
                                        dropbear_engine::model::AlphaMode::Mask,
                                        "Mask",
                                    );
                                    ui.selectable_value(
                                        &mut material.alpha_mode,
                                        dropbear_engine::model::AlphaMode::Blend,
                                        "Blend",
                                    );
                                });
                        });

                        let mut cutoff = material.alpha_cutoff.unwrap_or(0.5);
                        ui.horizontal(|ui| {
                            ui.label("Alpha Cutoff");
                            if ui.add(DragValue::new(&mut cutoff)
                                .speed(0.01)
                                .range(0.0..=1.0))
                                .changed()
                            {
                                material.alpha_cutoff = Some(cutoff);
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Double Sided");
                            ui.checkbox(&mut material.double_sided, "");
                        });

                        ui.horizontal(|ui| {
                            ui.label("Wrap");
                            egui::ComboBox::from_id_salt(format!("wrap_mode_{}", material_name))
                                .selected_text(match material.wrap_mode {
                                    dropbear_engine::texture::TextureWrapMode::Repeat => "Repeat",
                                    dropbear_engine::texture::TextureWrapMode::Clamp => "Clamp",
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut material.wrap_mode,
                                        dropbear_engine::texture::TextureWrapMode::Repeat,
                                        "Repeat",
                                    );
                                    ui.selectable_value(
                                        &mut material.wrap_mode,
                                        dropbear_engine::texture::TextureWrapMode::Clamp,
                                        "Clamp",
                                    );
                                });
                        });

                        ui.horizontal(|ui| {
                            ui.label("UV Tiling");
                            ui.add(
                                DragValue::new(&mut material.uv_tiling[0])
                                    .speed(0.05)
                                    .range(0.01..=10_000.0),
                            );
                            ui.label("x");
                            ui.add(
                                DragValue::new(&mut material.uv_tiling[1])
                                    .speed(0.05)
                                    .range(0.01..=10_000.0),
                            );
                        });

                        let mut texture_tag = material.texture_tag.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut texture_tag).changed() {
                            material.texture_tag = if texture_tag.trim().is_empty() {
                                None
                            } else {
                                Some(texture_tag)
                            };
                        }

                        // Texture customization slots
                        ui.add_space(8.0);
                        ui.label(RichText::new("Textures").strong());

                        // Check if a valid texture is being dragged
                        let drag_id = egui::Id::new(DRAGGED_ASSET_ID);
                        let dragging_valid_texture = ui.ctx().data_mut(|d| {
                            d.get_temp::<Option<ResourceReference>>(drag_id)
                                .unwrap_or(None)
                                .and_then(|r| r.as_uri().map(|u| is_probably_texture_uri(u)))
                                .unwrap_or(false)
                        });

                        // Helper to create texture slot UI
                        let texture_slot = |ui: &mut egui::Ui,
                                           label: &str,
                                           _id_salt: &str,
                                           current_texture: &Texture,
                                           _graphics: Arc<SharedGraphicsContext>,
                                           dragging_valid: bool| -> Option<Texture> {
                            let mut result = None;
                            ui.horizontal(|ui| {
                                ui.label(label);
                                
                                let texture_label = current_texture.label.clone()
                                    .unwrap_or_else(|| "Default".to_string());
                                
                                let (rect, response) = ui.allocate_exact_size(
                                    egui::vec2(ui.available_width().min(150.0), 24.0),
                                    egui::Sense::click(),
                                );

                                let fill = if dragging_valid && response.hovered() {
                                    ui.visuals().selection.bg_fill
                                } else if response.hovered() {
                                    ui.visuals().widgets.hovered.bg_fill
                                } else if dragging_valid {
                                    ui.visuals().widgets.active.bg_fill
                                } else {
                                    ui.visuals().widgets.inactive.bg_fill
                                };
                                ui.painter().rect_filled(rect, 2.0, fill);
                                ui.painter().rect_stroke(
                                    rect, 2.0,
                                    ui.visuals().widgets.inactive.bg_stroke,
                                    egui::StrokeKind::Inside,
                                );
                                ui.painter().text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    &texture_label,
                                    egui::FontId::default(),
                                    ui.visuals().text_color(),
                                );

                                // Handle texture drag-drop
                                let pointer_released = ui.input(|i| i.pointer.any_released());
                                if pointer_released && response.hovered() {
                                    let drag_id = egui::Id::new(DRAGGED_ASSET_ID);
                                    let dragged_reference = ui
                                        .ctx()
                                        .data_mut(|d| d.get_temp::<Option<ResourceReference>>(drag_id).unwrap_or(None));
                                    if let Some(reference) = dragged_reference {
                                        if let Some(uri) = reference.as_uri() {
                                            if is_probably_texture_uri(uri) {
                                                // Try to find the texture in the registry
                                                if let Some(handle) = ASSET_REGISTRY.read()
                                                    .get_texture_handle_by_reference(&reference)
                                                {
                                                    if let Some(tex) = ASSET_REGISTRY.read()
                                                        .get_texture(handle)
                                                        .cloned()
                                                    {
                                                        result = Some(tex);
                                                    }
                                                }
                                                ui.ctx().data_mut(|d| {
                                                    d.insert_temp(drag_id, None::<ResourceReference>)
                                                });
                                            }
                                        }
                                    }
                                }

                                if response.hovered() {
                                    response.on_hover_text("Drop a texture here");
                                }
                            });
                            result
                        };

                        // Diffuse texture slot
                        if let Some(new_tex) = texture_slot(
                            ui, "Diffuse", 
                            &format!("diffuse_tex_{}", material_name),
                            &material.diffuse_texture,
                            graphics.clone(),
                            dragging_valid_texture,
                        ) {
                            material.diffuse_texture = new_tex;
                        }

                        // Normal texture slot
                        if let Some(new_tex) = texture_slot(
                            ui, "Normal",
                            &format!("normal_tex_{}", material_name),
                            &material.normal_texture,
                            graphics.clone(),
                            dragging_valid_texture,
                        ) {
                            material.normal_texture = new_tex;
                        }

                        // Emissive texture slot (optional)
                        ui.horizontal(|ui| {
                            ui.label("Emissive");
                            
                            let texture_label = material.emissive_texture
                                .as_ref()
                                .and_then(|t| t.label.clone())
                                .unwrap_or_else(|| "None".to_string());
                            
                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width().min(150.0), 24.0),
                                egui::Sense::click(),
                            );

                            let fill = if dragging_valid_texture && response.hovered() {
                                ui.visuals().selection.bg_fill
                            } else if response.hovered() {
                                ui.visuals().widgets.hovered.bg_fill
                            } else if dragging_valid_texture {
                                ui.visuals().widgets.active.bg_fill
                            } else {
                                ui.visuals().widgets.inactive.bg_fill
                            };
                            ui.painter().rect_filled(rect, 2.0, fill);
                            ui.painter().rect_stroke(
                                rect, 2.0,
                                ui.visuals().widgets.inactive.bg_stroke,
                                egui::StrokeKind::Inside,
                            );
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                &texture_label,
                                egui::FontId::default(),
                                ui.visuals().text_color(),
                            );

                            let pointer_released = ui.input(|i| i.pointer.any_released());
                            if pointer_released && response.hovered() {
                                let drag_id = egui::Id::new(DRAGGED_ASSET_ID);
                                let dragged_reference = ui
                                    .ctx()
                                    .data_mut(|d| d.get_temp::<Option<ResourceReference>>(drag_id).unwrap_or(None));
                                if let Some(reference) = dragged_reference {
                                    if let Some(uri) = reference.as_uri() {
                                        if is_probably_texture_uri(uri) {
                                            if let Some(handle) = ASSET_REGISTRY.read()
                                                .get_texture_handle_by_reference(&reference)
                                            {
                                                if let Some(tex) = ASSET_REGISTRY.read()
                                                    .get_texture(handle)
                                                    .cloned()
                                                {
                                                    material.emissive_texture = Some(tex);
                                                }
                                            }
                                            ui.ctx().data_mut(|d| {
                                                d.insert_temp(drag_id, None::<ResourceReference>)
                                            });
                                        }
                                    }
                                }
                            }

                            if response.hovered() {
                                response.on_hover_text("Drop a texture here");
                            }

                            // Clear button for optional texture
                            if material.emissive_texture.is_some() && ui.small_button("X").clicked() {
                                material.emissive_texture = None;
                            }
                        });

                        // Metallic/Roughness texture slot (optional)
                        ui.horizontal(|ui| {
                            ui.label("Metal/Rough");
                            
                            let texture_label = material.metallic_roughness_texture
                                .as_ref()
                                .and_then(|t| t.label.clone())
                                .unwrap_or_else(|| "None".to_string());
                            
                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width().min(150.0), 24.0),
                                egui::Sense::click(),
                            );

                            let fill = if dragging_valid_texture && response.hovered() {
                                ui.visuals().selection.bg_fill
                            } else if response.hovered() {
                                ui.visuals().widgets.hovered.bg_fill
                            } else if dragging_valid_texture {
                                ui.visuals().widgets.active.bg_fill
                            } else {
                                ui.visuals().widgets.inactive.bg_fill
                            };
                            ui.painter().rect_filled(rect, 2.0, fill);
                            ui.painter().rect_stroke(
                                rect, 2.0,
                                ui.visuals().widgets.inactive.bg_stroke,
                                egui::StrokeKind::Inside,
                            );
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                &texture_label,
                                egui::FontId::default(),
                                ui.visuals().text_color(),
                            );

                            let pointer_released = ui.input(|i| i.pointer.any_released());
                            if pointer_released && response.hovered() {
                                let drag_id = egui::Id::new(DRAGGED_ASSET_ID);
                                let dragged_reference = ui
                                    .ctx()
                                    .data_mut(|d| d.get_temp::<Option<ResourceReference>>(drag_id).unwrap_or(None));
                                if let Some(reference) = dragged_reference {
                                    if let Some(uri) = reference.as_uri() {
                                        if is_probably_texture_uri(uri) {
                                            if let Some(handle) = ASSET_REGISTRY.read()
                                                .get_texture_handle_by_reference(&reference)
                                            {
                                                if let Some(tex) = ASSET_REGISTRY.read()
                                                    .get_texture(handle)
                                                    .cloned()
                                                {
                                                    material.metallic_roughness_texture = Some(tex);
                                                }
                                            }
                                            ui.ctx().data_mut(|d| {
                                                d.insert_temp(drag_id, None::<ResourceReference>)
                                            });
                                        }
                                    }
                                }
                            }

                            if response.hovered() {
                                response.on_hover_text("Drop a texture here");
                            }

                            if material.metallic_roughness_texture.is_some() && ui.small_button("X").clicked() {
                                material.metallic_roughness_texture = None;
                            }
                        });

                        // Occlusion texture slot (optional)
                        ui.horizontal(|ui| {
                            ui.label("Occlusion");
                            
                            let texture_label = material.occlusion_texture
                                .as_ref()
                                .and_then(|t| t.label.clone())
                                .unwrap_or_else(|| "None".to_string());
                            
                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width().min(150.0), 24.0),
                                egui::Sense::click(),
                            );

                            let fill = if dragging_valid_texture && response.hovered() {
                                ui.visuals().selection.bg_fill
                            } else if response.hovered() {
                                ui.visuals().widgets.hovered.bg_fill
                            } else if dragging_valid_texture {
                                ui.visuals().widgets.active.bg_fill
                            } else {
                                ui.visuals().widgets.inactive.bg_fill
                            };
                            ui.painter().rect_filled(rect, 2.0, fill);
                            ui.painter().rect_stroke(
                                rect, 2.0,
                                ui.visuals().widgets.inactive.bg_stroke,
                                egui::StrokeKind::Inside,
                            );
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                &texture_label,
                                egui::FontId::default(),
                                ui.visuals().text_color(),
                            );

                            let pointer_released = ui.input(|i| i.pointer.any_released());
                            if pointer_released && response.hovered() {
                                let drag_id = egui::Id::new(DRAGGED_ASSET_ID);
                                let dragged_reference = ui
                                    .ctx()
                                    .data_mut(|d| d.get_temp::<Option<ResourceReference>>(drag_id).unwrap_or(None));
                                if let Some(reference) = dragged_reference {
                                    if let Some(uri) = reference.as_uri() {
                                        if is_probably_texture_uri(uri) {
                                            if let Some(handle) = ASSET_REGISTRY.read()
                                                .get_texture_handle_by_reference(&reference)
                                            {
                                                if let Some(tex) = ASSET_REGISTRY.read()
                                                    .get_texture(handle)
                                                    .cloned()
                                                {
                                                    material.occlusion_texture = Some(tex);
                                                }
                                            }
                                            ui.ctx().data_mut(|d| {
                                                d.insert_temp(drag_id, None::<ResourceReference>)
                                            });
                                        }
                                    }
                                }
                            }

                            if response.hovered() {
                                response.on_hover_text("Drop a texture here");
                            }

                            if material.occlusion_texture.is_some() && ui.small_button("X").clicked() {
                                material.occlusion_texture = None;
                            }
                        });
                    }
                }
            });
        });
    }
}

