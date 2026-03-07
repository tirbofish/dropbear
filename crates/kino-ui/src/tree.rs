use crate::{KinoState, WidgetId};

#[cfg_attr(any(feature = "ser"), typetag::serde(tag = "type"))]
pub trait WidgetDescriptor: Send + Sync + 'static {
    fn id(&self) -> WidgetId;
    fn is_container(&self) -> bool {
        false
    }
    fn label(&self) -> &'static str;
    fn submit(self: Box<Self>, children: Vec<WidgetNode>, kino: &mut KinoState);
    fn clone_boxed(&self) -> Box<dyn WidgetDescriptor>;
}

impl Clone for Box<dyn WidgetDescriptor> {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}

#[cfg_attr(any(feature = "ser"), derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone)]
pub struct WidgetNode {
    pub id: WidgetId,
    pub widget: Box<dyn WidgetDescriptor>,
    pub children: Vec<WidgetNode>,
}

impl WidgetNode {
    pub fn new(widget: impl WidgetDescriptor) -> Self {
        let id = widget.id();
        Self {
            id,
            widget: Box::new(widget),
            children: vec![],
        }
    }

    pub fn with_child(mut self, child: WidgetNode) -> Self {
        self.children.push(child);
        self
    }

    pub fn with_children(mut self, children: impl IntoIterator<Item = WidgetNode>) -> Self {
        self.children.extend(children);
        self
    }
}

#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone)]
pub struct WidgetTree {
    pub roots: Vec<WidgetNode>,
}

impl Default for WidgetTree {
    fn default() -> Self {
        Self { roots: vec![] }
    }
}

impl WidgetTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, node: WidgetNode) {
        self.roots.push(node);
    }

    pub fn submit(self, kino: &mut KinoState) {
        for node in self.roots {
            submit_node(node, kino);
        }
    }
}

pub(crate) fn submit_node(node: WidgetNode, kino: &mut KinoState) {
    node.widget.submit(node.children, kino);
}