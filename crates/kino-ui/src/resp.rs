use crate::{WidgetId};

#[derive(Clone, Copy, Debug, Default)]
pub struct WidgetResponse {
    pub queried: WidgetId,
    pub clicked: bool,
    pub hovering: bool,
}