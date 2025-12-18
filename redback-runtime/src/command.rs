use dropbear_engine::graphics::RenderContext;
use eucalyptus_core::window::{CommandBufferPoller, COMMAND_BUFFER, CommandBuffer, WindowCommand, get_config};
use winit::window::CursorGrabMode;

use crate::scene::RuntimeScene;

impl CommandBufferPoller for RuntimeScene {
    fn poll(&mut self, graphics: &RenderContext) {
        while let Ok(cmd) = COMMAND_BUFFER.1.try_recv() {
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
                    self.scene_command = dropbear_engine::scene::SceneCommand::Quit;
                },
                CommandBuffer::SwitchScene(scene_name) => {
                    self.pending_scene_switch = Some(scene_name);
                }
            }
        }
    }
}