use serde::{Deserialize, Serialize};

/// Build profile this bundle was packed with.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildProfile {
    Release,
    Debug,
}

impl std::fmt::Display for BuildProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildProfile::Release => write!(f, "release"),
            BuildProfile::Debug => write!(f, "debug"),
        }
    }
}

/// Target platform a native library entry is intended for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    Windows,
    Linux,
    MacOs,
    /// Included on all platforms (e.g. pure-bytecode JARs).
    All,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::Windows => write!(f, "windows"),
            Platform::Linux => write!(f, "linux"),
            Platform::MacOs => write!(f, "macos"),
            Platform::All => write!(f, "all"),
        }
    }
}

/// CPU architecture a native library was compiled for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Arch {
    X64,
    Arm64,
}

impl std::fmt::Display for Arch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Arch::X64 => write!(f, "x64"),
            Arch::Arm64 => write!(f, "arm64"),
        }
    }
}

/// A native shared library entry inside the bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeLib {
    /// Path relative to the bundle root, e.g. `"libs/linux/x64/libplugin.so"`.
    pub path: String,
    pub platform: Platform,
    pub arch: Arch,
}

/// The category of a bundled asset — used by the runtime to route loading.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetKind {
    Texture,
    Font,
    Shader,
    Audio,
    Data,
}

/// A single asset entry inside the bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleAssetEntry {
    /// Path relative to the bundle root, e.g. `"assets/textures/albedo.png"`.
    pub path: String,
    pub kind: AssetKind,
    /// Whether this asset is stored with deflate compression in the ZIP.
    ///
    /// Defaults to `true` for Release (texture compression is a separate pipeline step),
    /// `false` for Debug (faster iteration, no re-compression on every pack).
    pub compressed: bool,
}

/// A dependency on another plugin, by name and semver requirement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleDependency {
    pub name: String,
    pub version: semver::VersionReq,
}

/// The manifest stored as `manifest.eucc` at the root of a `.eucplugin` bundle.
///
/// This is the authoritative description of a plugin bundle: its identity,
/// contained files, and ABI requirements for the runtime loader.
///
/// Written and verified by `currawong pack` / `currawong unpack`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleManifest {
    /// Plugin identifier — lowercase, hyphenated (e.g. `"surface-nets"`).
    pub name: String,
    pub version: semver::Version,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub license: Option<String>,

    /// Build profile this bundle was packed with.
    pub profile: BuildProfile,

    /// Minimum engine API version required to load this bundle.
    /// The runtime rejects bundles whose requirement is not satisfied.
    pub engine_api_version: semver::VersionReq,

    /// Native shared libraries (`.so` / `.dll` / `.dylib`).
    pub native_libs: Vec<NativeLib>,

    /// Path to the JVM JAR, relative to the bundle root (e.g. `"lib/plugin.jar"`).
    pub jar: Option<String>,

    /// Asset entries shipped with this plugin.
    pub assets: Vec<BundleAssetEntry>,

    /// Other plugins this plugin depends on.
    pub dependencies: Vec<BundleDependency>, // i need to figure out how to resolve this better.

    /// SHA-256 hex digest of all non-manifest bundle contents (set by `currawong pack`).
    /// `None` if the bundle was not packed with integrity verification enabled.
    pub content_hash: Option<String>,
}

impl BundleManifest {
    /// Returns the conventional output filename for this bundle.
    ///
    /// Example: `"surface-nets-1.0.0-release.eucplugin"`
    pub fn output_filename(&self) -> String {
        format!("{}-{}-{}.eucplugin", self.name, self.version, self.profile)
    }
}
