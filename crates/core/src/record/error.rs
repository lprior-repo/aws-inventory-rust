//! Typed boundary, format, and domain failures.
use minicbor::data::Type;
use std::error::Error;

use super::{FieldId, ValueKind};

/// A record operation failed; offsets are relative to the current borrowed view.
#[derive(Clone, Copy, Debug, thiserror::Error, PartialEq, Eq)]
pub enum RecordError {
    /// The root is not a map.
    #[error("top level is not a map")]
    NotMap,
    /// More than 32 containers.
    #[error("nesting depth limit exceeded")]
    DepthLimit,
    /// More than 16 MiB.
    #[error("record size limit exceeded")]
    SizeLimit,
    /// A second item or trailing bytes.
    #[error("trailing input at {offset}")]
    NotOneItem {
        /// Byte offset where the trailing input begins in the current borrowed view.
        offset: usize,
    },
    /// Truncated input.
    #[error("missing value at {offset}")]
    Missing {
        /// Byte offset in the current borrowed view; 0 if the decoder supplies none.
        offset: usize,
    },
    /// Invalid generic CBOR.
    #[error("malformed CBOR at {offset}")]
    Malformed {
        /// Byte offset in the current borrowed view; 0 if the decoder supplies none.
        offset: usize,
    },
    /// Unsigned value exceeds its domain.
    #[error("unsigned integer out of range at {offset}")]
    UnsignedOutOfRange {
        /// Byte offset of the unsigned value in the current borrowed view.
        offset: usize,
    },
    /// Negative value exceeds i64.
    #[error("signed integer out of range at {offset}")]
    SignedOutOfRange {
        /// Byte offset of the signed value in the current borrowed view.
        offset: usize,
    },
    /// Unsupported float width.
    #[error("float {type_name} not allowed at {offset}")]
    FloatType {
        /// Rejected floating-point width label: `f16` or `f32`.
        type_name: &'static str,
        /// Byte offset of the float in the current borrowed view.
        offset: usize,
    },
    /// Unsupported map key type.
    #[error("map key {found} not allowed at {offset}")]
    KeyType {
        /// CBOR type encountered where a supported map key was required.
        found: Type,
        /// Byte offset of the key in the current borrowed view.
        offset: usize,
    },
    /// A reachable map key is smaller than its predecessor; offset starts this key.
    #[error("map key out of order at {offset}")]
    KeyOutOfOrder {
        /// Byte offset of the out-of-order key in the current borrowed view.
        offset: usize,
    },
    /// A reachable map repeats its predecessor's key; offset starts the repeated key.
    #[error("duplicate map key at {offset}")]
    DuplicateKey {
        /// Byte offset of the repeated key in the current borrowed view.
        offset: usize,
    },
    /// Unsupported value type.
    #[error("value type {found} not allowed at {offset}")]
    ValueType {
        /// CBOR type encountered where a supported value was required.
        found: Type,
        /// Byte offset of the value in the current borrowed view.
        offset: usize,
    },
    /// Zero cannot identify a field.
    #[error("field ID zero at {offset}")]
    FieldIdZero {
        /// Byte offset of the key, or 0 when constructing a field ID without input bytes.
        offset: usize,
    },
    /// Nanoseconds must be below one billion.
    #[error("nanoseconds {nanos} out of range at {offset}")]
    NanosOutOfRange {
        /// Rejected fractional-second value in nanoseconds, at least one billion.
        nanos: u64,
        /// Byte offset of the fraction, or 0 when constructing a time without input bytes.
        offset: usize,
    },
    /// Invalid text, retaining the library's actual UTF-8 cause.
    #[error("text is not UTF-8 at {offset}: {source}")]
    NotUtf8 {
        /// Byte offset in the current borrowed view; 0 if the decoder supplies none.
        offset: usize,
        /// UTF-8 decoding failure retained from minicbor, not a synthesized replacement.
        source: std::str::Utf8Error,
    },
    /// Unsupported tag.
    #[error("tag {tag} not allowed at {offset}")]
    Tag {
        /// Unsupported CBOR tag number.
        tag: u64,
        /// Byte offset of the tag header in the current borrowed view.
        offset: usize,
    },
    /// Unsupported indefinite-length form.
    #[error("indefinite length at {offset}")]
    IndefiniteLength {
        /// Byte offset of the indefinite-length header in the current borrowed view.
        offset: usize,
    },
    /// Tag 1 did not contain a two-element array.
    #[error("invalid time representation at {offset}")]
    TimeShape {
        /// Byte offset of the time tag header in the current borrowed view.
        offset: usize,
    },
    /// Nanoseconds were negative, not malformed CBOR.
    #[error("negative nanoseconds {nanos} at {offset}")]
    NegativeNanos {
        /// Rejected negative fractional-second value in nanoseconds.
        nanos: i128,
        /// Byte offset of the fraction in the current borrowed view.
        offset: usize,
    },
    /// Structural validation could not reserve its input-bounded stack.
    #[error("CBOR validation stack allocation failed")]
    Allocation,
    /// A non-writer minicbor encoding failure.
    #[error("in-memory CBOR encoding failed")]
    Encode,
    /// An exclusion path is too short or too long.
    #[error("exclusion path length {length} outside 1..=32")]
    ExclusionPath {
        /// Rejected number of field IDs in the path, outside `1..=32`.
        length: usize,
    },
    /// A field has a different value kind than expected.
    #[error("field {} expected {expected:?}, found {found:?}", field.value())]
    KindMismatch {
        /// Field requested by the typed lookup.
        field: FieldId,
        /// Value kind required by the caller.
        expected: ValueKind,
        /// Value kind actually read from the field.
        found: ValueKind,
    },
}

impl From<minicbor::decode::Error> for RecordError {
    fn from(err: minicbor::decode::Error) -> Self {
        let offset = *err.position().get_or_insert(0);
        if let Some(source) = err
            .source()
            .and_then(|e| e.downcast_ref::<std::str::Utf8Error>())
        {
            return Self::NotUtf8 {
                offset,
                source: *source,
            };
        }
        if err.is_end_of_input() {
            Self::Missing { offset }
        } else {
            Self::Malformed { offset }
        }
    }
}

impl From<minicbor::encode::Error<RecordError>> for RecordError {
    fn from(error: minicbor::encode::Error<RecordError>) -> Self {
        let Some(error) = error.into_write() else {
            return Self::Encode;
        };
        error
    }
}

impl From<minicbor::encode::Error<std::convert::Infallible>> for RecordError {
    fn from(_: minicbor::encode::Error<std::convert::Infallible>) -> Self {
        Self::Encode
    }
}
