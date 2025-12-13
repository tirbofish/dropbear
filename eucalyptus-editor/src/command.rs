use dropbear_engine::graphics::RenderContext;
use eucalyptus_core::window::{CommandBufferPoller, GRAPHICS_COMMAND, CommandBuffer, WindowCommand, get_config};
use winit::window::CursorGrabMode;

use crate::editor::Editor;

impl CommandBufferPoller for Editor {
    fn poll(&mut self, graphics: &RenderContext) {
        while let Ok(cmd) = GRAPHICS_COMMAND.1.try_recv() {
            log::trace!("Received GRAPHICS_COMMAND update: {:?}", cmd);
            match cmd {
                CommandBuffer::WindowCommand(w_cmd) => match w_cmd {
                    WindowCommand::WindowGrab(is_locked) => {
                        let mut cfg = get_config().write();
                        if cfg.is_locked != is_locked {
                            if is_locked {
                                if let Err(e) = graphics.shared.window
                                    .set_cursor_grab(CursorGrabMode::Confined)
                                    .or_else(|_| graphics.shared.window.set_cursor_grab(CursorGrabMode::Locked))
                                {
                                    log_once::warn_once!("Failed to grab cursor: {:?}", e);
                                } else {
                                    log_once::info_once!("Grabbed cursor");
                                    cfg.is_locked = true;
                                }
                            } else if let Err(e) = graphics.shared.window.set_cursor_grab(CursorGrabMode::None) {
                                log_once::warn_once!("Failed to release cursor: {:?}", e);
                            } else {
                                log_once::info_once!("Released cursor");
                                cfg.is_locked = false;
                            }
                        }
                    }
                    WindowCommand::HideCursor(should_hide) => {
                        let cfg = get_config().write();
                        if cfg.is_hidden != should_hide {
                            if should_hide {
                                graphics.shared.window.set_cursor_visible(false);
                            } else {
                                graphics.shared.window.set_cursor_visible(true);
                            }
                        }
                    }
                },
                CommandBuffer::Quit => {
                    log::info!("Quit command received in editor, stopping play mode");
                    self.signal = crate::editor::Signal::StopPlaying;
                },
            }
        }
    }
}