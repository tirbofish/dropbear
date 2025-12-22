use dropbear_engine::graphics::RenderContext;
use eucalyptus_core::command::{CommandBufferPoller, COMMAND_BUFFER, CommandBuffer, WindowCommand, get_config};
use winit::window::CursorGrabMode;

use crate::editor::Editor;

impl CommandBufferPoller for Editor {
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
                    log::info!("Quit command received in editor, stopping play mode");
                    self.signal = crate::editor::Signal::StopPlaying;
                },
                CommandBuffer::SwitchScene(scene_name) => {
                    if matches!(self.editor_state, crate::editor::EditorState::Playing) {
                        log::info!(
                            "SwitchScene requested during play mode; switching play-world scene to '{}'",
                            scene_name
                        );
                        self.pending_play_scene_load = Some(scene_name);
                    } else if let Err(err) = self.queue_scene_load_by_name(&scene_name) {
                        log::error!(
                            "Failed to queue scene load for '{}': {}",
                            scene_name,
                            err
                        );
                    }
                }
            }
        }
    }
}