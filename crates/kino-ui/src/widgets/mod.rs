pub mod rect;
pub mod shorthand;
pub mod text;
pub mod layout;

use std::any::Any;
use glam::Vec2;
use crate::{KinoState, UiNode, WidgetId};

/// Determines how the object is anchored.
pub enum Anchor {
    /// A center anchor is when the position is based on the center of the object (such as the
    /// center of a circle)
    Center,
    /// A top left anchor is when the position is based on the top left corner of the rectangle.
    TopLeft,
}

/// Defines a widget with no children. 
pub trait NativeWidget: Send + Sync {
    /// Renders the widget. 
    /// 
    /// The state is provided for you to manipulate, such as adding a new response based on the
    /// [`WidgetId`]. 
    fn render(self: Box<Self>, state: &mut KinoState);
    fn size(&self) -> Vec2;
    fn id(&self) -> WidgetId;
    fn as_any(&self) -> &dyn Any;
}

pub trait ContaineredWidget: Send + Sync {
    fn render(self: Box<Self>, children: Vec<UiNode>, state: &mut KinoState);
    fn size(&self, children: &[UiNode]) -> Vec2;
    fn id(&self) -> WidgetId;
    fn as_any(&self) -> &dyn Any;
}

/// Describes the colour that the widget will be filled in with. 
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Fill {
    /// The colour of the fill described as RGBA between the range `0.0` <-> `1.0`.
    ///
    /// If a texture is applied to the colour, it will create a tinted texture on the quad.
    pub colour: [f32; 4],
}

impl Fill {
    /// Creates a new [`Fill`]
    pub fn new(colour: [f32; 4]) -> Self {
        Fill { colour }
    }
}

impl Default for Fill {
    fn default() -> Self {
        Fill { colour: [1.0, 1.0, 1.0, 1.0] }
    }
}

/// Describes the properties of the border/stroke of the widget. 
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Border {
    /// The colour of the border described as RGBA between the range `0.0` <-> `1.0`.
    pub colour: [f32; 4],
    
    /// The width of the border. 
    pub width: f32,
}

impl Border {
    /// Creates a new [`Border`]. 
    pub fn new(colour: [f32; 4], width: f32) -> Self {
        Self { colour, width }
    }
}