use crate::hierarchy::EntityTransformExt;
use crate::physics::PhysicsState;
use crate::ser::model::EucalyptusModel;
use crate::states::{SerializedMaterialCustomisation, SerializedMeshRenderer};
use crate::utils::{ResolveReference};
use downcast_rs::{Downcast, impl_downcast};
use dropbear_engine::asset::{ASSET_REGISTRY, Handle};
use dropbear_engine::entity::{EntityTransform, MeshRenderer, Transform};
use dropbear_engine::graphics::SharedGraphicsContext;
use dropbear_engine::model::Model;
use dropbear_engine::procedural::{ProcObjType, ProcedurallyGeneratedObject};
use dropbear_engine::texture::{Texture, TextureBuilder, TextureReference};
use dropbear_engine::utils::ResourceReference;
use egui::{CollapsingHeader, ComboBox, DragValue, Grid, RichText, UiBuilder};
use hecs::{Entity, World};
pub use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::future::Future;
use std::hash::{Hash, Hasher};
use std::pin::Pin;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

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
pub type ComponentInitFuture<'a, T> = std::pin::Pin<
    Box<
        dyn std::future::Future<Output = anyhow::Result<<T as Component>::RequiredComponentTypes>>
            + Send
            + Sync
            + 'a,
    >,
>;

type LoaderFuture<'a> = Pin<
    Box<
        dyn Future<
                Output = anyhow::Result<
                    Box<dyn for<'b> FnOnce(&'b mut hecs::EntityBuilder) + Send + Sync>,
                >,
            > + Send
            + Sync
            + 'a,
    >,
>;
type LoaderFn = Box<
    dyn for<'a> Fn(&'a dyn SerializedComponent, Arc<SharedGraphicsContext>) -> LoaderFuture<'a>
        + Send
        + Sync,
>;
type ExtractorFn =
    Box<dyn Fn(&hecs::World, hecs::Entity) -> Option<Box<dyn SerializedComponent>> + Send + Sync>;
type UpdateFn =
    Box<dyn Fn(&mut hecs::World, &mut PhysicsState, f32, Arc<SharedGraphicsContext>) + Send + Sync>;
type DefaultFn = Box<dyn Fn() -> Box<dyn SerializedComponent> + Send + Sync>;
type RemoveFn = Box<dyn Fn(&mut hecs::World, hecs::Entity) + Send + Sync>;
type FindFn = Box<dyn Fn(&hecs::World) -> Vec<hecs::Entity> + Send + Sync>;
type InspectFn = Box<
    dyn Fn(&mut hecs::World, hecs::Entity, &mut egui::Ui, Arc<SharedGraphicsContext>) + Send + Sync,
>;

// fn inspect(&mut self, world: &hecs::World, entity: hecs::Entity, ui: &mut egui::Ui, graphics: Arc<SharedGraphicsContext>);

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
            self.categories
                .entry(cat.clone())
                .or_default()
                .push(type_id);
        }
        self.descriptors.insert(type_id, desc);
        self.serialized_to_component
            .insert(serialized_type_id, type_id);

        self.extractors.insert(
            type_id,
            Box::new(|world, entity| {
                let Ok(c) = world.get::<&T>(entity) else {
                    return None;
                };
                Some(c.save(world, entity))
            }),
        );

        self.loaders.insert(
            serialized_type_id,
            Box::new(|serialized, graphics| {
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
            }),
        );

        self.defaults
            .insert(type_id, Box::new(|| Box::new(T::SerializedForm::default())));

        self.removers.insert(
            type_id,
            Box::new(|world, entity| {
                let _ = world.remove_one::<T>(entity);
            }),
        );

        self.finders.insert(
            type_id,
            Box::new(|world| {
                world
                    .query::<(hecs::Entity, &T)>()
                    .iter()
                    .map(|(entity, _)| entity)
                    .collect()
            }),
        );

        let disabled_flags = T::descriptor().disabled_flags;
        self.updaters.insert(
            type_id,
            Box::new(move |world, physics, dt, graphics| {
                let world_ptr = world as *mut hecs::World; // safe assuming world is kept at the DropbearAppBuilder application level (lifetime)
                let mut query = world.query::<(hecs::Entity, &mut T)>();
                for (entity, component) in query.iter() {
                    let world_ref = unsafe { &*world_ptr };
                    // skip update on DisabledFlags::Hidden
                    if !matches!(disabled_flags, DisabilityFlags::Never) {
                        if let Ok(status) = world_ref.get::<&crate::entity_status::EntityStatus>(entity) {
                            if status.disabled {
                                continue;
                            }
                            if status.hidden && matches!(disabled_flags, DisabilityFlags::Hidden) {
                                continue;
                            }
                        }
                    }
                    component.update_component(world_ref, physics, entity, dt, graphics.clone());
                }
            }),
        );

        self.inspectors.insert(
            type_id,
            Box::new(|world, entity, ui, graphics| {
                let world_ptr = world as *const hecs::World;
                if let Ok(mut comp) = world.get::<&mut T>(entity) {
                    let world_ref = unsafe { &*world_ptr };
                    comp.inspect(world_ref, entity, ui, graphics);
                }
            }),
        );
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
    pub fn remove_component_by_id(&self, world: &mut hecs::World, entity: hecs::Entity, id: u64) {
        if let Some(type_id) = self.type_id_from_numeric_id(id) {
            if let Some(remover) = self.removers.get(&type_id) {
                remover(world, entity);
            }
        }
    }

    /// Finds entities with the component matching a numeric id.
    pub fn find_entities_by_numeric_id(&self, world: &hecs::World, id: u64) -> Vec<hecs::Entity> {
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

            ui.separator();
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

#[derive(Debug, Clone, Default)]
pub enum DisabilityFlags {
    /// Skip this component's `update_component` when the entity is Disabled.
    #[default]
    Disabled,
    /// Skip this component's `update_component` when the entity is Hidden or Disabled.
    Hidden,
    /// Never skip this component. Used for meta-components such as `EntityStatus`.
    Never,
}

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
    /// Governs when this component's logic is skipped based on the entity's [`EntityStatus`].
    pub disabled_flags: DisabilityFlags,
    /// Internal components are not shown in the "Add Component" picker.
    pub internal: bool,
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
    fn init(
        ser: &'_ Self::SerializedForm,
        _graphics: Arc<SharedGraphicsContext>,
    ) -> ComponentInitFuture<'_, Self>;

    /// Called every frame to update the component's state.
    fn update_component(
        &mut self,
        world: &hecs::World,
        physics: &mut PhysicsState,
        entity: hecs::Entity,
        dt: f32,
        graphics: Arc<SharedGraphicsContext>,
    );

    /// Called when saving the scene to disk. Returns the [`Self::SerializedForm`] of the component that can be
    /// saved to disk.
    fn save(&self, _world: &hecs::World, _entity: hecs::Entity) -> Box<dyn SerializedComponent>;
}

pub trait InspectableComponent: Send + Sync {
    /// In the editor, how the component will be represented in the `Resource Viewer` dock.
    fn inspect(
        &mut self,
        world: &hecs::World,
        entity: hecs::Entity,
        ui: &mut egui::Ui,
        graphics: Arc<SharedGraphicsContext>,
    );
}

#[typetag::serde]
impl SerializedComponent for SerializedMeshRenderer {}

// sample for MeshRenderer
impl Component for MeshRenderer {
    type SerializedForm = SerializedMeshRenderer;
    type RequiredComponentTypes = (Self,);

    fn descriptor() -> ComponentDescriptor {
        ComponentDescriptor {
            fqtn: "dropbear_engine::entity::MeshRenderer".to_string(),
            type_name: "MeshRenderer".to_string(),
            category: Some("Rendering".to_string()),
            description: Some("Renders a mesh".to_string()),
            disabled_flags: DisabilityFlags::Disabled,
            internal: false,
        }
    }

    fn init(
        ser: &'_ Self::SerializedForm,
        graphics: Arc<SharedGraphicsContext>,
    ) -> ComponentInitFuture<'_, Self> {
        Box::pin(async move {
            let import_scale = ser.import_scale.unwrap_or(1.0);

            async fn load_model_from_reference(
                model_ref: ResourceReference,
                source_label: String,
                graphics: Arc<SharedGraphicsContext>,
            ) -> anyhow::Result<Handle<Model>> {
                let path = model_ref.resolve()?;
                let extension = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.to_ascii_lowercase());

                match extension.as_deref() {
                    Some("eucmdl") => {
                        let bytes = std::fs::read(&path)?;
                        let model = rkyv::from_bytes::<EucalyptusModel, rkyv::rancor::Error>(&bytes)
                            .map_err(|e| anyhow::anyhow!(
                                "Failed to deserialize .eucmdl '{}' into EucalyptusModel: {e}",
                                path.display()
                            ))?;

                        let runtime_model = model.load(model_ref.clone(), graphics);
                        let mut registry = ASSET_REGISTRY.write();
                        Ok(registry.add_model_with_label(source_label, runtime_model))
                    }
                    _ => {
                        let buffer = std::fs::read(&path)?;
                        Model::load_from_memory_raw(
                            graphics,
                            buffer,
                            Some(model_ref.clone()),
                            Some(source_label.as_str()),
                            ASSET_REGISTRY.clone(),
                        )
                        .await
                    }
                }
            }

            let handle = if let Some(uuid) = ser.uuid {
                let project_root = crate::states::PROJECT.read().project_path.clone();
                match crate::metadata::find_asset_by_uuid(&project_root, uuid) {
                    Ok(entry) => {
                        if let crate::resource::ResourceReference::File(rel) = &entry.location {
                            let abs = project_root.join(rel);
                            match ResourceReference::from_path(&abs) {
                                Ok(engine_ref) => {
                                    log::debug!("Loading model '{}' via UUID {}", entry.name, uuid);
                                    load_model_from_reference(
                                        engine_ref,
                                        entry.name.clone(),
                                        graphics.clone(),
                                    )
                                    .await?
                                }
                                Err(e) => {
                                    log::warn!("Failed to build engine reference for {abs:?}: {e}");
                                    Handle::NULL
                                }
                            }
                        } else {
                            log::warn!("Asset {} location is not a file path", uuid);
                            Handle::NULL
                        }
                    }
                    Err(e) => {
                        log::warn!("UUID {} not found in .eucmeta files: {e}", uuid);
                        Handle::NULL
                    }
                }
            } else if let Some(obj) = &ser.proc_obj {
                log::debug!("Rebuilding procedural mesh from saved geometry");
                obj.build_model(graphics.clone(), None, None, ASSET_REGISTRY.clone())
            } else {
                log::debug!("No model reference, setting to Handle::NULL");
                Handle::NULL
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

                    let get_tex_handle = async |resource: &Option<TextureReference>| -> Option<anyhow::Result<Handle<Texture>>> {
                        match resource {
                            Some(TextureReference::AssetUuid(uuid)) => {
                                let project_root = crate::states::PROJECT.read().project_path.clone();
                                match crate::metadata::find_asset_by_uuid(&project_root, *uuid) {
                                    Ok(entry) => {
                                        if let crate::resource::ResourceReference::File(rel) = &entry.location {
                                            let abs = project_root.join(rel);
                                            let path_str = abs.to_string_lossy().to_string();
                                            // return early if already loaded
                                            {
                                                let engine_ref = ResourceReference::from_path(&abs).ok();
                                                if let Some(ref r) = engine_ref {
                                                    let registry = ASSET_REGISTRY.read();
                                                    if let Some(h) = registry.get_texture_handle_by_reference(r) {
                                                        return Some(Ok(h));
                                                    }
                                                }
                                            }
                                            match std::fs::read(&abs) {
                                                Ok(bytes) => {
                                                    let engine_ref = ResourceReference::from_path(&abs).ok();
                                                    let mut texture = TextureBuilder::new(&graphics.device)
                                                        .with_bytes(graphics.clone(), bytes.as_slice())
                                                        .label(path_str.as_str())
                                                        .build();
                                                    texture.reference = engine_ref;
                                                    let mut registry = ASSET_REGISTRY.write();
                                                    Some(Ok(registry.add_texture_with_label(entry.name.clone(), texture)))
                                                }
                                                Err(e) => Some(Err(anyhow::anyhow!("Failed to read texture for UUID {}: {}", uuid, e))),
                                            }
                                        } else {
                                            Some(Err(anyhow::anyhow!("Texture asset {} has no file-backed location", uuid)))
                                        }
                                    }
                                    Err(e) => Some(Err(anyhow::anyhow!("UUID {} not found for texture: {}", uuid, e))),
                                }
                            }
                            Some(TextureReference::RGBAColour(rgba)) => {
                                let to_u8 = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                                let mut registry = ASSET_REGISTRY.write();
                                let handle = registry.solid_texture_rgba8(
                                    graphics.clone(),
                                    [to_u8(rgba[0]), to_u8(rgba[1]), to_u8(rgba[2]), to_u8(rgba[3])],
                                    Some(Texture::TEXTURE_FORMAT_BASE.add_srgb_suffix()),
                                );
                                Some(Ok(handle))
                            }
                            None => None,
                        }
                    };

                    if let Some(tex) = get_tex_handle(&m.diffuse_texture).await {
                        mat.diffuse_texture = tex?;
                    }

                    if let Some(tex) = get_tex_handle(&m.emissive_texture).await {
                        mat.emissive_texture = Some(tex?);
                    }

                    if let Some(tex) = get_tex_handle(&m.normal_texture).await {
                        mat.normal_texture = Some(tex?);
                    }

                    if let Some(tex) = get_tex_handle(&m.occlusion_texture).await {
                        mat.occlusion_texture = Some(tex?);
                    }

                    if let Some(tex) = get_tex_handle(&m.metallic_roughness_texture).await {
                        mat.metallic_roughness_texture = Some(tex?);
                    }

                    {
                        let mut registry = ASSET_REGISTRY.write();
                        mat.rebuild_bind_group(&mut registry, &graphics);
                    }
                    mat.sync_uniform(&graphics);
                }
            }

            Ok((renderer,))
        })
    }

    fn update_component(
        &mut self,
        world: &World,
        _physics: &mut PhysicsState,
        entity: Entity,
        _dt: f32,
        graphics: Arc<SharedGraphicsContext>,
    ) {
        if let Ok(transform) = world.query_one::<&EntityTransform>(entity).get() {
            self.update(&transform.propagate(&world, entity));
        } else {
            self.update(&Transform::new());
        }

        for (_, v) in self.material_snapshot.iter() {
            v.sync_uniform(&graphics);
        }
    }

    fn save(&self, _world: &World, _entity: Entity) -> Box<dyn SerializedComponent> {
        let save_optional_texture_reference = |tex_reference: Option<ResourceReference>| -> Option<TextureReference> {
            let resource = tex_reference?;
            if let ResourceReference::File(rel) = &resource {
                if !rel.is_empty() {
                    let project_root = crate::states::PROJECT.read().project_path.clone();
                    let abs = project_root.join("resources").join(rel);
                    if let Ok(entry) = crate::metadata::generate_eucmeta(&abs, &project_root) {
                        return Some(TextureReference::AssetUuid(entry.uuid));
                    }
                }
            }
            None
        };

        let asset = ASSET_REGISTRY.read();
        let model = asset.get_model(self.model());
        let (label, uuid, proc_obj) = if let Some(model) = model.as_ref() {
            match &model.path {
                ResourceReference::File(rel) if !rel.is_empty() => {
                    let project_root = crate::states::PROJECT.read().project_path.clone();
                    let abs = project_root.join("resources").join(rel);
                    let uuid = crate::metadata::generate_eucmeta(&abs, &project_root)
                        .ok()
                        .map(|e| e.uuid);
                    (model.label.clone(), uuid, None)
                }
                ResourceReference::Procedural(obj) => {
                    (model.label.clone(), None, Some(obj.clone()))
                }
                _ => (model.label.clone(), None, None),
            }
        } else {
            if !self.model().is_null() {
                log::warn!(
                    "MeshRenderer save: missing model handle {} in registry",
                    self.model().id
                );
            }
            ("None".to_string(), None, None)
        };

        let mut texture_override: HashMap<String, SerializedMaterialCustomisation> = HashMap::new();
        for (label, mat) in &self.material_snapshot {
            let default_material = model
                .as_ref()
                .and_then(|m| m.materials.iter().find(|default| default.name == *label));

            let diffuse_texture = if default_material
                .map(|default| default.diffuse_texture)
                == Some(mat.diffuse_texture)
            {
                None
            } else {
                asset
                    .get_texture(mat.diffuse_texture)
                    .and_then(|t| t.reference.clone())
            };
            let diffuse_texture = save_optional_texture_reference(diffuse_texture);

            let normal_texture = if default_material.map(|default| default.normal_texture)
                == Some(mat.normal_texture)
            {
                None
            } else {
                mat.normal_texture
                    .and_then(|h| asset.get_texture(h).and_then(|t| t.reference.clone()))
            };
            let normal_texture = save_optional_texture_reference(normal_texture);

            let emissive_texture = if default_material.map(|default| default.emissive_texture)
                == Some(mat.emissive_texture)
            {
                None
            } else {
                mat.emissive_texture
                    .and_then(|h| asset.get_texture(h).and_then(|t| t.reference.clone()))
            };
            let emissive_texture = save_optional_texture_reference(emissive_texture);

            let occlusion_texture = if default_material.map(|default| default.occlusion_texture)
                == Some(mat.occlusion_texture)
            {
                None
            } else {
                mat.occlusion_texture
                    .and_then(|h| asset.get_texture(h).and_then(|t| t.reference.clone()))
            };
            let occlusion_texture = save_optional_texture_reference(occlusion_texture);

            let metallic_roughness_texture = if default_material
                .map(|default| default.metallic_roughness_texture)
                == Some(mat.metallic_roughness_texture)
            {
                None
            } else {
                mat.metallic_roughness_texture
                    .and_then(|h| asset.get_texture(h).and_then(|t| t.reference.clone()))
            };
            let metallic_roughness_texture = save_optional_texture_reference(metallic_roughness_texture);

            texture_override.insert(
                label.to_string(),
                SerializedMaterialCustomisation {
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
                },
            );
        }

        Box::new(SerializedMeshRenderer {
            label,
            uuid,
            proc_obj,
            import_scale: Some(self.import_scale()),
            texture_override,
        })
    }
}

impl InspectableComponent for MeshRenderer {
    fn inspect(
        &mut self,
        _world: &hecs::World,
        entity: hecs::Entity,
        ui: &mut egui::Ui,
        graphics: Arc<SharedGraphicsContext>,
    ) {
        fn is_probably_model_uri(uri: &str) -> bool {
            let uri = uri.to_ascii_lowercase();
            uri.ends_with(".glb")
                || uri.ends_with(".gltf")
                || uri.ends_with(".obj")
                || uri.ends_with(".fbx")
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
                let mut model =
                    { proc_obj.construct(graphics.clone(), None, None, None, ASSET_REGISTRY.clone()) };
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
                let handle =
                    proc_obj.build_model(graphics.clone(), None, None, ASSET_REGISTRY.clone());
                renderer.set_model(handle);
            } else {
                let existing_label = {
                    let asset = ASSET_REGISTRY.read();
                    asset
                        .get_model(current_model)
                        .map(|model| model.label.clone())
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

                    {
                        let mut asset = ASSET_REGISTRY.write();
                        asset.update_model(current_model, model);
                    }
                } else {
                    let handle =
                        proc_obj.build_model(graphics.clone(), None, None, ASSET_REGISTRY.clone());
                    renderer.set_model(handle);
                }
            }

            renderer.reset_texture_override();
        };

        CollapsingHeader::new("Mesh Renderer")
            .default_open(true)
            .id_salt(format!("Mesh Renderer {}", entity.to_bits()))
            .show(ui, |ui| {
            let (model_reference, model_title, model_list) = {
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

                let model_title = match &model_reference {
                    ResourceReference::File(s) if s.is_empty() => "None".to_string(),
                    ResourceReference::Procedural(obj) => match obj.ty {
                        ProcObjType::Cuboid => "Cuboid".to_string(),
                    },
                    _ => model_label,
                };

                (model_reference, model_title, model_list)
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
                                        matches!(
                                            model_reference,
                                            ResourceReference::File(ref s) if s.is_empty()
                                        ),
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
                                            model_reference,
                                            ResourceReference::Procedural(_)
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
                    let dragged_reference = ui.ctx().data_mut(|d| {
                        d.get_temp::<Option<ResourceReference>>(drag_id)
                            .unwrap_or(None)
                    });
                    if let Some(reference) = dragged_reference {
                        if let Some(uri) = reference.as_uri() {
                            if is_probably_model_uri(uri) {
                                if let Some(handle) = ASSET_REGISTRY
                                    .read()
                                    .get_model_handle_by_reference(&reference)
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
                    let default_size = match &model_reference {
                        ResourceReference::Procedural(obj) => {
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

                    if let ResourceReference::File(reference) = &model_reference {
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

                    if let ResourceReference::Procedural(obj) = &model_reference {
                        if let Some(mut size) = proc_obj_size(obj) {
                            ui.label(RichText::new("Cuboid").strong());
                            ui.horizontal(|ui| {
                                ui.label("Extents:");
                                let mut changed = false;
                                ui.label("X");
                                changed |= ui
                                    .add(
                                        DragValue::new(&mut size[0])
                                            .speed(0.05)
                                            .range(0.01..=10_000.0),
                                    )
                                    .changed();
                                ui.label("Y");
                                changed |= ui
                                    .add(
                                        DragValue::new(&mut size[1])
                                            .speed(0.05)
                                            .range(0.01..=10_000.0),
                                    )
                                    .changed();
                                ui.label("Z");
                                changed |= ui
                                    .add(
                                        DragValue::new(&mut size[2])
                                            .speed(0.05)
                                            .range(0.01..=10_000.0),
                                    )
                                    .changed();

                                if changed {
                                    let saved_materials = self.material_snapshot.clone();
                                    apply_cuboid(self, size, false);
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

                    ui.add_space(4.0);
                    CollapsingHeader::new("Materials")
                        .default_open(true)
                        .id_salt(format!("Materials {}", entity.to_bits()))
                        .show(ui, |ui| {
                            let mut texture_options = {
                                let registry = ASSET_REGISTRY.read();
                                registry
                                    .list_textures()
                                    .into_iter()
                                    .filter_map(|(handle, label, reference)| {
                                        let is_file_reference = reference
                                            .as_ref()
                                            .is_some_and(|r| matches!(r, ResourceReference::File(s) if !s.is_empty()));
                                        if !is_file_reference {
                                            return None;
                                        }

                                        let display = label
                                            .or_else(|| {
                                                reference.and_then(|r| {
                                                    r.as_uri().map(|uri| {
                                                        uri.rsplit('/')
                                                            .next()
                                                            .filter(|v| !v.is_empty())
                                                            .unwrap_or(uri)
                                                            .to_string()
                                                    })
                                                })
                                            })
                                            .unwrap_or_else(|| format!("Texture {:016x}", handle.id));
                                        Some((handle, display))
                                    })
                                    .collect::<Vec<_>>()
                            };
                            texture_options.reverse();

                            let default_textures = {
                                let mut registry = ASSET_REGISTRY.write();
                                let white_srgb = registry.solid_texture_rgba8(
                                    graphics.clone(),
                                    [255, 255, 255, 255],
                                    Some(Texture::TEXTURE_FORMAT_BASE.add_srgb_suffix()),
                                );
                                let black_srgb = registry.solid_texture_rgba8(
                                    graphics.clone(),
                                    [0, 0, 0, 255],
                                    Some(Texture::TEXTURE_FORMAT_BASE.add_srgb_suffix()),
                                );
                                let white_linear = registry.solid_texture_rgba8(
                                    graphics.clone(),
                                    [255, 255, 255, 255],
                                    Some(Texture::TEXTURE_FORMAT_BASE),
                                );
                                let green_linear = registry.solid_texture_rgba8(
                                    graphics.clone(),
                                    [0, 255, 0, 255],
                                    Some(Texture::TEXTURE_FORMAT_BASE),
                                );
                                let flat_normal = registry.solid_texture_rgba8(
                                    graphics.clone(),
                                    [128, 128, 255, 255],
                                    Some(Texture::TEXTURE_FORMAT_BASE),
                                );

                                (
                                    Some(white_srgb),
                                    Some(flat_normal),
                                    Some(black_srgb),
                                    Some(green_linear),
                                    Some(white_linear),
                                )
                            };

                            let model_handle = self.model();

                            for (material_name, material) in self.material_snapshot.iter_mut() {
                                let default_material = {
                                    let registry = ASSET_REGISTRY.read();
                                    registry
                                        .get_model(model_handle)
                                        .and_then(|model| {
                                            model
                                                .materials
                                                .iter()
                                                .find(|mat| mat.name == *material_name)
                                                .cloned()
                                        })
                                };
                                let material_id = format!("material_{}", material_name);
                                CollapsingHeader::new(material_name.as_str())
                                    .id_salt(format!("{} {}", material_id, entity.to_bits()))
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            if ui.button("Reset Material").clicked() {
                                                if let Some(default) = default_material.as_ref() {
                                                    material.diffuse_texture = default.diffuse_texture;
                                                    material.normal_texture = default.normal_texture;
                                                    material.emissive_texture = default.emissive_texture;
                                                    material.metallic_roughness_texture =
                                                        default.metallic_roughness_texture;
                                                    material.occlusion_texture = default.occlusion_texture;
                                                    material.tint = default.tint;
                                                    material.emissive_factor = default.emissive_factor;
                                                    material.metallic_factor = default.metallic_factor;
                                                    material.roughness_factor = default.roughness_factor;
                                                    material.alpha_mode = default.alpha_mode;
                                                    material.alpha_cutoff = default.alpha_cutoff;
                                                    material.double_sided = default.double_sided;
                                                    material.occlusion_strength = default.occlusion_strength;
                                                    material.normal_scale = default.normal_scale;
                                                    material.uv_tiling = default.uv_tiling;
                                                    material.texture_tag = default.texture_tag.clone();
                                                    material.wrap_mode = default.wrap_mode;
                                                    {
                                                        let mut registry = ASSET_REGISTRY.write();
                                                        material.rebuild_bind_group(&mut registry, &graphics);
                                                    }
                                                    material.sync_uniform(&graphics);
                                                }
                                            }
                                        });

                                        ui.add_space(4.0);
                                        ui.label(
                                            RichText::new("Colors")
                                                .strong()
                                                .color(ui.visuals().text_color()),
                                        );
                                        Grid::new(format!("material_colors_{}", material_name))
                                            .num_columns(2)
                                            .spacing([12.0, 6.0])
                                            .striped(true)
                                            .show(ui, |ui| {
                                                ui.label("Tint");
                                                let mut tint_rgb =
                                                    [material.tint[0], material.tint[1], material.tint[2]];
                                                if egui::color_picker::color_edit_button_rgb(
                                                    ui,
                                                    &mut tint_rgb,
                                                )
                                                .changed()
                                                {
                                                    material.tint[0] = tint_rgb[0];
                                                    material.tint[1] = tint_rgb[1];
                                                    material.tint[2] = tint_rgb[2];
                                                }
                                                ui.end_row();

                                                ui.label("Emissive");
                                                let mut emissive_rgb = [
                                                    material.emissive_factor[0],
                                                    material.emissive_factor[1],
                                                    material.emissive_factor[2],
                                                ];
                                                if egui::color_picker::color_edit_button_rgb(
                                                    ui,
                                                    &mut emissive_rgb,
                                                )
                                                .changed()
                                                {
                                                    material.emissive_factor[0] = emissive_rgb[0];
                                                    material.emissive_factor[1] = emissive_rgb[1];
                                                    material.emissive_factor[2] = emissive_rgb[2];
                                                }
                                                ui.end_row();
                                            });

                                        ui.add_space(6.0);
                                        ui.label(RichText::new("Surface").strong());
                                        Grid::new(format!("material_surface_{}", material_name))
                                            .num_columns(2)
                                            .spacing([12.0, 6.0])
                                            .striped(true)
                                            .show(ui, |ui| {
                                                ui.label("Metallic");
                                                ui.add(
                                                    DragValue::new(&mut material.metallic_factor)
                                                        .speed(0.01)
                                                        .range(0.0..=1.0),
                                                );
                                                ui.end_row();

                                                ui.label("Roughness");
                                                ui.add(
                                                    DragValue::new(&mut material.roughness_factor)
                                                        .speed(0.01)
                                                        .range(0.0..=1.0),
                                                );
                                                ui.end_row();

                                                ui.label("Occlusion Strength");
                                                ui.add(
                                                    DragValue::new(&mut material.occlusion_strength)
                                                        .speed(0.01)
                                                        .range(0.0..=1.0),
                                                );
                                                ui.end_row();

                                                ui.label("Normal Scale");
                                                ui.add(
                                                    DragValue::new(&mut material.normal_scale)
                                                        .speed(0.01)
                                                        .range(0.0..=10.0),
                                                );
                                                ui.end_row();
                                            });

                                        ui.add_space(6.0);
                                        ui.label(RichText::new("Alpha").strong());
                                        Grid::new(format!("material_alpha_{}", material_name))
                                            .num_columns(2)
                                            .spacing([12.0, 6.0])
                                            .striped(true)
                                            .show(ui, |ui| {
                                                ui.label("Alpha Mode");
                                                egui::ComboBox::from_id_salt(format!(
                                                    "alpha_mode_{}",
                                                    material_name
                                                ))
                                                .selected_text(match material.alpha_mode {
                                                    dropbear_engine::model::AlphaMode::Opaque => {
                                                        "Opaque"
                                                    }
                                                    dropbear_engine::model::AlphaMode::Mask => {
                                                        "Mask"
                                                    }
                                                    dropbear_engine::model::AlphaMode::Blend => {
                                                        "Blend"
                                                    }
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
                                                ui.end_row();

                                                ui.label("Alpha Cutoff");
                                                let mut cutoff = material.alpha_cutoff.unwrap_or(0.5);
                                                if ui
                                                    .add(
                                                        DragValue::new(&mut cutoff)
                                                            .speed(0.01)
                                                            .range(0.0..=1.0),
                                                    )
                                                    .changed()
                                                {
                                                    material.alpha_cutoff = Some(cutoff);
                                                }
                                                ui.end_row();

                                                ui.label("Double Sided");
                                                ui.checkbox(&mut material.double_sided, "");
                                                ui.end_row();
                                            });

                                        ui.add_space(6.0);
                                        ui.label(RichText::new("UV & Wrap").strong());
                                        Grid::new(format!("material_uv_{}", material_name))
                                            .num_columns(2)
                                            .spacing([12.0, 6.0])
                                            .striped(true)
                                            .show(ui, |ui| {
                                                ui.label("Wrap");
                                                egui::ComboBox::from_id_salt(format!(
                                                    "wrap_mode_{}",
                                                    material_name
                                                ))
                                                .selected_text(match material.wrap_mode {
                                                    dropbear_engine::texture::TextureWrapMode::Repeat => {
                                                        "Repeat"
                                                    }
                                                    dropbear_engine::texture::TextureWrapMode::Clamp => {
                                                        "Clamp"
                                                    }
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
                                                ui.end_row();

                                                ui.label("UV Tiling");
                                                ui.horizontal(|ui| {
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
                                                ui.end_row();
                                            });

                                        ui.add_space(8.0);
                                        ui.label(RichText::new("Textures").strong());
                                        let texture_matches =
                                            |current: Option<Handle<Texture>>, candidate: Option<Handle<Texture>>| {
                                                match (current, candidate) {
                                                    (None, None) => true,
                                                    (Some(cur), Some(def)) => cur == def,
                                                    _ => false,
                                                }
                                            };
                                        let handle_label = |handle: Handle<Texture>| -> String {
                                            let registry = ASSET_REGISTRY.read();
                                            if let Some(texture) = registry.get_texture(handle) {
                                                if let Some(label) = &texture.label {
                                                    return label.clone();
                                                }
                                                if let Some(reference) = &texture.reference {
                                                    if let Some(uri) = reference.as_uri() {
                                                        return uri
                                                            .rsplit('/')
                                                            .next()
                                                            .filter(|v| !v.is_empty())
                                                            .unwrap_or(uri)
                                                            .to_string();
                                                    }
                                                }
                                            }
                                            format!("Texture {:016x}", handle.id)
                                        };
                                        let original_label = |original: Option<Handle<Texture>>| -> String {
                                            original
                                                .map(handle_label)
                                                .unwrap_or_else(|| "Original (None)".to_string())
                                        };
                                        let texture_combo =
                                            |ui: &mut egui::Ui,
                                             slot_label: &str,
                                             slot_id: &str,
                                             current_texture: Option<Handle<Texture>>,
                                             original_texture: Option<Handle<Texture>>,
                                             default_texture: Option<Handle<Texture>>|
                                             -> Option<Option<Handle<Texture>>> {
                                                let mut updated = None;
                                                let is_original = texture_matches(current_texture, original_texture);
                                                let is_default = texture_matches(current_texture, default_texture);
                                                let selected_text = if is_original {
                                                    original_label(original_texture)
                                                } else if is_default {
                                                    "Default".to_string()
                                                } else if let Some(handle) = current_texture {
                                                    handle_label(handle)
                                                } else {
                                                    "None".to_string()
                                                };

                                                ui.horizontal(|ui| {
                                                    ui.label(slot_label);
                                                    ComboBox::from_id_salt(format!(
                                                        "texture_slot_{}_{}",
                                                        material_name, slot_id
                                                    ))
                                                    .selected_text(selected_text)
                                                    .show_ui(ui, |ui| {
                                                        if ui
                                                            .selectable_label(
                                                                false,
                                                                original_label(original_texture),
                                                            )
                                                            .clicked()
                                                        {
                                                            updated = Some(original_texture);
                                                        }

                                                        ui.separator();

                                                        if ui
                                                            .selectable_label(false, "Default")
                                                            .clicked()
                                                        {
                                                            updated = Some(default_texture);
                                                        }

                                                        ui.separator();

                                                        for (handle, label) in &texture_options {
                                                            if ui
                                                                .selectable_label(false, label)
                                                                .clicked()
                                                            {
                                                                if let Some(texture) = ASSET_REGISTRY
                                                                    .read()
                                                                    .get_texture(*handle)
                                                                {
                                                                    let _ = texture;
                                                                    updated = Some(Some(*handle));
                                                                }
                                                            }
                                                        }
                                                    });
                                                });

                                                updated
                                            };

                                        let original_diffuse =
                                            default_material.as_ref().map(|m| m.diffuse_texture);
                                        let original_normal =
                                            default_material.as_ref().and_then(|m| m.normal_texture);
                                        let original_emissive =
                                            default_material.as_ref().and_then(|m| m.emissive_texture);
                                        let original_mr = default_material
                                            .as_ref()
                                            .and_then(|m| m.metallic_roughness_texture);
                                        let original_occ = default_material
                                            .as_ref()
                                            .and_then(|m| m.occlusion_texture);

                                        let (default_diffuse, default_normal, default_emissive, default_mr, default_occ) =
                                            default_textures.clone();

                                        if let Some(new_diffuse) = texture_combo(
                                            ui,
                                            "Diffuse",
                                            "diffuse",
                                            Some(material.diffuse_texture),
                                            original_diffuse,
                                            default_diffuse,
                                        ) {
                                            if let Some(tex) = new_diffuse {
                                                material.diffuse_texture = tex;
                                                {
                                                    let mut registry = ASSET_REGISTRY.write();
                                                    material.rebuild_bind_group(&mut registry, &graphics);
                                                }
                                                material.sync_uniform(&graphics);
                                            }
                                        }

                                        if let Some(new_normal) = texture_combo(
                                            ui,
                                            "Normal",
                                            "normal",
                                            material.normal_texture,
                                            original_normal,
                                            default_normal,
                                        ) {
                                            if let Some(tex) = new_normal {
                                                material.normal_texture = Some(tex);
                                                {
                                                    let mut registry = ASSET_REGISTRY.write();
                                                    material.rebuild_bind_group(&mut registry, &graphics);
                                                }
                                                material.sync_uniform(&graphics);
                                            }
                                        }

                                        if let Some(new_emissive) = texture_combo(
                                            ui,
                                            "Emissive",
                                            "emissive",
                                            material.emissive_texture,
                                            original_emissive,
                                            default_emissive,
                                        ) {
                                            material.emissive_texture = new_emissive;
                                            {
                                                let mut registry = ASSET_REGISTRY.write();
                                                material.rebuild_bind_group(&mut registry, &graphics);
                                            }
                                            material.sync_uniform(&graphics);
                                        }

                                        if let Some(new_mr) = texture_combo(
                                            ui,
                                            "Metal/Rough",
                                            "metal_rough",
                                            material.metallic_roughness_texture,
                                            original_mr,
                                            default_mr,
                                        ) {
                                            material.metallic_roughness_texture = new_mr;
                                            {
                                                let mut registry = ASSET_REGISTRY.write();
                                                material.rebuild_bind_group(&mut registry, &graphics);
                                            }
                                            material.sync_uniform(&graphics);
                                        }

                                        if let Some(new_occ) = texture_combo(
                                            ui,
                                            "Occlusion",
                                            "occlusion",
                                            material.occlusion_texture,
                                            original_occ,
                                            default_occ,
                                        ) {
                                            material.occlusion_texture = new_occ;
                                            {
                                                let mut registry = ASSET_REGISTRY.write();
                                                material.rebuild_bind_group(&mut registry, &graphics);
                                            }
                                            material.sync_uniform(&graphics);
                                        }
                                    });
                            }
                        });
                }
            });
        });
    }
}
