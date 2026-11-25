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
    ctx.children
        .iter()
        .for_each(|child| push_one_child_checks(checks, child, ctx));
}

fn push_one_child_checks(checks: &mut Vec<Value>, child: &Value, ctx: &ChildCheckContext) {
    let name = child.get("name").and_then(Value::as_str).unwrap_or("");
    let type_name = child.get("type_name").and_then(Value::as_str).unwrap_or("");
    let parent_key = child
        .get("parent_key")
        .and_then(Value::as_str)
        .unwrap_or("");

    let parent = ctx.parents.get(parent_key).cloned().unwrap_or(Value::Null);

    let (seed, parent_seed) = seed_pair(ctx.seeds, name, parent_key);
    push_seed_complete_checks(checks, name, &seed, &parent_seed);

    let identity = normalized_id(child, &seed);
    let parent_identity = normalized_id(&parent, &parent_seed);

    match (identity, parent_identity) {
        (Some(id), Some(_pid)) => {
            push_identified_child_checks(checks, child, ctx, name, type_name, &id);
        }
        _ => {
            add_check(checks, &format!("{name}:one_exact_child"), false, None);
        }
    }
}

fn seed_pair(
    seeds: &std::collections::HashMap<String, Value>,
    name: &str,
    parent_key: &str,
) -> (Value, Value) {
    let child_seed_key = format!("child:{name}");
    let seed = seeds.get(&child_seed_key).cloned().unwrap_or(Value::Null);
    let parent_seed_key = format!("parent:{parent_key}");
    let parent_seed = seeds.get(&parent_seed_key).cloned().unwrap_or(Value::Null);
    (seed, parent_seed)
}

fn push_seed_complete_checks(
    checks: &mut Vec<Value>,
    name: &str,
    seed: &Value,
    parent_seed: &Value,
) {
    let seed_status = seed.get("status").and_then(Value::as_str).unwrap_or("");
    let parent_seed_status = parent_seed
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("");

    let child_complete = seed_status == "complete";
    let parent_complete = parent_seed_status == "complete";

    add_check(
        checks,
        &format!("{name}:seed_complete"),
        child_complete,
        None,
    );
    add_check(
        checks,
        &format!("{name}:parent_seed_complete"),
        parent_complete,
        None,
    );
}

fn push_identified_child_checks(
    checks: &mut Vec<Value>,
    child: &Value,
    ctx: &ChildCheckContext,
    name: &str,
    type_name: &str,
    id: &str,
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

    if matches.len() == 1 {
        let resource = match matches.first() {
            Some(r) => r,
            None => return,
        };
        push_resource_checks(checks, child, ctx, name, type_name, id, resource);
    }
}

fn push_resource_checks(
    checks: &mut Vec<Value>,
    child: &Value,
    ctx: &ChildCheckContext,
    name: &str,
    type_name: &str,
    id: &str,
    resource: &Value,
) {
    let properties = resource.get("properties").cloned().unwrap_or(Value::Null);
    push_resource_type_checks(checks, name, type_name, child, &properties);

    let actual_edges: Vec<&Value> = ctx
        .parent_edges
        .iter()
        .filter(|e| e.get("from_id").and_then(Value::as_str).unwrap_or("") == id)
        .collect();

    push_edge_count_check(checks, name, child, actual_edges.len());
    push_edge_relation_checks(checks, name, child, &actual_edges);
    push_architecture_checks(checks, name, child, ctx.all_edges, id);
}

fn push_resource_type_checks(
    checks: &mut Vec<Value>,
    name: &str,
    type_name: &str,
    child: &Value,
    properties: &Value,
) {
    let child_type_ok = properties
        .get("cloudformation_type")
        .and_then(Value::as_str)
        .unwrap_or("")
        == type_name;
    add_check(
        checks,
        &format!("{name}:resource_type"),
        child_type_ok,
        None,
    );

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

fn push_edge_count_check(checks: &mut Vec<Value>, name: &str, child: &Value, actual_count: usize) {
    let expected_count = child
        .get("expected_edge_count")
        .and_then(Value::as_i64)
        .unwrap_or(-1);
    if expected_count >= 0 {
        let expected = usize::try_from(expected_count).unwrap_or(0);
        add_check(
            checks,
            &format!("{name}:edge_count"),
            actual_count == expected,
            None,
        );
    }
}

fn push_edge_relation_checks(
    checks: &mut Vec<Value>,
    name: &str,
    child: &Value,
    actual_edges: &[&Value],
) {
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
            .filter_map(|e| e.get("relation").and_then(Value::as_str).map(str::to_owned))
            .collect();

        let rel_ok = expected_rels.iter().all(|r| observed_rels.contains(r));
        add_check(checks, &format!("{name}:edge_relations"), rel_ok, None);
    }
}

fn push_architecture_checks(
    checks: &mut Vec<Value>,
    name: &str,
    child: &Value,
    all_edges: &[Value],
    id: &str,
) {
    let expected_architecture = child
        .get("expected_relationships")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let architecture: Vec<Value> = all_edges
        .iter()
        .filter(|e| {
            e.get("from_id").and_then(Value::as_str).unwrap_or("") == id
                && e.get("relation").and_then(Value::as_str).unwrap_or("") != "CHILD_OF"
        })
        .cloned()
        .collect();

    for edge in expected_architecture {
        push_architecture_edge_check(checks, name, &edge, &architecture);
    }
}

fn push_architecture_edge_check(
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
            Some(m) => {
                m.get("properties")
                    .and_then(|p| p.get("cloudformation_type"))
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    == type_name
            }
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
