use dropbear_engine::graphics::RenderContext;
use eucalyptus_core::command::{CommandBufferPoller, COMMAND_BUFFER, CommandBuffer, WindowCommand};
use winit::window::CursorGrabMode;
use crate::runtime::PlayMode;
use eucalyptus_core::scene::loading::IsSceneLoaded;

impl CommandBufferPoller for PlayMode {
    fn poll(&mut self, graphics: &mut RenderContext) {
        while let Ok(cmd) = COMMAND_BUFFER.1.try_recv() {
            log::trace!("Received GRAPHICS_COMMAND update: {:?}", cmd);
            match cmd {
                CommandBuffer::WindowCommand(w_cmd) => match w_cmd {
                    WindowCommand::WindowGrab(lock) => {
                        if lock {
                            if let Err(e) = graphics.shared.window
                                .set_cursor_grab(CursorGrabMode::Confined)
                                .or_else(|_| graphics.shared.window.set_cursor_grab(CursorGrabMode::Locked))
                            {
                                log_once::warn_once!("Failed to grab cursor: {:?}", e);
                            } else {
                                log_once::info_once!("Grabbed cursor");
                            }
                        } else if let Err(e) = graphics.shared.window.set_cursor_grab(CursorGrabMode::None) {
                            log_once::warn_once!("Failed to release cursor: {:?}", e);
                        } else {
                            log_once::info_once!("Released cursor");
                        }
                    }
                    WindowCommand::HideCursor(should_hide) => {
                        if should_hide {
                            graphics.shared.window.set_cursor_visible(false);
                        } else {
                            graphics.shared.window.set_cursor_visible(true);
                        }
                    }
                },
                CommandBuffer::Quit => {
                    self.scene_command = dropbear_engine::scene::SceneCommand::Quit(None);
                },
                CommandBuffer::SwitchSceneImmediate(scene_name) => {
                    log::debug!("Immediate scene switch requested: {}", scene_name);
                    let scene_to_load = IsSceneLoaded::new(scene_name);
                    self.request_immediate_scene_load(graphics, scene_to_load);
                }
                CommandBuffer::LoadSceneAsync(handle) => {
                    log::debug!("Load scene async requested");
                    let scene_to_load = IsSceneLoaded::new_with_id(handle.scene_name, handle.id);
                    self.request_async_scene_load(graphics, scene_to_load);
                }
                CommandBuffer::SwitchToAsync(handle) => {
                    if let Some(ref progress) = self.scene_progress {
                        if progress.requested_scene == handle.scene_name && progress.is_everything_loaded() {
                             self.switch_to(progress.clone(), graphics);
                        }
                    }
                }
            }
        }
    }
}