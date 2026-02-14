use std::path::PathBuf;

pub enum ErrorLevel {
    Info,
    Warn,
    Error,
}

pub struct ConsoleItem {
    pub id: u64,
    pub error_level: ErrorLevel,
    pub msg: String,
    pub file_location: Option<PathBuf>,
    pub line_ref: Option<String>,
}
