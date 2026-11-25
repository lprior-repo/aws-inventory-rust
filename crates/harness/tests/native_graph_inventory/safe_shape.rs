use super::TestResult;
use inventory_core::record::{Key, Value, ValueKind};
use std::{collections::BTreeSet, fmt};

#[derive(Default)]
pub(super) struct Violations(BTreeSet<String>);

impl Violations {
    pub(super) fn add(&mut self, kind: &str, path: &str, value: Option<&Value>, reason: &str) {
        let path = if path.is_empty() { "<root>" } else { path };
        self.0.insert(match value {
            Some(value) => format!("{kind} | {path} | {:?} | {reason}", value_kind(value)),
            None => format!("{kind} | {path} | Absent | {reason}"),
        });
    }

    pub(super) fn finish(self) -> TestResult {
        if self.0.is_empty() {
            return Ok(());
        }
        Err(self.into())
    }
}

impl fmt::Display for Violations {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(output, "strict safe-record comparison rejected:")?;
        for issue in &self.0 {
            write!(output, "\n{issue}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for Violations {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, output)
    }
}

impl std::error::Error for Violations {}

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

fn tree(kind: &str) -> Option<&'static [(&'static str, Shape)]> {
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
}

pub(super) fn inspect(kind: &str, value: &Value, issues: &mut Violations) {
    let Some(tree) = tree(kind) else {
        issues.add(
            kind,
            "",
            Some(value),
            "no independently reviewed safe member tree",
        );
        return;
    };
    visit(kind, value, "", "", tree, 0, issues);
}

fn visit(
    kind: &str,
    value: &Value,
    path: &str,
    pattern: &str,
    tree: &[(&str, Shape)],
    depth: usize,
    issues: &mut Violations,
) {
    if depth > 32 {
        issues.add(kind, path, Some(value), "safe tree nesting bound exceeded");
        return;
    }
    let shape = tree
        .iter()
        .find(|(candidate, _)| *candidate == pattern)
        .map(|(_, shape)| *shape);
    match shape {
        None => issues.add(kind, path, Some(value), "unreviewed safe member"),
        Some(shape) if !matches_shape(shape, value) => {
            let reason = format!("violates explicit {shape:?} predicate");
            issues.add(kind, path, Some(value), &reason);
        }
        _ => {}
    }
    let next = depth.saturating_add(1);
    let wildcard = || member_path(pattern, "*");
    match value {
        Value::Map(map) => {
            for (key, child) in map {
                let Key::Name(name) = key else {
                    issues.add(kind, path, Some(child), "safe tree requires named members");
                    continue;
                };
                let named = member_path(path, name);
                if forbidden_name(name) {
                    issues.add(kind, &named, Some(child), "forbidden safe member");
                }
                let named_pattern = (path != pattern).then(|| member_path(pattern, name));
                let candidate = named_pattern.as_deref().map_or(named.as_str(), |path| path);
                if tree.iter().any(|(reviewed, _)| *reviewed == candidate) {
                    visit(kind, child, &named, candidate, tree, next, issues);
                } else {
                    visit(kind, child, &named, &wildcard(), tree, next, issues);
                }
            }
        }
        Value::List(items) => {
            let item_pattern = wildcard();
            for (index, item) in items.iter().enumerate() {
                let indexed = format!("{path}[{index}]");
                visit(kind, item, &indexed, &item_pattern, tree, next, issues);
            }
        }
        _ => {}
    }
}

fn matches_shape(shape: Shape, value: &Value) -> bool {
    matches!(
        (shape, value),
        (Shape::Map, Value::Map(_))
            | (Shape::List, Value::List(_))
            | (Shape::Text, Value::Text(_))
            | (Shape::Bool, Value::Bool(_))
            | (Shape::Integer, Value::Int(_))
            | (Shape::Number, Value::Int(_) | Value::Float(_))
            | (Shape::Timestamp, Value::Time(_))
    )
}

pub(super) fn member_path(parent: &str, member: &str) -> String {
    if parent.is_empty() {
        member.to_owned()
    } else {
        format!("{parent}.{member}")
    }
}

pub(super) fn value_kind(value: &Value) -> ValueKind {
    match value {
        Value::Null => ValueKind::Null,
        Value::Bool(_) => ValueKind::Bool,
        Value::Int(_) => ValueKind::Int,
        Value::Float(_) => ValueKind::Float,
        Value::Text(_) => ValueKind::Text,
        Value::Bytes(_) => ValueKind::Bytes,
        Value::Time(_) => ValueKind::Time,
        Value::List(_) => ValueKind::List,
        Value::Map(_) => ValueKind::Map,
    }
}

fn forbidden_name(name: &str) -> bool {
    matches!(
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
    )
}
