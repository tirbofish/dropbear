use crate::editor::page::EditorTabVisibility;
use crate::editor::{EditorTabDock, EditorTabDockDescriptor, EditorTabViewer};
use egui::Ui;

pub struct UIInspector {}

impl EditorTabDock for UIInspector {
    fn desc() -> EditorTabDockDescriptor {
        EditorTabDockDescriptor {
            id: "Widget Inspector",
            title: "Widget Inspector".to_string(),
            visibility: EditorTabVisibility::UIEditor,
        }
    }

    fn display(_viewer: &mut EditorTabViewer<'_>, ui: &mut Ui) {
        ui.label("Not implemented yet.");
    }
}
