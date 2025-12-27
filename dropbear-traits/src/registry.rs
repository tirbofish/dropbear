use crate::{
    ComponentConverter, ComponentDeserializer, CustomConverter, CustomDeserializer,
    DirectConverter, DirectDeserializer, SerializableComponent,
};
use anyhow::Result;
use hecs::{Entity, EntityBuilder, World};
use std::any::TypeId;
use std::collections::HashMap;

// note: to anyone viewing this, yes i did use AI to generate the documentation for this module because
// sometimes i kept forgetting what function some did. mb

/// Registry of conversions between ECS components (`hecs`) and [`SerializableComponent`]
/// values.
///
/// # What this type does
///
/// `ComponentRegistry` is an adapter layer around a `hecs::World` that lets you:
///
/// - **Extract** one or more [`SerializableComponent`] values from an entity.
/// - **Deserialize** a [`SerializableComponent`] back into a `hecs` component
///   and insert it into an [`EntityBuilder`].
/// - **Address component "kinds" by numeric IDs** (useful for editor UI, network
///   messages, prefab formats, etc.).
/// - **Provide default/factory construction** for editor "Add component" flows.
///
/// # Numeric IDs and stability
///
/// Component IDs are assigned lazily (on registration) starting from `1`.
///
/// - IDs are **stable only for the lifetime of this registry instance**.
/// - IDs are **not guaranteed to be stable across runs** (or across different
///   registration orders), because they're assigned incrementally.
/// - Display / editor lists returned by [`iter_available_components`] will be in
///   **arbitrary order**, because `HashMap` iteration order is not deterministic.
///
/// If you need cross-run stable identifiers (e.g., long-lived save files), you
/// should layer an explicit, user-assigned ID scheme on top.
///
/// # Converters vs deserializers
///
/// A **converter** looks at an entity and tries to produce a serializable value.
/// A **deserializer** takes a serializable value and inserts a real ECS component
/// into an [`EntityBuilder`].
///
/// For directly-serializable components, [`register`] wires up both.
/// For custom flows, [`register_converter`] and [`register_deserializer`] can be
/// used independently.
pub struct ComponentRegistry {
    converters: HashMap<TypeId, Box<dyn ComponentConverter>>,
    deserializers: HashMap<TypeId, Box<dyn ComponentDeserializer>>,
    serializable_ids: HashMap<TypeId, u64>,
    id_to_serializable: HashMap<u64, TypeId>,
    default_creators: HashMap<u64, Box<dyn Fn() -> Box<dyn SerializableComponent> + Send + Sync>>,
    next_component_id: u64,
}

impl ComponentRegistry {
    /// Creates an empty registry.
    ///
    /// No components can be extracted or deserialized until they are registered.
    pub fn new() -> Self {
        Self {
            converters: HashMap::new(),
            deserializers: HashMap::new(),
            serializable_ids: HashMap::new(),
            id_to_serializable: HashMap::new(),
            default_creators: HashMap::new(),
            next_component_id: 1,
        }
    }

    /// Registers a component type that is already a [`SerializableComponent`].
    ///
    /// This is the common case: `T` is both a `hecs` component and a serializable
    /// value. The registry will:
    ///
    /// - Assign a numeric ID to `T` (if it doesn't already have one).
    /// - Register a direct converter (extract `T` from an entity and clone it).
    /// - Register a direct deserializer (insert a cloned `T` into a builder).
    ///
    /// Note: numeric IDs are assigned based on registration order; see the type
    /// docs for stability caveats.
    pub fn register<T>(&mut self)
    where
        T: SerializableComponent + hecs::Component + Clone + 'static,
    {
        let type_id = TypeId::of::<T>();
        self.ensure_serializable_id(type_id);
        self.converters
            .insert(type_id, Box::new(DirectConverter::<T>::new()));
        self.deserializers
            .insert(type_id, Box::new(DirectDeserializer::<T>::new()));
    }

    /// Registers `T` and also exposes it as an "available" component with a
    /// default constructor.
    ///
    /// This is primarily meant for editor tooling: values registered via this
    /// method will appear in [`iter_available_components`] and can be created via
    /// [`create_default_component`].
    pub fn register_with_default<T>(&mut self)
    where
        T: SerializableComponent + hecs::Component + Clone + Default + 'static,
    {
        self.register::<T>();
        let id = self.id_for_type::<T>().unwrap();
        self.default_creators
            .insert(id, Box::new(|| Box::new(T::default())));
    }

    /// Registers `T` for extraction/deserialization (if needed) and associates a
    /// custom factory used to create new instances of `T`.
    ///
    /// Like [`register_with_default`], this is intended for editor tooling.
    /// Use this when the best "empty" value can't be expressed as `Default`.
    pub fn register_factory<T, F>(&mut self, factory: F)
    where
        T: SerializableComponent + hecs::Component + Clone + 'static,
        F: Fn() -> Box<dyn SerializableComponent> + Send + Sync + 'static,
    {
        // Ensure registered first
        if self.id_for_type::<T>().is_none() {
            self.register::<T>();
        }
        let id = self.id_for_type::<T>().unwrap();
        self.default_creators.insert(id, Box::new(factory));
    }

    /// Creates a new component instance using the default constructor/factory
    /// registered for `component_id`.
    ///
    /// Returns `None` if no factory/default was registered for that ID.
    pub fn create_default_component(
        &self,
        component_id: u64,
    ) -> Option<Box<dyn SerializableComponent>> {
        self.default_creators.get(&component_id).map(|f| f())
    }

    /// Removes the (source) ECS component associated with the given numeric ID
    /// from `entity`.
    ///
    /// The registry resolves `component_id` to a *serializable* type, then finds
    /// the first registered converter whose output type matches and asks it to
    /// remove the underlying ECS component.
    ///
    /// ## Notes / edge cases
    ///
    /// - If no ID mapping exists or no converter matches, this is a no-op.
    /// - If multiple converters map to the same serializable type, only the first
    ///   match (in arbitrary `HashMap` order) will be used.
    pub fn remove_component_by_id(&self, world: &mut World, entity: Entity, component_id: u64) {
        if let Some(expected_type) = self.serializable_type_from_numeric(component_id) {
            // Find the converter that produces this serializable type
            for converter in self.converters.values() {
                if converter.serializable_type_id() == expected_type {
                    converter.remove_component(world, entity);
                    // We can stop after finding one, assuming one-to-one mapping for removal
                    // Or should we continue? Usually one component type per entity.
                    // But multiple converters might produce same serializable type?
                    // If so, we might try to remove all possible source components.
                    // But usually it's 1:1.
                    return;
                }
            }
        }
    }

    /// Iterates the set of components that are considered "addable" via defaults
    /// or factories.
    ///
    /// Yields pairs of `(numeric_id, type_name)`.
    ///
    /// The returned iterator only includes components for which:
    ///
    /// - a default/factory was registered (via [`register_with_default`] or
    ///   [`register_factory`]), and
    /// - a deserializer exists to provide a human-friendly type name.
    ///
    /// Ordering is arbitrary.
    pub fn iter_available_components(&self) -> impl Iterator<Item = (u64, &str)> {
        self.default_creators.keys().filter_map(move |id| {
            let type_id = self.id_to_serializable.get(id)?;
            let deserializer = self.deserializers.get(type_id)?;
            Some((*id, deserializer.serializable_type_name()))
        })
    }

    /// Registers a custom converter that extracts a serializable value `To` from
    /// an ECS component `From`.
    ///
    /// Use this when the runtime ECS component isn't directly serializable, but
    /// you can derive a serializable representation from it.
    ///
    /// The provided function receives `(world, entity, &From)` and may return
    /// `None` to indicate "not present / not applicable".
    ///
    /// Note: this registers an ID for `To` (the serializable output type), not for
    /// `From`.
    pub fn register_converter<From, To, F>(&mut self, converter_fn: F)
    where
        From: hecs::Component + 'static,
        To: SerializableComponent + 'static,
        F: Fn(&World, Entity, &From) -> Option<To> + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<From>();
        // converter output is To, so track its serializable id
        self.ensure_serializable_id(TypeId::of::<To>());
        self.converters
            .insert(type_id, Box::new(CustomConverter::new(converter_fn)));
    }

    /// Registers a custom deserializer that converts a serializable `From` into
    /// a concrete ECS component `To`.
    ///
    /// This is the inverse of [`register_converter`]. Use it when `From` is the
    /// type you store/transport, and `To` is the type you actually attach to an
    /// entity.
    pub fn register_deserializer<From, To, F>(&mut self, converter_fn: F)
    where
        From: SerializableComponent + 'static,
        To: hecs::Component + Clone + 'static,
        F: Fn(&From) -> To + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<From>();
        self.ensure_serializable_id(type_id);
        self.deserializers
            .insert(type_id, Box::new(CustomDeserializer::new(converter_fn)));
    }

    /// Extracts all registered serializable components from `entity`.
    ///
    /// This calls every registered converter and collects the values it returns.
    ///
    /// Ordering is arbitrary (depends on `HashMap` iteration order).
    pub fn extract_all_components(
        &self,
        world: &World,
        entity: Entity,
    ) -> Vec<Box<dyn SerializableComponent>> {
        let mut vec = vec![];
        for converter in self.converters.values() {
            if let Some(component) = converter.extract_serializable(world, entity) {
                vec.push(component);
            }
        }
        return vec;
    }

    /// Ensures a numeric ID exists for the given serializable `TypeId` and
    /// returns it.
    ///
    /// IDs are assigned incrementally and wrap on overflow. `0` is never used.
    fn ensure_serializable_id(&mut self, type_id: TypeId) -> u64 {
        if let Some(id) = self.serializable_ids.get(&type_id) {
            *id
        } else {
            let id = self.next_component_id;
            self.next_component_id = self.next_component_id.wrapping_add(1).max(1);
            self.serializable_ids.insert(type_id, id);
            self.id_to_serializable.insert(id, type_id);
            id
        }
    }

    /// Returns the numeric identifier assigned to the dynamic component value.
    ///
    /// Returns `None` if the component's concrete type has not been registered.
    pub fn id_for_component(&self, component: &dyn SerializableComponent) -> Option<u64> {
        let type_id = component.as_any().type_id();
        self.serializable_ids.get(&type_id).copied()
    }

    /// Returns the numeric identifier assigned to the serializable type `T`.
    ///
    /// Returns `None` if `T` has not been registered (directly or as a converter
    /// output).
    pub fn id_for_type<T>(&self) -> Option<u64>
    where
        T: SerializableComponent + 'static,
    {
        self.serializable_ids.get(&TypeId::of::<T>()).copied()
    }

    /// Looks up the serializable `TypeId` associated with a numeric identifier.
    fn serializable_type_from_numeric(&self, component_id: u64) -> Option<TypeId> {
        self.id_to_serializable.get(&component_id).copied()
    }

    /// Extracts a single serializable component from `entity` by numeric ID.
    ///
    /// Returns `None` if:
    ///
    /// - `component_id` is unknown, or
    /// - none of the registered converters produce a value of that type for the
    ///   given entity.
    ///
    /// If multiple converters can produce the same serializable type, the first
    /// match (in arbitrary order) wins.
    pub fn extract_component_by_numeric_id(
        &self,
        world: &World,
        entity: Entity,
        component_id: u64,
    ) -> Option<Box<dyn SerializableComponent>> {
        let expected_type = self.serializable_type_from_numeric(component_id)?;

        for converter in self.converters.values() {
            if let Some(component) = converter.extract_serializable(world, entity) {
                if component.as_any().type_id() == expected_type {
                    return Some(component);
                }
            }
        }

        None
    }

    /// Finds every entity in `world` that has a component matching `component_id`.
    ///
    /// This is a convenience wrapper that iterates all entities and uses
    /// [`extract_component_by_numeric_id`] to test each one.
    pub fn find_components_by_numeric_id(
        &self,
        world: &World,
        component_id: u64,
    ) -> Vec<(Entity, Box<dyn SerializableComponent>)> {
        let mut matches = Vec::new();
        for (entity, ()) in world.query::<()>().iter() {
            if let Some(component) =
                self.extract_component_by_numeric_id(world, entity, component_id)
            {
                matches.push((entity, component));
            }
        }
        matches
    }

    /// Deserializes a [`SerializableComponent`] into an ECS component and inserts
    /// it into `builder`.
    ///
    /// Returns:
    ///
    /// - `Ok(true)` if a deserializer was found and insertion succeeded.
    /// - `Ok(false)` if no deserializer is registered for this component type.
    /// - `Err(_)` if a deserializer was found but it failed.
    pub fn deserialize_into_builder(
        &self,
        component: &dyn SerializableComponent,
        builder: &mut EntityBuilder,
    ) -> Result<bool> {
        let type_id = component.as_any().type_id();
        if let Some(deserializer) = self.deserializers.get(&type_id) {
            deserializer.insert_into_builder(component, builder)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
