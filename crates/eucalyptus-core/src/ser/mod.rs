use std::fmt::Display;
use std::string::ToString;

pub mod model;
pub mod templates;

pub enum SerializedType {
    /// This is a `*.eucbin` file type.
    GenericBinary,

    /// This is a `*.eucmdl` file type.
    Model,
}

impl Display for SerializedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            SerializedType::GenericBinary => "eucbin".to_string(),
            SerializedType::Model => "eucmdl".to_string(),
        };
        write!(f, "{}", str)
    }
}

impl SerializedType {
    pub fn iter_extensions() -> impl Iterator<Item = String> {
        [
            Self::GenericBinary.to_string(),
            Self::Model.to_string()
        ].into_iter()
    }
}