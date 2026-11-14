use super::*;
use crate::request::Protocol;
use inventory_core::record::KeyRef;

fn one_based_index(index: usize) -> Result<usize, EncodeError> {
    index
        .checked_add(1)
        .ok_or(EncodeError::Limit("query traversal"))
}

pub(super) fn list<'a>(
    protocol: Protocol,
    shape: Shape<'a>,
    work: Work<'a>,
    pending: &mut Vec<Work<'a>>,
    body: &mut BoundedBody,
    first: &mut bool,
) -> Result<(), EncodeError> {
    let ValueRef::List(values) = work.value else {
        return Err(invalid("list input is not a list"));
    };
    let target = shape
        .element()?
        .ok_or(EncodeError::Unsupported("list element target is absent"))?;
    let element_wire = shape.element_wire()?;
    let shape_wire = shape.wire()?;
    let flattened =
        protocol == Protocol::Ec2Query || work.flattened || shape_wire.xml_flattened()?;
    let sparse = shape_wire.sparse()?;
    let member_name = element_wire.xml_name()?.unwrap_or("member");
    let timestamp_format = timestamp(element_wire.timestamp_format()?);
    let mut children = Vec::new();
    let mut count = 0usize;
    for (index, item) in values.iter()?.enumerate() {
        count = count
            .checked_add(1)
            .ok_or(EncodeError::Limit("query traversal"))?;
        let item = item?;
        let path = indexed_path(&work.path, one_based_index(index)?, flattened, member_name)?;
        if matches!(item, ValueRef::Null) {
            if sparse && protocol == Protocol::AwsQuery {
                push_work(
                    &mut children,
                    Work {
                        target,
                        value: ValueRef::Null,
                        path,
                        flattened: false,
                        timestamp_format,
                        empty: true,
                    },
                )?;
                continue;
            }
            if sparse {
                return Err(EncodeError::Unsupported(
                    "sparse EC2 Query list elements are not defined",
                ));
            }
            return Err(invalid("null element in nonsparse list"));
        }
        push_work(
            &mut children,
            Work {
                target,
                value: item,
                path,
                flattened: false,
                timestamp_format,
                empty: false,
            },
        )?;
    }
    if count == 0 && protocol == Protocol::AwsQuery && !flattened {
        return form::pair(body, first, path_str(&work.path)?, "");
    }
    push_children(pending, children)
}

pub(super) fn map<'a>(
    protocol: Protocol,
    shape: Shape<'a>,
    work: Work<'a>,
    pending: &mut Vec<Work<'a>>,
) -> Result<(), EncodeError> {
    let ValueRef::Map(values) = work.value else {
        return Err(invalid("map input is not a map"));
    };
    let key_target = shape
        .map_key()?
        .ok_or(EncodeError::Unsupported("map key target is absent"))?;
    let value_target = shape
        .map_value()?
        .ok_or(EncodeError::Unsupported("map value target is absent"))?;
    let shape_wire = shape.wire()?;
    let key_wire = shape.map_key_wire()?;
    let value_wire = shape.map_value_wire()?;
    let flattened =
        protocol == Protocol::Ec2Query || work.flattened || shape_wire.xml_flattened()?;
    let key_name = key_wire.xml_name()?.unwrap_or("key");
    let value_name = value_wire.xml_name()?.unwrap_or("value");
    let key_time = timestamp(key_wire.timestamp_format()?);
    let value_time = timestamp(value_wire.timestamp_format()?);
    let sparse = shape_wire.sparse()?;
    let mut children = Vec::new();
    for (index, entry) in values.iter()?.enumerate() {
        let (key, value) = entry?;
        let KeyRef::Name(key_text) = key else {
            return Err(invalid("map key is not a string"));
        };
        let prefix = indexed_path(&work.path, one_based_index(index)?, flattened, "entry")?;
        let key_path = protocol_path(&prefix, protocol, key_name)?;
        let value_path = protocol_path(&prefix, protocol, value_name)?;
        push_work(
            &mut children,
            Work {
                target: key_target,
                value: ValueRef::Text(key_text),
                path: key_path,
                flattened: false,
                timestamp_format: key_time,
                empty: false,
            },
        )?;
        if matches!(value, ValueRef::Null) {
            if sparse && protocol == Protocol::AwsQuery {
                push_work(
                    &mut children,
                    Work {
                        target: value_target,
                        value,
                        path: value_path,
                        flattened: false,
                        timestamp_format: value_time,
                        empty: true,
                    },
                )?;
            } else if sparse {
                return Err(EncodeError::Unsupported(
                    "sparse EC2 Query map values are not defined",
                ));
            } else {
                return Err(invalid("null value in nonsparse map"));
            }
        } else {
            push_work(
                &mut children,
                Work {
                    target: value_target,
                    value,
                    path: value_path,
                    flattened: false,
                    timestamp_format: value_time,
                    empty: false,
                },
            )?;
        }
    }
    push_children(pending, children)
}
