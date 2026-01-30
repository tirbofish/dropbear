use std::collections::HashMap;
use std::marker::PhantomData;
use crate::rendering::texture::Texture;

/// A handle with type [`T`] that provides an index to the [AssetServer] contents.
#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub struct Handle<T> {
    pub id: u64,
    _phanton: PhantomData<T>
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
}

impl AssetServer {
    /// Creates a new instance of an AssetServer.
    pub fn new() -> AssetServer {
        AssetServer {
            textures: Default::default(),
        }
    }

    /// Adds a texture and returns a handle.
    ///
    /// This assumes a [Texture] has already been created by you. To create a new texture,
    /// you can use [`Texture::from_bytes`].
    pub fn add_texture(&mut self, texture: Texture) -> Handle<Texture> {
        let handle = Handle::new(texture.hash);
        self.textures.insert(handle.id, texture);
        handle
    }

    /// Updates the asset server by inserting the texture provided at the location of the handle,
    /// and removing the old texture (by returning it back to you).
    pub fn update_texture(&mut self, handle: Handle<Texture>, texture: Texture) -> Option<Texture> {
        self.textures.insert(handle.id, texture)
    }

    pub fn get_texture(&mut self, handle: Handle<Texture>) -> Option<&Texture> {
        self.textures.get(&handle.id)
    }
}