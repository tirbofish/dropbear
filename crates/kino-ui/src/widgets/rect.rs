use std::any::Any;
use glam::{vec2, Mat2, Vec2};
use crate::{KinoState, WidgetId};
use crate::asset::Handle;
use crate::math::Rect;
use crate::rendering::texture::Texture;
use crate::rendering::vertex::Vertex;
use crate::widgets::{Anchor, NativeWidget};

pub struct Rectangle {
    /// The identifier of the widget.
    ///
    /// To make life easier, a text id works pretty well.
    pub id: WidgetId,

    pub anchor: Anchor,

    /// The position of the rectangle
    pub position: Vec2,

    /// The size of the rectangle
    pub size: Vec2,

    /// The texture that this
    pub texture: Option<Handle<Texture>>,

    /// Colour described as RGBA.
    ///
    /// If a texture is applied to the colour, it will create a tinted texture on the quad.
    pub colour: [f32; 4], // for now until colour is properly implemented

    /// Rotation of the rectangle in radians
    pub rotation: f32,

    /// The UV of the textures.
    pub uvs: [[f32; 2]; 4],
}

impl Rectangle {
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            anchor: Anchor::TopLeft,
            position: Vec2::ZERO,
            size: vec2(64.0, 64.0),
            rotation: 0.0,
            colour: [255.0, 255.0, 255.0, 255.0],
            uvs: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            texture: None,
        }
    }

    /// Sets the position & size from a [`Rect`].
    pub fn with(mut self, rect: &Rect) -> Self {
        self.position = rect.position;
        self.size = rect.size;
        self
    }
    /// Sets the anchor point of the rectangle
    /// Defaults to [`Anchor::TopLeft`].
    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }
    /// Sets the world-space position of the rectangle
    pub fn at(mut self, position: impl Into<Vec2>) -> Self {
        self.position = position.into();
        self
    }
    /// Sets the size of the rectangle
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
    /// Sets the color of the rectangle
    pub fn color(mut self, colour: [f32; 4]) -> Self {
        self.colour = colour;
        self
    }
    /// Sets rotation (in radians) around the rectangle's center
    /// 0 radians points up (positive Y), increasing clockwise
    pub fn rotate(mut self, angle: f32) -> Self {
        self.rotation = angle + std::f32::consts::FRAC_PI_2;
        self
    }
    /// Sets the texture ID for the rectangle
    pub fn texture(mut self, id: Handle<Texture>) -> Self {
        self.texture = Some(id);
        self
    }
    /// Custom UV coordinates
    /// Defaults to covering the full texture ((0,0) - (1,1))
    pub fn uv(mut self, coords: [[f32; 2]; 4]) -> Self {
        self.uvs = coords;
        self
    }
}

impl NativeWidget for Rectangle {
    fn render(self: Box<Self>, state: &mut KinoState) {
        let offset = match self.anchor {
            Anchor::TopLeft => Vec2::ZERO,
            Anchor::Center => -self.size / 2.0,
        };
        let top_left = self.position + offset;
        let rect = Rect::new(top_left, self.size);
        let rot = Mat2::from_angle(self.rotation);
        let verts: Vec<_> = rect
            .corners()
            .iter()
            .zip(self.uvs.iter())
            .map(|(&corner, &uv)| {
                let world = rot * (corner - rect.center()) + rect.center();
                Vertex::new(world.to_array(), self.colour, uv)
            })
            .collect();

        state.batch.push(&verts, &[0, 1, 2, 2, 3, 0], self.texture);
    }

    fn id(&self) -> WidgetId {
        self.id
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}