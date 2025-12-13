use crossbeam_channel::{Receiver, Sender, unbounded};
use dropbear_engine::graphics::RenderContext;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::sync::{OnceLock};

pub static GRAPHICS_COMMAND: Lazy<(Box<Sender<CommandBuffer>>, Receiver<CommandBuffer>)> =
    Lazy::new(|| {
        let (tx, rx) = unbounded::<CommandBuffer>();
        (Box::new(tx), rx)
    });
static PREVIOUS_CONFIG: OnceLock<RwLock<CommandCache>> = OnceLock::new();

pub fn get_config() -> &'static RwLock<CommandCache> {
    PREVIOUS_CONFIG.get_or_init(|| RwLock::new(CommandCache::new()))
}

pub struct CommandCache {
    pub is_locked: bool,
    pub is_hidden: bool,
}

impl CommandCache {
    fn new() -> Self {
        Self {
            is_locked: false,
            is_hidden: false,
        }
    }
}

#[derive(Debug)]
pub enum CommandBuffer {
    WindowCommand(WindowCommand),
    Quit,
}

#[derive(Debug)]
pub enum WindowCommand {
    WindowGrab(bool),
    HideCursor(bool),
}

/// Command buffer that is used for oneway communication between Kotlin to Rust.  
pub trait CommandBufferPoller {
    fn poll(&mut self, graphics: &RenderContext);
}