//! Types and functions that deal with loading scenes for scripting modules through the SceneManager API.  
use crate::utils::Progress;

/// The result of loading a scene asynchronously.
pub enum SceneLoadResult {
    /// The scene is currently in the process of loading
    Pending,
    /// The scene has successfully loaded
    Success,
    /// The scene has failed and provided an error message. 
    Error(anyhow::Error),
}

/// A handle that references the progress of a scene in the form of a handle. 
pub struct SceneLoadHandle {
    /// The unique number that identifies the scene load. 
    id: u64,
    /// The name of the planned scene. 
    scene_name: String,
    /// The result of the scene. 
    result: SceneLoadResult,
    /// The progress of the scene. 
    progress: Progress,
}