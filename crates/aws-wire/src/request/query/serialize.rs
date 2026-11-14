use super::{form, scalar};
#[path = "collections.rs"]
mod collections;
#[path = "paths.rs"]
mod paths;

use collections::{list, map};

use crate::request::{EncodeError, RequestContext, common::BoundedBody};
use inventory_core::{
    catalog::{Member, Shape, Target},
    record::ValueRef,
};
use paths::{indexed_path, member_path, path_str, protocol_path, timestamp};

const MAX_TRAVERSAL_STEPS: usize = 4_096;

struct Work<'a> {
    target: Target,
    value: ValueRef<'a>,
    path: Vec<u8>,
    flattened: bool,
    timestamp_format: Option<&'static str>,
    empty: bool,
}

pub(super) fn input(
    context: RequestContext<'_>,
    body: &mut BoundedBody,
    first: &mut bool,
) -> Result<(), EncodeError> {
    let mut pending = Vec::new();
    pending.try_reserve(1)?;
    pending.push(Work {
        target: context.operation.input()?,
        value: ValueRef::Map(context.input),
        path: Vec::new(),
        flattened: false,
        timestamp_format: None,
        empty: false,
    });
    let mut steps = 0usize;
    while let Some(work) = pending.pop() {
        steps = steps
            .checked_add(1)
            .ok_or(EncodeError::Limit("query traversal"))?;
        if steps > MAX_TRAVERSAL_STEPS {
            return Err(EncodeError::Limit("query traversal"));
        }
        expand(context, work, &mut pending, body, first)?;
    }
    Ok(())
}

fn expand<'a>(
    context: RequestContext<'a>,
    work: Work<'a>,
    pending: &mut Vec<Work<'a>>,
    body: &mut BoundedBody,
    first: &mut bool,
) -> Result<(), EncodeError> {
    if work.empty {
        return form::pair(body, first, path_str(&work.path)?, "");
    }
    match work.target {
        Target::Prelude("unit") => Ok(()),
        Target::Prelude(kind) => scalar::emit(
            body,
            first,
            path_str(&work.path)?,
            kind,
            work.value,
            work.timestamp_format,
            context.limits.body_bytes(),
        ),
        Target::Shape(index) => {
            let shape = context
                .service
                .shape(index)?
                .ok_or(EncodeError::Unsupported("shape reference is absent"))?;
            match shape.kind()? {
                "structure" | "union" => structure(context, shape, work, pending),
                "list" => list(context.protocol, shape, work, pending, body, first),
                "map" => map(context.protocol, shape, work, pending),
                "blob" | "boolean" | "byte" | "short" | "integer" | "long" | "float" | "double"
                | "bigInteger" | "bigDecimal" | "string" | "timestamp" | "enum" | "intEnum" => {
                    scalar::emit(
                        body,
                        first,
                        path_str(&work.path)?,
                        shape.kind()?,
                        work.value,
                        work.timestamp_format
                            .or(timestamp(shape.wire()?.timestamp_format()?)),
                        context.limits.body_bytes(),
                    )
                }
                "document" => Err(EncodeError::Unsupported(
                    "document values are not supported by AWS Query protocols",
                )),
                _ => Err(EncodeError::Unsupported("unsupported Query shape kind")),
            }
        }
        _ => Err(EncodeError::Unsupported("unknown Query target")),
    }
}

fn structure<'a>(
    context: RequestContext<'a>,
    shape: Shape<'a>,
    work: Work<'a>,
    pending: &mut Vec<Work<'a>>,
) -> Result<(), EncodeError> {
    let ValueRef::Map(values) = work.value else {
        return Err(invalid("structure input is not a map"));
    };
    let mut children = Vec::new();
    let union = shape.kind()? == "union";
    for member in shape.members()? {
        let member = member?;
        match values.get_field(member.id)? {
            Some(ValueRef::Null) if member.wire()?.idempotency_token()? => {
                add_token(context, member, &work, &mut children)?
            }
            Some(ValueRef::Null) => {}
            Some(value) => push_work(&mut children, member_work(context, member, value, &work)?)?,
            None if member.wire()?.idempotency_token()? => {
                add_token(context, member, &work, &mut children)?
            }
            None if !work.path.is_empty()
                && !union
                && !member.wire()?.client_optional()?
                && let Some(value) = member.wire()?.default_value()?
                && !matches!(value, ValueRef::Null) =>
            {
                push_work(&mut children, member_work(context, member, value, &work)?)?
            }
            None => {}
        }
    }
    if union && children.len() != 1 {
        return Err(invalid("union input must contain exactly one member"));
    }
    push_children(pending, children)
}

fn add_token<'a>(
    context: RequestContext<'a>,
    member: Member<'a>,
    parent: &Work<'a>,
    children: &mut Vec<Work<'a>>,
) -> Result<(), EncodeError> {
    if let Some(token) = context.idempotency_token {
        push_work(
            children,
            member_work(context, member, ValueRef::Text(token), parent)?,
        )?;
    }
    Ok(())
}

fn member_work<'a>(
    context: RequestContext<'a>,
    member: Member<'a>,
    value: ValueRef<'a>,
    parent: &Work<'a>,
) -> Result<Work<'a>, EncodeError> {
    let wire = member.wire()?;
    Ok(Work {
        target: member.target,
        value,
        path: member_path(context, &parent.path, member)?,
        flattened: wire.xml_flattened()?,
        timestamp_format: timestamp(wire.timestamp_format()?),
        empty: false,
    })
}

fn push_work<'a>(children: &mut Vec<Work<'a>>, work: Work<'a>) -> Result<(), EncodeError> {
    if children.len() == MAX_TRAVERSAL_STEPS {
        return Err(EncodeError::Limit("query traversal"));
    }
    children.try_reserve(1)?;
    children.push(work);
    Ok(())
}

fn push_children<'a>(
    pending: &mut Vec<Work<'a>>,
    children: Vec<Work<'a>>,
) -> Result<(), EncodeError> {
    let needed = pending
        .len()
        .checked_add(children.len())
        .ok_or(EncodeError::Limit("query traversal"))?;
    if needed > MAX_TRAVERSAL_STEPS {
        return Err(EncodeError::Limit("query traversal"));
    }
    pending.try_reserve(children.len())?;
    pending.extend(children.into_iter().rev());
    Ok(())
}

fn invalid(reason: &'static str) -> EncodeError {
    EncodeError::InvalidValue {
        field: None,
        reason,
    }
}
