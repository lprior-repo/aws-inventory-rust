use minicbor::data::Type;
use std::error::Error;

use super::{FieldId, ValueKind};

#[derive(Clone, Copy, Debug, thiserror::Error, PartialEq, Eq)]
pub enum RecordError {
    #[error("top level is not a map")]
    NotMap,

    #[error("nesting depth limit exceeded")]
    DepthLimit,

    #[error("record size limit exceeded")]
    SizeLimit,

    #[error("trailing input at {offset}")]
    NotOneItem { offset: usize },

    #[error("missing value at {offset}")]
    Missing { offset: usize },

    #[error("malformed CBOR at {offset}")]
    Malformed { offset: usize },

    #[error("unsigned integer out of range at {offset}")]
    UnsignedOutOfRange { offset: usize },

    #[error("signed integer out of range at {offset}")]
    SignedOutOfRange { offset: usize },

    #[error("float {type_name} not allowed at {offset}")]
    FloatType {
        type_name: &'static str,

        offset: usize,
    },

    #[error("map key {found} not allowed at {offset}")]
    KeyType { found: Type, offset: usize },

    #[error("map key out of order at {offset}")]
    KeyOutOfOrder { offset: usize },

    #[error("duplicate map key at {offset}")]
    DuplicateKey { offset: usize },

    #[error("value type {found} not allowed at {offset}")]
    ValueType { found: Type, offset: usize },

    #[error("field ID zero at {offset}")]
    FieldIdZero { offset: usize },

    #[error("nanoseconds {nanos} out of range at {offset}")]
    NanosOutOfRange { nanos: u64, offset: usize },

    #[error("text is not UTF-8 at {offset}: {source}")]
    NotUtf8 {
        offset: usize,

        source: std::str::Utf8Error,
    },

    #[error("tag {tag} not allowed at {offset}")]
    Tag { tag: u64, offset: usize },

    #[error("indefinite length at {offset}")]
    IndefiniteLength { offset: usize },

    #[error("invalid time representation at {offset}")]
    TimeShape { offset: usize },

    #[error("negative nanoseconds {nanos} at {offset}")]
    NegativeNanos { nanos: i128, offset: usize },

    #[error("CBOR validation stack allocation failed")]
    Allocation,

    #[error("in-memory CBOR encoding failed")]
    Encode,

    #[error("exclusion path length {length} outside 1..=32")]
    ExclusionPath { length: usize },

    #[error("field {} expected {expected:?}, found {found:?}", field.value())]
    KindMismatch {
        field: FieldId,

        expected: ValueKind,

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
