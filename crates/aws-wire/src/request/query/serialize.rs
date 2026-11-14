use crate::request::{EncodeError, Protocol, RequestContext, common::BoundedBody};
use inventory_core::{catalog::{Member, Shape, Target, TimestampFormat}, record::{KeyRef, ValueRef}};
use super::{form, scalar};

const MAX_TRAVERSAL_STEPS: usize = 4_096;
const MAX_PATH_BYTES: usize = 4_096;

struct Work<'a> {
    target: Target,
    value: ValueRef<'a>,
    path: Vec<u8>,
    flattened: bool,
    timestamp_format: Option<&'static str>,
    empty: bool,
}

pub(super) fn input(context: RequestContext<'_>, body: &mut BoundedBody, first: &mut bool) -> Result<(), EncodeError> {
    let mut pending = Vec::new();
    pending.try_reserve(1)?;
    pending.push(Work { target: context.operation.input()?, value: ValueRef::Map(context.input), path: Vec::new(), flattened: false, timestamp_format: None, empty: false });
    let mut steps = 0usize;
    while let Some(work) = pending.pop() {
        steps = steps.checked_add(1).ok_or(EncodeError::Limit("query traversal"))?;
        if steps > MAX_TRAVERSAL_STEPS { return Err(EncodeError::Limit("query traversal")); }
        expand(context, work, &mut pending, body, first)?;
    }
    Ok(())
}

fn expand<'a>(context: RequestContext<'a>, work: Work<'a>, pending: &mut Vec<Work<'a>>, body: &mut BoundedBody, first: &mut bool) -> Result<(), EncodeError> {
    if work.empty { return form::pair(body, first, path_str(&work.path)?, ""); }
    match work.target {
        Target::Prelude("unit") => Ok(()),
        Target::Prelude(kind) => scalar::emit(body, first, path_str(&work.path)?, kind, work.value, work.timestamp_format, context.limits.body_bytes()),
        Target::Shape(index) => {
            let shape = context.service.shape(index)?.ok_or(EncodeError::Unsupported("shape reference is absent"))?;
            match shape.kind()? {
                "structure" | "union" => structure(context, shape, work, pending),
                "list" => list(context.protocol, shape, work, pending, body, first),
                "map" => map(context.protocol, shape, work, pending, body, first),
                "blob" | "boolean" | "byte" | "short" | "integer" | "long" | "float" | "double" | "bigInteger" | "bigDecimal" | "string" | "timestamp" | "enum" | "intEnum" => scalar::emit(
                    body, first, path_str(&work.path)?, shape.kind()?, work.value, work.timestamp_format.or(timestamp(shape.wire()?.timestamp_format()?)), context.limits.body_bytes()),
                "document" => Err(EncodeError::Unsupported("document values are not supported by AWS Query protocols")),
                _ => Err(EncodeError::Unsupported("unsupported Query shape kind")),
            }
        }
    }
}

fn structure<'a>(context: RequestContext<'a>, shape: Shape<'a>, work: Work<'a>, pending: &mut Vec<Work<'a>>) -> Result<(), EncodeError> {
    let ValueRef::Map(values) = work.value else { return Err(invalid("structure input is not a map")); };
    let mut children = Vec::new();
    for member in shape.members()? {
        let member = member?;
        match values.get_field(member.id)? {
            Some(ValueRef::Null) if member.wire()?.idempotency_token() => add_token(context, member, &work, &mut children)?,
            Some(ValueRef::Null) => {}
            Some(value) => push_work(&mut children, member_work(context, member, value, &work)?)?,
            None if member.wire()?.idempotency_token() => add_token(context, member, &work, &mut children)?,
            None => {}
        }
    }
    if shape.kind()? == "union" && children.len() != 1 { return Err(invalid("union input must contain exactly one member")); }
    push_children(pending, children)
}

fn add_token<'a>(context: RequestContext<'a>, member: Member<'a>, parent: &Work<'a>, children: &mut Vec<Work<'a>>) -> Result<(), EncodeError> {
    if let Some(token) = context.idempotency_token { push_work(children, member_work(context, member, ValueRef::Text(token), parent)?)?; }
    Ok(())
}

fn member_work<'a>(context: RequestContext<'a>, member: Member<'a>, value: ValueRef<'a>, parent: &Work<'a>) -> Result<Work<'a>, EncodeError> {
    let wire = member.wire()?;
    Ok(Work { target: member.target, value, path: member_path(context, &parent.path, member)?, flattened: wire.xml_flattened(), timestamp_format: timestamp(wire.timestamp_format()?), empty: false })
}

fn list<'a>(protocol: Protocol, shape: Shape<'a>, work: Work<'a>, pending: &mut Vec<Work<'a>>, body: &mut BoundedBody, first: &mut bool) -> Result<(), EncodeError> {
    let ValueRef::List(values) = work.value else { return Err(invalid("list input is not a list")); };
    let target = shape.element()?.ok_or(EncodeError::Unsupported("list element target is absent"))?;
    let element_wire = shape.element_wire()?;
    let shape_wire = shape.wire()?;
    let flattened = protocol == Protocol::Ec2Query || work.flattened || shape_wire.xml_flattened();
    let member_name = element_wire.xml_name()?.unwrap_or("member");
    let timestamp_format = timestamp(element_wire.timestamp_format()?);
    let mut children = Vec::new();
    let mut count = 0usize;
    for (index, item) in values.iter()?.enumerate() {
        count = count.checked_add(1).ok_or(EncodeError::Limit("query traversal"))?;
        let item = item?;
        let path = indexed_path(&work.path, index + 1, flattened, member_name)?;
        if matches!(item, ValueRef::Null) {
            if shape_wire.sparse() && protocol == Protocol::AwsQuery {
                push_work(&mut children, Work { target, value: ValueRef::Null, path, flattened: false, timestamp_format, empty: true })?;
                continue;
            }
            if shape_wire.sparse() { return Err(EncodeError::Unsupported("sparse EC2 Query list elements are not defined")); }
            return Err(invalid("null element in nonsparse list"));
        }
        push_work(&mut children, Work { target, value: item, path, flattened: false, timestamp_format, empty: false })?;
    }
    if count == 0 && protocol == Protocol::AwsQuery && !flattened { return form::pair(body, first, path_str(&work.path)?, ""); }
    push_children(pending, children)
}

fn map<'a>(protocol: Protocol, shape: Shape<'a>, work: Work<'a>, pending: &mut Vec<Work<'a>>, body: &mut BoundedBody, first: &mut bool) -> Result<(), EncodeError> {
    let ValueRef::Map(values) = work.value else { return Err(invalid("map input is not a map")); };
    let key_target = shape.map_key()?.ok_or(EncodeError::Unsupported("map key target is absent"))?;
    let value_target = shape.map_value()?.ok_or(EncodeError::Unsupported("map value target is absent"))?;
    let shape_wire = shape.wire()?;
    let key_wire = shape.map_key_wire()?;
    let value_wire = shape.map_value_wire()?;
    let flattened = protocol == Protocol::Ec2Query || work.flattened || shape_wire.xml_flattened();
    let key_name = key_wire.xml_name()?.unwrap_or("key");
    let value_name = value_wire.xml_name()?.unwrap_or("value");
    let key_time = timestamp(key_wire.timestamp_format()?);
    let value_time = timestamp(value_wire.timestamp_format()?);
    let mut children = Vec::new();
    for (index, entry) in values.iter()?.enumerate() {
        let (key, value) = entry?;
        let KeyRef::Name(key_text) = key else { return Err(invalid("map key is not a string")); };
        let prefix = indexed_path(&work.path, index + 1, flattened, "entry")?;
        let key_path = protocol_path(&prefix, protocol, key_name)?;
        let value_path = protocol_path(&prefix, protocol, value_name)?;
        push_work(&mut children, Work { target: key_target, value: ValueRef::Text(key_text), path: key_path, flattened: false, timestamp_format: key_time, empty: false })?;
        if matches!(value, ValueRef::Null) {
            if shape_wire.sparse() && protocol == Protocol::AwsQuery {
                push_work(&mut children, Work { target: value_target, value, path: value_path, flattened: false, timestamp_format: value_time, empty: true })?;
            } else if shape_wire.sparse() {
                return Err(EncodeError::Unsupported("sparse EC2 Query map values are not defined"));
            } else {
                return Err(invalid("null value in nonsparse map"));
            }
        } else {
            push_work(&mut children, Work { target: value_target, value, path: value_path, flattened: false, timestamp_format: value_time, empty: false })?;
        }
    }
    push_children(pending, children)
}

fn member_path(context: RequestContext<'_>, parent: &[u8], member: Member<'_>) -> Result<Vec<u8>, EncodeError> {
    let wire = member.wire()?;
    let name = match context.protocol {
        Protocol::AwsQuery => wire.xml_name()?.unwrap_or(context.service.field_name(member.id)?),
        Protocol::Ec2Query => match wire.ec2_query_name()? {
            Some(name) => return append_ec2_explicit(parent, name),
            None => wire.xml_name()?.unwrap_or(context.service.field_name(member.id)?),
        },
        _ => return Err(EncodeError::Unsupported("protocol is not AWS Query or EC2 Query")),
    };
    let mut path = clone_path(parent)?;
    if context.protocol == Protocol::Ec2Query { append_ec2_name(&mut path, name)?; } else { append_segment(&mut path, name)?; }
    Ok(path)
}

fn append_ec2_explicit(parent: &[u8], name: &str) -> Result<Vec<u8>, EncodeError> {
    let mut path = clone_path(parent)?;
    let needed = path.len().checked_add(name.len()).and_then(|length| length.checked_add(usize::from(!path.is_empty()))).ok_or(EncodeError::Limit("query name"))?;
    reserve_path(&mut path, needed)?;
    if !path.is_empty() { path.push(b'.'); }
    path.extend_from_slice(name.as_bytes());
    Ok(path)
}


fn protocol_path(parent: &[u8], protocol: Protocol, name: &str) -> Result<Vec<u8>, EncodeError> {
    let mut path = clone_path(parent)?;
    match protocol { Protocol::AwsQuery => append_segment(&mut path, name)?, Protocol::Ec2Query => append_ec2_name(&mut path, name)?, _ => return Err(EncodeError::Unsupported("unsupported Query protocol")) }
    Ok(path)
}

fn append_ec2_name(path: &mut Vec<u8>, name: &str) -> Result<(), EncodeError> {
    let Some(first) = name.chars().next() else { return Ok(()); };
    let suffix = name.get(first.len_utf8()..).ok_or_else(|| invalid("EC2 Query name is not UTF-8"))?;
    let mut uppercase = [0u8; 12];
    let mut uppercase_len = 0usize;
    for character in first.to_uppercase() {
        let mut encoded = [0u8; 4];
        let bytes = character.encode_utf8(&mut encoded).as_bytes();
        let end = uppercase_len.checked_add(bytes.len()).ok_or(EncodeError::Limit("query name"))?;
        uppercase.get_mut(uppercase_len..end).ok_or(EncodeError::Limit("query name"))?.copy_from_slice(bytes);
        uppercase_len = end;
    }
    let needed = path.len().checked_add(uppercase_len).and_then(|length| length.checked_add(suffix.len()))
        .and_then(|length| length.checked_add(usize::from(!path.is_empty()))).ok_or(EncodeError::Limit("query name"))?;
    reserve_path(path, needed)?;
    if !path.is_empty() { path.push(b'.'); }
    path.extend_from_slice(&uppercase[..uppercase_len]);
    path.extend_from_slice(suffix.as_bytes());
    Ok(())
}

fn indexed_path(parent: &[u8], index: usize, flattened: bool, member_name: &str) -> Result<Vec<u8>, EncodeError> {
    let mut path = clone_path(parent)?;
    if !flattened { append_segment(&mut path, member_name)?; }
    let mut digits = [0; 20];
    let mut value = index;
    let mut position = digits.len();
    while value > 0 { position -= 1; digits[position] = b'0' + u8::try_from(value % 10).map_err(|_| EncodeError::Limit("query name"))?; value /= 10; }
    append_segment(&mut path, std::str::from_utf8(&digits[position..]).map_err(|_| invalid("list index is not UTF-8"))?)?;
    Ok(path)
}

fn clone_path(path: &[u8]) -> Result<Vec<u8>, EncodeError> {
    let mut clone = Vec::new(); clone.try_reserve_exact(path.len())?; clone.extend_from_slice(path); Ok(clone)
}

fn append_segment(path: &mut Vec<u8>, segment: &str) -> Result<(), EncodeError> {
    let needed = path.len().checked_add(segment.len()).and_then(|length| length.checked_add(usize::from(!path.is_empty()))).ok_or(EncodeError::Limit("query name"))?;
    reserve_path(path, needed)?;
    if !path.is_empty() { path.push(b'.'); }
    path.extend_from_slice(segment.as_bytes());
    Ok(())
}

fn reserve_path(path: &mut Vec<u8>, needed: usize) -> Result<(), EncodeError> {
    if needed > MAX_PATH_BYTES { return Err(EncodeError::Limit("query name")); }
    path.try_reserve(needed.saturating_sub(path.len()))?;
    Ok(())
}

fn path_str(path: &[u8]) -> Result<&str, EncodeError> {
    std::str::from_utf8(path).map_err(|_| invalid("query member name is not UTF-8"))
}

fn timestamp(format: Option<TimestampFormat>) -> Option<&'static str> {
    match format { Some(TimestampFormat::DateTime) => Some("date-time"), Some(TimestampFormat::HttpDate) => Some("http-date"), Some(TimestampFormat::EpochSeconds) => Some("epoch-seconds"), None => None }
}

fn push_work<'a>(children: &mut Vec<Work<'a>>, work: Work<'a>) -> Result<(), EncodeError> {
    if children.len() == MAX_TRAVERSAL_STEPS { return Err(EncodeError::Limit("query traversal")); }
    children.try_reserve(1)?; children.push(work); Ok(())
}

fn push_children<'a>(pending: &mut Vec<Work<'a>>, children: Vec<Work<'a>>) -> Result<(), EncodeError> {
    let needed = pending.len().checked_add(children.len()).ok_or(EncodeError::Limit("query traversal"))?;
    if needed > MAX_TRAVERSAL_STEPS { return Err(EncodeError::Limit("query traversal")); }
    pending.try_reserve(children.len())?; pending.extend(children.into_iter().rev()); Ok(())
}

fn invalid(reason: &'static str) -> EncodeError { EncodeError::InvalidValue { field: None, reason } }
