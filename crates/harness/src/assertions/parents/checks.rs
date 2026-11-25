//! Parent child check execution.

use serde_json::Value;

use crate::assertions::helpers::check as add_check;
use crate::assertions::parents::identity::normalized_id;

pub struct ChildCheckContext<'a> {
    pub children: &'a [Value],
    pub pages: &'a [Value],
    pub parents: &'a std::collections::HashMap<String, Value>,
    pub seeds: &'a std::collections::HashMap<String, Value>,
    pub observed_children: &'a [Value],
    pub parent_edges: &'a [Value],
    pub all_edges: &'a [Value],
    pub resources: &'a [Value],
    pub fixture: &'a Value,
}

pub fn child_checks(checks: &mut Vec<Value>, ctx: &ChildCheckContext) {
    for child in ctx.children {
        process_child(checks, ctx, child);
    }
}

fn process_child(
    checks: &mut Vec<Value>,
    ctx: &ChildCheckContext,
    child: &Value,
) {
    let name = child.get("name").and_then(Value::as_str).unwrap_or("");
    let type_name = child.get("type_name").and_then(Value::as_str).unwrap_or("");
    let parent_key = child
        .get("parent_key")
        .and_then(Value::as_str)
        .unwrap_or("");

    let parent = ctx
        .parents
        .get(parent_key)
        .cloned()
        .unwrap_or(Value::Null);

    let child_seed = ctx
        .seeds
        .get(&format!("child:{name}"))
        .cloned()
        .unwrap_or(Value::Null);
    let parent_seed = ctx
        .seeds
        .get(&format!("parent:{parent_key}"))
        .cloned()
        .unwrap_or(Value::Null);

    check_seed_status(checks, name, &child_seed, &parent_seed);

    let identity = normalized_id(child, &child_seed);
    let parent_identity = normalized_id(&parent, &parent_seed);

    check_child_identity(checks, name, child, type_name, identity, parent_identity, ctx);
}

fn check_seed_status(
    checks: &mut Vec<Value>,
    name: &str,
    child_seed: &Value,
    parent_seed: &Value,
) {
    let child_complete = child_seed
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("")
        == "complete";
    let parent_complete = parent_seed
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("")
        == "complete";

    add_check(checks, &format!("{name}:seed_complete"), child_complete, None);
    add_check(
        checks,
        &format!("{name}:parent_seed_complete"),
        parent_complete,
        None,
    );
}

fn check_child_identity(
    checks: &mut Vec<Value>,
    name: &str,
    child: &Value,
    type_name: &str,
    identity: Option<String>,
    parent_identity: Option<String>,
    ctx: &ChildCheckContext,
) {
    match (identity, parent_identity) {
        (Some(id), Some(_pid)) => {
            check_child_with_identity(checks, name, child, type_name, &id, ctx);
        }
        _ => {
            add_check(checks, &format!("{name}:one_exact_child"), false, None);
        }
    }
}

fn check_child_with_identity(
    checks: &mut Vec<Value>,
    name: &str,
    child: &Value,
    type_name: &str,
    id: &str,
    ctx: &ChildCheckContext,
) {
    let matches: Vec<&Value> = ctx
        .observed_children
        .iter()
        .filter(|r| r.get("id").and_then(Value::as_str).unwrap_or("") == id)
        .collect();

    add_check(
        checks,
        &format!("{name}:one_exact_child"),
        matches.len() == 1,
        None,
    );

    if matches.len() != 1 {
        return;
    }

    let Some(resource) = matches.first() else {
        return;
    };

    check_child_resource(checks, name, child, type_name, resource);
    check_child_edges(checks, name, child, id, ctx);
    check_child_architecture(checks, name, child, id, ctx);
}

fn check_child_resource(
    checks: &mut Vec<Value>,
    name: &str,
    child: &Value,
    type_name: &str,
    resource: &Value,
) {
    let properties = resource
        .get("properties")
        .cloned()
        .unwrap_or(Value::Null);

    let child_type_ok = properties
        .get("cloudformation_type")
        .and_then(Value::as_str)
        .unwrap_or("")
        == type_name;
    add_check(checks, &format!("{name}:resource_type"), child_type_ok, None);

    let type_kind = child
        .get("expected_kind")
        .and_then(Value::as_str)
        .unwrap_or("");
    let actual_kind = properties
        .get("details_available")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let kind_ok = match type_kind {
        "" | "resource" => actual_kind,
        "empty" => !actual_kind,
        _ => true,
    };
    add_check(checks, &format!("{name}:resource_kind"), kind_ok, None);
}

fn check_child_edges(
    checks: &mut Vec<Value>,
    name: &str,
    child: &Value,
    id: &str,
    ctx: &ChildCheckContext,
) {
    let actual_edges: Vec<&Value> = ctx
        .parent_edges
        .iter()
        .filter(|e| e.get("from_id").and_then(Value::as_str).unwrap_or("") == id)
        .collect();

    let expected_count = child
        .get("expected_edge_count")
        .and_then(Value::as_i64)
        .unwrap_or(-1);
    if expected_count >= 0 {
        let expected = usize::try_from(expected_count).unwrap_or(0);
        add_check(
            checks,
            &format!("{name}:edge_count"),
            actual_edges.len() == expected,
            None,
        );
    }

    let expected_rels: Vec<String> = child
        .get("expected_relations")
        .and_then(Value::as_array)
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect();

    if !expected_rels.is_empty() {
        let observed_rels: std::collections::HashSet<String> = actual_edges
            .iter()
            .filter_map(|e| {
                e.get("relation").and_then(Value::as_str).map(str::to_owned)
            })
            .collect();

        let rel_ok = expected_rels.iter().all(|r| observed_rels.contains(r));
        add_check(checks, &format!("{name}:edge_relations"), rel_ok, None);
    }
}

fn check_child_architecture(
    checks: &mut Vec<Value>,
    name: &str,
    child: &Value,
    id: &str,
    ctx: &ChildCheckContext,
) {
    let expected_architecture = child
        .get("expected_relationships")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let architecture: Vec<Value> = ctx
        .all_edges
        .iter()
        .filter(|e| {
            e.get("from_id").and_then(Value::as_str).unwrap_or("") == id
                && e.get("relation").and_then(Value::as_str).unwrap_or("") != "CHILD_OF"
        })
        .cloned()
        .collect();

    for edge in expected_architecture {
        check_architecture_edge(checks, name, &edge, &architecture);
    }
}

fn check_architecture_edge(
    checks: &mut Vec<Value>,
    name: &str,
    edge: &Value,
    architecture: &[Value],
) {
    let rel = edge
        .get("relationship")
        .and_then(Value::as_str)
        .unwrap_or("");
    let target = edge.get("target").and_then(Value::as_str).unwrap_or("");
    let type_name = edge.get("type_name").and_then(Value::as_str).unwrap_or("");

    let matching: Vec<Value> = architecture
        .iter()
        .filter(|e| {
            e.get("relation").and_then(Value::as_str).unwrap_or("") == rel
                && e.get("to_id").and_then(Value::as_str).unwrap_or("") == target
        })
        .cloned()
        .collect();

    let edge_passed = matching.len() == 1
        && match matching.first() {
            Some(m) => m
                .get("properties")
                .and_then(|p| p.get("cloudformation_type"))
                .and_then(Value::as_str)
                .unwrap_or("")
                == type_name,
            None => false,
        };

    add_check(
        checks,
        &format!("{name}:architecture_evidence:{rel}"),
        edge_passed,
        None,
    );
}

pub fn identity_checks(
    checks: &mut Vec<Value>,
    parents: &std::collections::HashMap<String, Value>,
    seeds: &std::collections::HashMap<String, Value>,
) {
    for (key, parent) in parents {
        let seed_key = format!("parent:{key}");
        let seed = seeds.get(&seed_key).cloned().unwrap_or(Value::Null);

        let identity = normalized_id(parent, &seed);
        if identity.is_some() {
            add_check(checks, &format!("{key}:normalized_identity"), true, None);
        }
    }
}

pub fn catalog_checks(
    checks: &mut Vec<Value>,
    children: &[Value],
    pages: &[Value],
    observed_children: &[Value],
) {
    let child_types: std::collections::HashSet<String> = children
        .iter()
        .chain(pages.iter())
        .filter_map(|c| {
            c.get("type_name")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .collect();

    let observed_types: std::collections::HashSet<String> = observed_children
        .iter()
        .filter_map(|r| {
            r.get("properties")
                .and_then(|p| p.get("cloudformation_type"))
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .collect();

    for t in &child_types {
        add_check(
            checks,
            &format!("catalog_type:{t}"),
            observed_types.contains(t),
            None,
        );
    }
}

pub fn scope_checks(checks: &mut Vec<Value>, fixture: &Value) {
    let children = fixture
        .get("children")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or(Vec::new());
    let pages = fixture
        .get("pagination_children")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or(Vec::new());

    let child_types: std::collections::HashSet<String> = children
        .iter()
        .chain(pages.iter())
        .filter_map(|c| {
            c.get("type_name")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .collect();

    let new_parent_types = [
        "AWS::ECS::Cluster",
        "AWS::EKS::Nodegroup",
        "AWS::EKS::PodIdentityAssociation",
    ];
    let mut discovery_types = child_types.clone();
    for t in &new_parent_types {
        discovery_types.insert(t.to_string());
    }

    let discovered: std::collections::HashSet<String> = children
        .iter()
        .filter_map(|c| {
            c.get("type_name")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .collect();

    for t in &discovery_types {
        add_check(
            checks,
            &format!("scope_discovery:{t}"),
            discovered.contains(t),
            None,
        );
    }
}
