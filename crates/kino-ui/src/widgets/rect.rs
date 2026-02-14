//! Defines the primitive [`Rectangle`] widget.

use std::any::Any;
use glam::{vec2, Mat2, Vec2};
use crate::{KinoState, UiNode, WidgetId};
use crate::asset::Handle;
use crate::math::Rect;
use crate::rendering::texture::Texture;
use crate::rendering::vertex::Vertex;
use crate::widgets::{Anchor, Border, ContaineredWidget, Fill, NativeWidget};
use crate::resp::WidgetResponse;
use winit::event::{ElementState, MouseButton};

/// A simple and humble rectangle.
pub struct Rectangle {
    /// The identifier of the widget.
    ///
    /// To make life easier, a text id works pretty well.
    pub id: WidgetId,

    /// The positioning of the rectangle.
    ///
    /// Default: [`Anchor::TopLeft`]
    pub anchor: Anchor,

    /// The position of the rectangle
    ///
    /// Default: [`Vec2::ZERO`]
    pub position: Vec2,

    /// The size of the rectangle
    ///
    /// Default: [`vec2(64.0, 64.0)`]
    pub size: Vec2,

    /// The texture that this
    ///
    /// Default: [`None`]
    pub texture: Option<Handle<Texture>>,

    /// Rotation of the rectangle in radians
    ///
    /// Default: `0.0` rad / `0.0` degrees
    pub rotation: f32,

    /// The UV of the textures.
    ///
    /// Default: `[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]`
    pub uvs: [[f32; 2]; 4],

    /// The fill properties of the rectangle
    ///
    /// Default: `[1.0, 1.0, 1.0, 1.0]`
    pub fill: Fill,

    /// The stroke/border properties of the rectangle
    ///
    /// Default: [`None`]
    pub border: Option<Border>,
}

impl Rectangle {
    pub fn new(id: impl Into<WidgetId>) -> Self {
        Self {
            id: id.into(),
            anchor: Anchor::TopLeft,
            position: Vec2::ZERO,
            size: vec2(64.0, 128.0),
            rotation: 0.0,
            uvs: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            fill: Fill::default(),
            texture: None,
            border: None,
        }
    }

    /// Sets the position & size from a [`Rect`].
    pub fn with(mut self, rect: &Rect) -> Self {
        self.position = rect.position;
        self.size = rect.size;
        self
    }

    /// Sets the anchor point of the rectangle
    ///
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

    /// Sets the fill properties of the rectangle
    pub fn fill(mut self, fill: Fill) -> Self {
        self.fill = fill;
        self
    }

    /// Sets the border properties of the rectangle
    pub fn border(mut self, border: Border) -> Self {
        self.border = Some(border);
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
    ///
    /// Defaults to covering the full texture ((0,0) - (1,1))
    pub fn uv(mut self, coords: [[f32; 2]; 4]) -> Self {
        self.uvs = coords;
        self
    }

    pub fn build(self) -> Box<Self> {
        Box::new(self)
    }

    fn compute_rect(&self, state: &KinoState) -> (Vec2, Rect, Mat2) {
        let container_offset = state.layout_offset();
        let offset = match self.anchor {
            Anchor::TopLeft => Vec2::ZERO,
            Anchor::Center => -self.size / 2.0,
        };
        let local_top_left = self.position + offset;
        let top_left = local_top_left + container_offset;
        let rect = Rect::new(top_left, self.size);
        let rot = Mat2::from_angle(self.rotation);
        (local_top_left, rect, rot)
    }

    fn render_body(&self, state: &mut KinoState, rect: &Rect, rot: Mat2) {
        let input = state.input();
        let hovering = rect.contains(input.mouse_position)
            && state.clip_contains(input.mouse_position);
        let clicked = hovering
            && input.mouse_button == MouseButton::Left
            && input.mouse_press_state == ElementState::Pressed;
        state.set_response(
            self.id,
            WidgetResponse {
                queried: self.id,
                clicked,
                hovering,
            },
        );

        let fill_verts: Vec<_> = rect
            .corners()
            .iter()
            .zip(self.uvs.iter())
            .map(|(&corner, &uv)| {
                let world = rot * (corner - rect.center()) + rect.center();
                Vertex::new(world.to_array(), self.fill.colour, uv)
            })
            .collect();

        state.batch.push(&fill_verts, &[0, 1, 2, 2, 3, 0], self.texture);

        if let Some(border) = self.border {
            let half_width = border.width / 2.0;
            let outer_rect = Rect::new(
                rect.position - Vec2::splat(half_width),
                rect.size + Vec2::splat(border.width)
            );
            let inner_rect = Rect::new(
                rect.position + Vec2::splat(half_width),
                rect.size - Vec2::splat(border.width)
            );

            let outer_corners = outer_rect.corners();
            let inner_corners = inner_rect.corners();

            let mut border_verts = Vec::with_capacity(8);
            for i in 0..4 {
                let outer_world = rot * (outer_corners[i] - rect.center()) + rect.center();
                border_verts.push(Vertex::new(outer_world.to_array(), border.colour, [0.0, 0.0]));

                let inner_world = rot * (inner_corners[i] - rect.center()) + rect.center();
                border_verts.push(Vertex::new(inner_world.to_array(), border.colour, [0.0, 0.0]));
            }

            let border_indices = [
                0, 1, 3, 3, 2, 0,
                2, 3, 5, 5, 4, 2,
                4, 5, 7, 7, 6, 4,
                6, 7, 1, 1, 0, 6,
            ];

            state.batch.push(&border_verts, &border_indices, None);
        }
    }
}

impl NativeWidget for Rectangle {
    fn render(self: Box<Self>, state: &mut KinoState) {
        let (_local_top_left, rect, rot) = self.compute_rect(state);
        self.render_body(state, &rect, rot);
    }

    fn size(&self) -> Vec2 {
        self.size
    }

    fn id(&self) -> WidgetId {
        self.id
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ContaineredWidget for Rectangle {
    fn render(self: Box<Self>, children: Vec<UiNode>, state: &mut KinoState) {
        let (local_top_left, rect, rot) = self.compute_rect(state);
        self.render_body(state, &rect, rot);
        state.push_container(Rect::new(local_top_left, self.size));
        state.render_tree(children);
        state.pop_container();
    }

    fn size(&self, _children: &[UiNode]) -> Vec2 {
        self.size
    }

    fn id(&self) -> WidgetId {
        self.id
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}