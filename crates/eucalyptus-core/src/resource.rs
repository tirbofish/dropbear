use std::path::PathBuf;
use std::sync::Arc;
use serde::{Serialize, Deserialize};

/// A resolved pointer to asset data, handed to the loader.
///
/// You should never construct this by hand with a raw string —
/// always derive it from an `AssetEntry` via `AssetEntry::to_reference()`,
/// or from a `PackedAssetEntry` via the pak reader.
///
/// This is the *loading strategy*, not the identity.
/// Identity is always the `Uuid` on `AssetEntry`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceReference {
    /// A file within the project's `resources/` directory.
    /// `PathBuf` is absolute at runtime, resolved by the registry
    /// from the project-relative `AssetLocation::ProjectFile`.
    File(PathBuf),

    /// Bytes compiled into the binary via `include_bytes!`.
    /// The `Arc<[u8]>` is cheap to clone and share across systems.
    Embedded(Arc<[u8]>),

    /// A byte range within a loaded `.eucpak` file.
    /// The loader holds the pak in memory and slices it directly —
    /// zero additional allocation.
    Packed {
        offset: u64,
        length: u64,
    },

    /// No backing data — generated entirely at runtime.
    Procedural,
}

impl ResourceReference {
    /// Resolves to a byte slice if the reference is `Embedded`.
    /// Returns `None` for all other variants.
    pub fn as_embedded_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Embedded(bytes) => Some(bytes),
            _ => None,
        }
    }

    /// Returns true if this reference requires filesystem I/O to load.
    pub fn is_file_backed(&self) -> bool {
        matches!(self, Self::File(_))
    }

    /// Returns true if this reference can be loaded without any I/O.
    pub fn is_in_memory(&self) -> bool {
        matches!(self, Self::Embedded(_) | Self::Packed { .. } | Self::Procedural)
    }
}