mod input;
mod scene;

use crate::scene::RuntimeScene;
use app_dirs2::AppInfo;
use dropbear_engine::future::FutureQueue;
use dropbear_engine::{MutableWindowConfiguration, WindowConfiguration, WindowedModes};
use eucalyptus_core::runtime::RuntimeProjectConfig;
use eucalyptus_core::states::ConfigFile;
use parking_lot::RwLock;
use std::env::current_exe;
use std::fs;
use std::rc::Rc;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

                write!(buf, "{}", console_line)?;

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

    let window_config_file = current_exe()?
        .parent()
        .ok_or(anyhow::anyhow!(
            "Unable to get parent of current executable"
        ))?
        .join("config.eucfg");

    let value = fs::read(&window_config_file);

    let config = match value {
        Ok(val) => {
            let (config, _): (ConfigFile, usize) =
                bincode::decode_from_slice(val.as_slice(), bincode::config::standard())?;
            config
        }
        Err(e) => {
            log::warn!("Unable to read config: {}", e);
            log::warn!("Creating new config file to overwrite old one");
            let window_configuration = MutableWindowConfiguration {
                max_fps: u32::MAX,
                windowed_mode: WindowedModes::Maximised,
            };
            let cfg = ConfigFile {
                jvm_args: None,
                window_configuration,
            };
            let vec = bincode::encode_to_vec(cfg.clone(), bincode::config::standard())?;
            if let Err(e) = fs::write(&window_config_file, vec) {
                log::warn!("Unable to write, still running game: {}", e);
            }
            cfg
        }
    };

    let scene_config = fs::read(
        current_exe()?
            .parent()
            .ok_or(anyhow::anyhow!(
                "Unable to locate parent folder for current executable"
            ))?
            .join("data.eupak"),
    )?;
    let (scene_config, _): (RuntimeProjectConfig, usize) =
        bincode::decode_from_slice(scene_config.as_slice(), bincode::config::standard())?;

    let runtime_scene = Rc::new(RwLock::new(RuntimeScene::new(scene_config.clone())?));
    let future_queue = Arc::new(FutureQueue::new());

    let name = current_exe()?
        .file_stem()
        .ok_or(anyhow::anyhow!("Unable to locate file name of current exe"))?
        .to_str()
        .ok_or(anyhow::anyhow!("Unable to convert file name to string"))?
        .to_string();

    let authors = scene_config.authors.developer.clone();
    let project_name = scene_config.project_name.clone();

    let win_cfg = WindowConfiguration {
        title: name,
        window_config: config.window_configuration,
        app_info: AppInfo {
            name: Box::leak(project_name.into_boxed_str()),
            author: Box::leak(authors.into_boxed_str()),
        },
    };

    dropbear_engine::run_app!(
        win_cfg,
        Some(future_queue),
        |mut scene_mgr, mut input_mgr| {
            dropbear_engine::scene::add_scene_with_input(
                &mut scene_mgr,
                &mut input_mgr,
                runtime_scene,
                "runtime_scene",
            );

            scene_mgr.switch("runtime_scene");

            (scene_mgr, input_mgr)
        }
    )
    .await?;

    Ok(())
}
