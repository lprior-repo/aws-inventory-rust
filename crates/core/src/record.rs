//! Binary record encoding as CBOR.
//!
//! Encoding follows the fixed forms below; equal values produce equal bytes.
//! `from_bytes` checks CBOR framing and strictly ascending supported keys in
//! reader-reachable maps. Unknown values remain skippable; readers check their forms.
//! The writer's depth bound includes timestamp arrays, but not their tag headers.
//!
//! # Encoding table
//!
//! | Value | CBOR written |
//! |---|---|
//! | null | `null` |
//! | boolean | `true` / `false` |
//! | integer (i64) | major type 0 or 1, shortest form |
//! | float | always the 64-bit form; every NaN as bits `0x7FF8_0000_0000_0000` |
//! | text | definite-length text string |
//! | bytes | definite-length byte string |
//! | time | tag 1, then array of two: seconds (i64), nanoseconds (u32) |
//! | list | definite-length array |
//! | map | definite-length map; field-ID keys first ascending, then name keys ascending |
//!
//! Nothing else is ever written: no indefinite lengths, no other tags, no
//! 16-bit or 32-bit floats.
//!
//! # Limits
//!
//! - Nesting depth at most 32 containers.
//! - A record at most 16 MiB (16,777,216 bytes).
//!
//! Generic structural readers retain deep unknown values. `MapRef::from_bytes_bounded`
//! additionally enforces 32 container frames, including containers inside opaque values.

mod containers;
mod encode;
mod error;
mod exclusions;
mod hash;
mod iter;
mod readers;
mod skip;
mod structure;
#[cfg(test)]
mod tests;
mod value;

pub use containers::{ListRef, MapRef};
pub use error::RecordError;
pub use exclusions::Exclusions;
pub use hash::ContentHash;
pub use value::{FieldId, Float, Key, KeyRef, Time, Value, ValueKind, ValueRef};

/// Maximum record size: 16 MiB, inclusive.
pub const MAX_RECORD_BYTES: usize = 16_777_216;
/// Maximum nested containers, including the root map.
pub const MAX_DEPTH: usize = 32;
/// Record encoding format version. Version 1 is the bare CBOR root map.
pub const FORMAT_VERSION: u8 = 1;

/// Owned CBOR map bytes with supported, strictly ascending keys in reachable maps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record(Vec<u8>);

impl Record {
    /// Encode the fixed forms in the encoding table.
    /// # Errors
    /// Returns `NotMap`, `DepthLimit`, `SizeLimit`, or an encoding error.
    pub fn encode(value: &Value) -> Result<Self, RecordError> {
        if !matches!(value, Value::Map(_)) {
            return Err(RecordError::NotMap);
        }
        encode::encode(value).map(Self)
    }
    /// Validate CBOR framing and supported, strictly ascending reachable map keys.
    /// Tags and indefinite containers remain opaque; readers check selected value forms.
    /// # Errors
    /// Rejects oversized, malformed, non-map, trailing input, or invalid reachable keys.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, RecordError> {
        validate(&bytes)?;
        Ok(Self(bytes))
    }
    /// Borrow the encoded bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    /// Consume the record and recover its bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
    /// Borrow the root map, without allocation.
    pub fn root(&self) -> MapRef<'_> {
        MapRef { bytes: &self.0 }
    }

    /// Compute the BLAKE3 content hash, excluding the given field-ID paths.
    /// Two records that differ only inside excluded entries, or in whether an
    /// excluded entry is present, have equal hashes. Equal values give equal
    /// hashes whatever integer widths their bytes use; key order is as stored.
    /// # Errors
    /// Returns a reader error for unsupported forms or `DepthLimit` past 32.
    /// The value of an excluded entry is still read once: an unsupported
    /// scalar there is an error, while the inside of an excluded container
    /// is skipped unread.
    pub fn content_hash(&self, exclusions: &Exclusions) -> Result<ContentHash, RecordError> {
        hash::compute(self, exclusions)
    }
}

fn container_depth(depth: usize) -> Result<usize, RecordError> {
    depth
        .checked_add(1)
        .filter(|next| *next <= MAX_DEPTH)
        .ok_or(RecordError::DepthLimit)
}

fn validate(bytes: &[u8]) -> Result<(), RecordError> {
    validate_with_depth(bytes, None)
}

fn validate_with_depth(bytes: &[u8], limit: Option<usize>) -> Result<(), RecordError> {
    if bytes.len() > MAX_RECORD_BYTES {
        return Err(RecordError::SizeLimit);
    }
    let mut decoder = minicbor::Decoder::new(bytes);
    if !matches!(
        decoder.datatype()?,
        minicbor::data::Type::Map | minicbor::data::Type::MapIndef
    ) {
        return Err(RecordError::NotMap);
    }
    structure::validate(&mut decoder, limit)?;
    if decoder.position() != bytes.len() {
        return Err(RecordError::NotOneItem {
            offset: decoder.position(),
        });
    }
    Ok(())
}
