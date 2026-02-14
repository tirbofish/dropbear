use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::marker::PhantomData;
use crate::rendering::texture::Texture;

/// A handle with type [`T`] that provides an index to the [AssetServer] contents.
#[derive(Hash, Eq, PartialEq, Debug)]
pub struct Handle<T> {
    pub id: u64,
    _phanton: PhantomData<T>
}

impl<T> Copy for Handle<T> {}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Handle<T> {
    pub fn new(id: u64) -> Self {
        Self { id, _phanton: Default::default() }
    }
}

/// Serves assets as a [Handle], allowing you to reference any object.
#[derive(Default)]
pub struct AssetServer {
    textures: HashMap<u64, Texture>,
    texture_labels: HashMap<String, Handle<Texture>>,
}

impl AssetServer {
    /// Creates a new instance of an AssetServer.
    pub fn new() -> AssetServer {
        AssetServer {
            textures: Default::default(),
            texture_labels: Default::default(),
        }
    }

    /// Adds a texture and returns a handle.
    ///
    /// This assumes a [Texture] has already been created by you. To create a new texture,
    /// you can use [`Texture::from_bytes`].
    pub fn add_texture(&mut self, texture: Texture) -> Handle<Texture> {
        let handle = Handle::new(texture.hash);
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

    pub fn get_texture_handle(&self, label: &str) -> Option<Handle<Texture>> {
        self.texture_labels.get(label).cloned()
    }

    pub fn texture_handle_by_hash(&self, hash: u64) -> Option<Handle<Texture>> {
        self.textures.contains_key(&hash).then(|| Handle::new(hash))
    }

    pub fn hash_bytes(data: &[u8]) -> u64 {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }
}