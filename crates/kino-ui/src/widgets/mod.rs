pub mod rect;

use std::any::Any;
use crate::{KinoState, UiNode, WidgetId};

pub enum Anchor {
    Center,
    TopLeft,
}

pub trait NativeWidget: Send + Sync {
    fn render(self: Box<Self>, state: &mut KinoState);
    fn id(&self) -> WidgetId;
    fn as_any(&self) -> &dyn Any;
}

pub trait ContaineredWidget: Send + Sync {
    fn render(self: Box<Self>, children: Vec<UiNode>, state: &mut KinoState);
    fn id(&self) -> WidgetId;
    fn as_any(&self) -> &dyn Any;
}