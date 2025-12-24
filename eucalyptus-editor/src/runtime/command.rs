use dropbear_engine::graphics::RenderContext;
use eucalyptus_core::command::{CommandBufferPoller, COMMAND_BUFFER, CommandBuffer, WindowCommand};
use winit::window::CursorGrabMode;
use crate::runtime::PlayMode;

impl CommandBufferPoller for PlayMode {
    fn poll(&mut self, graphics: &RenderContext) {
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
                    self.scene_command = dropbear_engine::scene::SceneCommand::Quit;
                },

                CommandBuffer::SwitchSceneImmediate(_) => {}
                CommandBuffer::LoadSceneAsync(_) => {}
                CommandBuffer::SwitchToAsync(_) => {}
            }
        }
    }
}