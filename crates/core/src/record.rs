//! Binary record encoding as CBOR.
//!
//! Encoding follows the fixed forms below; equal values produce equal bytes.
//! `from_bytes` checks generic CBOR structure, not supported forms or depth.
//! Readers reject unsupported forms; lookups rely on the encoder's sorted keys.
//! The depth bound includes timestamp arrays, but not their tag headers.
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

mod containers;
mod encode;
mod error;
mod iter;
mod readers;
mod skip;
mod structure;
#[cfg(test)]
mod tests;
mod value;

pub use containers::{ListRef, MapRef};
pub use error::RecordError;
pub use value::{FieldId, Float, Key, KeyRef, Time, Value, ValueRef};

/// Maximum record size: 16 MiB, inclusive.
pub const MAX_RECORD_BYTES: usize = 16_777_216;
/// Maximum nested containers, including the root map.
pub const MAX_DEPTH: usize = 32;

/// Owned bytes containing exactly one structurally valid CBOR map.
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
    /// Validate generic CBOR, deferring supported-form checks to readers.
    /// # Errors
    /// Rejects oversized, malformed, non-map, or trailing input.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, RecordError> {
        if bytes.len() > MAX_RECORD_BYTES {
            return Err(RecordError::SizeLimit);
        }
        let mut decoder = minicbor::Decoder::new(&bytes);
        if !matches!(
            decoder.datatype()?,
            minicbor::data::Type::Map | minicbor::data::Type::MapIndef
        ) {
            return Err(RecordError::NotMap);
        }
        structure::validate(&mut decoder)?;
        if decoder.position() != bytes.len() {
            return Err(RecordError::NotOneItem {
                offset: decoder.position(),
            });
        }
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
}

fn container_depth(depth: usize) -> Result<usize, RecordError> {
    depth
        .checked_add(1)
        .filter(|next| *next <= MAX_DEPTH)
        .ok_or(RecordError::DepthLimit)
}
