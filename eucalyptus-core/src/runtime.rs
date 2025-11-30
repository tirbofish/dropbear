use crate::scene::SceneConfig;
use crate::states::{PROJECT, SCENES};
use anyhow::Context;
use semver::Version;

/// The settings of a project in its runtime.
///
/// This is different to [`SceneSettings`], which contains settings for ONLY
/// that specific scene. This is for any configurations of the project during its runtime,
/// such as initial scene and stuff like that.
#[derive(bincode::Decode, bincode::Encode, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct RuntimeSettings {
    /// The first scene that shows up when redback-runtime is ran. 
    /// 
    /// The first scene is not set is expected to be the first scene out of the
    /// projects scene list, or just a normal anyhow error. 
    pub initial_scene: Option<String>,
}

impl RuntimeSettings {
    /// Creates a new [`RuntimeSettings`] config.
    pub fn new() -> Self {
        Self {
            initial_scene: None,
        }
    }
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self::new()
    }
}

/// A struct that represents the authors and creators of the eucalyptus project.
#[derive(bincode::Decode, bincode::Encode, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Authoring {
    /// The team behind the game
    pub developer: String,
}

impl Default for Authoring {
    fn default() -> Self {
        Self {
            developer: String::from("Unknown"),
        }
    }
}

/// The configuration of a packaged eucalyptus project.
///
/// Often stored as a single .eupak file, it contains all the scenes and the references of different
/// resources.
#[derive(bincode::Decode, bincode::Encode, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct RuntimeProjectConfig {
    /// The name of the project
    #[bincode(with_serde)]
    pub project_name: String,

    /// The initial/first scene that will show up. 
    /// 
    /// Access to other scenes are done with the game's scripting. 
    #[bincode(with_serde)]
    pub initial_scene: String,

    /// Authors and creators of the game
    #[bincode(with_serde)]
    pub authors: Authoring,

    /// The version of dropbear engine and eucalyptus-editor.
    ///
    /// dropbear and eucalyptus all share the same semver version.
    #[bincode(with_serde)]
    pub editor_version: Version,

    /// The version of the project. By default, it returns `0.1.0` if none has been specified.
    #[bincode(with_serde)]
    pub project_version: Version,

    /// Any specific settings to do with the runtime.
    #[bincode(with_serde)]
    pub runtime_settings: RuntimeSettings,

    /// All scenes that are available in the project.
    #[bincode(with_serde)]
    pub scenes: Vec<SceneConfig>,
}

impl RuntimeProjectConfig {
    /// Creates a [RuntimeProjectConfig] from a loaded [PROJECT] and [SCENES] states.
    pub fn from_memory() -> anyhow::Result<Self> {
        let project = PROJECT.read();
        let scenes = SCENES.read();

        let initial_scene = match &project.runtime_settings.initial_scene {
            Some(val) => val.clone(),
            None => {
                log::warn!("Unable to fetch initial settings, using first scene available");
                let scene = scenes.first().ok_or(anyhow::anyhow!("Unable to locate first scene in SCENES"))?;
                scene.scene_name.clone()
            },
        };

        let result = Self {
            project_name: project.project_name.clone(),
            authors: Authoring::default(),
            editor_version: Version::parse(env!("CARGO_PKG_VERSION")).context("This should not happen, unless some issue is happening with env!(\"CARGO_PKG_VERSION\")")?,
            project_version: Version::parse(
                project
                    .project_version
                    .clone()
                    .unwrap_or("0.1.0".to_string())
                    .as_str(),
            )
            .unwrap_or(Version::new(0, 1, 0)),
            runtime_settings: project.runtime_settings.clone(),
            scenes: scenes.to_vec(),
            initial_scene,
        };

        Ok(result)
    }
}
