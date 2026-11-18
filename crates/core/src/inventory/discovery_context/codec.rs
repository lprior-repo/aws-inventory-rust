use super::{
    ContextError, DiscoveryContext, DiscoveryFrame, MAX_DISCOVERY_CONTEXT_BYTES,
    MAX_DISCOVERY_FRAMES,
};
use crate::{
    inventory::{OperationId, ServiceId},
    record::{FieldId, Key, MapRef, Record, Value, ValueRef},
};
use std::collections::BTreeMap;

pub(super) fn encode(context: &DiscoveryContext) -> Result<Record, ContextError> {
    encoded_len(context, None)?;
    let frames = context
        .frames
        .iter()
        .map(|frame| {
            object(
                [
                    (1, Value::Text(frame.service.as_str().to_owned())),
                    (2, Value::Text(frame.operation.as_str().to_owned())),
                    (3, Value::Int(i64::from(frame.ordinal))),
                    (4, Value::Bytes(frame.selected.as_bytes().to_vec())),
                    (5, Value::Bytes(frame.response.as_bytes().to_vec())),
                    (6, Value::Bytes(frame.request.as_bytes().to_vec())),
                ]
                .into_iter()
                .chain(
                    frame
                        .branch
                        .iter()
                        .map(|branch| (7, Value::Text(branch.to_string()))),
                ),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let record = Record::encode(&object([(1, Value::Int(1)), (2, Value::List(frames))])?)?;
    if record.as_bytes().len() > MAX_DISCOVERY_CONTEXT_BYTES {
        return Err(ContextError::Bounds);
    }
    Ok(record)
}

pub(super) fn decode(bytes: &[u8]) -> Result<DiscoveryContext, ContextError> {
    if bytes.len() > MAX_DISCOVERY_CONTEXT_BYTES {
        return Err(ContextError::Bounds);
    }
    let record = Record::from_bytes(bytes.to_vec())?;
    only(record.root(), &[1, 2])?;
    if !matches!(field(record.root(), 1)?, ValueRef::Int(1)) {
        return Err(ContextError::Invalid);
    }
    let ValueRef::List(list) = field(record.root(), 2)? else {
        return Err(ContextError::Invalid);
    };
    let frames = list.iter()?.try_fold(Vec::new(), |mut frames, entry| {
        if frames.len() >= MAX_DISCOVERY_FRAMES {
            return Err(ContextError::Bounds);
        }
        let ValueRef::Map(map) = entry? else {
            return Err(ContextError::Invalid);
        };
        only(map, &[1, 2, 3, 4, 5, 6, 7])?;
        let ValueRef::Int(ordinal) = field(map, 3)? else {
            return Err(ContextError::Invalid);
        };
        frames.push(
            DiscoveryFrame {
                service: ServiceId::parse(text(map, 1)?)?,
                operation: OperationId::parse(text(map, 2)?)?,
                ordinal: u32::try_from(ordinal).map_err(|_| ContextError::Invalid)?,
                branch: None,
                selected: nested(map, 4)?,
                response: nested(map, 5)?,
                request: nested(map, 6)?,
            }
            .with_branch(match map.get_field(FieldId::new(7)?)? {
                None => None,
                Some(ValueRef::Text(branch)) => Some(branch),
                _ => return Err(ContextError::Invalid),
            })?,
        );
        Ok::<_, ContextError>(frames)
    })?;
    Ok(DiscoveryContext { frames })
}

fn object(fields: impl IntoIterator<Item = (u32, Value)>) -> Result<Value, ContextError> {
    fields
        .into_iter()
        .map(|(id, value)| Ok((Key::Field(FieldId::new(id)?), value)))
        .collect::<Result<BTreeMap<_, _>, ContextError>>()
        .map(Value::Map)
}
fn field(map: MapRef<'_>, id: u32) -> Result<ValueRef<'_>, ContextError> {
    map.get_field(FieldId::new(id)?)?
        .ok_or(ContextError::Invalid)
}
fn text(map: MapRef<'_>, id: u32) -> Result<&str, ContextError> {
    match field(map, id)? {
        ValueRef::Text(text) => Ok(text),
        _ => Err(ContextError::Invalid),
    }
}
fn nested(map: MapRef<'_>, id: u32) -> Result<Record, ContextError> {
    match field(map, id)? {
        ValueRef::Bytes(bytes) => Record::from_bytes(bytes.to_vec()).map_err(ContextError::from),
        _ => Err(ContextError::Invalid),
    }
}
fn only(map: MapRef<'_>, ids: &[u32]) -> Result<(), ContextError> {
    map.iter()?.try_for_each(|entry| match entry? {
        (crate::record::KeyRef::Field(id), _) if ids.contains(&id.value()) => Ok(()),
        _ => Err(ContextError::Invalid),
    })
}

pub(super) fn encoded_len(
    context: &DiscoveryContext,
    additional: Option<&DiscoveryFrame>,
) -> Result<usize, ContextError> {
    let count = context
        .frames
        .len()
        .checked_add(usize::from(additional.is_some()))
        .ok_or(ContextError::Bounds)?;
    if count > MAX_DISCOVERY_FRAMES {
        return Err(ContextError::Bounds);
    }
    // Root map has two keys, the version, and the definite-length frame list.
    let count_len = u64::try_from(count).map_err(|_| ContextError::Bounds)?;
    let initial = 4_usize
        .checked_add(argument_len(count_len))
        .ok_or(ContextError::Bounds)?;
    context
        .frames
        .iter()
        .chain(additional)
        .try_fold(initial, |total, frame| {
            // A six-field frame map has a one-byte map header and six one-byte keys.
            let initial = total
                .checked_add(7)
                .and_then(|size| size.checked_add(argument_len(u64::from(frame.ordinal))))
                .ok_or(ContextError::Bounds)?;
            let initial = if let Some(branch) = &frame.branch {
                let branch_len = branch.len();
                let branch_arg_len = u64::try_from(branch_len).map_err(|_| ContextError::Bounds)?;
                initial
                    .checked_add(1)
                    .and_then(|size| size.checked_add(argument_len(branch_arg_len)))
                    .and_then(|size| size.checked_add(branch_len))
                    .ok_or(ContextError::Bounds)?
            } else {
                initial
            };
            [
                frame.service.as_str().len(),
                frame.operation.as_str().len(),
                frame.selected.as_bytes().len(),
                frame.response.as_bytes().len(),
                frame.request.as_bytes().len(),
            ]
            .into_iter()
            .try_fold(initial, |size, length| {
                let length_arg_len = u64::try_from(length).map_err(|_| ContextError::Bounds)?;
                size.checked_add(argument_len(length_arg_len))
                    .and_then(|size| size.checked_add(length))
                    .filter(|size| *size <= MAX_DISCOVERY_CONTEXT_BYTES)
                    .ok_or(ContextError::Bounds)
            })
        })
}

fn argument_len(value: u64) -> usize {
    match value {
        0..=23 => 1,
        24..=255 => 2,
        256..=65_535 => 3,
        65_536..=4_294_967_295 => 5,
        _ => 9,
    }
}
