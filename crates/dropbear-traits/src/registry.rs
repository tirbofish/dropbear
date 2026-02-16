use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

use egui::Ui;
use hecs::{Entity, World};

use crate::{Component, ComponentResources, ComponentUpdateContext, SerializableComponent};

pub struct ComponentRegistry {
    next_id: u32,
    entries: HashMap<u32, ComponentEntry>,
    type_to_id: HashMap<TypeId, u32>,
    converters: Vec<ConverterEntry>,
}

struct ComponentEntry {
    #[allow(dead_code)]
    type_id: TypeId,
    fqtn: &'static str,
    display_name: String,
    create_default: Option<fn() -> Box<dyn SerializableComponent>>,
    extract_fn: Option<fn(&World, Entity) -> Option<Box<dyn SerializableComponent>>>,
    remove_fn: Option<fn(&mut World, Entity)>,
    inspect_fn: Option<fn(&mut World, Entity, &mut Ui) -> bool>,
    update_fn: Option<fn(&mut World, f32, &Arc<ComponentResources>)>,
}

struct ConverterEntry {
    from_type: TypeId,
    convert_fn: Box<dyn Fn(&World, Entity) -> Option<Box<dyn SerializableComponent>> + Send + Sync>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            entries: HashMap::new(),
            type_to_id: HashMap::new(),
            converters: Vec::new(),
        }
    }

    pub fn register_with_default<T>(&mut self)
    where
        T: SerializableComponent + Default + Clone + 'static,
    {
        self.register_entry::<T>(None, None);
    }

    pub fn register_with_default_component<T>(&mut self)
    where
        T: Component + SerializableComponent + Default + Clone + 'static,
    {
        let inspect_fn: fn(&mut World, Entity, &mut Ui) -> bool = |world, entity, ui| {
            if let Ok(mut component) = world.get::<&mut T>(entity) {
                component.inspect(ui);
                true
            } else {
                false
            }
        };

        let update_fn: fn(&mut World, f32, &Arc<ComponentResources>) =
            |world, dt, resources| {
                let world_ptr = world as *const World;
                let mut query = world.query::<(Entity, &mut T)>();
                for (entity, component) in query.iter() {
                    let mut ctx = ComponentUpdateContext::new(
                        entity,
                        dt,
                        resources.clone(),
                        unsafe { &*world_ptr },
                    );
                    component.update(&mut ctx);
                }
            };

        self.register_entry::<T>(Some(inspect_fn), Some(update_fn));
    }

    fn register_entry<T>(
        &mut self,
        inspect_fn: Option<fn(&mut World, Entity, &mut Ui) -> bool>,
        update_fn: Option<fn(&mut World, f32, &Arc<ComponentResources>)>,
    )
    where
        T: SerializableComponent + Default + Clone + 'static,
    {
        let type_id = TypeId::of::<T>();
        if self.type_to_id.contains_key(&type_id) {
            return;
        }

        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);

        let entry = ComponentEntry {
            type_id,
            fqtn: std::any::type_name::<T>(),
            display_name: short_name(std::any::type_name::<T>()),
            create_default: Some(|| Box::new(T::default())),
            extract_fn: Some(|world, entity| {
                world
                    .get::<&T>(entity)
                    .ok()
                    .map(|component| component.clone_box())
            }),
            remove_fn: Some(|world, entity| {
                let _ = world.remove_one::<T>(entity);
            }),
            inspect_fn,
            update_fn,
        };

        self.entries.insert(id, entry);
        self.type_to_id.insert(type_id, id);
    }

    pub fn register_converter<From, To, F>(&mut self, converter: F)
    where
        From: Sync + Send + 'static,
        To: SerializableComponent + 'static,
        F: Fn(&World, Entity, &From) -> Option<To> + Sync + Send + 'static,
    {
        let from_id = TypeId::of::<From>();
        let convert_fn = move |world: &World, entity: Entity| -> Option<Box<dyn SerializableComponent>> {
            world
                .get::<&From>(entity)
                .ok()
                .and_then(|component| converter(world, entity, &component))
                .map(|converted| Box::new(converted) as Box<dyn SerializableComponent>)
        };

        self.converters.push(ConverterEntry {
            from_type: from_id,
            convert_fn: Box::new(convert_fn),
        });
    }

    pub fn iter_available_components(&self) -> impl Iterator<Item = (u32, &str)> {
        self.entries
            .iter()
            .map(|(id, entry)| (*id, entry.fqtn))
    }

    pub fn create_default_component(&self, id: u32) -> Option<Box<dyn SerializableComponent>> {
        self.entries
            .get(&id)
            .and_then(|entry| entry.create_default.map(|ctor| ctor()))
    }

    pub fn id_for_component(&self, component: &dyn SerializableComponent) -> Option<u32> {
        self.type_to_id.get(&component.as_any().type_id()).copied()
    }

    pub fn remove_component_by_id(&self, world: &mut World, entity: Entity, id: u32) {
        if let Some(entry) = self.entries.get(&id) {
            if let Some(remove_fn) = entry.remove_fn {
                remove_fn(world, entity);
            }
        }
    }

    pub fn extract_all_components(
        &self,
        world: &World,
        entity: Entity,
    ) -> Vec<Box<dyn SerializableComponent>> {
        let mut components = Vec::new();

        for entry in self.entries.values() {
            if let Some(extract_fn) = entry.extract_fn {
                if let Some(component) = extract_fn(world, entity) {
                    components.push(component);
                }
            }
        }

        for converter in &self.converters {
            if let Some(component) = (converter.convert_fn)(world, entity) {
                components.push(component);
            }
        }

        components
    }

    pub fn inspect_components(&self, world: &mut World, entity: Entity, ui: &mut Ui) {
        let mut ids: Vec<u32> = self.entries.keys().copied().collect();
        ids.sort_unstable();

        for id in ids {
            let Some(entry) = self.entries.get(&id) else {
                continue;
            };

            if let Some(inspect_fn) = entry.inspect_fn {
                if inspect_fn(world, entity, ui) {
                    ui.add_space(6.0);
                }
            }
        }
    }

    pub fn update_components(
        &self,
        world: &mut World,
        dt: f32,
        resources: &Arc<ComponentResources>,
    ) {
        let mut ids: Vec<u32> = self.entries.keys().copied().collect();
        ids.sort_unstable();

        for id in ids {
            let Some(entry) = self.entries.get(&id) else {
                continue;
            };

            if let Some(update_fn) = entry.update_fn {
                update_fn(world, dt, resources);
            }
        }
    }
    
    pub fn find_components_by_numeric_id(
        &self,
        id: u64
    ) -> Vec<(u32, &str)> {
        self.entries
            .iter()
            .filter(|(_, entry)| entry.display_name.starts_with(&id.to_string()))
            .map(|(id, entry)| (*id, entry.fqtn))
            .collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = (u32, &str)> {
        self.iter_available_components()
    }
}

fn short_name(fqtn: &str) -> String {
    fqtn.split("::").last().unwrap_or(fqtn).to_string()
}
