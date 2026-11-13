//! One allocation-free pass that decides whether bytes are a record.

use super::cursor::Cursor;
use super::error::RecordError;
use super::{MAX_DEPTH, MAX_RECORD_BYTES, tag};

/// Accepts `bytes` only if they are the canonical encoding of some record.
pub(super) fn validate(bytes: &[u8]) -> Result<(), RecordError> {
    if bytes.len() > MAX_RECORD_BYTES {
        return Err(RecordError::TooLarge { len: bytes.len() });
    }
    let mut cursor = Cursor::new(bytes);
    if cursor.byte()? != tag::STRUCT {
        return Err(RecordError::TopLevelNotStruct);
    }
    validate_struct(&mut cursor, 1)?;
    if cursor.is_empty() {
        Ok(())
    } else {
        Err(RecordError::TrailingBytes {
            offset: cursor.offset(),
        })
    }
}

/// Checks one value; `depth` is the depth a container here would have.
fn validate_value(cursor: &mut Cursor<'_>, depth: usize) -> Result<(), RecordError> {
    let offset = cursor.offset();
    match cursor.byte()? {
        tag::NULL | tag::FALSE | tag::TRUE => Ok(()),
        tag::INT => cursor.varint().map(drop),
        tag::FLOAT => cursor.float().map(drop),
        tag::TEXT => cursor.text().map(drop),
        tag::BYTES => cursor.blob().map(drop),
        tag::TIME => cursor.time().map(drop),
        tag::LIST => validate_list(cursor, depth),
        tag::MAP => validate_map(cursor, depth),
        tag::STRUCT => validate_struct(cursor, depth),
        tag => Err(RecordError::UnknownTag { offset, tag }),
    }
}

fn validate_list(cursor: &mut Cursor<'_>, depth: usize) -> Result<(), RecordError> {
    let (count, mut body) = open(cursor, depth)?;
    for _ in 0..count {
        validate_value(&mut body, depth.saturating_add(1))?;
    }
    close(&body)
}

fn validate_map(cursor: &mut Cursor<'_>, depth: usize) -> Result<(), RecordError> {
    let (count, mut body) = open(cursor, depth)?;
    let mut previous: Option<&[u8]> = None;
    for _ in 0..count {
        let offset = body.offset();
        let key = body.text()?.as_bytes();
        if previous.is_some_and(|previous| previous >= key) {
            return Err(RecordError::OrderViolation { offset });
        }
        previous = Some(key);
        validate_value(&mut body, depth.saturating_add(1))?;
    }
    close(&body)
}

fn validate_struct(cursor: &mut Cursor<'_>, depth: usize) -> Result<(), RecordError> {
    let (count, mut body) = open(cursor, depth)?;
    let mut previous = 0u32;
    for _ in 0..count {
        let offset = body.offset();
        let id = body.field_id()?.get();
        if id <= previous {
            return Err(RecordError::OrderViolation { offset });
        }
        previous = id;
        validate_value(&mut body, depth.saturating_add(1))?;
    }
    close(&body)
}

/// Enters a container: checks depth, bounds its body and reads its count.
///
/// A count larger than the body can hold ends in `UnexpectedEnd`, because
/// every entry consumes at least one byte of the bounded body.
fn open<'a>(cursor: &mut Cursor<'a>, depth: usize) -> Result<(u64, Cursor<'a>), RecordError> {
    if depth > MAX_DEPTH {
        return Err(RecordError::DepthExceeded);
    }
    let mut body = cursor.body()?;
    let count = body.varint()?;
    Ok((count, body))
}

/// Leaves a container: its body must be fully read.
fn close(body: &Cursor<'_>) -> Result<(), RecordError> {
    if body.is_empty() {
        Ok(())
    } else {
        Err(RecordError::BodyNotConsumed {
            offset: body.offset(),
        })
    }
}
