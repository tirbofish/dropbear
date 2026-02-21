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
pub use redback_runtime as runtime;

use crate::editor::{
    EditorTabRegistry, asset_viewer::AssetViewerDock, build_console::BuildConsoleDock,
    dock::ConsoleDock, entity_list::EntityListDock, resource::ResourceInspectorDock,
    viewport::ViewportDock,
};

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
}
