use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::marker::PhantomData;
use std::sync::{Arc, LazyLock};
use parking_lot::RwLock;
use crate::{
    texture::Texture,
};
use crate::graphics::SharedGraphicsContext;
use crate::model::Model;

pub static ASSET_REGISTRY: LazyLock<Arc<RwLock<AssetRegistry>>> = LazyLock::new(|| Arc::new(RwLock::new(AssetRegistry::new())));

/// A handle with type [`T`] that provides an index to the [AssetRegistry] contents.
#[derive(Hash, Eq, Debug)]
pub struct Handle<T> {
    pub id: u64,
    _phantom: PhantomData<T>
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.is_null() && other.is_null() {
            return false;
        }
        self.id == other.id
    }


}

impl<T> Copy for Handle<T> {}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Handle<T> {
    /// Creates a null handle, for when there is no way to uniquely identify a hash (such as a viewport texture).
    ///
    /// # Safety
    /// You will want to watch out, as adding this onto the asset registry with a type
    /// where there already is a null handle item, it will be overwritten and data
    /// will not be saved. It is the reason why you will want to consider using the [Self::is_null]
    /// function to verify if the storage of the type has gone through correctly.
    pub const NULL: Self = Self { id: 0, _phantom: PhantomData };

    /// Creates a new handle with the given ID.
    pub fn new(id: u64) -> Self {
        Self { id, _phantom: Default::default() }
    }

    /// Returns true if the handle is null.
    pub fn is_null(&self) -> bool {
        self.id == 0
    }
}

pub struct AssetRegistry {
    textures: HashMap<u64, Texture>,
    texture_labels: HashMap<String, Handle<Texture>>,

    models: HashMap<u64, Model>,
    model_labels: HashMap<String, Handle<Model>>,
}

#[repr(C)]
#[derive(Debug, Clone)]
#[dropbear_macro::repr_c_enum]
pub enum AssetKind {
    Texture,
    Model,
}

/// Common
impl AssetRegistry {
    pub fn new() -> Self {
        Self {
            textures: Default::default(),
            texture_labels: Default::default(),
            models: Default::default(),
            model_labels: Default::default(),
        }
    }

    /// A convenient helper function for hashing a byte slice of data.
    pub(crate) fn hash_bytes(data: &[u8]) -> u64 {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }

    /// A convenient helper function for hashing a byte slice of data.
    pub(crate) fn hash_contents<T: Hash>(data: T) -> u64 {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }

    /// Checks if the asset registry contains a handle with the given hash.
    ///
    /// It will check all different types, so it does not point out specifically where.
    pub fn contains_hash(&self, hash: u64) -> bool {
        self.textures.contains_key(&hash) || self.models.contains_key(&hash)
    }

    /// Checks if the asset registry contains a handle with the given string label.
    ///
    /// It will check all different types, so it does not point out specifically where.
    pub fn contains_label(&self, label: &str) -> bool {
        self.texture_labels.contains_key(label) || self.model_labels.contains_key(label)
    }
}

/// Texture stuff
impl AssetRegistry {
    /// Adds a texture and returns a handle.
    ///
    /// This assumes a [Texture] has already been created by you. To create a new texture,
    /// you can use [`Texture::from_bytes`].
    pub fn add_texture(&mut self, texture: Texture) -> Handle<Texture> {
        let handle = texture.hash.map(|v| Handle::new(v)).unwrap_or_else(|| Handle::NULL);
        self.textures.entry(handle.id).or_insert(texture);
        handle
    }

    /// Adds a texture with a label. If the texture already exists (by hash),
    /// returns the existing handle and updates the label to point at it.
    pub fn add_texture_with_label(&mut self, label: impl Into<String>, texture: Texture) -> Handle<Texture> {
        let handle = self.add_texture(texture);
        self.texture_labels.insert(label.into(), handle.clone());
        handle
    }

    /// Maps a label to an existing texture handle.
    pub fn label_texture(&mut self, label: impl Into<String>, handle: Handle<Texture>) {
        self.texture_labels.insert(label.into(), handle.clone());
    }

    /// Removes a label from the texture registry, but keeps it in the registry.
    ///
    /// When the label is removed, the [Handle] is still valid.
    pub fn remove_label_texture(&mut self, label: &str) {
        self.texture_labels.remove(label);
    }

    /// Updates the asset server by inserting the texture provided at the location of the handle,
    /// and removing the old texture (by returning it back to you).
    pub fn update_texture(&mut self, handle: Handle<Texture>, texture: Texture) -> Option<Texture> {
        self.textures.insert(handle.id, texture)
    }

    pub fn get_texture(&self, handle: Handle<Texture>) -> Option<&Texture> {
        self.textures.get(&handle.id)
    }

    pub fn get_texture_by_label(&self, label: &str) -> Option<&Texture> {
        self.texture_labels
            .get(label)
            .and_then(|handle| self.textures.get(&handle.id))
    }

    pub fn get_texture_handle_from_label(&self, label: &str) -> Option<Handle<Texture>> {
        self.texture_labels.get(label).cloned()
    }

    pub fn texture_handle_by_hash(&self, hash: u64) -> Option<Handle<Texture>> {
        self.textures.contains_key(&hash).then(|| Handle::new(hash))
    }

    pub fn grey_texture(&mut self, graphics: Arc<SharedGraphicsContext>) -> Handle<Texture> {
        let grey_handle = Handle::new(Self::hash_contents("Solid texture [128, 128, 128, 255]"));
        
        if self.contains_hash(grey_handle.id) {
            return grey_handle;
        }

        self.solid_texture_rgba8(graphics, [128, 128, 128, 255])
    }

    pub fn solid_texture_rgba8(
        &mut self,
        graphics: Arc<SharedGraphicsContext>,
        rgba: [u8; 4],
    ) -> Handle<Texture> {
        let handle = Handle::new(Self::hash_bytes(&rgba));

        if self.contains_hash(handle.id) {
            return handle;
        }

        let label = format!("Solid texture [{}, {}, {}, {}]", rgba[0], rgba[1], rgba[2], rgba[3]);

        let texture = Texture::from_bytes_verbose_mipmapped(
            graphics,
            &rgba,
            Some((1, 1)),
            None,
            None,
            Some(label.as_str())
        );
        
        self.add_texture_with_label(label, texture)
    }

    pub fn get_label_from_texture_handle(&self, handle: Handle<Texture>) -> Option<String> {
        self.texture_labels.iter().find_map(|(label, h)| if *h == handle { Some(label.clone()) } else { None })
    }
}

/// Model stuff
impl AssetRegistry {
    pub fn add_model(&mut self, model: Model) -> Handle<Model> {
        let handle = Handle::new(model.hash);
        self.models.entry(handle.id).or_insert(model);
        handle
    }

    pub fn add_model_with_label(&mut self, label: impl Into<String>, model: Model) -> Handle<Model> {
        let handle = self.add_model(model);
        self.model_labels.insert(label.into(), handle.clone());
        handle
    }

    pub fn label_model(&mut self, label: impl Into<String>, handle: Handle<Model>) {
        self.model_labels.insert(label.into(), handle.clone());
    }

    pub fn update_model(&mut self, handle: Handle<Model>, model: Model) -> Option<Model> {
        self.models.insert(handle.id, model)
    }

    pub fn get_model(&self, handle: Handle<Model>) -> Option<&Model> {
        self.models.get(&handle.id)
    }

    pub fn get_model_by_label(&self, label: &str) -> Option<&Model> {
        self.model_labels
            .get(label)
            .and_then(|handle| self.models.get(&handle.id))
    }

    pub fn get_model_handle_from_label(&self, label: &str) -> Option<Handle<Model>> {
        self.model_labels.get(label).cloned()
    }

    pub fn model_handle_by_hash(&self, hash: u64) -> Option<Handle<Model>> {
        self.models.contains_key(&hash).then(|| Handle::new(hash))
    }

    pub fn get_label_from_model_handle(&self, handle: Handle<Model>) -> Option<String> {
        self.model_labels.iter().find_map(|(label, h)| if *h == handle { Some(label.clone()) } else { None })
    }
}

impl Default for AssetRegistry {
    fn default() -> Self {
        Self::new()
    }
}