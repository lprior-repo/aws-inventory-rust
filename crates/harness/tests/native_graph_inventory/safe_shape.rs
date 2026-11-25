//! Frozen fixture safe member trees, reviewed against the pinned provider shapes.
//! Independent of compiled runtime plans and production filtering: changing
//! either cannot silently expand this oracle. Optional provider metadata has
//! explicit scalar predicates; configured values are checked in safe_fields.
use super::TestResult;
use inventory_core::record::{Key, Value};

#[derive(Clone, Copy, Debug)]
enum Shape {
    Map,
    List,
    Text,
    Bool,
    Integer,
    Number,
    Timestamp,
}

#[path = "safe_shape/compute.rs"]
mod compute;
#[path = "safe_shape/ec2.rs"]
mod ec2;
#[path = "safe_shape/global.rs"]
mod global;
#[path = "safe_shape/messaging.rs"]
mod messaging;
#[path = "safe_shape/network.rs"]
mod network;
#[path = "safe_shape/storage.rs"]
mod storage;

fn tree(kind: &str) -> TestResult<&'static [(&'static str, Shape)]> {
    [
        ec2::tree(kind),
        network::tree(kind),
        compute::tree(kind),
        storage::tree(kind),
        global::tree(kind),
        messaging::tree(kind),
    ]
    .into_iter()
    .flatten()
    .next()
    .ok_or_else(|| format!("no independently reviewed safe member tree for {kind}").into())
}

pub(super) fn validate(kind: &str, value: &Value) -> TestResult {
    visit(value, "", tree(kind)?, 0)
}

fn visit(value: &Value, path: &str, tree: &[(&str, Shape)], depth: usize) -> TestResult {
    if depth > 32 {
        return Err("fixture safe tree nesting bound exceeded".into());
    }
    let next_depth = depth.saturating_add(1);
    let shape = tree
        .iter()
        .find(|(candidate, _)| *candidate == path)
        .map(|(_, shape)| *shape)
        .ok_or_else(|| format!("unreviewed safe member {path}"))?;
    match (shape, value) {
        (Shape::Map, Value::Map(map)) => {
            for (key, child) in map {
                let Key::Name(name) = key else {
                    return Err("safe tree requires named members".into());
                };
                assert_safe_name(name)?;
                let named = if path.is_empty() {
                    name.clone()
                } else {
                    format!("{path}.{name}")
                };
                let child_path = if tree.iter().any(|(candidate, _)| *candidate == named) {
                    named
                } else {
                    format!("{path}.*")
                };
                visit(child, &child_path, tree, next_depth)?;
            }
        }
        (Shape::List, Value::List(list)) => {
            for child in list {
                visit(child, &format!("{path}.*"), tree, next_depth)?;
            }
        }
        (Shape::Text, Value::Text(_))
        | (Shape::Bool, Value::Bool(_))
        | (Shape::Integer, Value::Int(_))
        | (Shape::Number, Value::Int(_) | Value::Float(_)) => {}
        (Shape::Timestamp, Value::Text(value)) if !value.is_empty() => {}
        (Shape::Timestamp, Value::Int(value)) if *value >= 0 => {}
        (Shape::Timestamp, Value::Float(_)) => {}
        _ => {
            return Err(format!(
                "safe member {path} violates explicit {shape:?} predicate: {value:?}"
            )
            .into());
        }
    }
    Ok(())
}

fn assert_safe_name(name: &str) -> TestResult {
    if matches!(
        name,
        "SecretAccessKey"
            | "SecretString"
            | "SecretBinary"
            | "Password"
            | "PrivateKey"
            | "AccessToken"
            | "RefreshToken"
            | "ClientSecret"
            | "SessionToken"
            | "UnsafeSentinel"
    ) {
        return Err(format!("forbidden safe member {name}").into());
    }
    Ok(())
}
