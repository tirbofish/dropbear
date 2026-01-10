//! The scripting module, primarily for JVM based languages and Kotlin/Native generated libraries. 
//! 
//! Other native languages are available (not tested) such as Python or C++, 
//! it is that JVM and Kotlin/Native languages are prioritised in the dropbear project. 
pub mod error;
pub mod jni;
pub mod native;
pub mod utils;
pub mod result;

pub static JVM_ARGS: OnceLock<String> = OnceLock::new();
pub static AWAIT_JDB: OnceLock<bool> = OnceLock::new();

use std::sync::OnceLock;
use crate::ptr::{AssetRegistryPtr, CommandBufferPtr, InputStatePtr, PhysicsStatePtr, SceneLoaderPtr, WorldPtr};
use crate::scripting::jni::JavaContext;
use crate::scripting::native::NativeLibrary;
use crate::states::{Script};
use anyhow::Context;
use crossbeam_channel::Sender;
use dropbear_engine::asset::ASSET_REGISTRY;
use hecs::{Entity, World};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use dropbear_engine::asset::PointerKind::Const;
use dropbear_engine::model::MODEL_CACHE;
use magna_carta::Target;
use crate::scene::loading::SCENE_LOADER;
use crate::types::{CollisionEvent, ContactForceEvent};

/// The target of the script. This can be either a JVM or a native library.
#[derive(Default, Clone, Debug)]
pub enum ScriptTarget {
    #[default]
    /// The default target. Using this will always return an error.
    None,
    /// JVM target. This will load the script into a dropbear hosted JVM instance.  
    JVM {
        /// Path to the JAR file. This is the file that will be loaded into the JVM.
        library_path: PathBuf,
    },
    /// Native target. This will load the library_path of this enum.
    Native {
        /// Path to the library. This is the file that will be loaded into the JVM.
        library_path: PathBuf,
    },
}

/// An enum representing the status of the build process.
///
/// This is used for cross-thread [`crossbeam_channel::unbounded`] channels
#[derive(Debug, Clone)]
pub enum BuildStatus {
    Started,
    Building(String),
    Completed,
    Failed(String),
}

pub struct ScriptManager {
    /// The JVM instance. This is only set if the [`ScriptTarget`] is [`ScriptTarget::JVM`].
    jvm: Option<JavaContext>,
    /// The library instance. This is only set if the [`ScriptTarget`] is [`ScriptTarget::Native`].
    library: Option<NativeLibrary>,
    /// The target of the script. This can be either a JVM or a native library (or None, but why
    /// would you set it as that?)
    script_target: ScriptTarget,
    /// The entity tag database. This is a map of tag<->list of entities.
    entity_tag_database: HashMap<String, Vec<Entity>>,
    /// Whether or not the JVM has been created.
    ///
    /// This bool is required as the JNI specifications only allow for one JVM per process.
    jvm_created: bool,
    /// The path to the library. This is set if the [`ScriptTarget`] is [`ScriptTarget::Native`] or
    /// [`ScriptTarget::JVM`]
    lib_path: Option<PathBuf>,

    /// Tags that have been instantiated/loaded into the scripting runtime.
    ///
    /// This is intentionally independent of the current scene's entity tag database so that
    /// scripts can keep state across scene switches.
    loaded_tags: HashSet<String>,

    /// Tags that are currently in-scope for the active scene/world.
    active_tags: HashSet<String>,

    /// True once `load_script` has successfully initialised the current target.
    scripts_loaded: bool,
}

impl ScriptManager {
    /// Creates a new [`ScriptManager`] uninitialised instance, as well as a new
    /// JVM instance (if the JVM flag is enabled)
    pub fn new() -> anyhow::Result<Self> {
        #[allow(unused_mut)]
        let mut result = Self {
            jvm: None,
            library: None,
            script_target: Default::default(),
            entity_tag_database: HashMap::new(),
            jvm_created: false,
            lib_path: None,
            loaded_tags: HashSet::new(),
            active_tags: HashSet::new(),
            scripts_loaded: false,
        };

        #[cfg(feature = "jvm")]
        // using this feature is automatically supported by the "editor" feature flag
        {
            let jvm_args = JVM_ARGS.get().map(|v| v.clone());
            // JavaContext will only be created if developer explicitly specifies.
            let jvm = JavaContext::new(jvm_args)?;
            result.jvm = Some(jvm);
            result.jvm_created = true;
            log::debug!("Created new JVM instance");
        }

        Ok(result)
    }

    /// Initialises the library by loading it into memory or into the JVM depending on the
    /// target.
    ///
    /// This function required a [`HashMap<String, Vec<Entity>>`], which has a tag<->list of entities
    /// link. It is stored in memory until the script is reinitialised.
    ///
    /// This function is only required to be run once at the start of the session.
    pub fn init_script(
        &mut self,
        jvm_args: Option<String>,
        entity_tag_database: HashMap<String, Vec<Entity>>,
        target: ScriptTarget,
    ) -> anyhow::Result<()> {
        let previous_target = self.script_target.clone();
        let previous_path = self.lib_path.clone();

        let next_active: HashSet<String> = entity_tag_database.keys().cloned().collect();

        let new_path = match &target {
            ScriptTarget::JVM { library_path } => Some(library_path.clone()),
            ScriptTarget::Native { library_path } => Some(library_path.clone()),
            ScriptTarget::None => None,
        };

        let target_kind_changed = std::mem::discriminant(&previous_target) != std::mem::discriminant(&target);
        let path_changed = previous_path != new_path;

        if target_kind_changed || path_changed {
            self.destroy_all().ok();
        } else if self.scripts_loaded {
            let removed: Vec<String> = self
                .active_tags
                .difference(&next_active)
                .cloned()
                .collect();

            for tag in removed {
                let _ = self.destroy_in_scope_tagged(&tag);
            }
        }

        self.active_tags = next_active;

        self.entity_tag_database = entity_tag_database;
        self.script_target = target.clone();
        self.lib_path = new_path;

        match &target {
            ScriptTarget::JVM { library_path } => {
                if !self.jvm_created {
                    let jvm = JavaContext::new(jvm_args)?;
                    self.jvm = Some(jvm);
                    self.jvm_created = true;
                    log::debug!("Created new JVM instance");
                } else {
                    log::debug!("Reusing existing JVM instance");
                    if let Some(jvm) = &mut self.jvm {
                        jvm.jar_path = library_path.clone();
                    }
                }
            }
            ScriptTarget::Native { library_path } => {
                if path_changed || self.library.is_none() {
                    self.library = Some(NativeLibrary::new(library_path)?);
                }
            }
            ScriptTarget::None => {
                self.jvm = None;
                self.library = None;
                self.jvm_created = false;
                self.lib_path = None;
                self.loaded_tags.clear();
                self.active_tags.clear();
                self.scripts_loaded = false;
            }
        }

        Ok(())
    }

    /// Loads and initialises the script for the specified script target.
    ///
    /// This function only needs to be called once at the start of the session.
    ///
    /// # ScriptTarget behaviours
    /// - [`ScriptTarget::JVM`] - This initialises the JVM by setting specific contexts such
    ///   as necessary pointer/handles with [`JavaContext::load_systems_for_tag`]. After it
    ///   loads each system for each tag.
    /// - [`ScriptTarget::Native`] - This initialises the library using [`NativeLibrary::init`].
    ///   After it loads the necessary system with the tag.
    /// - [`ScriptTarget::None`] - This returns an [`Err`], as no script target would have been
    ///   set.
    pub fn load_script(
        &mut self,
        world: WorldPtr,
        input: InputStatePtr,
        graphics: CommandBufferPtr,
        physics_state: PhysicsStatePtr,
    ) -> anyhow::Result<()> {
        let assets = &raw const *ASSET_REGISTRY;
        let scene_loader = &raw const *SCENE_LOADER;
        
        let model_cache_ptr = &raw const *MODEL_CACHE;
        ASSET_REGISTRY.add_pointer(Const("model_cache"), model_cache_ptr as usize);

        let context = DropbearContext {
            world,
            input,
            graphics,
            assets,
            scene_loader,
            physics_state,
        };

        if world.is_null() { log::error!("World pointer is null"); }
        if input.is_null() { log::error!("InputState pointer is null"); }
        if graphics.is_null() { log::error!("CommandBuffer pointer is null"); }
        if assets.is_null() { log::error!("AssetRegistry pointer is null"); }
        if scene_loader.is_null() { log::error!("SceneLoader pointer is null"); }
        if physics_state.is_null() { log::error!("PhysicsState pointer is null"); }

        match &self.script_target {
            ScriptTarget::JVM { .. } => {
                if let Some(jvm) = &mut self.jvm {
                    jvm.init(&context)?;
                    for (tag, entities) in &self.entity_tag_database {
                        log::trace!("Loading systems for tag: {}", tag);

                        let entity_ids: Vec<u64> = entities
                            .iter()
                            .map(|entity| entity.to_bits().get())
                            .collect();

                        if entity_ids.is_empty() {
                            jvm.load_systems_for_tag(tag)?;
                        } else {
                            jvm.load_systems_for_entities(tag, &entity_ids)?;
                        }

                        self.loaded_tags.insert(tag.clone());
                    }
                    self.scripts_loaded = true;
                    return Ok(());
                }
            }
            ScriptTarget::Native { .. } => {
                if let Some(library) = &mut self.library {
                    library.init(&context)?;
                    for (tag, entities) in &self.entity_tag_database {
                        log::trace!("Loading systems for tag: {}", tag);

                        let entity_ids: Vec<u64> = entities
                            .iter()
                            .map(|entity| entity.to_bits().get())
                            .collect();

                        if entity_ids.is_empty() {
                            library.load_systems(tag.to_string())?;
                        } else {
                            library.load_systems_for_entities(tag, &entity_ids)?;
                        }

                        self.loaded_tags.insert(tag.clone());
                    }
                    self.scripts_loaded = true;
                    return Ok(());
                }
            }
            ScriptTarget::None => {
                return Err(anyhow::anyhow!("No script target set"));
            }
        }

        Err(anyhow::anyhow!("Invalid script target configuration"))
    }

    pub fn collision_event_script(&mut self, world: &World, event: &CollisionEvent) -> anyhow::Result<()> {
        self.rebuild_entity_tag_database(world)?;

        let a = event.collider1_entity_id();
        let b = event.collider2_entity_id();

        for (tag, entities) in &self.entity_tag_database {
            let entity_ids: Vec<u64> = entities
                .iter()
                .map(|entity| entity.to_bits().get())
                .collect();

            if entity_ids.is_empty() {
                continue;
            }

            let mut relevant = Vec::new();
            if entity_ids.iter().any(|id| *id == a) {
                relevant.push(a);
            }
            if b != a && entity_ids.iter().any(|id| *id == b) {
                relevant.push(b);
            }
            if relevant.is_empty() {
                continue;
            }

            match &self.script_target {
                ScriptTarget::JVM { .. } => {
                    if let Some(jvm) = &self.jvm {
                        for current in relevant {
                            jvm.collision_event(tag, current, event)?;
                        }
                    }
                }
                ScriptTarget::Native { .. } => {
                    if let Some(library) = &self.library {
                        for current in relevant {
                            library.collision_event(tag, current, event)?;
                        }
                    }
                }
                ScriptTarget::None => {}
            }
        }

        Ok(())
    }

    pub fn contact_force_event_script(&mut self, world: &World, event: &ContactForceEvent) -> anyhow::Result<()> {
        self.rebuild_entity_tag_database(world)?;

        let a = event.collider1_entity_id();
        let b = event.collider2_entity_id();

        for (tag, entities) in &self.entity_tag_database {
            let entity_ids: Vec<u64> = entities
                .iter()
                .map(|entity| entity.to_bits().get())
                .collect();

            if entity_ids.is_empty() {
                continue;
            }

            let mut relevant = Vec::new();
            if entity_ids.iter().any(|id| *id == a) {
                relevant.push(a);
            }
            if b != a && entity_ids.iter().any(|id| *id == b) {
                relevant.push(b);
            }
            if relevant.is_empty() {
                continue;
            }

            match &self.script_target {
                ScriptTarget::JVM { .. } => {
                    if let Some(jvm) = &self.jvm {
                        for current in relevant {
                            jvm.contact_force_event(tag, current, event)?;
                        }
                    }
                }
                ScriptTarget::Native { .. } => {
                    if let Some(library) = &self.library {
                        for current in relevant {
                            library.contact_force_event(tag, current, event)?;
                        }
                    }
                }
                ScriptTarget::None => {}
            }
        }

        Ok(())
    }

    /// Updates the script as loaded into [`ScriptManager`].
    ///
    /// This function needs to be called every frame.
    ///
    /// # ScriptTarget behaviours
    /// - [`ScriptTarget::JVM`] - This runs [`JavaContext::update_all_systems`] if the database is
    ///   empty, [`JavaContext::update_systems_for_tag`] if there are tags but no entities, and
    ///   [`JavaContext::update_systems_for_entities`] if there are entities.
    /// - [`ScriptTarget::Native`] - This runs [`NativeLibrary::update_all`] if the database is
    ///   empty or [`NativeLibrary::update_systems_for_entities`] if there are tags.
    /// - [`ScriptTarget::None`] - This returns an error.
    pub fn update_script(
        &mut self,
        world: &World,
        dt: f64,
    ) -> anyhow::Result<()> {
        self.rebuild_entity_tag_database(world)?;

        match self.script_target {
            ScriptTarget::None => Err(anyhow::anyhow!(
                "ScriptTarget is set to None. Either set to JVM or Native"
            )),
            ScriptTarget::JVM { .. } => {
                if let Some(jvm) = &self.jvm {
                    if self.entity_tag_database.is_empty() {
                        jvm.update_all_systems(dt)?;
                    } else {
                        for (tag, entities) in &self.entity_tag_database {
                            let entity_ids: Vec<u64> = entities
                                .iter()
                                .map(|entity| entity.to_bits().get())
                                .collect();

                            if entity_ids.is_empty() {
                                jvm.update_systems_for_tag(tag, dt)?;
                            } else {
                                jvm.update_systems_for_entities(tag, &entity_ids, dt)?;
                            }
                        }
                    }
                    return Ok(());
                }
                Err(anyhow::anyhow!(
                    "ScriptTarget is set to JVM but JVM is None"
                ))
            }
            ScriptTarget::Native { .. } => {
                if let Some(library) = &mut self.library {
                    if self.entity_tag_database.is_empty() {
                        library.update_all(dt)?;
                    } else {
                        for (tag, entities) in &self.entity_tag_database {
                            let entity_ids: Vec<u64> = entities
                                .iter()
                                .map(|entity| entity.to_bits().get())
                                .collect();

                            if entity_ids.is_empty() {
                                library.update_tagged(tag, dt)?;
                            } else {
                                library.update_systems_for_entities(tag, entity_ids.as_slice(), dt)?;
                            }
                        }
                    }
                    return Ok(());
                }
                Err(anyhow::anyhow!(
                    "ScriptTarget is set to Native but library is None"
                ))
            }
        }
    }

    /// Updates the world on every physics update.
    ///
    /// A physics update is determined by [dropbear_engine::PHYSICS_STEP_RATE], which is set to a
    /// constant `50`.
    pub fn physics_update_script(
        &mut self,
        world: &World,
        dt: f64,
    ) -> anyhow::Result<()> {
        self.rebuild_entity_tag_database(world)?;

        match self.script_target {
            ScriptTarget::None => Err(anyhow::anyhow!(
                "ScriptTarget is set to None. Either set to JVM or Native"
            )),
            ScriptTarget::JVM { .. } => {
                if let Some(jvm) = &self.jvm {
                    if self.entity_tag_database.is_empty() {
                        jvm.physics_update_all_systems(dt)?;
                    } else {
                        for (tag, entities) in &self.entity_tag_database {
                            let entity_ids: Vec<u64> = entities
                                .iter()
                                .map(|entity| entity.to_bits().get())
                                .collect();

                            if entity_ids.is_empty() {
                                jvm.physics_update_systems_for_tag(tag, dt)?;
                            } else {
                                jvm.physics_update_systems_for_entities(tag, &entity_ids, dt)?;
                            }
                        }
                    }
                    return Ok(());
                }
                Err(anyhow::anyhow!(
                    "ScriptTarget is set to JVM but JVM is None"
                ))
            }
            ScriptTarget::Native { .. } => {
                if let Some(library) = &mut self.library {
                    if self.entity_tag_database.is_empty() {
                        library.physics_update_all(dt)?;
                    } else {
                        for (tag, entities) in &self.entity_tag_database {
                            let entity_ids: Vec<u64> = entities
                                .iter()
                                .map(|entity| entity.to_bits().get())
                                .collect();

                            if entity_ids.is_empty() {
                                library.physics_update_tagged(tag, dt)?;
                            } else {
                                library.physics_update_systems_for_entities(tag, entity_ids.as_slice(), dt)?;
                            }
                        }
                    }
                    return Ok(());
                }
                Err(anyhow::anyhow!(
                    "ScriptTarget is set to Native but library is None"
                ))
            }
        }
    }

    /// Reloads the .jar file by unloading the previous classes and reloading them back in,
    /// allowing for hot reloading.
    ///
    /// # ScriptTarget behaviours
    /// - [`ScriptTarget::JVM`] - This target is the only target that allows this function.
    /// - [`ScriptTarget::Native`] - This target does not do anything, but does not result in an
    ///   error (returns [`Ok`])
    /// - [`ScriptTarget::None`] - This target does not do anything, but does not result in an
    ///   error (returns [`Ok`])
    pub fn reload(&mut self, world_ptr: WorldPtr) -> anyhow::Result<()> {
        if let Some(jvm) = &mut self.jvm {
            jvm.reload(world_ptr)?
        }
        Ok(())
    }

    /// Destroys all scripts for the current target.
    pub fn destroy_all(&mut self) -> anyhow::Result<()> {
        match self.script_target {
            ScriptTarget::None => Ok(()),
            ScriptTarget::JVM { .. } => {
                if let Some(jvm) = &self.jvm {
                    let _ = jvm.unload_all_systems();
                }
                self.loaded_tags.clear();
                self.active_tags.clear();
                self.scripts_loaded = false;
                Ok(())
            }
            ScriptTarget::Native { .. } => {
                if let Some(library) = &mut self.library {
                    library.destroy_all()?;
                }
                self.loaded_tags.clear();
                self.active_tags.clear();
                self.scripts_loaded = false;
                Ok(())
            }
        }
    }

    fn destroy_in_scope_tagged(&mut self, tag: &str) -> anyhow::Result<()> {
        if !self.scripts_loaded {
            return Ok(());
        }

        match self.script_target {
            ScriptTarget::None => Ok(()),
            ScriptTarget::JVM { .. } => {
                if let Some(jvm) = &self.jvm {
                    let _ = jvm.destroy_systems_for_tag(tag);
                }
                Ok(())
            }
            ScriptTarget::Native { .. } => {
                if let Some(library) = &mut self.library {
                    library.destroy_in_scope_tagged(tag.to_string())?;
                }
                Ok(())
            }
        }
    }

    fn load_tagged(&mut self, tag: &str) -> anyhow::Result<()> {
        self.loaded_tags.insert(tag.to_string());

        match self.script_target {
            ScriptTarget::None => Ok(()),
            ScriptTarget::JVM { .. } => {
                if let Some(jvm) = &mut self.jvm {
                    jvm.load_systems_for_tag(tag)?;
                }
                Ok(())
            }
            ScriptTarget::Native { .. } => {
                if let Some(library) = &mut self.library {
                    library.load_systems(tag.to_string())?;
                }
                Ok(())
            }
        }
    }

    /// Rebuilds the ScriptManagers entity database by parsing a [`World`].
    ///
    /// If scripts are already loaded, this also:
    /// - loads tags entering scope, and
    /// - calls `destroy()` for tags leaving scope (without unloading instances).
    fn rebuild_entity_tag_database(&mut self, world: &World) -> anyhow::Result<()> {
        let mut new_map: HashMap<String, Vec<Entity>> = HashMap::new();

        for (entity, script) in world.query::<&Script>().iter() {
            for tag in &script.tags {
                new_map.entry(tag.clone()).or_default().push(entity);
            }
        }

        if self.scripts_loaded {
            let next_active: HashSet<String> = new_map.keys().cloned().collect();
            let removed: Vec<String> = self
                .active_tags
                .difference(&next_active)
                .cloned()
                .collect();
            let added: Vec<String> = next_active
                .difference(&self.active_tags)
                .cloned()
                .collect();

            for tag in removed {
                self.destroy_in_scope_tagged(&tag)?;
            }
            for tag in added {
                self.load_tagged(&tag)?;
            }

            self.active_tags = next_active;
        } else {
            self.active_tags = new_map.keys().cloned().collect();
        }

        self.entity_tag_database = new_map;
        Ok(())
    }
}

impl Drop for ScriptManager {
    fn drop(&mut self) {
        let _ = self.destroy_all();
    }
}

/// Fetches the gradle command available for that operating system.
///
/// # Platform-specific behaviours
/// - `windows` - Windows uses `gradlew.bat`
/// - `linux` - Linux uses `./gradlew`
/// - `macos` - macOS uses `./gradlew`
fn get_gradle_command(project_root: impl AsRef<Path>) -> String {
    let project_root = project_root.as_ref().to_owned();
    if cfg!(target_os = "windows") {
        let gradlew = project_root.join("gradlew.bat");
        if gradlew.exists() {
            gradlew.to_string_lossy().to_string()
        } else {
            "gradle.bat".to_string()
        }
    } else {
        let gradlew = project_root.join("gradlew");
        if gradlew.exists() {
            "./gradlew".to_string()
        } else {
            "gradle".to_string()
        }
    }
}

/// Asynchronously builds a project for the JVM using gradle.
pub async fn build_jvm(
    project_root: impl AsRef<Path>,
    status_sender: Sender<BuildStatus>,
) -> anyhow::Result<PathBuf> {
    let project_root = project_root.as_ref();

    if !project_root.exists() {
        let err = format!("Project root does not exist: {:?}", project_root);
        let _ = status_sender.send(BuildStatus::Failed(err.clone()));
        return Err(anyhow::anyhow!(err));
    }

    let _ = status_sender.send(BuildStatus::Started);

    let _ = status_sender.send(BuildStatus::Building(format!("Building manifest at {}", project_root.join("build/magna-carta/jvmMain/RunnableRegistry.kt").display())));
    if let Err(e) = magna_carta::parse(project_root.join("src"), Target::Jvm, project_root.join("build/magna-carta/jvmMain")) {
        let _ = status_sender.send(BuildStatus::Failed(format!("Failed to build manifest: {}", e)));
        return Err(e);
    }
    let _ = status_sender.send(BuildStatus::Building(String::from("Successfully built manifest!")));

    if !(project_root.join("build.gradle").exists()
        || project_root.join("build.gradle.kts").exists())
    {
        let err = format!("No Gradle build script found in: {:?}", project_root);
        let _ = status_sender.send(BuildStatus::Failed(err.clone()));
        return Err(anyhow::anyhow!(err));
    }

    let gradle_cmd = get_gradle_command(project_root);

    let _ = status_sender.send(BuildStatus::Building(format!("Running: {}", gradle_cmd)));

    let mut child = Command::new(&gradle_cmd)
        .current_dir(project_root)
        .args(["--console=plain", "fatJar"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context(format!("Failed to spawn `{} fatJar`", gradle_cmd))?;

    let stdout = child.stdout.take().expect("Stdout was piped");
    let stderr = child.stderr.take().expect("Stderr was piped");

    let tx_out = status_sender.clone();
    let stdout_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = tx_out.send(BuildStatus::Building(line));
        }
    });

    let tx_err = status_sender.clone();
    let stderr_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = tx_err.send(BuildStatus::Building(line));
        }
    });

    let status = child
        .wait()
        .await
        .context("Failed to wait for gradle process")?;

    let _ = tokio::join!(stdout_task, stderr_task);

    if !status.success() {
        let code = status.code().unwrap_or(-1);
        let err_msg = format!("Gradle build failed with exit code {}", code);
        let _ = status_sender.send(BuildStatus::Failed(err_msg.clone()));
        return Err(anyhow::anyhow!(err_msg));
    }

    let libs_dir = project_root.join("build").join("libs");
    if !libs_dir.exists() {
        let err = "Build succeeded but 'build/libs' directory is missing".to_string();
        let _ = status_sender.send(BuildStatus::Failed(err.clone()));
        return Err(anyhow::anyhow!(err));
    }

    let jar_files: Vec<PathBuf> = std::fs::read_dir(&libs_dir)
        .context("Failed to read 'build/libs'")?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.extension()
                .map_or(false, |ext| ext.eq_ignore_ascii_case("jar"))
                && !path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .contains("-sources")
                && !path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .contains("-javadoc")
        })
        .collect();

    if jar_files.is_empty() {
        let err = "No JAR artifact found in 'build/libs'".to_string();
        let _ = status_sender.send(BuildStatus::Failed(err.clone()));
        return Err(anyhow::anyhow!(err));
    }

    let fat_jar = jar_files.iter().find(|path| {
        path.file_name()
            .and_then(|n| n.to_str())
            .map_or(false, |name| name.contains("-all"))
    });

    let jar_path = if let Some(fat) = fat_jar {
        fat.clone()
    } else {
        jar_files
            .into_iter()
            .max_by_key(|path| std::fs::metadata(path).map(|m| m.len()).unwrap_or(0))
            .unwrap()
    };

    let _ = status_sender.send(BuildStatus::Completed);
    Ok(jar_path)
}

/// Asynchronously builds a project for Kotlin/Native using gradle.
pub async fn build_native(
    project_root: impl AsRef<Path>,
    status_sender: Sender<BuildStatus>,
) -> anyhow::Result<PathBuf> {
    let project_root = project_root.as_ref();

    if !project_root.exists() {
        let err = format!("Project root does not exist: {:?}", project_root);
        let _ = status_sender.send(BuildStatus::Failed(err.clone()));
        return Err(anyhow::anyhow!(err));
    }

    let _ = status_sender.send(BuildStatus::Started);

    let _ = status_sender.send(BuildStatus::Building("Copying core library...".to_string()));
    let libs_dir = project_root.join("libs");
    if !libs_dir.exists() {
        std::fs::create_dir_all(&libs_dir).context("Failed to create libs directory")?;
    }

    let (lib_name, lib_ext) = if cfg!(target_os = "windows") {
        ("eucalyptus_core", "dll")
    } else if cfg!(target_os = "macos") {
        ("libeucalyptus_core", "dylib")
    } else {
        ("libeucalyptus_core", "so")
    };

    let lib_filename = format!("{}.{}", lib_name, lib_ext);

    let current_exe = std::env::current_exe().context("Failed to get current executable path")?;
    let exe_dir = current_exe
        .parent()
        .context("Failed to get executable directory")?;
    let source_lib_path = exe_dir.join(&lib_filename);

    if source_lib_path.exists() {
        std::fs::copy(&source_lib_path, libs_dir.join(&lib_filename))
            .context(format!("Failed to copy {} to libs", lib_filename))?;
    } else {
        let cwd_lib_path = std::env::current_dir()?.join(&lib_filename);
        if cwd_lib_path.exists() {
            std::fs::copy(&cwd_lib_path, libs_dir.join(&lib_filename))
                .context(format!("Failed to copy {} to libs", lib_filename))?;
        } else {
            let err = format!("Could not find core library {} to copy", lib_filename);
            let _ = status_sender.send(BuildStatus::Failed(err.clone()));
            return Err(anyhow::anyhow!(err));
        }
    }

    let _ = status_sender.send(BuildStatus::Building(format!("Building manifest at {}", project_root.join("build/magna-carta/jvmMain/RunnableRegistry.kt").display())));
    if let Err(e) = magna_carta::parse(project_root.join("src"), Target::Jvm, project_root.join("build/magna-carta/jvmMain")) {
        let _ = status_sender.send(BuildStatus::Failed(format!("Failed to build manifest: {}", e)));
        return Err(e);
    }
    let _ = status_sender.send(BuildStatus::Building(String::from("Successfully built manifest!")));

    let gradle_cmd = get_gradle_command(project_root);
    let _ = status_sender.send(BuildStatus::Building(format!(
        "Running: {} build",
        gradle_cmd
    )));

    let mut child = Command::new(&gradle_cmd)
        .current_dir(project_root)
        .args(["--console=plain", "build"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context(format!("Failed to spawn `{} build`", gradle_cmd))?;

    let stdout = child.stdout.take().expect("Stdout was piped");
    let stderr = child.stderr.take().expect("Stderr was piped");

    let tx_out = status_sender.clone();
    let stdout_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = tx_out.send(BuildStatus::Building(line));
        }
    });

    let tx_err = status_sender.clone();
    let stderr_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = tx_err.send(BuildStatus::Building(line));
        }
    });

    let status = child
        .wait()
        .await
        .context("Failed to wait for gradle process")?;

    let _ = tokio::join!(stdout_task, stderr_task);

    if !status.success() {
        let code = status.code().unwrap_or(-1);
        let err_msg = format!("Gradle build failed with exit code {}", code);
        let _ = status_sender.send(BuildStatus::Failed(err_msg.clone()));
        return Err(anyhow::anyhow!(err_msg));
    }

    let output_dir = project_root.join("build/bin/nativeLib/releaseShared");
    if !output_dir.exists() {
        let err = format!(
            "Build succeeded but output directory missing: {:?}",
            output_dir
        );
        let _ = status_sender.send(BuildStatus::Failed(err.clone()));
        return Err(anyhow::anyhow!(err));
    }

    let mut found_lib = None;
    if let Ok(entries) = std::fs::read_dir(&output_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == lib_ext {
                    found_lib = Some(path);
                    break;
                }
            }
        }
    }

    if let Some(lib_path) = found_lib {
        let _ = status_sender.send(BuildStatus::Completed);
        Ok(lib_path)
    } else {
        let err = format!("No .{} file found in {:?}", lib_ext, output_dir);
        let _ = status_sender.send(BuildStatus::Failed(err.clone()));
        Err(anyhow::anyhow!(err))
    }
}

/// Describes all the different pointers that can be passed into a scripting
/// module.
#[repr(C)]
pub struct DropbearContext {
    pub world: WorldPtr,
    pub input: InputStatePtr,
    pub graphics: CommandBufferPtr,
    pub assets: AssetRegistryPtr,
    pub scene_loader: SceneLoaderPtr,
    pub physics_state: PhysicsStatePtr,
}