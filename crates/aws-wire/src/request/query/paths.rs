use super::invalid;
use crate::request::{EncodeError, Protocol, RequestContext};
use inventory_core::catalog::{Member, TimestampFormat};

const MAX_PATH_BYTES: usize = 4_096;

pub(super) fn member_path(
    context: RequestContext<'_>,
    parent: &[u8],
    member: Member<'_>,
) -> Result<Vec<u8>, EncodeError> {
    let wire = member.wire()?;
    let name = match context.protocol {
        Protocol::AwsQuery => wire
            .xml_name()?
            .unwrap_or(context.service.field_name(member.id)?),
        Protocol::Ec2Query => match wire.ec2_query_name()? {
            Some(name) => return append_ec2_explicit(parent, name),
            None => wire
                .xml_name()?
                .unwrap_or(context.service.field_name(member.id)?),
        },
        _ => {
            return Err(EncodeError::Unsupported(
                "protocol is not AWS Query or EC2 Query",
            ));
        }
    };
    let mut path = clone_path(parent)?;
    if context.protocol == Protocol::Ec2Query {
        append_ec2_name(&mut path, name)?;
    } else {
        append_segment(&mut path, name)?;
    }
    Ok(path)
}

fn append_ec2_explicit(parent: &[u8], name: &str) -> Result<Vec<u8>, EncodeError> {
    let mut path = clone_path(parent)?;
    let needed = path
        .len()
        .checked_add(name.len())
        .and_then(|length| length.checked_add(usize::from(!path.is_empty())))
        .ok_or(EncodeError::Limit("query name"))?;
    reserve_path(&mut path, needed)?;
    if !path.is_empty() {
        path.push(b'.');
    }
    path.extend_from_slice(name.as_bytes());
    Ok(path)
}

pub(super) fn protocol_path(
    parent: &[u8],
    protocol: Protocol,
    name: &str,
) -> Result<Vec<u8>, EncodeError> {
    let mut path = clone_path(parent)?;
    match protocol {
        Protocol::AwsQuery => append_segment(&mut path, name)?,
        Protocol::Ec2Query => append_ec2_name(&mut path, name)?,
        _ => return Err(EncodeError::Unsupported("unsupported Query protocol")),
    }
    Ok(path)
}

fn append_ec2_name(path: &mut Vec<u8>, name: &str) -> Result<(), EncodeError> {
    let Some(first) = name.chars().next() else {
        return Ok(());
    };
    let suffix = name
        .get(first.len_utf8()..)
        .ok_or_else(|| invalid("EC2 Query name is not UTF-8"))?;
    let mut uppercase = [0u8; 12];
    let mut uppercase_len = 0usize;
    for character in first.to_uppercase() {
        let mut encoded = [0u8; 4];
        let bytes = character.encode_utf8(&mut encoded).as_bytes();
        let end = uppercase_len
            .checked_add(bytes.len())
            .ok_or(EncodeError::Limit("query name"))?;
        uppercase
            .get_mut(uppercase_len..end)
            .ok_or(EncodeError::Limit("query name"))?
            .copy_from_slice(bytes);
        uppercase_len = end;
    }
    let needed = path
        .len()
        .checked_add(uppercase_len)
        .and_then(|length| length.checked_add(suffix.len()))
        .and_then(|length| length.checked_add(usize::from(!path.is_empty())))
        .ok_or(EncodeError::Limit("query name"))?;
    reserve_path(path, needed)?;
    if !path.is_empty() {
        path.push(b'.');
    }
    path.extend_from_slice(
        uppercase
            .get(..uppercase_len)
            .ok_or(EncodeError::Limit("query name"))?,
    );
    path.extend_from_slice(suffix.as_bytes());
    Ok(())
}

pub(super) fn indexed_path(
    parent: &[u8],
    index: usize,
    flattened: bool,
    member_name: &str,
) -> Result<Vec<u8>, EncodeError> {
    let mut path = clone_path(parent)?;
    if !flattened {
        append_segment(&mut path, member_name)?;
    }
    let mut digits = [0; 20];
    let mut value = index;
    let mut position = digits.len();
    while value > 0 {
        position = position
            .checked_sub(1)
            .ok_or(EncodeError::Limit("query name"))?;
        let digit = u8::try_from(value % 10).map_err(|_| EncodeError::Limit("query name"))?;
        *digits
            .get_mut(position)
            .ok_or(EncodeError::Limit("query name"))? = b'0'
            .checked_add(digit)
            .ok_or(EncodeError::Limit("query name"))?;
        value /= 10;
    }
    let index = digits
        .get(position..)
        .ok_or(EncodeError::Limit("query name"))?;
    append_segment(
        &mut path,
        std::str::from_utf8(index).map_err(|_| invalid("list index is not UTF-8"))?,
    )?;
    Ok(path)
}

fn clone_path(path: &[u8]) -> Result<Vec<u8>, EncodeError> {
    let mut clone = Vec::new();
    clone.try_reserve_exact(path.len())?;
    clone.extend_from_slice(path);
    Ok(clone)
}

fn append_segment(path: &mut Vec<u8>, segment: &str) -> Result<(), EncodeError> {
    let needed = path
        .len()
        .checked_add(segment.len())
        .and_then(|length| length.checked_add(usize::from(!path.is_empty())))
        .ok_or(EncodeError::Limit("query name"))?;
    reserve_path(path, needed)?;
    if !path.is_empty() {
        path.push(b'.');
    }
    path.extend_from_slice(segment.as_bytes());
    Ok(())
}

fn reserve_path(path: &mut Vec<u8>, needed: usize) -> Result<(), EncodeError> {
    if needed > MAX_PATH_BYTES {
        return Err(EncodeError::Limit("query name"));
    }
    path.try_reserve(needed.saturating_sub(path.len()))?;
    Ok(())
}

pub(super) fn path_str(path: &[u8]) -> Result<&str, EncodeError> {
    std::str::from_utf8(path).map_err(|_| invalid("query member name is not UTF-8"))
}

pub(super) fn timestamp(format: Option<TimestampFormat>) -> Option<&'static str> {
    match format {
        Some(TimestampFormat::DateTime) => Some("date-time"),
        Some(TimestampFormat::HttpDate) => Some("http-date"),
        Some(TimestampFormat::EpochSeconds) => Some("epoch-seconds"),
        None => None,
    }
}
