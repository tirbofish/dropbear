use app_dirs2::AppInfo;
use dropbear_engine::future::FutureQueue;
use eucalyptus_core::runtime::RuntimeProjectConfig;
use parking_lot::RwLock;
use redback_runtime::PlayMode;
use std::env::current_exe;
use std::fs;
use std::rc::Rc;
use std::sync::Arc;
use ron::ser::PrettyConfig;
use winit::window::WindowAttributes;
use dropbear_engine::{DropbearAppBuilder, DropbearWindowBuilder};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    env_logger::init();

    dropbear_engine::panic::set_hook();
    log::debug!("Set panic hook");

    let window_config_file = current_exe().unwrap()
        .parent()
        .ok_or(anyhow::anyhow!(
            "Unable to get parent of current executable"
        )).unwrap()
        .join("config.eucfg");
    log::debug!("Fetched window config file path: {}", window_config_file.display());

    log::debug!("Reading from window config file");
    let value = fs::read(&window_config_file);

    let config = match value {
        Ok(val) => {
            log::debug!("Config file exists, reading contents");
            let config = ron::de::from_bytes::<ConfigFile>(val.as_slice()).unwrap();
            log::debug!("File converted to ConfigFile");
            config
        }
        Err(e) => {
            log::warn!("Unable to read config: {}", e);
            log::warn!("Creating new config file to overwrite old one");
            let cfg = ConfigFile {
                jvm_args: None,
                max_fps: 60,
                target_resolution: WindowModes::Windowed(1920, 1080),
            };
            let vec = ron::ser::to_string_pretty(&cfg, PrettyConfig::default()).unwrap();
            if let Err(e) = fs::write(&window_config_file, vec) {
                log::warn!("Unable to write, still running game: {}", e);
            }
            cfg
        }
    };

    let path = current_exe().unwrap()
        .parent()
        .ok_or(anyhow::anyhow!(
                "Unable to locate parent folder for current executable"
            )).unwrap()
        .join("data.eupak");
    log::debug!("scene config (potential) file path: {}", path.display());

    let scene_config = fs::read(&path).unwrap();
    log::debug!("Located scene config file: [{}] ({} bytes)", path.display(), scene_config.len());

    let scene_config: RuntimeProjectConfig = postcard::from_bytes(&scene_config).unwrap();

    log::debug!("Converted scene config file to RuntimeProjectConfig");

    let runtime_scene = Rc::new(RwLock::new(PlayMode::new(None).unwrap()));
    let future_queue = Arc::new(FutureQueue::new());

    let authors = scene_config.authors.developer.clone();
    let project_name = scene_config.project_name.clone();

    let name = Box::leak(project_name.into_boxed_str());
    let author = Box::leak(authors.into_boxed_str());

    log::debug!("Loading {} by {}", name, author);

    let attributes = WindowAttributes::default();

    match config.target_resolution {
        WindowModes::Windowed(_, _) => {}
        WindowModes::Maximised => {}
        WindowModes::Fullscreen => {}
    }

    let window = DropbearWindowBuilder::new()
        .with_attributes(attributes)
        .add_scene_with_input(runtime_scene, "runtime_scene")
        .set_initial_scene("runtime_scene")
        .build();

    log::debug!("Running dropbear app");

    DropbearAppBuilder::new()
        .add_window(window)
        .max_fps(config.max_fps)
        .app_data(AppInfo {
            name,
            author,
        })
        .with_future_queue(future_queue)
        .run().await.unwrap();
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConfigFile {
    pub jvm_args: Option<String>,
    pub max_fps: u32,
    pub target_resolution: WindowModes
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum WindowModes {
    Windowed(u32, u32),
    Maximised,
    Fullscreen,
}