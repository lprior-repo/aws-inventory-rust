//! Generic CBOR framing. Minicbor still decodes every header and scalar.
use super::RecordError;
use minicbor::{Decoder, data::Type};

#[derive(Clone, Copy)]
enum Frame {
    Items(u64),
    Array,
    MapKey,
    MapValue,
}

pub(super) fn validate(d: &mut Decoder<'_>) -> Result<(), RecordError> {
    let mut frames = Vec::new();
    push(&mut frames, Frame::Items(1))?;
    // Each step consumes a byte, pops a previously pushed frame, or finishes.
    let steps = d
        .input()
        .len()
        .checked_mul(2)
        .and_then(|n| n.checked_add(2))
        .ok_or(RecordError::SizeLimit)?;
    let Some(result) = (0..steps).find_map(|_| match step(d, &mut frames) {
        Ok(true) => Some(Ok(())),
        Ok(false) => None,
        Err(err) => Some(Err(err)),
    }) else {
        return Err(RecordError::Malformed {
            offset: d.position(),
        });
    };
    result
}

fn step(d: &mut Decoder<'_>, frames: &mut Vec<Frame>) -> Result<bool, RecordError> {
    let Some(frame) = frames.last().copied() else {
        return Ok(true);
    };
    let offset = d.position();
    let error = RecordError::Malformed { offset };
    if matches!(frame, Frame::Items(0)) {
        frames.pop().ok_or(error)?;
        return Ok(false);
    }
    if d.datatype()? == Type::Break {
        return close(d, frames, frame);
    }
    advance(frames.last_mut().ok_or(error)?, offset)?;
    read_item(d, frames)?;
    Ok(false)
}

fn advance(frame: &mut Frame, offset: usize) -> Result<(), RecordError> {
    *frame = match *frame {
        Frame::Items(n) => Frame::Items(n.checked_sub(1).ok_or(RecordError::Malformed { offset })?),
        Frame::MapKey => Frame::MapValue,
        Frame::MapValue => Frame::MapKey,
        Frame::Array => Frame::Array,
    };
    Ok(())
}

fn close(d: &mut Decoder<'_>, frames: &mut Vec<Frame>, frame: Frame) -> Result<bool, RecordError> {
    if !matches!(frame, Frame::Array | Frame::MapKey) {
        return Err(RecordError::Malformed {
            offset: d.position(),
        });
    }
    d.skip()?;
    frames.pop().ok_or(RecordError::Malformed {
        offset: d.position(),
    })?;
    Ok(false)
}

fn read_item(d: &mut Decoder<'_>, frames: &mut Vec<Frame>) -> Result<(), RecordError> {
    let frame = match d.datatype()? {
        Type::Array | Type::ArrayIndef => match d.array()? {
            Some(n) => Frame::Items(n),
            None => Frame::Array,
        },
        Type::Map | Type::MapIndef => map_frame(d.map()?, d.position())?,
        Type::Tag => {
            d.tag()?;
            Frame::Items(1)
        }
        _ => {
            d.skip()?;
            return Ok(());
        }
    };
    push(frames, frame)
}

fn map_frame(count: Option<u64>, offset: usize) -> Result<Frame, RecordError> {
    match count {
        Some(n) => n
            .checked_mul(2)
            .map(Frame::Items)
            .ok_or(RecordError::Malformed { offset }),
        None => Ok(Frame::MapKey),
    }
}

fn push(frames: &mut Vec<Frame>, frame: Frame) -> Result<(), RecordError> {
    frames.try_reserve(1).map_err(|_| RecordError::Allocation)?;
    frames.push(frame);
    Ok(())
}
