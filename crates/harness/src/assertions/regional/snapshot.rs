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
    let edges = &data.edges;
    let scopes = &data.scopes;

    let network = fixtures
        .get("ec2")
        .unwrap_or(&Value::Object(serde_json::Map::new()));
    let roles = fixtures
        .get("roles")
        .unwrap_or(&Value::Object(serde_json::Map::new()));
    let lambda = fixtures
        .get("lambda")
        .unwrap_or(&Value::Object(serde_json::Map::new()));
    let dynamodb = fixtures
        .get("dynamodb")
        .unwrap_or(&Value::Object(serde_json::Map::new()));
    let ecs = fixtures
        .get("ecs")
        .unwrap_or(&Value::Object(serde_json::Map::new()));
    let eks = fixtures
        .get("eks")
        .unwrap_or(&Value::Object(serde_json::Map::new()));
    let elbv2 = fixtures
        .get("elbv2")
        .unwrap_or(&Value::Object(serde_json::Map::new()));
    let sqs = fixtures
        .get("sqs")
        .unwrap_or(&Value::Object(serde_json::Map::new()));
    let sns = fixtures
        .get("sns")
        .unwrap_or(&Value::Object(serde_json::Map::new()));
    let kms = fixtures
        .get("kms")
        .unwrap_or(&Value::Object(serde_json::Map::new()));
    let workflows = fixtures
        .get("step_functions")
        .unwrap_or(&Value::Object(serde_json::Map::new()));

    // Kind coverage
    let regional_kinds = vec![
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
    ];

    let kind_counts: HashMap<String, u32> = resources
        .values()
        .filter_map(|r| r.get("kind").and_then(Value::as_str).map(|s| (s.to_owned(), 1u32)))
        .fold(HashMap::new(), |mut map, (kind, count)| {
            *map.entry(kind).or_insert(0) += count;
            map
        });

    for kind in &regional_kinds {
        let observed = kind_counts.get(*kind).copied().unwrap_or(0);
        add_check(
            checks,
            &format!("regional:kind-observed:{kind}"),
            observed > 0,
            Some(&json!({"category": "coverage", "observed": observed})),
        );
    }

    // EC2 network resource identity checks
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
        let ids = network
            .get(key)
            .and_then(Value::as_array)
            .unwrap_or(&Value::Array(vec![]));
        for id_value in ids {
            if let Some(id) = id_value.as_str() {
                let identity = ec2_identity(region, path, id);
                let row = resources.get(&identity);

                if row.is_none() {
                    add_check(
                        checks,
                        &format!("regional:fixture:{identity}"),
                        false,
                        Some(&json!({"category": "identity", "kind": kind})),
                    );
                } else {
                    let kind_actual = row
                        .and_then(|r| r.get("kind"))
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    add_check(
                        checks,
                        &format!("regional:fixture:{identity}"),
                        kind_actual == kind,
                        Some(&json!({"category": "identity", "actual": kind_actual, "expected": kind})),
                    );

                    // Direct source check
                    let source = row
                        .and_then(|r| r.get("source"))
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    add_check(
                        checks,
                        &format!("regional:direct-source:{identity}"),
                        source == "direct",
                        Some(&json!({"category": "source"})),
                    );

                    // Account check
                    let account = row
                        .and_then(|r| r.get("account_id"))
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    add_check(
                        checks,
                        &format!("regional:account:{identity}"),
                        account == "000000000000",
                        Some(&json!({"category": "source"})),
                    );

                    // Region check
                    let region_actual = row
                        .and_then(|r| r.get("region"))
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    add_check(
                        checks,
                        &format!("regional:region:{identity}"),
                        region_actual == region,
                        Some(&json!({"category": "source"})),
                    );

                    // Field check
                    let props = row.and_then(|r| r.get("properties")).unwrap_or(&Value::Object(serde_json::Map::new()));
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
            }
        }
    }

    // Scope pagination checks
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

    let root_collectors: Vec<&str> = network_map
        .iter()
        .map(|entry| entry[2])
        .chain(["aws_dynamodb_table", "aws_lambda_function", "aws_ecs_cluster", "aws_eks_cluster", "aws_elbv2_load_balancer", "aws_elbv2_target_group", "aws_sqs_queue", "aws_sns_topic", "aws_sns_subscription", "aws_kms_key", "aws_kms_alias", "aws_step_functions_state_machine", "aws_step_functions_activity"])
        .collect();

    for collector in root_collectors {
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

    for scope in &regional_scopes {
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

    Ok(())
}