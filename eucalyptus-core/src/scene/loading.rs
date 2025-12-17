use crate::utils::Progress;

/// The result of loading a scene asynchronously.
pub enum SceneLoadResult {
    Pending,
    Success,
    Error(anyhow::Error),
}

pub struct SceneLoadHandle {
    id: u64,
    scene_name: String,
    result: SceneLoadResult,
    progress: Progress,
}