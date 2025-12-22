mod input;
mod scene;
mod debug;
mod utils;
mod command;

use crate::scene::RuntimeScene;
use app_dirs2::AppInfo;
use dropbear_engine::future::FutureQueue;
use eucalyptus_core::runtime::RuntimeProjectConfig;
use parking_lot::RwLock;
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
    #[cfg(not(target_os = "android"))]
    {
        use chrono::offset::Local;
        use colored::Colorize;
        use env_logger::Builder;
        use log::LevelFilter;
        use parking_lot::Mutex;
        use std::fs::OpenOptions;

        let log_dir =
            app_dirs2::app_root(app_dirs2::AppDataType::UserData, &eucalyptus_core::APP_INFO)
                .expect("Failed to get app data directory")
                .join("logs");
        fs::create_dir_all(&log_dir).expect("Failed to create log dir");

        let datetime_str = Local::now().format("%Y-%m-%d_%H-%M-%S");
        let log_filename = format!("{}.{}.log", "redback-runtime", datetime_str);
        let log_path = log_dir.join(log_filename);

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .expect("Failed to open log file");
        let file = Mutex::new(file);

        Builder::new()
            .format(move |buf, record| {
                use std::io::Write;

                let ts = Local::now().format("%Y-%m-%dT%H:%M:%S");

                let colored_level = match record.level() {
                    log::Level::Error => record.level().to_string().red().bold(),
                    log::Level::Warn => record.level().to_string().yellow().bold(),
                    log::Level::Info => record.level().to_string().green().bold(),
                    log::Level::Debug => record.level().to_string().blue().bold(),
                    log::Level::Trace => record.level().to_string().cyan().bold(),
                };

                let colored_timestamp = ts.to_string().bright_black();

                let file_info = format!(
                    "{}:{}",
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0)
                )
                .bright_black();

                let console_line = format!(
                    "{} {} [{}] - {}\n",
                    file_info,
                    colored_timestamp,
                    colored_level,
                    record.args()
                );

                let file_line = format!(
                    "{}:{} {} [{}] - {}\n",
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0),
                    ts,
                    record.level(),
                    record.args()
                );

                write!(buf, "{}", console_line).unwrap();

                let mut fh = file.lock();
                let _ = fh.write_all(file_line.as_bytes());

                Ok(())
            })
            .filter(Some("dropbear_engine"), LevelFilter::Trace)
            .filter(
                Some("redback-runtime".replace('-', "_").as_str()),
                LevelFilter::Debug,
            )
            .filter(Some("eucalyptus_core"), LevelFilter::Debug)
            .filter(Some("dropbear_traits"), LevelFilter::Debug)
            .init();
        log::info!("Initialised logger");
    }

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

    let (scene_config, _): (RuntimeProjectConfig, usize) =
        bincode::decode_from_slice(scene_config.as_slice(), bincode::config::standard()).unwrap();
    log::debug!("Converted scene config file to RuntimeProjectConfig");

    let runtime_scene = Rc::new(RwLock::new(RuntimeScene::new(scene_config.clone(), config.clone()).unwrap()));
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

#[derive(Debug, Clone, Deserialize, Serialize, bincode::Encode, bincode::Decode)]
pub struct ConfigFile {
    pub jvm_args: Option<String>,
    pub max_fps: u32,
    pub target_resolution: WindowModes
}

#[derive(Debug, Clone, Deserialize, Serialize, bincode::Encode, bincode::Decode)]
pub enum WindowModes {
    Windowed(u32, u32),
    Maximised,
    Fullscreen,
}
