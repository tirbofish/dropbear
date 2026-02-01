use crate::{KinoState, WidgetId};
use crate::widgets::rect::Rectangle;
use crate::widgets::text::Text;

/// Shorthand for a standard rectangle widget.
pub fn rect<F>(kino: &mut KinoState, id: impl Into<WidgetId>, configure: F) -> WidgetId
where
    F: FnOnce(&mut Rectangle),
{
    let id = id.into();
    let mut rect = Rectangle::new(id);
    configure(&mut rect);
    kino.add_widget(Box::new(rect));
    id
}

/// Shorthand for a rectangle container.
///
/// `configure` sets up the rectangle, and `contents` emits child widgets between
/// start/end instructions.
pub fn rect_container<C>(
    kino: &mut KinoState,
    rect: Rectangle,
    contents: C,
) -> WidgetId
where
    C: FnOnce(&mut KinoState),
{
    let id = rect.id;

    kino.add_container(Box::new(rect));
    contents(kino);
    kino.end_container(id);
    id
}

/// Shorthand for a standard label. 
pub fn label<F>(kino: &mut KinoState, text: impl ToString, configure: F) -> WidgetId
where
    F: FnOnce(&mut Text),
{
    let mut text = Text::new(text);
    configure(&mut text);
    kino.add_widget(Box::new(text))
}