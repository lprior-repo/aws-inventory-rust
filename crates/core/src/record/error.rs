//! Typed boundary, format, and domain failures.
use minicbor::data::Type;
use std::error::Error;

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
    NotOneItem { offset: usize },
    /// Truncated input.
    #[error("missing value at {offset}")]
    Missing { offset: usize },
    /// Invalid generic CBOR.
    #[error("malformed CBOR at {offset}")]
    Malformed { offset: usize },
    /// Unsigned value exceeds its domain.
    #[error("unsigned integer out of range at {offset}")]
    UnsignedOutOfRange { offset: usize },
    /// Negative value exceeds i64.
    #[error("signed integer out of range at {offset}")]
    SignedOutOfRange { offset: usize },
    /// Unsupported float width.
    #[error("float {type_name} not allowed at {offset}")]
    FloatType {
        type_name: &'static str,
        offset: usize,
    },
    /// Unsupported map key type.
    #[error("map key {found} not allowed at {offset}")]
    KeyType { found: Type, offset: usize },
    /// Unsupported value type.
    #[error("value type {found} not allowed at {offset}")]
    ValueType { found: Type, offset: usize },
    /// Zero cannot identify a field.
    #[error("field ID zero at {offset}")]
    FieldIdZero { offset: usize },
    /// Nanoseconds must be below one billion.
    #[error("nanoseconds {nanos} out of range at {offset}")]
    NanosOutOfRange { nanos: u64, offset: usize },
    /// Invalid text, retaining the library's actual UTF-8 cause.
    #[error("text is not UTF-8 at {offset}: {source}")]
    NotUtf8 {
        offset: usize,
        source: std::str::Utf8Error,
    },
    /// Unsupported tag.
    #[error("tag {tag} not allowed at {offset}")]
    Tag { tag: u64, offset: usize },
    /// Unsupported indefinite-length form.
    #[error("indefinite length at {offset}")]
    IndefiniteLength { offset: usize },
    /// Tag 1 did not contain a two-element array.
    #[error("invalid time representation at {offset}")]
    TimeShape { offset: usize },
    /// Nanoseconds were negative, not malformed CBOR.
    #[error("negative nanoseconds {nanos} at {offset}")]
    NegativeNanos { nanos: i128, offset: usize },
    /// Structural validation could not reserve its input-bounded stack.
    #[error("CBOR validation stack allocation failed")]
    Allocation,
    /// A non-writer minicbor encoding failure.
    #[error("in-memory CBOR encoding failed")]
    Encode,
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
