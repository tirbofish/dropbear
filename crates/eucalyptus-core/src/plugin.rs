use std::any::TypeId;
use std::collections::HashMap;
use crate::component::{Component, InspectableComponent, LanguageTypeId};
use libloading as lib;

/// FFI signature for the plugin entry point exported as `eucalyptus_plugin_init`.
pub type PluginInitFn = unsafe extern "C" fn(*mut PluginRegistry);

pub struct PluginRegistry {
    tokens: HashMap<PluginRegistrationToken, LanguageTypeId>,
    /// Maps plugin id string to plugin type id.
    ty: HashMap<PluginManifest, LanguageTypeId>,
    /// Maps plugin token to the list of component type ids registered under that plugin.
    components: HashMap<uuid::Uuid, Vec<LanguageTypeId>>,
    plugins: HashMap<LanguageTypeId, lib::Library>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
            ty: HashMap::new(),
            components: HashMap::new(),
            plugins: HashMap::new(),
        }
    }

    pub fn load_plugins(&mut self) {

    }

    pub fn register_plugin<T>(&mut self) -> PluginRegistrationToken
    where T: ExternalPlugin + Send + Sync + 'static
    {
        let type_id = LanguageTypeId::Rust(TypeId::of::<T>());
        let token = PluginRegistrationToken(uuid::Uuid::new_v4());
        self.ty.insert(T::plugin_manifest(), type_id.clone());
        self.tokens.insert(token.clone(), type_id);
        token
    }

    pub fn register_component<T>(&mut self, token: PluginRegistrationToken)
    where T: Component + InspectableComponent + Send + Sync + 'static,
        T::SerializedForm: Send + Sync + 'static,
        T::RequiredComponentTypes: Send + Sync + 'static,
    {
        let component_type_id = LanguageTypeId::Rust(TypeId::of::<T>());
        self.components
            .entry(token.0)
            .or_default()
            .push(component_type_id);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&PluginManifest, &lib::Library)> {
        self.ty.iter().filter_map(|(manifest, type_id)| {
            self.plugins.get(type_id).map(|lib| (manifest, lib))
        })
    }
}

/// Used as a temporary form of registering Components and other types under one plugin.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PluginRegistrationToken(pub(crate) uuid::Uuid);

pub trait ExternalPlugin {
    fn plugin_manifest() -> PluginManifest;
}

#[derive(Hash, Eq, PartialEq)]
pub struct PluginManifest {
    pub display_name: String,
    pub description: String,
    pub version: String,
    pub authors: Vec<String>,
    pub dependencies: Vec<String>,
}

