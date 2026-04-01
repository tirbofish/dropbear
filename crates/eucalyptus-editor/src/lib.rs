pub mod about;
pub mod build;
pub mod camera;
pub mod debug;
pub mod editor;
pub mod menu;
pub mod plugin;
pub mod process;
pub mod signal;
pub mod spawn;
pub mod stats;
pub mod utils;
// pub mod outline;

use editor::docks::asset_viewer::AssetViewerDock;
use editor::docks::build_console::BuildConsoleDock;
use editor::docks::entity_list::EntityListDock;
use editor::docks::resource::ResourceInspectorDock;
use editor::docks::viewport::ViewportDock;
pub use redback_runtime as runtime;

use crate::editor::ui::inspector::UIInspector;
use crate::editor::ui::widget_tree::UIWidgetTree;
use crate::editor::{EditorTabRegistry, dock::ConsoleDock, ui::viewport::UICanvas};

dropbear_engine::features! {
    pub mod features {
        const ShowComponentTypeIDInEditor = 0b00000001
    }
}

pub fn register_docks(registry: &mut EditorTabRegistry) {
    registry.register::<ViewportDock>();
    registry.register::<EntityListDock>();
    registry.register::<AssetViewerDock>();
    registry.register::<ResourceInspectorDock>();
    registry.register::<BuildConsoleDock>();
    registry.register::<ConsoleDock>();
    registry.register::<UICanvas>();
    registry.register::<UIInspector>();
    registry.register::<UIWidgetTree>();
}
