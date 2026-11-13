//! Canonical binary record.
//!
//! A record is what one AWS resource becomes after its response is parsed:
//! the bytes that are hashed to detect change, stored as a blob, and read
//! field by field without decoding the rest. Equal values always encode to
//! equal bytes, and a byte string is either rejected or is exactly the
//! encoding of the value it decodes to. Field names are not stored; structure
//! members are keyed by numeric [`FieldId`]s assigned elsewhere.
//!
//! # Format
//!
//! ```text
//! record  := value                 the top-level value must be a struct
//! value   := tag payload           tag is one byte
//!
//! tag  0  null     no payload
//! tag  1  false    no payload
//! tag  2  true     no payload
//! tag  3  int      zigzag varint of an i64
//! tag  4  float    8 bytes, big-endian IEEE-754 bits; NaN is 0x7FF8_0000_0000_0000
//! tag  5  text     varint byte length, UTF-8 bytes
//! tag  6  bytes    varint byte length, bytes
//! tag  7  time     zigzag varint seconds, varint nanoseconds below 1_000_000_000
//! tag  8  list     varint body length, body: varint count, values
//! tag  9  map      varint body length, body: varint count, (text key, value) pairs,
//!                  keys strictly ascending by bytes
//! tag 10  struct   varint body length, body: varint count, (varint field id, value)
//!                  pairs, ids non-zero and strictly ascending
//! ```
//!
//! Varints are unsigned LEB128, at most ten bytes, and minimal. The body
//! length in front of every container lets a reader skip it without parsing.
//! Containers nest at most [`MAX_DEPTH`] deep and a record is at most
//! [`MAX_RECORD_BYTES`] long.

mod containers;
mod cursor;
mod encode;
mod error;
mod owned;
mod reader;
mod scalar;
mod validate;
mod value;
mod varint;

#[cfg(test)]
mod tests;

pub use containers::{ListIter, ListRef, MapIter, MapRef, StructIter, StructRef};
pub use error::RecordError;
pub use owned::Record;
pub use reader::ValueRef;
pub use value::{FieldId, Float, Time, Value};

/// Deepest allowed container nesting; the top-level struct is depth one.
pub const MAX_DEPTH: usize = 32;
/// Largest allowed record, in bytes.
pub const MAX_RECORD_BYTES: usize = 16_777_216;

/// One-byte value tags of the wire format.
mod tag {
    pub(super) const NULL: u8 = 0;
    pub(super) const FALSE: u8 = 1;
    pub(super) const TRUE: u8 = 2;
    pub(super) const INT: u8 = 3;
    pub(super) const FLOAT: u8 = 4;
    pub(super) const TEXT: u8 = 5;
    pub(super) const BYTES: u8 = 6;
    pub(super) const TIME: u8 = 7;
    pub(super) const LIST: u8 = 8;
    pub(super) const MAP: u8 = 9;
    pub(super) const STRUCT: u8 = 10;
}
