use std::any::Any;
use glam::Vec2;
use glyphon::{Attrs, AttrsOwned, Buffer, Color, Metrics, Shaping};
use crate::{KinoState, WidgetId};
use crate::math::Rect;
use crate::rendering::text::TextEntry;
use crate::resp::WidgetResponse;
use crate::widgets::NativeWidget;
use winit::event::{ElementState, MouseButton};

/// Creates a label with the specified text and properties. 
/// 
/// # Input
/// Responses are weird for text, as it recognises the input when you touch the text itself. 
/// 
/// If you want an area, you might be interested in [`crate::rect_container`] (with a transparent colour). 
pub struct Text {
    pub id: WidgetId,
    pub text: String,
    pub position: Vec2,
    pub size: Vec2,
    pub metrics: Metrics,
    pub attributes: AttrsOwned,
}

impl Text {
    /// Create a new text builder that will push text to the renderer
    pub fn new(text: impl ToString) -> Self {
        Self {
            id: text.to_string().into(),
            text: text.to_string(),
            position: Vec2::new(10.0, 10.0),
            size: Vec2::ZERO,
            metrics: Metrics::new(16.0, 1.0),
            attributes: AttrsOwned::new(&Attrs::new().color(Color::rgb(0, 0, 0))),
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

    /// Sets the position & size from a [`Rect`].
    pub fn with(mut self, rect: &Rect) -> Self {
        self.position = rect.position;
        self.size = rect.size;
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
        if self.size != Vec2::ZERO {
            buffer.set_size(
                &mut state.renderer.text.font_system,
                Some(self.size.x),
                Some(self.size.y),
            );
        }
        buffer.set_text(
            &mut state.renderer.text.font_system,
            &self.text,
            &self.attributes.as_attrs(),
            Shaping::Basic,
        );

        let mut max_x = 0.0f32;
        let mut max_y = 0.0f32;
        for run in buffer.layout_runs() {
            if let Some(last) = run.glyphs.last() {
                max_x = max_x.max(last.x + last.w);
            }
            max_y = max_y.max(run.line_top + run.line_height);
        }
        let intrinsic_size = Vec2::new(
            max_x.max(1.0),
            max_y.max(self.metrics.line_height),
        );
        let size = if self.size == Vec2::ZERO {
            intrinsic_size
        } else {
            self.size
        };

        let top_left = self.position + state.layout_offset();
        let rect = Rect::new(top_left, size);

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

        state.renderer.text.entries.push(TextEntry {
            buffer,
            position: top_left,
        });
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