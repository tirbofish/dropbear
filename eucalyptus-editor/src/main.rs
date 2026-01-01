// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// note to self: when it becomes release, remember to re-add this back

use anyhow::{bail, Context};
use clap::{Arg, Command};
use dropbear_engine::future::FutureQueue;
use eucalyptus_editor::{build, editor, menu};
use parking_lot::RwLock;
use std::sync::Arc;
use std::{fs, path::{Path, PathBuf}, rc::Rc};
use winit::window::WindowAttributes;
use dropbear_engine::DropbearWindowBuilder;
use eucalyptus_core::config::ProjectConfig;
use eucalyptus_core::scripting::jni::{RuntimeMode, RUNTIME_MODE};
use eucalyptus_core::scripting::{AWAIT_JDB, JVM_ARGS};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(not(target_os = "android"))]
    {
        use colored::Colorize;
        use env_logger::Builder;
        use log::LevelFilter;
        use std::fs::OpenOptions;

        let log_dir =
            app_dirs2::app_root(app_dirs2::AppDataType::UserData, &eucalyptus_core::APP_INFO)
                .expect("Failed to get app data directory")
                .join("logs");
        fs::create_dir_all(&log_dir).expect("Failed to create log dir");

        let datetime_str = chrono::offset::Local::now().format("%Y-%m-%d_%H-%M-%S");
        let log_filename = format!("{}.{}.log", "eucalyptus-editor", datetime_str);
        let log_path = log_dir.join(log_filename);

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .expect("Failed to open log file");
        let file = parking_lot::Mutex::new(file);

        let app_target = "eucalyptus-editor".replace('-', "_");
        let log_config = format!("dropbear_engine=trace,{}=debug,warn", app_target);
        unsafe { std::env::set_var("RUST_LOG", log_config) };

        Builder::new()
            .format(move |buf, record| {
                use std::io::Write;

                let ts = chrono::offset::Local::now().format("%Y-%m-%dT%H:%M:%S");

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
                Some("eucalyptus-editor".replace('-', "_").as_str()),
                LevelFilter::Debug,
            )
            .filter(Some("eucalyptus_core"), LevelFilter::Debug)
            .filter(Some("dropbear_traits"), LevelFilter::Debug)
            .init();
        log::info!("Initialised logger");
    }

    dropbear_engine::panic::set_hook();
    let matches = Command::new("eucalyptus-editor")
        .about("A visual game editor")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand_required(false)
        .arg_required_else_help(false)
        .arg(
            Arg::new("jvm-args")
                .long("jvm-args")
                .help("Additional JVM arguments to pass to the Java runtime")
                .value_name("ARGS")
                .global(true)
                .required(false),
        )
        .arg(
            Arg::new("await-jdb")
                .long("await-jdb")
                .help("Waits for you to enable the java debugger (either through IntelliJ or JDB cli). This assumes no custom arguments exist. It is either a \"true\" or \"false\" value. ")
                .value_name("AWAIT_JDB")
                .global(true)
                .required(false)
        )
        .subcommand(
            Command::new("build")
                .about("Build a eucalyptus project, but only the .eupak file and its resources")
                .arg(
                    Arg::new("project")
                        .help("Path to the .eucp project file")
                        .value_name("PROJECT_FILE")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("package")
                .about("Package a eucalyptus project into a runnable bundle")
                .arg(
                    Arg::new("project")
                        .help("Path to the project directory or .eucp file")
                        .value_name("PROJECT_PATH")
                        .required(true),
                )
                .arg(
                    Arg::new("debug")
                        .long("debug")
                        .help("Use debug build of the native library (default: release)")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("read").about("Reads a .eupak file").arg(
                Arg::new("eupak_file")
                    .help("Path to the .eupak data file")
                    .value_name("EUPAK_FILE")
                    .required(true),
            ),
        )
        .subcommand(
            Command::new("play")
                .about("Starts a debuggable play mode session of the specified project")
                .arg(
                    Arg::new("project")
                        .help("Path to the project directory or .eucp file")
                        .value_name("PROJECT_PATH")
                        .required(true),
                )
                .arg(
                    Arg::new("initial_scene")
                        .help("Sets the first scene to load. Default is the initial scene set by the project")
                        .value_name("INITIAL_SCENE")
                        .required(false),
                ),
        )
        .get_matches();

    let jvm_args = matches.get_one::<String>("jvm-args");
    let await_jdb = matches.get_one::<String>("await-jdb");

    if let Some(args) = jvm_args {
        let _ = JVM_ARGS.set(args.clone());
    }

    if let Some(args) = await_jdb {
        match args.as_str() {
            "true" => {let _ = AWAIT_JDB.set(true);}
            "false" => {let _ = AWAIT_JDB.set(false);}
            _ => {log::warn!("\"await-jdb\" args were recognised as {}, however it is not \"true\" or \"false\", therefore it is not set. ", args)}
        }
    }

    match matches.subcommand() {
        Some(("build", sub_matches)) => {
            let path = resolve_project_argument(sub_matches.get_one::<String>("project"))?;
            log::info!("Building project at {:?}", path);
            build::build(path)?;
        }
        Some(("package", sub_matches)) => {
            let path = resolve_project_argument(sub_matches.get_one::<String>("project"))?;
            let use_debug = sub_matches.get_flag("debug");
            log::info!("Packaging project at {:?} (debug: {})", path, use_debug);
            build::package(path, None, use_debug).await?;
        }
        Some(("read", sub_matches)) => {
            let eupak = match sub_matches.get_one::<String>("eupak_file") {
                Some(path) => PathBuf::from(path),
                None => {
                    log::error!("Eupak file returned none");
                    std::process::exit(1)
                }
            };

            build::read(eupak)?;
        },
        Some(("play", sub_matches)) => {
            let _ = RUNTIME_MODE.set(RuntimeMode::PlayMode);

            let mut path = resolve_project_argument(sub_matches.get_one::<String>("project"))?;
            let initial_scene = sub_matches.get_one::<String>("initial_scene").and_then(|s| Some(s.clone()));

            if path.is_dir() {
                path = find_eucp_in_dir(path.as_path())?;
            }

            let config = ProjectConfig::read_from(path.clone())?;

            {
                let mut project = eucalyptus_core::states::PROJECT.write();
                *project = config.clone();
            }

            let scene_to_load = initial_scene
                .as_ref()
                .or(config.runtime_settings.initial_scene.as_ref())
                .ok_or_else(|| anyhow::anyhow!("No initial scene specified and no default scene in project config"))?;

            eucalyptus_core::states::load_scene_into_memory(scene_to_load)?;
            log::info!("Loaded initial scene '{}' for play mode", scene_to_load);

            let future_queue = Arc::new(FutureQueue::new());

            let play_mode =
                Rc::new(RwLock::new(eucalyptus_editor::runtime::PlayMode::new(initial_scene).unwrap_or_else(|e| {
                    panic!("Unable to initialise eucalyptus play mode session: {}", e)
                })));

            let window = DropbearWindowBuilder::new()
                .with_attributes(
                    WindowAttributes::default().with_title(config.project_name.clone())
                )
                .add_scene_with_input(play_mode, "play_mode")
                .set_initial_scene("play_mode")
                .build();

            dropbear_engine::DropbearAppBuilder::new()
                .with_future_queue(future_queue)
                .add_window(window)
                .run().await?;
        },
        None => {
            let _ = RUNTIME_MODE.set(RuntimeMode::Editor);

            let future_queue = Arc::new(FutureQueue::new());

            let main_menu = Rc::new(RwLock::new(menu::MainMenu::new()));
            let editor =
                Rc::new(RwLock::new(editor::Editor::new().unwrap_or_else(|e| {
                    panic!("Unable to initialise Eucalyptus Editor: {}", e)
                })));

            let window = DropbearWindowBuilder::new()
                .with_attributes(
                    WindowAttributes::default().with_title(
                        format!(
                            "Eucalyptus, built with dropbear | Version {} on commit {}",
                            env!("CARGO_PKG_VERSION"),
                            env!("GIT_HASH")
                        )
                    )
                        .with_maximized(true)
                )
                .add_scene_with_input(editor, "editor")
                .add_scene_with_input(main_menu, "main_menu")
                .set_initial_scene("main_menu")
                .build();

            dropbear_engine::DropbearAppBuilder::new()
                .with_future_queue(future_queue)
                .add_window(window)
                .run().await?;
        }
        _ => unreachable!(),
    }
    Ok(())
}

fn resolve_project_argument(arg: Option<&String>) -> anyhow::Result<PathBuf> {
    match arg {
        Some(path) => {
            let provided = PathBuf::from(path);
            if provided.is_dir() {
                find_eucp_in_dir(&provided)
            } else if provided.exists() {
                Ok(provided)
            } else {
                bail!("Provided project path does not exist: {}", provided.display());
            }
        }
        None => find_eucp_file(),
    }
}

fn find_eucp_in_dir(dir: &Path) -> anyhow::Result<PathBuf> {
    if !dir.exists() {
        bail!("Directory does not exist: {}", dir.display());
    }

    let mut matches = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("Unable to read {}", dir.display()))? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        if entry
            .path()
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("eucp"))
            .unwrap_or(false)
        {
            matches.push(entry.path());
        }
    }

    match matches.len() {
        0 => bail!("No .eucp file found in {}", dir.display()),
        1 => Ok(matches.remove(0)),
        _ => bail!(
            "Multiple .eucp files found in {}. Please specify one explicitly.",
            dir.display()
        ),
    }
}

fn find_eucp_file() -> anyhow::Result<PathBuf> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;
    let entries =
        fs::read_dir(&current_dir).context("Failed to read current directory for .eucp files")?;

    let mut eucp_files = Vec::new();

    for entry in entries {
        if let Ok(entry) = entry
            && let Some(file_name) = entry.file_name().to_str()
            && file_name.ends_with(".eucp")
        {
            eucp_files.push(entry.path());
        }
    }

    match eucp_files.len() {
        0 => bail!("No .eucp files found in current directory"),
        1 => Ok(eucp_files[0].clone()),
        _ => bail!(
            "Multiple .eucp files found: {:#?}. Please specify which one to use.",
            eucp_files
        ),
    }
}
