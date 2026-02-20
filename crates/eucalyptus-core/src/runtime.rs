//! Configuration and metadata information about redback-runtime based data.

use crate::config::ProjectConfig;
use crate::scene::SceneConfig;
use crate::states::{PROJECT, SCENES};
use crate::utils::option::HistoricalOption;
use anyhow::Context;
use chrono::Utc;
use semver::Version;

/// The settings of a project in its runtime.
///
/// This is different to [`SceneSettings`], which contains settings for ONLY
/// that specific scene. This is for any configurations of the project during its runtime,
/// such as initial scene and stuff like that.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct RuntimeSettings {
    /// The first scene that shows up when redback-runtime is ran.
    ///
    /// The first scene is not set is expected to be the first scene out of the
    /// projects scene list, or just a normal anyhow error.
    #[serde(default)]
    pub initial_scene: Option<String>,
    #[serde(default)]
    pub target_fps: HistoricalOption<u32>,
}

impl RuntimeSettings {
    /// Creates a new [`RuntimeSettings`] config.
    pub fn new() -> Self {
        Self {
            initial_scene: None,
            target_fps: HistoricalOption::none(),
        }
    }
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self::new()
    }
}

/// A struct that represents the authors and creators of the eucalyptus project.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Authoring {
    /// The team behind the game
    pub developer: String,
}

impl Default for Authoring {
    fn default() -> Self {
        Self {
            developer: String::from("Some pretty good developers"),
        }
    }
}

/// The configuration of a packaged eucalyptus project.
///
/// Often stored as a single .eupak file, it contains all the scenes and the references of different
/// resources.
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct RuntimeProjectConfig {
    /// The name of the project
    pub project_name: String,

    /// The initial/first scene that will show up.
    ///
    /// Access to other scenes are done with the game's scripting.
    pub initial_scene: String,

    /// Authors and creators of the game
    pub authors: Authoring,

    /// The version of dropbear engine and eucalyptus-editor.
    ///
    /// dropbear and eucalyptus all share the same semver version.
    pub editor_version: Version,

    /// The version of the project. By default, it returns `0.1.0` if none has been specified.
    pub project_version: Version,

    /// Any specific settings to do with the runtime.
    pub runtime_settings: RuntimeSettings,

    /// All scenes that are available in the project.
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
                let scene = scenes
                    .first()
                    .ok_or(anyhow::anyhow!("Unable to locate first scene in SCENES"))?;
                scene.scene_name.clone()
            }
        };

        let result = Self {
            project_name: project.project_name.clone(),
            authors: Authoring::default(),
            editor_version: Version::parse(env!("CARGO_PKG_VERSION")).context("This should not happen, unless some issue is happening with env!(\"CARGO_PKG_VERSION\")")?,
            project_version: Version::parse(
                project
                    .project_version
                    .clone()
                    .as_str(),
            )
            .unwrap_or(Version::new(0, 1, 0)),
            runtime_settings: project.runtime_settings.clone(),
            scenes: scenes.to_vec(),
            initial_scene,
        };

        Ok(result)
    }

    /// Populates the states (such as [PROJECT]) with all the context from the RuntimeProjectConfig.
    pub fn populate(&self) -> anyhow::Result<()> {
        let exe_dir = std::env::current_exe()
            .context("Unable to locate runtime executable")?
            .parent()
            .ok_or_else(|| {
                anyhow::anyhow!("Unable to locate parent directory of runtime executable")
            })?
            .to_path_buf();

        let now = format!("{}", Utc::now().format("%Y-%m-%d %H:%M:%S"));

        let mut runtime_settings = self.runtime_settings.clone();
        if runtime_settings.initial_scene.is_none() {
            runtime_settings.initial_scene = Some(self.initial_scene.clone());
        }

        let project_config = ProjectConfig {
            project_name: self.project_name.clone(),
            project_path: exe_dir,
            date_created: now.clone(),
            date_last_accessed: now,
            project_version: self.project_version.to_string(),
            authors: self.authors.clone(),
            runtime_settings,
            last_opened_scene: Some(self.initial_scene.clone()),
        };

        {
            let mut project = PROJECT.write();
            *project = project_config;
        }

        {
            let mut scenes = SCENES.write();
            scenes.clear();
            scenes.extend(self.scenes.iter().cloned());
        }

        Ok(())
    }
}
