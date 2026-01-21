use std::hash::{DefaultHasher, Hash, Hasher};
use parking_lot::Mutex;
use crate::math::Size;
use crate::rendering::KinoRenderer;

pub mod rendering;
pub(crate) mod math;
pub(crate) mod primitives;
pub(crate) mod utils;

pub mod prelude {
    pub use crate::{
        rendering::KinoRenderer,
        math::*,
        primitives::*,
        WidgetId,
        Widget,
    };
}

pub struct GumContext {
    screen_size: Size,
}

impl GumContext {
    pub fn new() -> Self {
        Self { screen_size: Size::default() }
    }
}

pub struct KinoUICommandBuffer {
    contents: Mutex<Vec<Box<dyn Widget>>>,
}

impl KinoUICommandBuffer {
    pub(crate) fn new() -> Self {
        Self {
            contents: Mutex::new(Vec::new()),
        }
    }

    /// Drains the [self.contents] and sends back to the renderer
    pub(crate) fn process<'a>(&self) -> Vec<Box<dyn Widget>> {
        let mut contents = self.contents.lock();
        contents.drain(..).collect()
    }

    pub fn add<T: Widget + 'static>(&self, widget: T) {
        self.contents.lock().push(Box::new(widget));
    }
}

pub trait Widget {
    fn id(&self) -> WidgetId;
    fn draw<'a>(&mut self, renderer: &KinoRenderer, pass: &mut wgpu::RenderPass<'a>);
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct WidgetId(u64);

impl WidgetId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl Into<WidgetId> for String {
    fn into(self) -> WidgetId {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        WidgetId(hasher.finish())
    }
}