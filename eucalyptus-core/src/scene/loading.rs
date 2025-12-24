//! Types and functions that deal with loading scenes for scripting modules through the SceneManager API.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use hecs::{Entity, World};
use parking_lot::Mutex;

use dropbear_engine::future::FutureHandle;
use dropbear_engine::graphics::SharedGraphicsContext;
use dropbear_traits::registry::ComponentRegistry;

use crate::states::{WorldLoadingStatus, SCENES};
use crate::utils::Progress;

pub static SCENE_LOADER: LazyLock<Mutex<SceneLoader>> = LazyLock::new(|| Mutex::new(SceneLoader::new()));

pub struct SceneLoader {
    scenes_to_load: HashMap<u64, SceneLoadEntry>,
    pub next_id: u64,
}

pub struct LoadedScene {
    pub scene_name: String,
    pub world: World,
    pub active_camera: Entity,
}

struct SceneLoadEntry {
    scene_name: String,
    result: SceneLoadResult,
    progress: Progress,
    status: crossbeam_channel::Receiver<WorldLoadingStatus>,
    loaded: crossbeam_channel::Receiver<anyhow::Result<(World, Entity)>>,
    loaded_scene: Option<(World, Entity)>,
    thread_handle: FutureHandle,
}

impl SceneLoader {
    pub fn new() -> Self {
        Self {
            scenes_to_load: HashMap::new(),
            next_id: 0,
        }
    }
}

/// The result of loading a scene asynchronously.
#[derive(Clone, Debug)]
pub enum SceneLoadResult {
    /// The scene is currently in the process of loading
    Pending,
    /// The scene has successfully loaded
    Success,
    /// The scene has failed and provided an error message.
    Error(String),
}

/// A handle that references the progress of a scene in the form of a handle.
#[derive(Clone, Debug)]
pub struct SceneLoadHandle {
    /// The unique number that identifies the scene load.
    pub id: u64,
    /// The name of the planned scene.
    pub scene_name: String,
}