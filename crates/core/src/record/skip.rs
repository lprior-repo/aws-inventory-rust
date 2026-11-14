use super::RecordError;
use minicbor::{Decoder, data::Type};

pub(super) fn value(d: &mut Decoder<'_>) -> Result<(), RecordError> {
    walk(d, skip_item)
}

fn walk<'a>(
    d: &mut Decoder<'a>,
    mut step: impl FnMut(&mut Decoder<'a>, u64) -> Result<u64, RecordError>,
) -> Result<(), RecordError> {
    let mut pending = 1;

    let Some(result) =
        (0..d.input().len().saturating_sub(d.position())).find_map(|_| match step(d, pending) {
            Ok(0) => Some(Ok(())),
            Ok(next) => {
                pending = next;
                None
            }
            Err(error) => Some(Err(error)),
        })
    else {
        return Err(RecordError::Malformed {
            offset: d.position(),
        });
    };
    result
}

fn skip_item(d: &mut Decoder<'_>, pending: u64) -> Result<u64, RecordError> {
    let error = RecordError::Malformed {
        offset: d.position(),
    };
    let children = match d.datatype()? {
        found @ (Type::Array | Type::Map) => definite_arity(d, found)?,
        Type::Tag => {
            d.tag()?;
            1
        }
        Type::ArrayIndef | Type::MapIndef => {
            indefinite(d)?;
            0
        }
        _ => {
            d.skip()?;
            0
        }
    };
    pending
        .checked_sub(1)
        .and_then(|n| n.checked_add(children))
        .ok_or(error)
}

fn definite_arity(d: &mut Decoder<'_>, found: Type) -> Result<u64, RecordError> {
    let error = RecordError::Malformed {
        offset: d.position(),
    };
    match found {
        Type::Array => d.array()?.ok_or(error),
        _ => d.map()?.and_then(|n| n.checked_mul(2)).ok_or(error),
    }
}

fn indefinite(d: &mut Decoder<'_>) -> Result<(), RecordError> {
    header_or_scalar(d)?;
    walk(d, skip_group)
}

fn skip_group(d: &mut Decoder<'_>, groups: u64) -> Result<u64, RecordError> {
    let found = d.datatype()?;
    let offset = d.position();
    header_or_scalar(d)?;

    match found {
        Type::ArrayIndef | Type::MapIndef => groups.checked_add(1),
        Type::Break => groups.checked_sub(1),
        _ => Some(groups),
    }
    .ok_or(RecordError::Malformed { offset })
}

fn header_or_scalar(d: &mut Decoder<'_>) -> Result<(), RecordError> {
    match d.datatype()? {
        Type::Array | Type::ArrayIndef => d.array().map(|_| ()),
        Type::Map | Type::MapIndef => d.map().map(|_| ()),
        Type::Tag => d.tag().map(|_| ()),

        _ => d.skip(),
    }
    .map_err(Into::into)
}
