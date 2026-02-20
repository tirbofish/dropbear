// In widgets/layout.rs

use crate::widgets::{Anchor, ContaineredWidget};
use crate::{ContaineredWidgetType, KinoState, UiInstructionType, UiNode, WidgetId};
use glam::{Vec2, vec2};
use std::any::Any;

fn calculate_node_size(node: &UiNode) -> Vec2 {
    match &node.instruction {
        UiInstructionType::Widget(widget) => widget.size(),
        UiInstructionType::Containered(ContaineredWidgetType::Start { widget, .. }) => {
            if let Some(row) = widget.as_any().downcast_ref::<Row>() {
                calculate_row_size(&node.children, row.spacing)
            } else if let Some(column) = widget.as_any().downcast_ref::<Column>() {
                calculate_column_size(&node.children, column.spacing)
            } else {
                Vec2::ZERO
            }
        }
        _ => Vec2::ZERO,
    }
}

fn calculate_row_size(children: &[UiNode], spacing: f32) -> Vec2 {
    if children.is_empty() {
        return Vec2::ZERO;
    }

    let mut total_width = 0.0;
    let mut max_height: f32 = 0.0;

    for (i, child) in children.iter().enumerate() {
        let child_size = calculate_node_size(child);
        total_width += child_size.x;
        max_height = max_height.max(child_size.y);

        if i < children.len() - 1 {
            total_width += spacing;
        }
    }

    vec2(total_width, max_height)
}

fn calculate_column_size(children: &[UiNode], spacing: f32) -> Vec2 {
    if children.is_empty() {
        return Vec2::ZERO;
    }

    let mut total_height = 0.0;
    let mut max_width: f32 = 0.0;

    for (i, child) in children.iter().enumerate() {
        let child_size = calculate_node_size(child);
        total_height += child_size.y;
        max_width = max_width.max(child_size.x);

        if i < children.len() - 1 {
            total_height += spacing;
        }
    }

    vec2(max_width, total_height)
}

pub struct Row {
    pub id: WidgetId,
    pub anchor: Anchor,
    pub position: Vec2,
    pub spacing: f32,
}

impl Row {
    pub fn new(id: impl Into<WidgetId>) -> Self {
        Self {
            id: id.into(),
            anchor: Anchor::TopLeft,
            position: Vec2::ZERO,
            spacing: 8.0,
        }
    }

    pub fn at(mut self, position: impl Into<Vec2>) -> Self {
        self.position = position.into();
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn build(self) -> Box<Self> {
        Box::new(self)
    }
}

impl ContaineredWidget for Row {
    fn render(self: Box<Self>, children: Vec<UiNode>, state: &mut KinoState) {
        let total_size = calculate_row_size(&children, self.spacing);

        let offset = match self.anchor {
            Anchor::TopLeft => Vec2::ZERO,
            Anchor::Center => vec2(-total_size.x / 2.0, 0.0),
        };

        let start_pos = self.position + offset;
        let mut x_offset = 0.0;

        for child in children {
            let child_size = calculate_node_size(&child);

            state.push_container(crate::math::Rect::new(
                start_pos + vec2(x_offset, 0.0),
                vec2(child_size.x, total_size.y),
            ));

            state.render_tree(vec![child]);
            state.pop_container();

            x_offset += child_size.x + self.spacing;
        }
    }

    fn size(&self, children: &[UiNode]) -> Vec2 {
        calculate_row_size(children, self.spacing)
    }

    fn id(&self) -> WidgetId {
        self.id
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct Column {
    pub id: WidgetId,
    pub anchor: Anchor,
    pub position: Vec2,
    pub spacing: f32,
}

impl Column {
    pub fn new(id: impl Into<WidgetId>) -> Self {
        Self {
            id: id.into(),
            anchor: Anchor::TopLeft,
            position: Vec2::ZERO,
            spacing: 8.0,
        }
    }

    pub fn at(mut self, position: impl Into<Vec2>) -> Self {
        self.position = position.into();
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn build(self) -> Box<Self> {
        Box::new(self)
    }
}

impl ContaineredWidget for Column {
    fn render(self: Box<Self>, children: Vec<UiNode>, state: &mut KinoState) {
        let total_size = calculate_column_size(&children, self.spacing);

        let offset = match self.anchor {
            Anchor::TopLeft => Vec2::ZERO,
            Anchor::Center => vec2(0.0, -total_size.y / 2.0),
        };

        let start_pos = self.position + offset;
        let mut y_offset = 0.0;

        for child in children {
            let child_size = calculate_node_size(&child);

            state.push_container(crate::math::Rect::new(
                start_pos + vec2(0.0, y_offset),
                vec2(total_size.x, child_size.y),
            ));

            state.render_tree(vec![child]);
            state.pop_container();

            y_offset += child_size.y + self.spacing;
        }
    }

    fn size(&self, children: &[UiNode]) -> Vec2 {
        calculate_column_size(children, self.spacing)
    }

    fn id(&self) -> WidgetId {
        self.id
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
