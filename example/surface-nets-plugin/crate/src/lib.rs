mod component;
mod exports;
pub use component::SurfaceNets;

use eucalyptus_core::plugin::{ExternalPlugin, PluginManifest, PluginRegistry};

pub mod surface_nets;

pub struct SurfaceNetsPlugin;

#[unsafe(no_mangle)]
extern "C" fn eucalyptus_plugin_init(plugin_registry: *mut PluginRegistry) {
    if plugin_registry.is_null() {
        println!("Plugin registry is null");
        return;
    }

    let plugin_registry = unsafe { &mut *plugin_registry };

    let token = plugin_registry.register_plugin::<SurfaceNetsPlugin>();
    plugin_registry.register_component::<SurfaceNets>(token);
}

impl ExternalPlugin for SurfaceNetsPlugin {
    fn plugin_manifest() -> PluginManifest {
        PluginManifest {
            display_name: "Surface Nets".to_string(),
            description: "Isosurface mesh generation from signed distance fields using the Surface Nets algorithm.".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            authors: vec!["tirbofish".to_string()],
            dependencies: vec![],
        }
    }
}
