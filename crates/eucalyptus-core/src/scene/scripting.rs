
pub mod shared {
    use crate::command::CommandBuffer;
    use crate::scene::loading::{SceneLoadResult, SceneLoader};
    use crate::scripting::native::DropbearNativeError;
    use crate::scripting::result::DropbearNativeResult;
    use crossbeam_channel::Sender;
    use parking_lot::Mutex;

    pub fn load_scene_async(
        command_buffer: &Sender<CommandBuffer>,
        scene_loader: &Mutex<SceneLoader>,
        scene_name: String,
        _loading_scene: Option<String>,
    ) -> DropbearNativeResult<u64> {
        let mut loader = scene_loader.lock();

        if let Some(existing_id) = loader.find_pending_id_by_name(&scene_name) {
            return Ok(existing_id);
        }

        let id = loader.register_load(scene_name.clone());

        let handle = crate::scene::loading::SceneLoadHandle {
            id,
            scene_name: scene_name.clone(),
        };

        command_buffer
            .try_send(CommandBuffer::LoadSceneAsync(handle))
            .map_err(|_| DropbearNativeError::SendError)?;

        Ok(id)
    }

    pub fn switch_to_scene_immediate(
        command_buffer: &Sender<CommandBuffer>,
        scene_name: String,
    ) -> DropbearNativeResult<()> {
        command_buffer
            .try_send(CommandBuffer::SwitchSceneImmediate(scene_name))
            .map_err(|_| DropbearNativeError::SendError)?;
        Ok(())
    }

    pub fn switch_to_scene_async(
        command_buffer: &Sender<CommandBuffer>,
        scene_loader: &Mutex<SceneLoader>,
        scene_id: u64,
    ) -> DropbearNativeResult<()> {
        let loader = scene_loader.lock();

        if let Some(entry) = loader.get_entry(scene_id) {
            if matches!(entry.result, SceneLoadResult::Success) {
                let handle = crate::scene::loading::SceneLoadHandle {
                    id: scene_id,
                    scene_name: entry.scene_name.clone(),
                };

                command_buffer
                    .try_send(CommandBuffer::SwitchToAsync(handle))
                    .map_err(|_| DropbearNativeError::SendError)?;
                Ok(())
            } else {
                Err(DropbearNativeError::PrematureSceneSwitch)
            }
        } else {
            Err(DropbearNativeError::NoSuchHandle)
        }
    }

    pub fn get_scene_load_handle_scene_name(
        scene_loader: &Mutex<SceneLoader>,
        scene_id: u64,
    ) -> DropbearNativeResult<String> {
        let loader = scene_loader.lock();

        if let Some(entry) = loader.get_entry(scene_id) {
            Ok(entry.scene_name.clone())
        } else {
            Err(DropbearNativeError::NoSuchHandle)
        }
    }

    pub fn get_scene_load_progress(
        scene_loader: &Mutex<SceneLoader>,
        scene_id: u64,
    ) -> DropbearNativeResult<crate::utils::Progress> {
        let mut loader = scene_loader.lock();

        if let Some(entry) = loader.get_entry_mut(scene_id) {
            if let Some(ref rx) = entry.status {
                while let Ok(status) = rx.try_recv() {
                    match status {
                        crate::states::WorldLoadingStatus::Idle => {
                            entry.progress.message = "Idle".to_string();
                        }
                        crate::states::WorldLoadingStatus::LoadingEntity { index, name, total } => {
                            entry.progress.current = index;
                            entry.progress.total = total;
                            entry.progress.message = format!("Loading entity: {}", name);
                        }
                        crate::states::WorldLoadingStatus::Completed => {
                            entry.progress.current = entry.progress.total;
                            entry.progress.message = "Completed".to_string();
                        }
                    }
                }
            }

            Ok(entry.progress.clone())
        } else {
            Err(DropbearNativeError::NoSuchHandle)
        }
    }

    pub fn get_scene_load_status(
        scene_loader: &Mutex<SceneLoader>,
        scene_id: u64,
    ) -> DropbearNativeResult<u32> {
        let loader = scene_loader.lock();

        if let Some(entry) = loader.get_entry(scene_id) {
            let status = match entry.result {
                SceneLoadResult::Pending => 0,
                SceneLoadResult::Success => 1,
                SceneLoadResult::Error(_) => 2,
            };
            Ok(status)
        } else {
            Err(DropbearNativeError::NoSuchHandle)
        }
    }
}

