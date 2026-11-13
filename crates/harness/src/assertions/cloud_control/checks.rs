//! Per-fixture expectations for the Cloud Control architecture assertions.

use std::collections::BTreeSet;

use serde_json::{Value, json};

use super::expected_id;
use crate::HarnessError;

/// One authored relationship the collector must have produced.
struct ExpectedEdge {
    relation: String,
    target: String,
    kind: String,
    owner: String,
    paths: Vec<String>,
}

fn typed_rows<'a>(rows: &'a [Value], type_name: &str) -> Vec<&'a Value> {
    rows.iter()
        .filter(|row| {
            row.pointer("/properties/cloudformation_type")
                .and_then(Value::as_str)
                == Some(type_name)
        })
        .collect()
}

fn typed_read_check(name: &str, observed: usize) -> Value {
    json!({
        "check": format!("cloud_control:{name}:one_typed_read"),
        "passed": observed == 1,
    })
}

/// Every assertion that one authored fixture implies.
pub(super) fn fixture_checks(fixture: &Value, rows: &[Value], edges: &[Value]) -> Vec<Value> {
    let name = text(fixture, "name");
    let type_name = text(fixture, "type_name");
    let identifier = match expected_id(fixture) {
        Ok(value) => value,
        Err(error) => return vec![failure(&format!("{name}:exact_identity"), &error)],
    };
    let observed = typed_rows(rows, &type_name);
    let mut checks = vec![typed_read_check(&name, observed.len())];
    let Some(resource) = observed.first().copied() else {
        return checks;
    };
    checks.extend(resource_checks(&name, &identifier, resource));
    checks.extend(edge_checks(
        &name,
        &identifier,
        &type_name,
        resource,
        fixture,
        edges,
    ));
    checks.extend(identity_checks(&name, fixture, resource));
    checks
}

fn identity_checks(name: &str, fixture: &Value, resource: &Value) -> Vec<Value> {
    let resolution = resource
        .pointer("/properties/identity_resolution")
        .and_then(Value::as_str);
    let mut checks = Vec::new();
    if fixture.get("opaque_identity") == Some(&Value::Bool(true)) {
        checks.push(json!({
            "check": format!("cloud_control:{name}:compound_identity_opaque"),
            "passed": resolution == Some("namespaced_provider_identifier"),
        }));
    }
    if name == "snapshot_self_identity_uses_accountless_arn" {
        checks.push(json!({
            "check": format!("cloud_control:{name}:accountless_identity"),
            "passed": resolution == Some("canonical_template_without_account_component"),
        }));
    }
    checks
}

fn resource_checks(name: &str, identifier: &str, resource: &Value) -> Vec<Value> {
    let encoded = serde_json::to_string(resource).unwrap_or_default();
    let detail = resource.get("source").and_then(Value::as_str) == Some("cloud_control")
        && resource.pointer("/properties/details_available") == Some(&Value::Bool(true))
        && resource
            .pointer("/properties/observation_detail")
            .and_then(Value::as_str)
            == Some("get_resource");
    let observed_at = resource.get("observed_at").and_then(Value::as_i64);
    vec![
        json!({"check": format!("cloud_control:{name}:exact_identity"),
            "passed": resource.get("id").and_then(Value::as_str) == Some(identifier)}),
        json!({"check": format!("cloud_control:{name}:completed_detail"), "passed": detail}),
        json!({"check": format!("cloud_control:{name}:redaction"),
            "passed": !encoded.contains("HIDDEN")}),
        json!({"check": format!("cloud_control:{name}:valid_observation_time"),
            "passed": observed_at.is_some_and(|at| at > 0)}),
    ]
}

fn edge_set_check(name: &str, expected: &[ExpectedEdge], actual: &[&Value]) -> Value {
    let actual_keys = edge_keys(actual);
    let expected_keys = expected_keys_of(expected);
    json!({
        "check": format!("cloud_control:{name}:exact_edge_set"),
        "passed": actual_keys == expected_keys && actual.len() == expected_keys.len(),
        "detail": {"expected": expected_keys.len(), "observed": actual.len()},
    })
}

fn edge_checks(
    name: &str,
    identifier: &str,
    type_name: &str,
    resource: &Value,
    fixture: &Value,
    edges: &[Value],
) -> Vec<Value> {
    let expected = match expected_edges(fixture) {
        Ok(values) => values,
        Err(error) => return vec![failure(&format!("{name}:exact_edge_set"), &error)],
    };
    let actual = outgoing_edges(edges, identifier);
    let mut checks = vec![edge_set_check(name, &expected, &actual)];
    let observed_at = resource.get("observed_at").cloned().unwrap_or(Value::Null);
    for want in &expected {
        checks.push(edge_result(name, want, &actual, type_name, &observed_at));
    }
    checks
}

fn outgoing_edges<'a>(edges: &'a [Value], identifier: &str) -> Vec<&'a Value> {
    edges
        .iter()
        .filter(|edge| edge.get("from_id").and_then(Value::as_str) == Some(identifier))
        .collect()
}

fn edge_keys(edges: &[&Value]) -> BTreeSet<(String, String)> {
    edges
        .iter()
        .map(|edge| (text(edge, "relation"), text(edge, "to_id")))
        .collect()
}

fn expected_keys_of(expected: &[ExpectedEdge]) -> BTreeSet<(String, String)> {
    expected
        .iter()
        .map(|edge| (edge.relation.clone(), edge.target.clone()))
        .collect()
}

fn relation_matches(edge: &Value, want: &ExpectedEdge) -> bool {
    edge.get("relation").and_then(Value::as_str) == Some(want.relation.as_str())
        && edge.get("to_id").and_then(Value::as_str) == Some(want.target.as_str())
}

fn edge_result(
    name: &str,
    want: &ExpectedEdge,
    actual: &[&Value],
    type_name: &str,
    observed_at: &Value,
) -> Value {
    let matching: Vec<&&Value> = actual
        .iter()
        .filter(|edge| relation_matches(edge, want))
        .collect();
    let label = format!("{name}:edge:{}:{}", want.relation, want.target);
    let passed = matching.len() == 1
        && matching.first().is_some_and(|edge| {
            edge_matches(edge, want, type_name) && edge.get("observed_at") == Some(observed_at)
        });
    json!({"check": format!("cloud_control:{label}"), "passed": passed})
}

fn expected_edge(row: &Value) -> Result<ExpectedEdge, HarnessError> {
    Ok(ExpectedEdge {
        relation: text(row, "relation"),
        target: text(row, "target"),
        kind: text(row, "kind"),
        owner: text(row, "owner"),
        paths: edge_paths(row)?,
    })
}

fn edge_paths(row: &Value) -> Result<Vec<String>, HarnessError> {
    row.get("paths")
        .and_then(Value::as_array)
        .ok_or_else(|| HarnessError::invalid("fixture edge paths are missing"))
        .map(|paths| {
            paths
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
}

fn expected_edges(fixture: &Value) -> Result<Vec<ExpectedEdge>, HarnessError> {
    let rows = fixture
        .get("edges")
        .and_then(Value::as_array)
        .ok_or_else(|| HarnessError::invalid("fixture edges are missing"))?;
    rows.iter().map(expected_edge).collect()
}

fn text(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn edge_matches(edge: &Value, want: &ExpectedEdge, type_name: &str) -> bool {
    let evidence = edge.get("properties").unwrap_or(&Value::Null);
    let paths: Vec<&str> = evidence
        .get("property_paths")
        .and_then(Value::as_array)
        .map(|rows| rows.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let expected_paths: BTreeSet<&str> = want.paths.iter().map(String::as_str).collect();
    let observed_paths: BTreeSet<&str> = paths.iter().copied().collect();
    evidence.get("target_kind").and_then(Value::as_str) == Some(want.kind.as_str())
        && evidence.get("owner_resolution").and_then(Value::as_str) == Some(want.owner.as_str())
        && evidence.get("cloudformation_type").and_then(Value::as_str) == Some(type_name)
        && observed_paths == expected_paths
        && paths.len() == want.paths.len()
        && edge.get("source").and_then(Value::as_str) == Some("cloud_control")
}

pub(super) fn failure(name: &str, error: &HarnessError) -> Value {
    json!({
        "check": format!("cloud_control:{name}"),
        "passed": false,
        "detail": error.to_string().chars().take(300).collect::<String>(),
    })
}
