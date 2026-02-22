#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AntiAliasingMode {
    #[default]
    None,
    MSAA4,
    // todo: implement TAA
}

impl Into<u32> for AntiAliasingMode {
    fn into(self) -> u32 {
        match self {
            AntiAliasingMode::None => 1,
            AntiAliasingMode::MSAA4 => 4,
        }
    }
}