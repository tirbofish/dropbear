//! Types and functions that deal with loading scenes for scripting modules through the SceneManager API.

use std::collections::HashMap;
use std::sync::LazyLock;

use hecs::{Entity, World};
use parking_lot::Mutex;

use dropbear_engine::future::FutureHandle;

use crate::states::WorldLoadingStatus;
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

pub struct SceneLoadEntry {
    pub scene_name: String,
    pub result: SceneLoadResult,
    pub progress: Progress,
    pub status: Option<crossbeam_channel::Receiver<WorldLoadingStatus>>,
    pub loaded: Option<crossbeam_channel::Receiver<anyhow::Result<(World, Entity)>>>,
    pub loaded_scene: Option<(World, Entity)>,
    pub thread_handle: Option<FutureHandle>,
}

impl SceneLoader {
    pub fn new() -> Self {
        Self {
            scenes_to_load: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn register_load(&mut self, scene_name: String) -> u64 {
        for (id, entry) in &self.scenes_to_load {
            if entry.scene_name == scene_name {
                if let SceneLoadResult::Pending = entry.result {
                    return *id;
                }
            }
        }

        self.next_id += 1;
        let id = self.next_id;
        self.scenes_to_load.insert(id, SceneLoadEntry {
            scene_name,
            result: SceneLoadResult::Pending,
            progress: Progress {
                current: 0,
                total: 1,
                message: "Idle".to_string(),
            },
            status: None,
            loaded: None,
            loaded_scene: None,
            thread_handle: None,
        });
        id
    }

    pub fn find_pending_id_by_name(&self, scene_name: &str) -> Option<u64> {
        self.scenes_to_load.iter().find_map(|(id, entry)| {
            if entry.scene_name == scene_name {
                if let SceneLoadResult::Pending = entry.result {
                    return Some(*id);
                }
            }
            None
        })
    }

    pub fn get_entry(&self, id: u64) -> Option<&SceneLoadEntry> {
        self.scenes_to_load.get(&id)
    }

    pub fn get_entry_mut(&mut self, id: u64) -> Option<&mut SceneLoadEntry> {
        self.scenes_to_load.get_mut(&id)
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