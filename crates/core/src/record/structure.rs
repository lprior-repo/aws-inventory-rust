//! CBOR framing and strictly ascending supported keys for reader-reachable maps.
//!
//! Reachable maps are the definite root and definite maps reached transitively
//! through map values and list items. Tags and indefinite containers make their
//! entire contents opaque: only generic framing is checked there, so version 1
//! can still skip unknown value kinds. Minicbor decodes every header and scalar.
use super::{KeyRef, RecordError, readers};
use minicbor::{Decoder, data::Type};
use std::cmp::Ordering;

#[derive(Clone, Copy)]
enum Reachability {
    Reachable,
    Opaque,
}

#[derive(Clone, Copy)]
enum Frame<'a> {
    Items(u64, Reachability),
    MapKey {
        remaining: u64,
        previous: Option<KeyRef<'a>>,
    },
    MapValue {
        remaining: u64,
        previous: KeyRef<'a>,
    },
    Array,
    IndefMapKey,
    IndefMapValue,
}

impl Frame<'_> {
    fn complete(self) -> bool {
        matches!(self, Self::Items(0, _) | Self::MapKey { remaining: 0, .. })
    }

    fn reachability(self) -> Reachability {
        match self {
            Self::Items(_, reachability) => reachability,
            Self::MapValue { .. } => Reachability::Reachable,
            _ => Reachability::Opaque,
        }
    }
}

pub(super) fn validate(d: &mut Decoder<'_>) -> Result<(), RecordError> {
    let mut frames = Vec::new();
    push(&mut frames, Frame::Items(1, Reachability::Reachable))?;
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

fn step<'a>(d: &mut Decoder<'a>, frames: &mut Vec<Frame<'a>>) -> Result<bool, RecordError> {
    let Some(frame) = frames.last().copied() else {
        return Ok(true);
    };
    let error = RecordError::Malformed {
        offset: d.position(),
    };
    if frame.complete() {
        frames.pop().ok_or(error)?;
        return Ok(false);
    }
    if d.datatype()? == Type::Break {
        return close(d, frames, frame);
    }
    read_next(d, frames, frame)
}

fn read_next<'a>(
    d: &mut Decoder<'a>,
    frames: &mut Vec<Frame<'a>>,
    frame: Frame<'a>,
) -> Result<bool, RecordError> {
    let error = RecordError::Malformed {
        offset: d.position(),
    };
    let current = frames.last_mut().ok_or(error)?;
    if let Frame::MapKey {
        remaining,
        previous,
    } = frame
    {
        read_key(d, current, remaining, previous)?;
        return Ok(false);
    }
    let reachability = frame.reachability();
    advance(current, d.position())?;
    read_item(d, frames, reachability)?;
    Ok(false)
}

fn read_key<'a>(
    d: &mut Decoder<'a>,
    frame: &mut Frame<'a>,
    remaining: u64,
    previous: Option<KeyRef<'a>>,
) -> Result<(), RecordError> {
    let offset = d.position();
    let key = readers::read_key(d)?;
    match previous.map(|previous| previous.cmp(&key)) {
        Some(Ordering::Equal) => return Err(RecordError::DuplicateKey { offset }),
        Some(Ordering::Greater) => return Err(RecordError::KeyOutOfOrder { offset }),
        _ => {}
    }
    *frame = Frame::MapValue {
        remaining,
        previous: key,
    };
    Ok(())
}

fn advance(frame: &mut Frame<'_>, offset: usize) -> Result<(), RecordError> {
    let decrement = |n: u64| n.checked_sub(1).ok_or(RecordError::Malformed { offset });
    *frame = match *frame {
        Frame::Items(n, reachability) => Frame::Items(decrement(n)?, reachability),
        Frame::MapValue {
            remaining,
            previous,
        } => Frame::MapKey {
            remaining: decrement(remaining)?,
            previous: Some(previous),
        },
        Frame::IndefMapKey => Frame::IndefMapValue,
        Frame::IndefMapValue => Frame::IndefMapKey,
        Frame::MapKey { .. } => return Err(RecordError::Malformed { offset }),
        Frame::Array => Frame::Array,
    };
    Ok(())
}

fn close(
    d: &mut Decoder<'_>,
    frames: &mut Vec<Frame<'_>>,
    frame: Frame<'_>,
) -> Result<bool, RecordError> {
    if !matches!(frame, Frame::Array | Frame::IndefMapKey) {
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

fn read_item<'a>(
    d: &mut Decoder<'a>,
    frames: &mut Vec<Frame<'a>>,
    reachability: Reachability,
) -> Result<(), RecordError> {
    let frame = match d.datatype()? {
        Type::Array | Type::ArrayIndef => match d.array()? {
            Some(n) => Frame::Items(n, reachability),
            None => Frame::Array,
        },
        Type::Map | Type::MapIndef => map_frame(d.map()?, d.position(), reachability)?,
        Type::Tag => {
            d.tag()?;
            Frame::Items(1, Reachability::Opaque)
        }
        _ => {
            d.skip()?;
            return Ok(());
        }
    };
    push(frames, frame)
}

fn map_frame(
    count: Option<u64>,
    offset: usize,
    reachability: Reachability,
) -> Result<Frame<'static>, RecordError> {
    match (count, reachability) {
        (Some(remaining), Reachability::Reachable) => {
            remaining
                .checked_mul(2)
                .ok_or(RecordError::Malformed { offset })?;
            Ok(Frame::MapKey {
                remaining,
                previous: None,
            })
        }
        (Some(n), Reachability::Opaque) => n
            .checked_mul(2)
            .map(|n| Frame::Items(n, Reachability::Opaque))
            .ok_or(RecordError::Malformed { offset }),
        (None, _) => Ok(Frame::IndefMapKey),
    }
}

fn push<'a>(frames: &mut Vec<Frame<'a>>, frame: Frame<'a>) -> Result<(), RecordError> {
    frames.try_reserve(1).map_err(|_| RecordError::Allocation)?;
    frames.push(frame);
    Ok(())
}
