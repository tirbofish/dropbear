use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use hecs::{Entity, World, DynamicBundle};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

pub struct ComponentDescriptor {
    pub fqtn: String,
    pub type_name: String,
    pub category: Option<String>,
    pub description: Option<String>,
}

pub struct ComponentRepository {
    components: HashMap<TypeId, ComponentDescriptor>,
}

impl ComponentRepository {
    pub fn new() -> Self {
        Self {
            components: Default::default(),
        }
    }

    pub fn register<T: Component + 'static>(&mut self) {
        let type_id = TypeId::of::<T>();

        let descriptor = T::static_descriptor();

        self.components.insert(type_id, descriptor);
    }

    pub fn get_descriptor<T: Component + 'static>(&self) -> Option<&ComponentDescriptor> {
        self.components.get(&TypeId::of::<T>())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&TypeId, &ComponentDescriptor)> {
        self.components.iter()
    }

    pub fn descriptors(&self) -> impl Iterator<Item = &ComponentDescriptor> {
        self.components.values()
    }
}

pub trait Component {
    type Serialized: Serialize + for<'de> Deserialize<'de>;

    fn static_descriptor() -> ComponentDescriptor;

    fn deserialize(serialized: &Self::Serialized) -> Self;
    fn serialize(&self) -> Self::Serialized;
    fn inspect(&mut self, ui: &mut egui::Ui);
}

pub mod registry;

pub struct ComponentResources {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ComponentResources {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert<T: Any + Send + Sync>(&mut self, value: T) {
        self.map.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn get<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    pub fn get_mut<T: Any + Send + Sync>(&mut self) -> Option<&mut T> {
        self.map
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }
}

#[derive(Clone)]
pub struct ComponentInitContext {
    pub entity: Entity,
    pub resources: Arc<ComponentResources>,
}

pub type ComponentInitFuture = Pin<Box<dyn Future<Output = anyhow::Result<Box<dyn ComponentInsert>>> + Send + 'static>>;

pub trait ComponentInsert: Send {
    fn insert(self: Box<Self>, world: &mut World, entity: Entity) -> anyhow::Result<()>;
}

pub struct InsertBundle<T: DynamicBundle + Send + 'static>(pub T);

impl<T: DynamicBundle + Send + 'static> ComponentInsert for InsertBundle<T> {
    fn insert(self: Box<Self>, world: &mut World, entity: Entity) -> anyhow::Result<()> {
        world.insert(entity, self.0).map_err(|e| anyhow::anyhow!(e.to_string()))?;
        Ok(())
    }
}

#[typetag::serde(tag = "type")]
pub trait SerializableComponent: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn SerializableComponent>;

    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn display_name(&self) -> String {
        self.type_name()
            .split("::")
            .last()
            .unwrap_or(self.type_name())
            .to_string()
    }

    fn init(&self, ctx: ComponentInitContext) -> ComponentInitFuture;
}

impl Clone for Box<dyn SerializableComponent> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}