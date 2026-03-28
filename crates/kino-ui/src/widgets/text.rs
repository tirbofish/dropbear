use crate::math::Rect;
use crate::rendering::text::TextEntry;
use crate::resp::WidgetResponse;
use crate::widgets::NativeWidget;
#[cfg(feature = "ser")]
use crate::widgets::text::ser::{SerializedAttrs, SerializedMetrics};
use crate::{KinoState, WidgetId};
use glam::Vec2;
use glyphon::{Attrs, AttrsOwned, Buffer, Color, Metrics, Shaping};
use std::any::Any;
use winit::event::{ElementState, MouseButton};

/// Creates a label with the specified text and properties.
///
/// # Input
/// Responses are weird for text, as it recognises the input when you touch the text itself.
///
/// If you want an area, you might be interested in [`crate::rect_container`] (with a transparent colour).
#[derive(Clone)]
#[cfg_attr(any(feature = "ser"), derive(serde::Serialize, serde::Deserialize))]
pub struct Text {
    pub id: WidgetId,
    pub text: String,
    pub position: Vec2,
    pub size: Vec2,

    #[cfg(not(feature = "ser"))]
    pub metrics: Metrics,
    #[cfg(feature = "ser")]
    pub metrics: SerializedMetrics,

    #[cfg(not(feature = "ser"))]
    pub attributes: AttrsOwned,
    #[cfg(feature = "ser")]
    pub attributes: SerializedAttrs,
}

impl Text {
    /// Create a new text builder that will push text to the renderer
    pub fn new(text: impl ToString) -> Self {
        Self {
            id: text.to_string().into(),
            text: text.to_string(),
            position: Vec2::new(10.0, 10.0),
            size: Vec2::ZERO,
            metrics: Metrics::new(16.0, 1.0).into(),
            attributes: AttrsOwned::new(&Attrs::new().color(Color::rgb(0, 0, 0))).into(),
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
        self.attributes = attributes.into();
        self
    }

    pub fn with_metrics(mut self, metrics: Metrics) -> Self {
        self.metrics = metrics.into();
        self
    }
}

impl NativeWidget for Text {
    fn render(self: Box<Self>, state: &mut KinoState) {
        #[cfg(not(feature = "ser"))]
        let (metrics, attributes) = (self.metrics, self.attributes);
        #[cfg(feature = "ser")]
        let (metrics, attributes): (Metrics, AttrsOwned) =
            (self.metrics.into(), self.attributes.into());

        let mut buffer = Buffer::new(&mut state.renderer.text.font_system, metrics);
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
            &attributes.as_attrs(),
            Shaping::Basic,
            None, // todo: figure out what to put in here
        );

        let mut max_x = 0.0f32;
        let mut max_y = 0.0f32;
        for run in buffer.layout_runs() {
            if let Some(last) = run.glyphs.last() {
                max_x = max_x.max(last.x + last.w);
            }
            max_y = max_y.max(run.line_top + run.line_height);
        }
        let intrinsic_size = Vec2::new(max_x.max(1.0), max_y.max(metrics.line_height));
        let size = if self.size == Vec2::ZERO {
            intrinsic_size
        } else {
            self.size
        };

        let top_left = self.position + state.layout_offset();
        let rect = Rect::new(top_left, size);

        let input = state.input();
        let hovering =
            rect.contains(input.mouse_position) && state.clip_contains(input.mouse_position);
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

        state.push_text_entry(TextEntry {
            buffer,
            position: top_left,
            size,
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

#[cfg_attr(any(feature = "ser"), typetag::serde)]
impl crate::WidgetDescriptor for Text {
    fn id(&self) -> crate::WidgetId {
        self.id
    }

    fn label(&self) -> &'static str {
        "Text"
    }

    fn submit(self: Box<Self>, _children: Vec<crate::WidgetNode>, kino: &mut crate::KinoState) {
        kino.add_widget(self);
    }

    fn clone_boxed(&self) -> Box<dyn crate::WidgetDescriptor> {
        Box::new(self.clone())
    }
}

#[cfg(feature = "ser")]
pub mod ser {
    use glyphon::{AttrsOwned, Color, FamilyOwned, Metrics, Stretch, Style, Weight};

    #[derive(Clone, serde::Serialize, serde::Deserialize)]
    pub struct SerializedMetrics {
        pub font_size: f32,
        pub line_height: f32,
    }

    impl From<Metrics> for SerializedMetrics {
        fn from(value: Metrics) -> Self {
            Self {
                font_size: value.font_size,
                line_height: value.line_height,
            }
        }
    }

    impl From<SerializedMetrics> for Metrics {
        fn from(value: SerializedMetrics) -> Self {
            Metrics::new(value.font_size, value.line_height)
        }
    }

    #[derive(Clone, serde::Serialize, serde::Deserialize)]
    pub struct SerializedAttrs {
        pub color: Option<u32>,
        pub family: SerializedFamily,
        pub weight: u16,
        pub style: u8,
        pub stretch: u8,
    }

    #[derive(Clone, serde::Serialize, serde::Deserialize)]
    pub enum SerializedFamily {
        Name(String),
        Serif,
        SansSerif,
        Cursive,
        Fantasy,
        Monospace,
    }

    impl From<AttrsOwned> for SerializedAttrs {
        fn from(a: AttrsOwned) -> Self {
            Self {
                color: a.color_opt.map(|c| c.0),
                family: match a.family_owned {
                    FamilyOwned::Name(s) => SerializedFamily::Name(s.to_string()),
                    FamilyOwned::Serif => SerializedFamily::Serif,
                    FamilyOwned::SansSerif => SerializedFamily::SansSerif,
                    FamilyOwned::Cursive => SerializedFamily::Cursive,
                    FamilyOwned::Fantasy => SerializedFamily::Fantasy,
                    FamilyOwned::Monospace => SerializedFamily::Monospace,
                },
                weight: a.weight.0,
                style: match a.style {
                    Style::Normal => 0,
                    Style::Italic => 1,
                    Style::Oblique => 2,
                },
                stretch: a.stretch.to_number() as u8,
            }
        }
    }

    impl From<SerializedAttrs> for AttrsOwned {
        fn from(s: SerializedAttrs) -> Self {
            use glyphon::Attrs;
            let family_owned = match s.family {
                SerializedFamily::Name(n) => FamilyOwned::Name(n.into()),
                SerializedFamily::Serif => FamilyOwned::Serif,
                SerializedFamily::SansSerif => FamilyOwned::SansSerif,
                SerializedFamily::Cursive => FamilyOwned::Cursive,
                SerializedFamily::Fantasy => FamilyOwned::Fantasy,
                SerializedFamily::Monospace => FamilyOwned::Monospace,
            };
            let style = match s.style {
                1 => Style::Italic,
                2 => Style::Oblique,
                _ => Style::Normal,
            };
            let stretch = match s.stretch {
                1 => Stretch::UltraCondensed,
                2 => Stretch::ExtraCondensed,
                3 => Stretch::Condensed,
                4 => Stretch::SemiCondensed,
                6 => Stretch::SemiExpanded,
                7 => Stretch::Expanded,
                8 => Stretch::ExtraExpanded,
                9 => Stretch::UltraExpanded,
                _ => Stretch::Normal,
            };
            AttrsOwned::new(
                &Attrs::new()
                    .family(family_owned.as_family())
                    .weight(Weight(s.weight))
                    .style(style)
                    .stretch(stretch)
                    .color(Color(s.color.unwrap_or(0))),
            )
        }
    }
}
