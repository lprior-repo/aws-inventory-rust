//! Why bytes are not a record, or a value cannot become one.

use thiserror::Error;

/// A rejected record. Offsets are byte positions from the start of the record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RecordError {
    /// The input ends inside a value.
    #[error("record ends unexpectedly at byte {offset}")]
    UnexpectedEnd { offset: usize },
    /// A tag byte outside the format.
    #[error("unknown tag {tag} at byte {offset}")]
    UnknownTag { offset: usize, tag: u8 },
    /// A varint with a padding continuation byte.
    #[error("varint at byte {offset} is not minimal")]
    NonMinimalVarint { offset: usize },
    /// A varint that does not fit in 64 bits.
    #[error("varint at byte {offset} overflows 64 bits")]
    VarintOverflow { offset: usize },
    /// A length that reaches past the enclosing body.
    #[error("length at byte {offset} exceeds the enclosing body")]
    LengthOutOfBounds { offset: usize },
    /// A container body with bytes left after its last entry.
    #[error("container body has unread bytes at byte {offset}")]
    BodyNotConsumed { offset: usize },
    /// Bytes after the top-level value.
    #[error("bytes follow the record at byte {offset}")]
    TrailingBytes { offset: usize },
    /// Text or a map key that is not UTF-8.
    #[error("invalid UTF-8 at byte {offset}")]
    InvalidUtf8 { offset: usize },
    /// A map key or field id that is not greater than the one before it.
    #[error("entry at byte {offset} is not in strictly ascending order")]
    OrderViolation { offset: usize },
    /// A field id of zero or one that does not fit in 32 bits.
    #[error("field id at byte {offset} is zero or exceeds 32 bits")]
    InvalidFieldId { offset: usize },
    /// Nanoseconds of one second or more.
    #[error("nanoseconds at byte {offset} are not below one second")]
    InvalidNanoseconds { offset: usize },
    /// A NaN with any bit pattern other than the canonical one.
    #[error("NaN at byte {offset} is not canonical")]
    NonCanonicalNan { offset: usize },
    /// Containers nested deeper than the limit.
    #[error("containers nest deeper than the depth limit")]
    DepthExceeded,
    /// A record longer than the limit.
    #[error("record of {len} bytes exceeds the size limit")]
    TooLarge { len: usize },
    /// A top-level value that is not a struct.
    #[error("the top-level value must be a struct")]
    TopLevelNotStruct,
}
