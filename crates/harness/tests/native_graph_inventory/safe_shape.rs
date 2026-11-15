//! Frozen fixture safe member trees, reviewed against the pinned provider shapes.
//! Independent of compiled runtime plans and production filtering: changing
//! either cannot silently expand this oracle. Optional provider metadata has
//! explicit scalar predicates; configured values are checked in safe_fields.
use super::TestResult;
use inventory_core::record::{Key, Value};

#[derive(Clone, Copy, Debug)]
enum Shape { Map, List, Text, Bool, Integer, Number, Timestamp }

mod ec2;
mod network;
mod compute;
mod storage;
mod global;
mod messaging;

fn tree(kind: &str) -> TestResult<&'static [(&'static str, Shape)]> {
    [ec2::tree(kind), network::tree(kind), compute::tree(kind),
        storage::tree(kind), global::tree(kind), messaging::tree(kind)]
        .into_iter().flatten().next()
        .ok_or_else(|| format!("no independently reviewed safe member tree for {kind}").into())
}

pub(super) fn validate(kind: &str, value: &Value) -> TestResult {
    visit(value, "", tree(kind)?, 0)
}

fn visit(value: &Value, path: &str, tree: &[(&str, Shape)], depth: usize) -> TestResult {
    if depth > 32 { return Err("fixture safe tree nesting bound exceeded".into()); }
    let shape = tree.iter().find(|(candidate, _)| *candidate == path).map(|(_, shape)| *shape)
        .ok_or_else(|| format!("unreviewed safe member {path}"))?;
    match (shape, value) {
        (Shape::Map, Value::Map(map)) => for (key, child) in map {
            let Key::Name(name) = key else { return Err("safe tree requires named members".into()); };
            if matches!(name.as_str(), "SecretAccessKey" | "SecretString" | "SecretBinary" | "Password" | "PrivateKey" | "AccessToken" | "RefreshToken" | "ClientSecret" | "SessionToken" | "UnsafeSentinel") {
                return Err(format!("forbidden safe member {name}").into());
            }
            let named = if path.is_empty() { name.clone() } else { format!("{path}.{name}") };
            let child_path = if tree.iter().any(|(candidate, _)| *candidate == named) { named } else { format!("{path}.*") };
            visit(child, &child_path, tree, depth + 1)?;
        },
        (Shape::List, Value::List(list)) => for child in list { visit(child, &format!("{path}.*"), tree, depth + 1)?; },
        (Shape::Text, Value::Text(_)) | (Shape::Bool, Value::Bool(_)) | (Shape::Integer, Value::Int(_)) |
        (Shape::Number, Value::Int(_) | Value::Float(_)) => {},
        (Shape::Timestamp, Value::Text(value)) if !value.is_empty() => {},
        (Shape::Timestamp, Value::Int(value)) if *value >= 0 => {},
        (Shape::Timestamp, Value::Float(_)) => {},
        _ => return Err(format!("safe member {path} violates explicit {shape:?} predicate: {value:?}").into()),
    }
    Ok(())
}
