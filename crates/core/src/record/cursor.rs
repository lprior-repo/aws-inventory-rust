//! Bounds-checked forward reader over record bytes.

use super::error::RecordError;
use super::varint::unzigzag;

/// Forward-only view of record bytes that knows its absolute offset.
#[derive(Debug, Clone, Copy)]
pub(super) struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    /// A cursor at the start of `bytes`.
    pub(super) const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    /// Absolute position of the next unread byte.
    pub(super) const fn offset(&self) -> usize {
        self.offset
    }

    /// Whether every byte has been read.
    pub(super) const fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Reads one byte.
    pub(super) fn byte(&mut self) -> Result<u8, RecordError> {
        let (first, rest) = self.bytes.split_first().ok_or(self.end())?;
        self.bytes = rest;
        self.offset = self.offset.saturating_add(1);
        Ok(*first)
    }

    /// Reads exactly `len` bytes.
    pub(super) fn take(&mut self, len: usize) -> Result<&'a [u8], RecordError> {
        let (head, rest) = self.bytes.split_at_checked(len).ok_or(self.end())?;
        self.bytes = rest;
        self.offset = self.offset.saturating_add(len);
        Ok(head)
    }

    /// Reads a minimal unsigned LEB128 varint.
    pub(super) fn varint(&mut self) -> Result<u64, RecordError> {
        let offset = self.offset;
        let mut value = 0u64;
        for shift in (0u32..64).step_by(7) {
            let byte = self.byte()?;
            let low = u64::from(byte & 0x7f);
            if shift == 63 && low > 1 {
                break;
            }
            value |= low.checked_shl(shift).unwrap_or(0);
            if byte & 0x80 == 0 {
                let padded = byte == 0 && shift != 0;
                return if padded {
                    Err(RecordError::NonMinimalVarint { offset })
                } else {
                    Ok(value)
                };
            }
        }
        Err(RecordError::VarintOverflow { offset })
    }

    /// Reads a zigzag varint.
    pub(super) fn zigzag(&mut self) -> Result<i64, RecordError> {
        self.varint().map(unzigzag)
    }

    /// Reads a length and checks that many bytes remain.
    pub(super) fn length(&mut self) -> Result<usize, RecordError> {
        let offset = self.offset;
        let len = self.varint()?;
        usize::try_from(len)
            .ok()
            .filter(|len| *len <= self.bytes.len())
            .ok_or(RecordError::LengthOutOfBounds { offset })
    }

    /// Reads a length-prefixed byte string.
    pub(super) fn blob(&mut self) -> Result<&'a [u8], RecordError> {
        let len = self.length()?;
        self.take(len)
    }

    /// Reads a length-prefixed container body as its own cursor.
    pub(super) fn body(&mut self) -> Result<Self, RecordError> {
        let len = self.length()?;
        let offset = self.offset;
        let bytes = self.take(len)?;
        Ok(Self { bytes, offset })
    }

    /// The error for running out of input here.
    const fn end(&self) -> RecordError {
        RecordError::UnexpectedEnd {
            offset: self.offset.saturating_add(self.bytes.len()),
        }
    }
}
