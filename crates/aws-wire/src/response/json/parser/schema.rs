use super::{DecodeError, Key, ResponseContext};
use inventory_core::{
    catalog::{Member, Shape, Target},
    record::{KeyRef, RecordBuilder},
};

pub(super) fn member_target(
    context: ResponseContext<'_>,
    target: Option<Target>,
    name: &str,
) -> Result<(Option<Target>, Key), DecodeError> {
    let Some(target) = target else {
        return Ok((None, Key::Ignore));
    };
    match target {
        Target::Prelude("document") => Ok((Some(target), Key::Name(copy_name(name)?, None))),
        Target::Shape(index) => {
            let shape = context
                .service
                .shape(index)?
                .ok_or(DecodeError::Invalid("missing shape"))?;
            named_member(shape, name)
        }
        _ => Ok((None, Key::Ignore)),
    }
}

fn named_member(shape: Shape<'_>, name: &str) -> Result<(Option<Target>, Key), DecodeError> {
    match shape.kind()? {
        "document" => Ok((
            Some(Target::Prelude("document")),
            Key::Name(copy_name(name)?, None),
        )),
        "map" => Ok((
            shape.map_value()?,
            Key::Name(
                copy_name(name)?,
                shape.map_value_wire()?.timestamp_format()?,
            ),
        )),
        "structure" | "union" => find_member(shape, name),
        _ => Ok((None, Key::Ignore)),
    }
}

fn find_member(shape: Shape<'_>, name: &str) -> Result<(Option<Target>, Key), DecodeError> {
    for item in shape.members()? {
        let member = item?;
        if member
            .wire()?
            .json_name()?
            .map_or(member.name()?, |name| name)
            == name
        {
            return member_key(member);
        }
    }
    Ok((None, Key::Ignore))
}

fn member_key(member: Member<'_>) -> Result<(Option<Target>, Key), DecodeError> {
    let wire = member.wire()?;
    if !crate::response::defaults::is_document(wire.binding()?) {
        return Ok((None, Key::Ignore));
    }
    let defaulted = !wire.client_optional()?
        && (member.required
            || wire
                .default_value()?
                .is_some_and(|value| !matches!(value, inventory_core::record::ValueRef::Null)));
    Ok((
        Some(member.target),
        Key::Field(member.id, wire.timestamp_format()?, defaulted),
    ))
}

pub(super) fn object_target(
    context: ResponseContext<'_>,
    target: Option<Target>,
) -> Result<Option<Target>, DecodeError> {
    let Some(target) = target else {
        return Ok(None);
    };
    match target {
        Target::Prelude("document" | "unit") => Ok(Some(target)),
        Target::Shape(index) => {
            let shape = context
                .service
                .shape(index)?
                .ok_or(DecodeError::Invalid("missing shape"))?;
            match shape.kind()? {
                "structure" | "union" | "map" | "document" => Ok(Some(target)),
                _ => Err(DecodeError::Invalid("object for scalar shape")),
            }
        }
        _ => Err(DecodeError::Invalid("object for scalar shape")),
    }
}

pub(super) fn shape_kind(
    context: ResponseContext<'_>,
    target: Target,
) -> Result<&'static str, DecodeError> {
    match target {
        Target::Prelude(kind) => Ok(kind),
        Target::Shape(index) => Ok(context
            .service
            .shape(index)?
            .ok_or(DecodeError::Invalid("missing shape"))?
            .kind()?),
        _ => Err(DecodeError::Unsupported("non-exhaustive target")),
    }
}

pub(super) fn element_target(
    context: ResponseContext<'_>,
    target: Option<Target>,
) -> Result<Option<Target>, DecodeError> {
    match target {
        None => Ok(None),
        Some(Target::Prelude("document")) => Ok(Some(Target::Prelude("document"))),
        Some(Target::Shape(index)) => {
            let shape = context
                .service
                .shape(index)?
                .ok_or(DecodeError::Invalid("missing list shape"))?;
            match shape.kind()? {
                "list" => Ok(shape.element()?),
                "document" => Ok(Some(Target::Prelude("document"))),
                _ => Err(DecodeError::Invalid("array for non-list shape")),
            }
        }
        _ => Err(DecodeError::Invalid("array for non-list shape")),
    }
}

pub(super) fn preserves_null(
    context: ResponseContext<'_>,
    target: Option<Target>,
) -> Result<bool, DecodeError> {
    match target {
        Some(Target::Prelude("document")) => Ok(true),
        Some(Target::Shape(index)) => {
            let shape = context
                .service
                .shape(index)?
                .ok_or(DecodeError::Invalid("missing shape"))?;
            Ok(shape.kind()? == "document" || shape.wire()?.sparse()?)
        }
        _ => Ok(false),
    }
}

pub(super) fn nodes_add(
    nodes: &mut usize,
    context: ResponseContext<'_>,
) -> Result<(), DecodeError> {
    *nodes = nodes
        .checked_add(1)
        .ok_or(DecodeError::Limit("response nodes"))?;
    if *nodes > context.limits.nodes() {
        return Err(DecodeError::Limit("response nodes"));
    }
    Ok(())
}

pub(super) fn nested_depth(
    depth: usize,
    context: ResponseContext<'_>,
) -> Result<usize, DecodeError> {
    let next = depth
        .checked_add(1)
        .ok_or(DecodeError::Limit("response depth"))?;
    if next > context.limits.depth() {
        return Err(DecodeError::Limit("response depth"));
    }
    Ok(next)
}

pub(super) fn copy_name(name: &str) -> Result<String, DecodeError> {
    let mut value = String::new();
    value.try_reserve_exact(name.len())?;
    value.push_str(name);
    Ok(value)
}

pub(super) fn syntax<E>(_: E) -> DecodeError {
    DecodeError::Invalid("JSON syntax")
}

pub(super) fn begin_map(builder: &mut RecordBuilder, key: Key) -> Result<(), DecodeError> {
    match key {
        Key::Field(id, _, _) => builder.begin_map(Some(KeyRef::Field(id)))?,
        Key::Name(name, _) => builder.begin_map(Some(KeyRef::Name(&name)))?,
        Key::List(_) => builder.begin_map(None)?,
        Key::Ignore => return Err(DecodeError::Invalid("map without key")),
    }
    Ok(())
}

pub(super) fn begin_list(builder: &mut RecordBuilder, key: Key) -> Result<(), DecodeError> {
    match key {
        Key::Field(id, _, _) => builder.begin_list(Some(KeyRef::Field(id)))?,
        Key::Name(name, _) => builder.begin_list(Some(KeyRef::Name(&name)))?,
        Key::List(_) => builder.begin_list(None)?,
        Key::Ignore => return Err(DecodeError::Invalid("list without key")),
    }
    Ok(())
}
