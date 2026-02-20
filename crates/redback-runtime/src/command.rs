use std::sync::Arc;

use crate::PlayMode;
use dropbear_engine::graphics::SharedGraphicsContext;
use dropbear_engine::scene::SceneCommand;
use eucalyptus_core::command::{COMMAND_BUFFER, CommandBuffer, CommandBufferPoller, WindowCommand};
use eucalyptus_core::scene::loading::IsSceneLoaded;
use winit::window::CursorGrabMode;

impl CommandBufferPoller for PlayMode {
    fn poll(&mut self, graphics: Arc<SharedGraphicsContext>) {
        while let Ok(cmd) = COMMAND_BUFFER.1.try_recv() {
            log::trace!("Received GRAPHICS_COMMAND update: {:?}", cmd);
            match cmd {
                CommandBuffer::WindowCommand(w_cmd) => match w_cmd {
                    WindowCommand::WindowGrab(lock) => {
                        if lock {
                            let window = &graphics.window;
                            window.set_cursor_visible(false);
                            if let Err(e) =
                                window.set_cursor_grab(CursorGrabMode::Locked).or_else(|_| {
                                    log_once::warn_once!(
                                        "Using cursor grab fallback: CursorGrabMode::Confined"
                                    );
                                    window.set_cursor_grab(CursorGrabMode::Confined)
                                })
                            {
                                log_once::error_once!("Unable to grab mouse: {}", e);
                            }
                        } else if let Err(e) = graphics
                            .clone()
                            .window
                            .set_cursor_grab(CursorGrabMode::None)
                        {
                            log_once::warn_once!("Failed to release cursor: {:?}", e);
                        } else {
                            log_once::info_once!("Released cursor");
                        }
                    }
                    WindowCommand::HideCursor(should_hide) => {
                        if should_hide {
                            graphics.window.set_cursor_visible(false);
                        } else {
                            graphics.window.set_cursor_visible(true);
                        }
                    }
                },
                CommandBuffer::Quit => {
                    self.scene_command = SceneCommand::CloseWindow(graphics.window.id());
                }
                CommandBuffer::SwitchSceneImmediate(scene_name) => {
                    log::debug!("Immediate scene switch requested: {}", scene_name);
                    let scene_to_load = IsSceneLoaded::new(scene_name);
                    self.request_immediate_scene_load(graphics.clone(), scene_to_load);
                }
                CommandBuffer::LoadSceneAsync(handle) => {
                    log::debug!("Load scene async requested");
                    let scene_to_load = IsSceneLoaded::new_with_id(handle.scene_name, handle.id);
                    self.request_async_scene_load(graphics.clone(), scene_to_load);
                }
                CommandBuffer::SwitchToAsync(handle) => {
                    if let Some(ref progress) = self.scene_progress {
                        if progress.requested_scene == handle.scene_name
                            && progress.is_everything_loaded()
                        {
                            self.switch_to(progress.clone(), graphics.clone());
                        }
                    }
                }
            }
        }
    }
}
