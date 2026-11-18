use super::{BindingValue, RuntimeCatalogError, tags};
use crate::{
    catalog::{Service, Shape, Target},
    record::{Key, KeyRef, ValueRef},
};
use std::ops::ControlFlow;

enum Work<'a> {
    Read(ValueRef<'a>, usize),
    List(usize),
    Object(Vec<Key>),
}

pub(super) fn decode(value: ValueRef<'_>) -> Result<BindingValue, RuntimeCatalogError> {
    let mut work = vec![Work::Read(value, 0)];
    let mut output = Vec::new();
    let mut nodes = 0usize;
    let mut bytes = 0usize;
    let outcome = (0..BindingValue::MAX_NODES.saturating_mul(2)).try_for_each(|_| {
        let Some(next) = work.pop() else {
            return ControlFlow::Break(Ok(()));
        };
        let result = (|| {
            match next {
                Work::Read(value, depth) => {
                    nodes = nodes.checked_add(1).ok_or(RuntimeCatalogError::Bounds)?;
                    if nodes > BindingValue::MAX_NODES || depth > BindingValue::MAX_DEPTH {
                        return Err(RuntimeCatalogError::Bounds);
                    }
                    read(value, depth, &mut work, &mut output, &mut bytes)?;
                }
                Work::List(length) => {
                    let children = split(&mut output, length)?;
                    let value = if children
                        .iter()
                        .all(|value| matches!(value, BindingValue::String(_)))
                    {
                        BindingValue::Strings(
                            children
                                .into_iter()
                                .map(|value| match value {
                                    BindingValue::String(text) => Ok(text),
                                    _ => Err(RuntimeCatalogError::Capability),
                                })
                                .collect::<Result<_, _>>()?,
                        )
                    } else {
                        BindingValue::List(children.into_boxed_slice())
                    };
                    output.push(value);
                }
                Work::Object(keys) => {
                    let children = split(&mut output, keys.len())?;
                    output.push(BindingValue::Object(
                        keys.into_iter().zip(children).collect(),
                    ));
                }
            }
            if work.len().saturating_add(nodes) > BindingValue::MAX_NODES.saturating_mul(2) {
                return Err(RuntimeCatalogError::Bounds);
            }
            Ok(())
        })();
        match result {
            Ok(()) => ControlFlow::Continue(()),
            Err(error) => ControlFlow::Break(Err(error)),
        }
    });
    if let ControlFlow::Break(result) = outcome {
        result?;
    }
    if !work.is_empty() || output.len() != 1 {
        return Err(RuntimeCatalogError::Bounds);
    }
    output.pop().ok_or(RuntimeCatalogError::Capability)
}

fn read<'a>(
    value: ValueRef<'a>,
    depth: usize,
    work: &mut Vec<Work<'a>>,
    output: &mut Vec<BindingValue>,
    bytes: &mut usize,
) -> Result<(), RuntimeCatalogError> {
    let scalar = match value {
        ValueRef::Text(value) => {
            text_bound(value.len(), bytes)?;
            Some(BindingValue::String(value.to_owned()))
        }
        ValueRef::Int(value) => Some(BindingValue::Integer(value)),
        ValueRef::Bool(value) => Some(BindingValue::Boolean(value)),
        ValueRef::Float(value) if value.to_f64().is_finite() => Some(BindingValue::Float(value)),
        ValueRef::Bytes(value) => {
            text_bound(value.len(), bytes)?;
            Some(BindingValue::Bytes(value.into()))
        }
        ValueRef::Time(value) => Some(BindingValue::Timestamp(value)),
        ValueRef::List(values) => {
            let children = values
                .iter()?
                .take(BindingValue::MAX_COLLECTION.saturating_add(1))
                .collect::<Result<Vec<_>, _>>()?;
            collection(children.len())?;
            work.push(Work::List(children.len()));
            work.extend(
                children
                    .into_iter()
                    .rev()
                    .map(|value| Work::Read(value, depth.saturating_add(1))),
            );
            None
        }
        ValueRef::Map(values) => {
            let entries = values
                .iter()?
                .take(BindingValue::MAX_COLLECTION.saturating_add(1))
                .collect::<Result<Vec<_>, _>>()?;
            collection(entries.len())?;
            let keys = entries
                .iter()
                .map(|(key, _)| {
                    if let KeyRef::Name(text) = key {
                        text_bound(text.len(), bytes)?;
                    }
                    Ok(key.to_key())
                })
                .collect::<Result<Vec<_>, RuntimeCatalogError>>()?;
            work.push(Work::Object(keys));
            work.extend(
                entries
                    .into_iter()
                    .rev()
                    .map(|(_, value)| Work::Read(value, depth.saturating_add(1))),
            );
            None
        }
        _ => return Err(RuntimeCatalogError::Capability),
    };
    if let Some(value) = scalar {
        output.push(value);
    }
    Ok(())
}
fn split(
    values: &mut Vec<BindingValue>,
    length: usize,
) -> Result<Vec<BindingValue>, RuntimeCatalogError> {
    let start = values
        .len()
        .checked_sub(length)
        .ok_or(RuntimeCatalogError::Capability)?;
    Ok(values.split_off(start))
}
fn text_bound(length: usize, bytes: &mut usize) -> Result<(), RuntimeCatalogError> {
    *bytes = bytes
        .checked_add(length)
        .ok_or(RuntimeCatalogError::Bounds)?;
    if length > BindingValue::MAX_STRING || *bytes > BindingValue::MAX_TEXT_BYTES {
        Err(RuntimeCatalogError::Bounds)
    } else {
        Ok(())
    }
}
fn collection(length: usize) -> Result<(), RuntimeCatalogError> {
    if length > BindingValue::MAX_COLLECTION {
        Err(RuntimeCatalogError::Bounds)
    } else {
        Ok(())
    }
}

struct Node<'a> {
    target: Target,
    value: &'a BindingValue,
    depth: usize,
}
struct Validation<'a> {
    pending: Vec<Node<'a>>,
    nodes: usize,
    bytes: usize,
}

pub(super) fn validate(
    service: Service<'_>,
    target: Target,
    value: &BindingValue,
) -> Result<(), RuntimeCatalogError> {
    let mut state = Validation {
        pending: vec![Node {
            target,
            value,
            depth: 0,
        }],
        nodes: 0,
        bytes: 0,
    };
    let outcome = (0..BindingValue::MAX_NODES).try_for_each(|_| {
        let Some(node) = state.pending.pop() else {
            return ControlFlow::Break(Ok(()));
        };
        let result = (|| {
            state.nodes = state
                .nodes
                .checked_add(1)
                .ok_or(RuntimeCatalogError::Bounds)?;
            if node.depth > BindingValue::MAX_DEPTH {
                return Err(RuntimeCatalogError::Bounds);
            }
            validate_node(service, node, &mut state)?;
            if state.nodes.saturating_add(state.pending.len()) > BindingValue::MAX_NODES {
                return Err(RuntimeCatalogError::Bounds);
            }
            Ok(())
        })();
        match result {
            Ok(()) => ControlFlow::Continue(()),
            Err(error) => ControlFlow::Break(Err(error)),
        }
    });
    if let ControlFlow::Break(result) = outcome {
        result?;
    }
    if state.pending.is_empty() {
        Ok(())
    } else {
        Err(RuntimeCatalogError::Bounds)
    }
}
fn validate_node<'a>(
    service: Service<'_>,
    node: Node<'a>,
    state: &mut Validation<'a>,
) -> Result<(), RuntimeCatalogError> {
    let kind = tags::target_kind(service, node.target)?;
    match node.value {
        BindingValue::Strings(values) if kind == "list" => strings(service, node, values, state),
        BindingValue::List(values) if kind == "list" => {
            collection(values.len())?;
            let target = tags::shape(service, node.target)?
                .element()?
                .ok_or(RuntimeCatalogError::Capability)?;
            state.pending.extend(values.iter().rev().map(|value| Node {
                target,
                value,
                depth: node.depth.saturating_add(1),
            }));
            Ok(())
        }
        BindingValue::Object(values) if matches!(kind, "structure" | "map") => {
            object(service, node, kind, values, state)
        }
        _ => scalar(service, node.target, kind, node.value, &mut state.bytes),
    }
}
fn scalar(
    service: Service<'_>,
    target: Target,
    kind: &str,
    value: &BindingValue,
    bytes: &mut usize,
) -> Result<(), RuntimeCatalogError> {
    match value {
        BindingValue::String(text) if matches!(kind, "string" | "enum") => {
            text_bound(text.len(), bytes)?;
            enum_value(service, target, ValueRef::Text(text))
        }
        BindingValue::Integer(number) if integer(kind, *number) => {
            enum_value(service, target, ValueRef::Int(*number))
        }
        BindingValue::Boolean(_) if kind == "boolean" => Ok(()),
        BindingValue::Float(number)
            if matches!(kind, "float" | "double")
                && number.to_f64().is_finite()
                && (kind == "double" || number.to_f64().abs() <= f64::from(f32::MAX)) =>
        {
            Ok(())
        }
        BindingValue::Bytes(value) if kind == "blob" => text_bound(value.len(), bytes),
        BindingValue::Timestamp(_) if kind == "timestamp" => Ok(()),
        _ => Err(RuntimeCatalogError::Capability),
    }
}
fn strings(
    service: Service<'_>,
    node: Node<'_>,
    values: &[String],
    state: &mut Validation<'_>,
) -> Result<(), RuntimeCatalogError> {
    collection(values.len())?;
    let element = tags::shape(service, node.target)?
        .element()?
        .ok_or(RuntimeCatalogError::Capability)?;
    if (!values.is_empty() && !matches!(tags::target_kind(service, element)?, "string" | "enum"))
        || (node.depth >= BindingValue::MAX_DEPTH && !values.is_empty())
    {
        return Err(RuntimeCatalogError::Capability);
    }
    state.nodes = state
        .nodes
        .checked_add(values.len())
        .ok_or(RuntimeCatalogError::Bounds)?;
    values.iter().try_for_each(|text| {
        text_bound(text.len(), &mut state.bytes)?;
        enum_value(service, element, ValueRef::Text(text))
    })
}
fn object<'a>(
    service: Service<'_>,
    node: Node<'a>,
    kind: &str,
    values: &'a [(Key, BindingValue)],
    state: &mut Validation<'a>,
) -> Result<(), RuntimeCatalogError> {
    collection(values.len())?;
    let shape = tags::shape(service, node.target)?;
    if kind == "structure" {
        shape.members()?.try_for_each(|member| {
            let member = member?;
            if member.required && !values.iter().any(|(key, _)| *key == Key::Field(member.id)) {
                return Err(RuntimeCatalogError::Capability);
            }
            Ok(())
        })?;
    }
    values
        .iter()
        .enumerate()
        .try_for_each(|(index, (key, value))| {
            if values.iter().take(index).any(|(prior, _)| prior == key) {
                return Err(RuntimeCatalogError::Capability);
            }
            let target = object_child(service, shape, kind, key, &mut state.bytes)?;
            state.pending.push(Node {
                target,
                value,
                depth: node.depth.saturating_add(1),
            });
            Ok(())
        })
}
fn object_child(
    service: Service<'_>,
    shape: Shape<'_>,
    kind: &str,
    key: &Key,
    bytes: &mut usize,
) -> Result<Target, RuntimeCatalogError> {
    match (kind, key) {
        ("structure", Key::Field(id)) => Ok(tags::member(shape, *id)?.target),
        ("map", Key::Name(name)) => {
            text_bound(name.len(), bytes)?;
            let key = shape.map_key()?.ok_or(RuntimeCatalogError::Capability)?;
            if !matches!(tags::target_kind(service, key)?, "string" | "enum") {
                return Err(RuntimeCatalogError::Capability);
            }
            enum_value(service, key, ValueRef::Text(name))?;
            shape.map_value()?.ok_or(RuntimeCatalogError::Capability)
        }
        _ => Err(RuntimeCatalogError::Capability),
    }
}
fn integer(kind: &str, value: i64) -> bool {
    match kind {
        "byte" => i8::try_from(value).is_ok(),
        "short" => i16::try_from(value).is_ok(),
        "integer" | "intEnum" => i32::try_from(value).is_ok(),
        "long" => true,
        _ => false,
    }
}
fn enum_value(
    service: Service<'_>,
    target: Target,
    value: ValueRef<'_>,
) -> Result<(), RuntimeCatalogError> {
    if let Target::Shape(_) = target
        && !tags::shape(service, target)?.accepts_enum_value(value)?
    {
        return Err(RuntimeCatalogError::Capability);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::{FieldId, Record, Value};
    use std::collections::BTreeMap;
    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn decoded(value: Value) -> Result<BindingValue, RuntimeCatalogError> {
        let id = FieldId::new(1)?;
        let record = Record::encode(&Value::Map(BTreeMap::from([(Key::Field(id), value)])))?;
        decode(
            record
                .root()
                .get_field(id)?
                .ok_or(RuntimeCatalogError::Capability)?,
        )
    }

    #[test]
    fn given_legacy_lists_when_loaded_then_empty_and_sixty_four_keep_exact_semantics() -> TestResult
    {
        assert_eq!(
            decoded(Value::List(Vec::new()))?,
            BindingValue::Strings(Box::new([]))
        );
        assert_eq!(
            decoded(Value::List(vec![Value::Text("x".into()); 64]))?,
            BindingValue::Strings(vec!["x".to_owned(); 64].into_boxed_slice())
        );
        assert!(matches!(
            decoded(Value::List(vec![Value::Text("x".into()); 65])),
            Err(RuntimeCatalogError::Bounds)
        ));
        Ok(())
    }

    #[test]
    fn given_literal_depth_when_at_boundary_then_only_one_over_is_rejected() -> TestResult {
        for depth in [
            BindingValue::MAX_DEPTH.saturating_sub(1),
            BindingValue::MAX_DEPTH,
        ] {
            assert!(
                decoded((0..depth).fold(Value::Int(1), |value, _| Value::List(vec![value])))
                    .is_ok()
            );
        }
        assert!(matches!(
            decoded(
                (0..BindingValue::MAX_DEPTH.saturating_add(1))
                    .fold(Value::Int(1), |value, _| Value::List(vec![value]))
            ),
            Err(RuntimeCatalogError::Bounds)
        ));
        Ok(())
    }

    #[test]
    fn given_literal_node_count_when_at_boundary_then_only_one_over_is_rejected() -> TestResult {
        for (first_length, accepted) in [(61, true), (62, true), (63, false)] {
            // One root + sixteen child lists + first_length + 15*63 leaves.
            let mut children = vec![Value::List(vec![Value::Int(1); 63]); 15];
            children.push(Value::List(vec![Value::Int(1); first_length]));
            let result = decoded(Value::List(children));
            if accepted {
                assert!(result.is_ok());
            } else {
                assert!(matches!(result, Err(RuntimeCatalogError::Bounds)));
            }
        }
        Ok(())
    }

    #[test]
    fn given_literal_string_length_when_at_boundary_then_only_one_over_is_rejected() -> TestResult {
        for length in [
            BindingValue::MAX_STRING.saturating_sub(1),
            BindingValue::MAX_STRING,
        ] {
            assert_eq!(
                decoded(Value::Text("x".repeat(length)))?,
                BindingValue::String("x".repeat(length))
            );
        }
        assert!(matches!(
            decoded(Value::Text(
                "x".repeat(BindingValue::MAX_STRING.saturating_add(1))
            )),
            Err(RuntimeCatalogError::Bounds)
        ));
        Ok(())
    }

    #[test]
    fn given_literal_total_bytes_when_at_boundary_then_only_one_over_is_rejected() -> TestResult {
        for (last_length, accepted) in [
            (BindingValue::MAX_STRING.saturating_sub(1), true),
            (BindingValue::MAX_STRING, true),
            (BindingValue::MAX_STRING.saturating_add(1), false),
        ] {
            let mut right = vec![Value::Text("x".repeat(BindingValue::MAX_STRING)); 31];
            // Keep each string within its independent bound even at total+1.
            right.push(Value::Text(
                "x".repeat(last_length.min(BindingValue::MAX_STRING)),
            ));
            if last_length > BindingValue::MAX_STRING {
                right.push(Value::Text("x".into()));
            }
            let result = decoded(Value::List(vec![
                Value::List(vec![Value::Text("x".repeat(BindingValue::MAX_STRING)); 32]),
                Value::List(right),
            ]));
            if accepted {
                assert!(result.is_ok());
            } else {
                assert!(matches!(result, Err(RuntimeCatalogError::Bounds)));
            }
        }
        Ok(())
    }

    #[test]
    fn given_structured_member_count_when_at_boundary_then_only_one_over_is_rejected() -> TestResult
    {
        for (length, accepted) in [(63, true), (64, true), (65, false)] {
            let entries = (1..=length)
                .map(|index| Ok((Key::Field(FieldId::new(index)?), Value::Int(1))))
                .collect::<Result<BTreeMap<_, _>, crate::record::RecordError>>()?;
            let result = decoded(Value::Map(entries));
            if accepted {
                assert!(result.is_ok());
            } else {
                assert!(matches!(result, Err(RuntimeCatalogError::Bounds)));
            }
        }
        assert!(matches!(
            decoded(Value::Null),
            Err(RuntimeCatalogError::Capability)
        ));
        Ok(())
    }

    #[test]
    fn given_numeric_structures_and_named_maps_when_decoded_then_keys_remain_typed() -> TestResult {
        let id = FieldId::new(9)?;
        let value = Value::List(vec![Value::Map(BTreeMap::from([(
            Key::Field(id),
            Value::Map(BTreeMap::from([(
                Key::Name("engine".into()),
                Value::Text("neptune".into()),
            )])),
        )]))]);
        assert_eq!(
            decoded(value)?,
            BindingValue::List(
                vec![BindingValue::Object(
                    vec![(
                        Key::Field(id),
                        BindingValue::Object(
                            vec![(
                                Key::Name("engine".into()),
                                BindingValue::String("neptune".into())
                            ),]
                            .into_boxed_slice()
                        )
                    )]
                    .into_boxed_slice()
                ),]
                .into_boxed_slice()
            )
        );
        Ok(())
    }
}
