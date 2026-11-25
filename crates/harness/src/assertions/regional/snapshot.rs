//! Regional snapshot assertion logic.

use std::collections::HashMap;

use serde_json::{Value, json};

use crate::{assertions::helpers::check as add_check, HarnessError};

use super::observations::RegionalData;

fn ec2_identity(region: &str, path: &str, identity: &str) -> String {
    format!("arn:aws:ec2:{region}:000000000000:{path}/{identity}")
}

fn edge_exists(edges: &[(String, String, String)], from: &str, relation: &str, to: &str) -> bool {
    edges.iter().any(|(f, r, t)| f == from && r == relation && t == to)
}

pub fn check(
    checks: &mut Vec<Value>,
    data: &RegionalData,
    fixtures: &Value,
    region: &str,
) -> Result<(), HarnessError> {
    let resources = &data.resources;
    let _edges = &data.edges;
    let scopes = &data.scopes;

    let network = fixtures
        .get("ec2")
        .unwrap_or(&Value::Object(serde_json::Map::new()));

    let regional_kinds = get_regional_kinds();
    let kind_counts = compute_kind_counts(resources);
    check_kind_coverage(checks, &kind_counts, &regional_kinds);
    check_ec2_network_identities(checks, resources, network, region);
    check_scope_pagination(checks, scopes, &regional_kinds);

    Ok(())
}

fn get_regional_kinds() -> Vec<&'static str> {
    vec![
        "aws_vpc", "aws_vpc_subnet", "aws_vpc_security_group",
        "aws_ec2_network_interface", "aws_ebs_volume", "aws_ec2_instance",
        "aws_vpc_route_table", "aws_vpc_internet_gateway", "aws_vpc_nat_gateway",
        "aws_vpc_endpoint", "aws_vpc_network_acl", "aws_vpc_peering_connection",
        "aws_dynamodb_table", "aws_lambda_function", "aws_ecs_cluster",
        "aws_ecs_service", "aws_ecs_task", "aws_ecs_task_definition",
        "aws_eks_cluster", "aws_eks_node_group", "aws_elbv2_load_balancer",
        "aws_elbv2_listener", "aws_elbv2_rule", "aws_elbv2_target_group",
        "aws_elbv2_target_health", "aws_sqs_queue", "aws_sns_topic",
        "aws_sns_subscription", "aws_kms_key", "aws_kms_alias",
        "aws_step_functions_state_machine", "aws_step_functions_activity",
    ]
}

fn compute_kind_counts(
    resources: &HashMap<String, Value>,
) -> HashMap<String, u32> {
    resources
        .values()
        .filter_map(|r| {
            r.get("kind").and_then(Value::as_str).map(|s| (s.to_owned(), 1u32))
        })
        .fold(HashMap::new(), |mut map, (kind, count)| {
            let entry = map.entry(kind).or_insert(0);
            *entry = entry.saturating_add(count);
            map
        })
}

fn check_kind_coverage(
    checks: &mut Vec<Value>,
    kind_counts: &HashMap<String, u32>,
    regional_kinds: &[&'static str],
) {
    for kind in regional_kinds {
        let observed = kind_counts.get(*kind).copied().unwrap_or(0);
        add_check(
            checks,
            &format!("regional:kind-observed:{kind}"),
            observed > 0,
            Some(&json!({"category": "coverage", "observed": observed})),
        );
    }
}

fn check_ec2_network_identities(
    checks: &mut Vec<Value>,
    resources: &HashMap<String, Value>,
    network: &Value,
    region: &str,
) {
    let network_map = [
        ("vpc_ids", "vpc", "aws_vpc", "vpc_id"),
        ("subnet_ids", "subnet", "aws_vpc_subnet", "subnet_id"),
        ("security_group_ids", "security-group", "aws_vpc_security_group", "group_id"),
        ("network_interface_ids", "network-interface", "aws_ec2_network_interface", "network_interface_id"),
        ("volume_ids", "volume", "aws_ebs_volume", "volume_id"),
        ("instance_ids", "instance", "aws_ec2_instance", "instance_id"),
        ("route_table_ids", "route-table", "aws_vpc_route_table", "route_table_id"),
        ("internet_gateway_ids", "internet-gateway", "aws_vpc_internet_gateway", "internet_gateway_id"),
        ("nat_gateway_ids", "natgateway", "aws_vpc_nat_gateway", "nat_gateway_id"),
        ("vpc_endpoint_ids", "vpc-endpoint", "aws_vpc_endpoint", "vpc_endpoint_id"),
        ("network_acl_ids", "network-acl", "aws_vpc_network_acl", "network_acl_id"),
        ("vpc_peering_connection_ids", "vpc-peering-connection", "aws_vpc_peering_connection", "vpc_peering_connection_id"),
    ];

    for (key, path, kind, field) in network_map {
        check_network_resource_type(checks, resources, network, key, path, kind, field, region);
    }
}

fn check_network_resource_type(
    checks: &mut Vec<Value>,
    resources: &HashMap<String, Value>,
    network: &Value,
    key: &str,
    path: &str,
    kind: &str,
    field: &str,
    region: &str,
) {
    let ids = network
        .get(key)
        .and_then(Value::as_array)
        .unwrap_or(&Value::Array(vec![]));

    for id_value in ids {
        if let Some(id) = id_value.as_str() {
            check_single_network_identity(checks, resources, id, path, kind, field, region);
        }
    }
}

fn check_single_network_identity(
    checks: &mut Vec<Value>,
    resources: &HashMap<String, Value>,
    id: &str,
    path: &str,
    kind: &str,
    field: &str,
    region: &str,
) {
    let identity = ec2_identity(region, path, id);
    let row = resources.get(&identity);

    if row.is_none() {
        add_check(
            checks,
            &format!("regional:fixture:{identity}"),
            false,
            Some(&json!({"category": "identity", "kind": kind})),
        );
        return;
    }

    check_identity_kind(checks, identity, row, kind);
    check_identity_source(checks, identity, row);
    check_identity_account(checks, identity, row);
    check_identity_region(checks, identity, row, region);
    check_identity_field(checks, identity, row, field, id);
}

fn check_identity_kind(
    checks: &mut Vec<Value>,
    identity: String,
    row: &Value,
    kind: &str,
) {
    let kind_actual = row
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("");
    add_check(
        checks,
        &format!("regional:fixture:{identity}"),
        kind_actual == kind,
        Some(&json!({"category": "identity", "actual": kind_actual, "expected": kind})),
    );
}

fn check_identity_source(
    checks: &mut Vec<Value>,
    identity: String,
    row: &Value,
) {
    let source = row
        .get("source")
        .and_then(Value::as_str)
        .unwrap_or("");
    add_check(
        checks,
        &format!("regional:direct-source:{identity}"),
        source == "direct",
        Some(&json!({"category": "source"})),
    );
}

fn check_identity_account(
    checks: &mut Vec<Value>,
    identity: String,
    row: &Value,
) {
    let account = row
        .get("account_id")
        .and_then(Value::as_str)
        .unwrap_or("");
    add_check(
        checks,
        &format!("regional:account:{identity}"),
        account == "000000000000",
        Some(&json!({"category": "source"})),
    );
}

fn check_identity_region(
    checks: &mut Vec<Value>,
    identity: String,
    row: &Value,
    region: &str,
) {
    let region_actual = row
        .get("region")
        .and_then(Value::as_str)
        .unwrap_or("");
    add_check(
        checks,
        &format!("regional:region:{identity}"),
        region_actual == region,
        Some(&json!({"category": "source"})),
    );
}

fn check_identity_field(
    checks: &mut Vec<Value>,
    identity: String,
    row: &Value,
    field: &str,
    id: &str,
) {
    let props = row.get("properties").unwrap_or(&Value::Object(serde_json::Map::new()));
    let field_actual = props
        .get(field)
        .and_then(Value::as_str)
        .unwrap_or("");
    add_check(
        checks,
        &format!("regional:{identity}:{field}"),
        field_actual == id,
        Some(&json!({"category": "field"})),
    );
}

fn check_scope_pagination(
    checks: &mut Vec<Value>,
    scopes: &[Value],
    regional_kinds: &[&'static str],
) {
    let regional_scopes: Vec<&Value> = scopes
        .iter()
        .filter(|s| {
            let collector = s.get("collector").and_then(Value::as_str).unwrap_or("");
            regional_kinds.contains(&collector)
                || collector.starts_with("aws_ecs_")
                || collector.starts_with("aws_eks_")
                || collector.starts_with("aws_elbv2_")
        })
        .collect();

    check_root_scopes(checks, &regional_scopes);
    check_individual_scopes(checks, &regional_scopes);
}

fn check_root_scopes(
    checks: &mut Vec<Value>,
    regional_scopes: &[&Value],
) {
    let root_collectors = [
        "aws_vpc", "aws_vpc_subnet", "aws_vpc_security_group",
        "aws_ec2_network_interface", "aws_ebs_volume", "aws_ec2_instance",
        "aws_vpc_route_table", "aws_vpc_internet_gateway", "aws_vpc_nat_gateway",
        "aws_vpc_endpoint", "aws_vpc_network_acl", "aws_vpc_peering_connection",
        "aws_dynamodb_table", "aws_lambda_function", "aws_ecs_cluster",
        "aws_eks_cluster", "aws_elbv2_load_balancer", "aws_elbv2_target_group",
        "aws_sqs_queue", "aws_sns_topic", "aws_sns_subscription", "aws_kms_key",
        "aws_kms_alias", "aws_step_functions_state_machine",
        "aws_step_functions_activity",
    ];

    for collector in root_collectors {
        check_root_scope_present(checks, collector, regional_scopes);
    }
}

fn check_root_scope_present(
    checks: &mut Vec<Value>,
    collector: &str,
    regional_scopes: &[&Value],
) {
    let selected: Vec<&Value> = regional_scopes
        .iter()
        .filter(|s| {
            s.get("collector").and_then(Value::as_str) == Some(collector)
                && s.get("parent_id").is_none()
        })
        .collect();

    add_check(
        checks,
        &format!("regional:{collector}:root-scope-present"),
        selected.len() == 1,
        Some(&json!({"category": "pagination", "observed": selected.len()})),
    );
}

fn check_individual_scopes(
    checks: &mut Vec<Value>,
    regional_scopes: &[&Value],
) {
    for scope in regional_scopes {
        check_scope_status(checks, scope);
    }
}

fn check_scope_status(
    checks: &mut Vec<Value>,
    scope: &Value,
) {
    let collector = scope
        .get("collector")
        .and_then(Value::as_str)
        .unwrap_or("");
    let parent_id = scope
        .get("parent_id")
        .and_then(Value::as_str)
        .unwrap_or("");
    let label = if parent_id.is_empty() {
        collector.to_owned()
    } else {
        format!("{collector}:{parent_id}")
    };

    let complete = scope
        .get("complete")
        .and_then(Value::as_i64)
        .unwrap_or(-1);
    add_check(
        checks,
        &format!("regional:{label}:scope-complete"),
        complete == 1,
        Some(&json!({"category": "pagination"})),
    );

    let failed = scope
        .get("failed")
        .and_then(Value::as_i64)
        .unwrap_or(-1);
    add_check(
        checks,
        &format!("regional:{label}:scope-no-failure"),
        failed == 0,
        Some(&json!({"category": "pagination"})),
    );
}