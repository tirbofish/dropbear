use bitflags::bitflags;

bitflags! {
    #[derive(PartialEq, Copy, Clone)]
    pub struct EditorTabVisibility : u8 {
        /// The editor for the game design.
        const GameEditor = 1;
        /// The editor for UI
        const UIEditor = 1 << 1;
    }
}
