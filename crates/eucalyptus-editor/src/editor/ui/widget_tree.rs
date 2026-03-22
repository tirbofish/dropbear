use egui::Ui;
use crate::editor::{EditorTabDock, EditorTabDockDescriptor, EditorTabViewer};
use crate::editor::page::EditorTabVisibility;

pub struct UIWidgetTree {

}

impl EditorTabDock for UIWidgetTree {
    fn desc() -> EditorTabDockDescriptor {
        EditorTabDockDescriptor {
            id: "Widget Tree",
            title: "Widget Tree".to_string(),
            visibility: EditorTabVisibility::UIEditor,
        }
    }

    fn display(_viewer: &mut EditorTabViewer<'_>, ui: &mut Ui) {
        ui.label("Not implemented yet.");
    }
}