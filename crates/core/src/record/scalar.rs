//! Typed scalar reads shared by validation and the borrowed readers.

use super::cursor::Cursor;
use super::error::RecordError;
use super::value::{FieldId, Float, Time};

impl<'a> Cursor<'a> {
    /// Reads a float and rejects a non-canonical NaN.
    pub(super) fn float(&mut self) -> Result<Float, RecordError> {
        let offset = self.offset();
        let bytes = self.take(8)?;
        <[u8; 8]>::try_from(bytes)
            .ok()
            .and_then(|bytes| Float::from_canonical_bits(u64::from_be_bytes(bytes)))
            .ok_or(RecordError::NonCanonicalNan { offset })
    }

    /// Reads a time and rejects nanoseconds of one second or more.
    pub(super) fn time(&mut self) -> Result<Time, RecordError> {
        let seconds = self.zigzag()?;
        let offset = self.offset();
        let nanos = self.varint()?;
        u32::try_from(nanos)
            .ok()
            .and_then(|nanos| Time::new(seconds, nanos))
            .ok_or(RecordError::InvalidNanoseconds { offset })
    }

    /// Reads length-prefixed UTF-8 text.
    pub(super) fn text(&mut self) -> Result<&'a str, RecordError> {
        let len = self.length()?;
        let offset = self.offset();
        let bytes = self.take(len)?;
        std::str::from_utf8(bytes).map_err(|_| RecordError::InvalidUtf8 { offset })
    }

    /// Reads a field id and rejects zero and values beyond 32 bits.
    pub(super) fn field_id(&mut self) -> Result<FieldId, RecordError> {
        let offset = self.offset();
        let id = self.varint()?;
        u32::try_from(id)
            .ok()
            .and_then(FieldId::new)
            .ok_or(RecordError::InvalidFieldId { offset })
    }
}
