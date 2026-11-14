use super::super::{DecodeError, ResponseContext};
use super::scalar;
use super::schema::{element_target, invalid_cbor, shape, shape_option, target_kind};
use super::unknown;
use inventory_core::{
    catalog::Target,
    record::{FieldId, KeyRef, Record, RecordBuilder},
};
use minicbor::{Decoder, data::Type};
mod containers;
mod walk;
use containers::{open_child_list, open_child_map};
use walk::walk;

pub(super) struct Frame {
    pub(super) map: bool,
    pub(super) remaining: Option<u64>,
    pub(super) target: Option<Target>,
    pub(super) modeled: bool,
    pub(super) seen: Vec<FieldId>,
}

pub(super) fn decode(context: ResponseContext<'_>) -> Result<Record, DecodeError> {
    if context.body.len() > context.limits.body_bytes() {
        return Err(DecodeError::Limit("response body"));
    }
    let builder = RecordBuilder::new(
        context.limits.body_bytes(),
        context.limits.nodes(),
        context.limits.depth(),
    )?;
    if context.body.is_empty() {
        return empty(context, builder);
    }
    decode_body(context, builder)
}

fn empty(context: ResponseContext<'_>, mut builder: RecordBuilder) -> Result<Record, DecodeError> {
    builder.begin_map(None)?;
    crate::response::defaults::apply(
        &mut builder,
        Some(context.operation.output()?),
        &[],
        context.service,
    )?;
    builder.end()?;
    builder.finish().map_err(DecodeError::Builder)
}

fn decode_body(
    context: ResponseContext<'_>,
    mut builder: RecordBuilder,
) -> Result<Record, DecodeError> {
    let mut decoder = Decoder::new(context.body);
    let root = open_root(&mut decoder, &mut builder, context.operation.output()?)?;
    let mut frames = Vec::new();
    frames.try_reserve(1)?;
    frames.push(root);
    let mut nodes = 1_usize;
    walk(
        &mut decoder,
        &mut builder,
        &mut frames,
        &context,
        &mut nodes,
    )?;
    if decoder.position() != context.body.len() {
        return Err(DecodeError::Invalid("trailing CBOR data"));
    }
    builder.finish().map_err(DecodeError::Builder)
}

fn open_root(
    decoder: &mut Decoder<'_>,
    builder: &mut RecordBuilder,
    target: Target,
) -> Result<Frame, DecodeError> {
    if !matches!(
        decoder.datatype().map_err(|_| invalid_cbor())?,
        Type::Map | Type::MapIndef
    ) {
        return Err(DecodeError::Invalid("expected CBOR structure map"));
    }
    let count = decoder.map().map_err(|_| invalid_cbor())?;
    builder.begin_map(None)?;
    Ok(Frame {
        map: true,
        remaining: count,
        target: Some(target),
        modeled: true,
        seen: Vec::new(),
    })
}

fn frame_done(decoder: &mut Decoder<'_>, frame: &mut Frame) -> Result<bool, DecodeError> {
    match frame.remaining {
        Some(0) => Ok(true),
        Some(left) => {
            frame.remaining = Some(
                left.checked_sub(1)
                    .ok_or(DecodeError::Invalid("CBOR count"))?,
            );
            Ok(false)
        }
        None if decoder.datatype().map_err(|_| invalid_cbor())? == Type::Break => {
            decoder.skip().map_err(|_| invalid_cbor())?;
            Ok(true)
        }
        None => Ok(false),
    }
}

fn child<'a>(
    decoder: &mut Decoder<'a>,
    frame: &mut Frame,
    service: inventory_core::catalog::Service<'_>,
) -> Result<(Option<Target>, Option<KeyRef<'a>>, bool), DecodeError> {
    if !frame.map {
        return Ok((element_target(frame.target, service)?, None, frame.modeled));
    }
    let name = read_text(decoder)?;
    let Some(target) = frame.target else {
        return Ok((None, Some(KeyRef::Name(name)), frame.modeled));
    };
    child_for_target(target, name, service)
}

fn child_for_target<'a>(
    target: Target,
    name: &'a str,
    service: inventory_core::catalog::Service<'_>,
) -> Result<(Option<Target>, Option<KeyRef<'a>>, bool), DecodeError> {
    if target == Target::Prelude("document") {
        return Ok((Some(target), Some(KeyRef::Name(name)), true));
    }
    match shape(target, service)?.kind()? {
        "structure" | "union" => structure_child(target, name, service),
        "map" => Ok((
            shape(target, service)?.map_value()?,
            Some(KeyRef::Name(name)),
            true,
        )),
        "document" => Ok((
            Some(Target::Prelude("document")),
            Some(KeyRef::Name(name)),
            true,
        )),
        _ => Ok((None, None, false)),
    }
}

fn structure_child<'a>(
    target: Target,
    name: &'a str,
    service: inventory_core::catalog::Service<'_>,
) -> Result<(Option<Target>, Option<KeyRef<'a>>, bool), DecodeError> {
    let Some(shape) = shape_option(target, service)? else {
        return Ok((None, None, false));
    };
    let mut found = None;
    for entry in shape.members()? {
        let member = entry?;
        if member.name()? == name {
            found = Some(member);
            break;
        }
    }
    match found {
        Some(member) => Ok((Some(member.target), Some(KeyRef::Field(member.id)), true)),
        None => Ok((None, None, false)),
    }
}

fn read_text<'a>(decoder: &mut Decoder<'a>) -> Result<&'a str, DecodeError> {
    match decoder.datatype().map_err(|_| invalid_cbor())? {
        Type::String => decoder.str().map_err(|_| invalid_cbor()),
        _ => Err(DecodeError::Invalid("expected CBOR map text key")),
    }
}

pub(super) fn count_node(nodes: &mut usize, limit: usize) -> Result<(), DecodeError> {
    *nodes = nodes
        .checked_add(1)
        .ok_or(DecodeError::Limit("response nodes"))?;
    if *nodes > limit {
        return Err(DecodeError::Limit("response nodes"));
    }
    Ok(())
}

fn preserves_null(
    frame: &Frame,
    service: inventory_core::catalog::Service<'_>,
) -> Result<bool, DecodeError> {
    match frame.target {
        Some(Target::Prelude("document")) => Ok(true),
        Some(target) => match shape_option(target, service)? {
            Some(shape) => Ok(shape.kind()? == "document" || shape.wire()?.sparse()?),
            None => Ok(false),
        },
        None => Ok(false),
    }
}
