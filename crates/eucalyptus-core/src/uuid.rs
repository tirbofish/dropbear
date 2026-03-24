use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A newtype around the raw bytes of a UUID that satisfies rkyv's `Archive` bound.
///
/// The inner `[u8; 16]` is trivially archivable, giving us zero-copy access at
/// runtime.  Use `From`/`Into` to convert to/from the standard `uuid::Uuid` type.
#[derive(
    Clone, Debug, PartialEq, Eq, Hash,
    Archive, RkyvSerialize, RkyvDeserialize,
    Serialize, Deserialize,
)]
#[repr(transparent)]
pub struct UuidV4([u8; 16]);

impl UuidV4 {
    pub fn new_v4() -> Self {
        UuidV4(*Uuid::new_v4().as_bytes())
    }

    pub fn as_uuid(&self) -> Uuid {
        Uuid::from_bytes(self.0)
    }
}

impl From<Uuid> for UuidV4 {
    fn from(u: Uuid) -> Self {
        UuidV4(*u.as_bytes())
    }
}

impl From<UuidV4> for Uuid {
    fn from(v: UuidV4) -> Self {
        Uuid::from_bytes(v.0)
    }
}

impl Default for UuidV4 {
    fn default() -> Self {
        UuidV4([0u8; 16])
    }
}
