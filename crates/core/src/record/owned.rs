//! The owned record: bytes that are known to be canonical.

use super::containers::StructRef;
use super::cursor::Cursor;
use super::error::RecordError;
use super::value::Value;
use super::{encode, validate};

/// Canonical record bytes. Every `Record` is valid: it can only be built by
/// encoding a value or by validating bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Record(Vec<u8>);

impl Record {
    /// Encodes a top-level struct.
    ///
    /// # Errors
    /// The value is not a struct, nests deeper than
    /// [`MAX_DEPTH`](super::MAX_DEPTH), or encodes to more than
    /// [`MAX_RECORD_BYTES`](super::MAX_RECORD_BYTES).
    pub fn encode(value: &Value) -> Result<Self, RecordError> {
        encode::encode(value).map(Self)
    }

    /// Takes ownership of `bytes` if they are a canonical record.
    ///
    /// # Errors
    /// The first rule of the format that `bytes` break, with its offset.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, RecordError> {
        validate::validate(&bytes)?;
        Ok(Self(bytes))
    }

    /// The canonical bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Gives up the canonical bytes.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }

    /// The top-level struct, read in place.
    #[must_use]
    pub fn root(&self) -> StructRef<'_> {
        let mut cursor = Cursor::new(&self.0);
        let body = cursor.byte().ok().and_then(|_| cursor.blob().ok());
        StructRef::new(body.unwrap_or(&[]))
    }
}
