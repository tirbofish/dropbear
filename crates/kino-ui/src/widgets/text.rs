use std::any::Any;
use glam::Vec2;
use glyphon::{Attrs, AttrsOwned, Buffer, Color, Metrics, Shaping};
use crate::{KinoState, WidgetId};
use crate::rendering::text::TextEntry;
use crate::widgets::NativeWidget;

pub struct Text {
    pub id: WidgetId,
    pub text: String,
    pub position: Vec2,
    pub metrics: Metrics,
    pub attributes: AttrsOwned,
}

impl Text {
    /// Create a new text builder that will push text to the renderer
    pub fn new(text: String) -> Self {
        Self {
            id: WidgetId::from_str(&text),
            text,
            position: Vec2::new(10.0, 10.0),
            metrics: Metrics::new(16.0, 1.0),
            attributes: AttrsOwned::new(Attrs::new()),
        }
    }

    /// Set a custom ID before sending the widget off
    pub fn with_id(mut self, id: WidgetId) -> Self {
        self.id = id;
        self
    }

    /// Set the position of text in screen space
    pub fn at(mut self, position: impl Into<Vec2>) -> Self {
        self.position = position.into();
        self
    }

    pub fn with_attrs(mut self, attributes: AttrsOwned) -> Self {
        self.attributes = attributes;
        self
    }

    pub fn with_metrics(mut self, metrics: Metrics) -> Self {
        self.metrics = metrics;
        self
    }
}

impl NativeWidget for Text {
    fn render(self: Box<Self>, state: &mut KinoState) {
        let mut buffer = Buffer::new(&mut state.renderer.text.font_system, self.metrics);
        buffer.set_text(
            &mut state.renderer.text.font_system,
            &self.text,
            self.attributes.as_attrs(),
            Shaping::Basic,
        );

        state.renderer.text.entries.push(TextEntry {
            buffer,
            position: self.position,
        });
    }

    fn id(&self) -> WidgetId {
        self.id
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}