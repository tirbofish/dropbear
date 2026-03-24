use std::fs;
use std::time::SystemTime;
use uuid::Uuid;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use rkyv::Archive;
use ron::ser::PrettyConfig;
use serde::{Serialize, Deserialize};
use sha2::{Digest, Sha256};
use crate::resource::ResourceReference;
use crate::uuid::UuidV4;

/// The type of asset, used for filtering in the asset browser
/// and determining which importer to invoke.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetType {
    Mesh,
    Texture,
    Audio,
    Material,
    Scene,
    Script,
}

/// A single entry in the editor's asset registry.
///
/// This is an editor-only struct — it is never rkyv-archived or packed.
/// It is rebuilt at editor startup by scanning `.eucmeta` files.
/// The serialized form of this struct (written to `.eucmeta`) uses serde.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetEntry {
    /// The stable, permanent identity of this asset.
    /// Never changes even if the file is renamed or moved.
    /// All in-engine references to this asset use this UUID exclusively.
    pub uuid: Uuid,

    /// Display name shown in the editor asset browser.
    /// Derived from filename on first import, but user-overridable.
    pub name: String,

    /// The kind of asset. Used for browser filtering and importer dispatch.
    pub asset_type: AssetType,

    /// Where the source file lives. Purely a location — identity is the UUID.
    pub location: ResourceReference,

    /// Path to the compiled output (e.g. `compiled/meshes/f47ac10b.eucmdl`),
    /// relative to the project root.
    /// `None` for `Embedded` and `Procedural` assets which have no compiled form.
    pub compiled_path: Option<PathBuf>,

    /// SHA-256 (or xxHash) of the source file at last successful import.
    ///
    /// On editor startup, the source is rehashed and compared.
    ///
    /// If it differs, the asset is marked dirty and recompilation is triggered.
    ///
    /// More reliable than `import_time` — timestamps can lie (e.g. git checkouts).
    pub source_hash: [u8; 32],

    /// When the asset was last successfully imported.
    ///
    /// Used for "recently imported" sorting in the browser — not for staleness.
    pub import_time: SystemTime,

    /// UUIDs of other assets this asset directly references.
    ///
    /// e.g. a Mesh entry lists its Material UUIDs; a Material lists its Texture UUIDs.
    pub dependencies: Vec<Uuid>,
}

impl AssetEntry {
    /// Resolves this entry to a `ResourceReference` for use by the asset loader.
    ///
    /// `project_root` is the absolute path to the project's `resources/` directory.
    /// Requires the caller (typically the `AssetRegistry`) to supply it —
    /// `AssetEntry` itself only stores project-relative paths.
    ///
    /// Returns `None` if the location is `Embedded` — use `to_embedded_reference`
    /// instead, supplying the actual bytes.
    pub fn to_reference(&self, project_root: &Path) -> Option<ResourceReference> {
        match &self.location {
            ResourceReference::File(relative) => {
                Some(ResourceReference::File(project_root.join(relative)))
            }
            ResourceReference::Procedural => {
                Some(ResourceReference::Procedural)
            }
            ResourceReference::Embedded(_) | ResourceReference::Packed { .. } => None,
        }
    }

    /// Resolves an `Embedded` asset to a `ResourceReference`.
    /// The caller is responsible for supplying the correct bytes
    /// (typically a `&'static [u8]` from `include_bytes!`).
    pub fn to_embedded_reference(&self, bytes: &'static [u8]) -> ResourceReference {
        ResourceReference::Embedded(Arc::from(bytes))
    }

    /// Returns true if the asset's source file needs to be reimported.
    /// Call this on editor startup after rehashing the source file.
    pub fn is_stale(&self, current_hash: &[u8; 32]) -> bool {
        &self.source_hash != current_hash
    }
}

/// Detects the `AssetType` for a file based on its extension.
/// Returns `None` if the extension is unknown or absent.
pub fn detect_asset_type(path: &Path) -> Option<AssetType> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "obj" | "gltf" | "glb" | "fbx" | "eucmdl" | "eucbin" => Some(AssetType::Mesh),
        "png" | "jpg" | "jpeg" | "webp" | "hdr" | "tga" | "bmp" | "exr" => Some(AssetType::Texture),
        "wav" | "ogg" | "flac" | "mp3" => Some(AssetType::Audio),
        "eucs" => Some(AssetType::Scene),
        "kt" | "kts" => Some(AssetType::Script),
        _ => None,
    }
}

fn hash_file(path: &Path) -> anyhow::Result<[u8; 32]> {
    let bytes = fs::read(path)?;
    let digest = Sha256::digest(&bytes);
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&digest);
    Ok(hash)
}

/// Writes a `.eucmeta` sidecar next to `source_path` and returns the created `AssetEntry`.
///
/// If a sidecar already exists it is read and returned unchanged —
/// the file's UUID and other metadata are preserved across re-imports.
///
/// `source_path` — absolute path to the resource file inside the project.
/// `project_root` — absolute path to the project root, used to store a project-relative
///                  path inside the entry so that the project stays relocatable.
pub fn generate_eucmeta(source_path: &Path, project_root: &Path) -> anyhow::Result<AssetEntry> {
    let meta_path = PathBuf::from(format!("{}.eucmeta", source_path.display()));

    if meta_path.exists() {
        let ron_str = fs::read_to_string(&meta_path)?;
        let entry: AssetEntry = ron::de::from_str(&ron_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse {}: {}", meta_path.display(), e))?;
        return Ok(entry);
    }

    let asset_type = detect_asset_type(source_path)
        .ok_or_else(|| anyhow::anyhow!("Unknown asset type for: {}", source_path.display()))?;

    let name = source_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unnamed")
        .to_string();

    let relative = source_path
        .strip_prefix(project_root)
        .unwrap_or(source_path)
        .to_path_buf();

    let source_hash = hash_file(source_path)?;

    let entry = AssetEntry {
        uuid: Uuid::new_v4(),
        name,
        asset_type,
        location: ResourceReference::File(relative),
        compiled_path: None,
        source_hash,
        import_time: SystemTime::now(),
        dependencies: vec![],
    };

    let ron_str = ron::ser::to_string_pretty(&entry, PrettyConfig::default())
        .map_err(|e| anyhow::anyhow!("RON serialization error: {}", e))?;
    fs::write(&meta_path, &ron_str)?;
    log::info!("Generated .eucmeta for {}", source_path.display());
    Ok(entry)
}

/// The asset type, duplicated here as a lean copy without editor-only variants.
/// Must stay in sync with `AssetType` in the editor crate.
#[derive(Clone, Debug, PartialEq, Eq, Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum PackedAssetType {
    Mesh,
    Texture,
    Audio,
    Material,
    Scene,
    Script,
}

/// A single entry in the runtime packed asset manifest.
///
/// Stored in the header block of a `.eucpak` file, rkyv-archived.
/// The loader uses this to resolve a UUID to a byte range within the pak,
/// then reads and deserializes the asset from those bytes.
///
/// Everything editor-specific (name, source path, hash, import time,
/// thumbnail) is stripped — the runtime has no use for it.
#[derive(Clone, Debug, Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct PackedAssetEntry {
    /// The same UUID as the editor's `AssetEntry`.
    /// This is the only field shared between the two representations.
    pub uuid: UuidV4,

    /// The type of asset. Needed so the loader knows which
    /// deserializer to invoke without having to peek at the bytes.
    pub asset_type: PackedAssetType,

    /// Byte offset from the start of the `.eucpak` data block
    /// where this asset's compiled bytes begin.
    pub offset: u64,

    /// Length in bytes of this asset's compiled data.
    /// `offset + length` is the exclusive end of the asset slice.
    pub length: u64,

    /// UUIDs of assets that must be loaded before this one.
    /// Kept from `AssetEntry::dependencies` for streaming and
    /// load-order resolution at runtime. Can be empty for leaf assets
    /// (e.g. a Texture that references nothing).
    pub dependencies: Vec<UuidV4>,
}

/// Scans all `.eucmeta` files under `<project_root>/resources/` and returns
/// the [`AssetEntry`] whose UUID matches `uuid`.
///
/// Returns an error if no matching entry is found.
pub fn find_asset_by_uuid(project_root: &Path, uuid: Uuid) -> anyhow::Result<AssetEntry> {
    fn scan_dir(dir: &Path, uuid: Uuid) -> Option<AssetEntry> {
        let read = std::fs::read_dir(dir).ok()?;
        for entry in read.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(found) = scan_dir(&path, uuid) {
                    return Some(found);
                }
            } else if path.extension().and_then(|e| e.to_str()) == Some("eucmeta") {
                if let Ok(s) = std::fs::read_to_string(&path) {
                    if let Ok(entry) = ron::de::from_str::<AssetEntry>(&s) {
                        if entry.uuid == uuid {
                            return Some(entry);
                        }
                    }
                }
            }
        }
        None
    }

    let resources_dir = project_root.join("resources");
    scan_dir(&resources_dir, uuid)
        .ok_or_else(|| anyhow::anyhow!("No .eucmeta file found for UUID {}", uuid))
}