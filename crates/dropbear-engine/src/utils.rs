//! Utilities and helper functions for the dropbear renderer.

use crate::procedural::ProcedurallyGeneratedObject;
use rkyv::Archive;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::sync::Arc;

pub const EUCA_SCHEME: &str = "euca://";

pub const INTERNAL_MODELS: &[&str] = &["cube"];

/// Converts any supported resource reference into the canonical `euca://` form.
///
/// The function trims whitespace, normalizes path separators, ensures the scheme
/// prefix is present, and collapses redundant separators. Legacy strings without
/// the scheme are accepted and automatically upgraded to the canonical form.
///
/// # Examples
/// ```ignore
/// let canonical = dropbear_engine::utils::canonicalize_euca_uri("textures/diffuse.png").unwrap();
/// assert_eq!(canonical, "euca://textures/diffuse.png");
/// ```
pub fn canonicalize_euca_uri(uri: &str) -> anyhow::Result<String> {
    let trimmed = uri.trim();
    if trimmed.is_empty() {
        anyhow::bail!("euca URI cannot be empty");
    }

    let normalized = trimmed.replace('\\', "/");
    let (had_scheme, without_scheme) = if let Some(rest) = normalized.strip_prefix(EUCA_SCHEME) {
        (true, rest)
    } else {
        (false, normalized.as_str())
    };

    let stripped = without_scheme.trim_start_matches('/');
    if stripped.is_empty() {
        anyhow::bail!("euca URI '{}' must contain a resource path", uri);
    }

    let mut clean = stripped
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("/");

    if let Some(rest) = clean.strip_prefix("resources/") {
        clean = rest.to_string();
    }

    if clean.is_empty() {
        anyhow::bail!("euca URI '{}' must contain a resource path", uri);
    }

    if !had_scheme {
        log::debug!(
            "Canonicalized legacy resource reference '{}' to '{}{}'",
            uri,
            EUCA_SCHEME,
            clean
        );
    }

    Ok(format!("{EUCA_SCHEME}{clean}"))
}

pub fn relative_path_from_euca(uri: &str) -> anyhow::Result<&str> {
    let without_scheme = uri.strip_prefix(EUCA_SCHEME).unwrap_or(uri);

    let stripped = without_scheme.trim_start_matches('/');
    if stripped.is_empty() {
        anyhow::bail!("euca URI '{}' must contain a resource path", uri);
    }

    Ok(stripped.strip_prefix("resources/").unwrap_or(stripped))
}

/// A reference to an asset used to identify and load it.
///
/// `File` holds a path relative to the project's `resources/` directory
/// (e.g. `"models/cube.glb"`).  `Embedded` carries raw bytes.
/// `Procedural` stores the procedurally generated geometry directly.
///
/// An empty `File("")` acts as a "no asset" sentinel wherever an
/// `Option<ResourceReference>` is not available (e.g. rkyv-archived structs).
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum ResourceReference {
    /// A resource-root-relative file path, e.g. `"models/cube.glb"`.
    File(String),
    /// Raw bytes embedded in memory (e.g. from `include_bytes!`).
    Embedded(Arc<[u8]>),
    /// A procedurally generated object with embedded geometry.
    Procedural(ProcedurallyGeneratedObject),
}

impl Default for ResourceReference {
    fn default() -> Self {
        // Empty path = "no asset" sentinel.
        Self::File(String::new())
    }
}

impl Display for ResourceReference {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File(path) if path.is_empty() => write!(f, "File(<none>)"),
            Self::File(path) => write!(f, "File({path})"),
            Self::Embedded(bytes) => write!(f, "Embedded({} bytes)", bytes.len()),
            Self::Procedural(_) => write!(f, "Procedural"),
        }
    }
}

impl ResourceReference {
    /// Creates a `ResourceReference::File` with a resource-root-relative path.
    pub fn file(path: impl Into<String>) -> Self {
        Self::File(path.into())
    }

    /// Creates a bytes-backed reference.
    pub fn from_bytes(bytes: impl AsRef<[u8]>) -> Self {
        Self::Embedded(Arc::<[u8]>::from(bytes.as_ref()))
    }

    /// Returns true when this reference points to a non-empty file path.
    pub fn is_file_backed(&self) -> bool {
        matches!(self, Self::File(s) if !s.is_empty())
    }

    /// Creates a `ResourceReference` from a full absolute path by extracting
    /// the component after `resources/`.
    ///
    /// # Examples
    /// ```
    /// use dropbear_engine::utils::ResourceReference;
    ///
    /// let path = "/home/tk/project/resources/models/cube.obj";
    /// let r = ResourceReference::from_path(path).unwrap();
    /// assert_eq!(r.relative_path(), Some("models/cube.obj"));
    /// ```
    pub fn from_path(full_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        puffin::profile_function!(full_path.as_ref().display().to_string());
        let path = full_path.as_ref();

        let components: Vec<_> = path.components().collect();
        for (i, component) in components.iter().enumerate() {
            if let std::path::Component::Normal(name) = component
                && *name == "resources"
            {
                let remaining = &components[i + 1..];
                if remaining.is_empty() {
                    anyhow::bail!(
                        "Path has no components after 'resources/': {}",
                        path.display()
                    );
                }
                let resource_path = remaining
                    .iter()
                    .map(|c| match c {
                        std::path::Component::Normal(n) => n.to_str().unwrap_or(""),
                        _ => "",
                    })
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join("/");

                return Ok(Self::File(resource_path));
            }
        }

        anyhow::bail!(
            "Path does not contain a 'resources' component: {}",
            path.display()
        )
    }

    /// Creates a `ResourceReference` directly from an euca URI
    /// (e.g. `euca://models/cube.glb`).
    ///
    /// Strips the `euca://` scheme and any `resources/` prefix, storing
    /// only the resource-root-relative path.
    pub fn from_euca_uri(uri: impl AsRef<str>) -> anyhow::Result<Self> {
        let canonical = canonicalize_euca_uri(uri.as_ref())?;
        // canonical is "euca://models/cube.glb" — resources/ is already stripped.
        let relative = canonical.strip_prefix(EUCA_SCHEME).unwrap_or(&canonical);
        Ok(Self::File(relative.to_string()))
    }

    /// Returns the resource-root-relative path for `File` references.
    /// Returns `None` for empty paths and all other variants.
    pub fn as_uri(&self) -> Option<&str> {
        match self {
            Self::File(s) if !s.is_empty() => Some(s.as_str()),
            _ => None,
        }
    }

    /// Returns the resource path relative to the `resources/` directory.
    /// Alias for [`as_uri`].
    pub fn relative_path(&self) -> Option<&str> {
        self.as_uri()
    }

    /// Returns the raw bytes for `Embedded` references.
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Embedded(bytes) => Some(bytes.as_ref()),
            _ => None,
        }
    }
}

/// Neat lil macro to create a resource reference easier
#[macro_export]
macro_rules! resource {
    ($path:expr) => {
        ::dropbear_engine::utils::ResourceReference::from_euca_uri($path).expect("Invalid euca URI")
    };
}

/// Helper trait for converting `Option<T: ToString>` to [`Option<String>`] without looking into its contents.
pub trait ToPotentialString {
    /// Converts an [`Option<T>`], where [`T`] can be converted to a [`String`], into an [`Option<String>`].
    fn to_potential_string(&self) -> Option<String>;
}

impl<T> ToPotentialString for Option<T>
where
    T: ToString,
{
    fn to_potential_string(&self) -> Option<String> {
        match self {
            None => None,
            Some(v) => Some(v.to_string()),
        }
    }
}
