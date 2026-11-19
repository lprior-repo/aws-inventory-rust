//! Negative controls for the regional architecture assertion oracle.

use serde_json::json;
use sqlx::sqlite::{SqliteConnectOptions, SqliteConnection};
use sqlx::{Connection, Executor};

use inventory_harness::assertions::regional;
use inventory_harness::HarnessError;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn has_failed_check(checks: &[serde_json::Value]) -> bool {
    checks.iter().any(|check| {
        check
            .get("passed")
            .and_then(serde_json::Value::as_bool)
            == Some(false)
    })
}

#[test]
fn fixture_validation_requires_vpc_ids() -> TestResult {
    let fixtures = json!({});
    let mut checks = Vec::new();
    let valid = regional::fixture::validate(&fixtures, &mut checks);
    assert!(!valid, "empty fixtures should not validate");
    assert!(has_failed_check(&checks), "validation should report failures");
    Ok(())
}

#[test]
fn fixture_validation_requires_iam_roles() -> TestResult {
    let fixtures = json!({
        "ec2": {"vpc_ids": ["vpc-1", "vpc-2"], "vpc_cidr_blocks": ["10.0.0.0/16", "10.1.0.0/16"], "subnet_ids": ["subnet-1", "subnet-2"], "security_group_ids": ["sg-1", "sg-2"], "network_interface_ids": ["eni-1", "eni-2"], "volume_ids": ["vol-1", "vol-2"], "route_table_ids": ["rt-1", "rt-2"], "internet_gateway_ids": ["igw-1", "igw-2"], "nat_gateway_ids": ["nat-1", "nat-2"], "vpc_endpoint_ids": ["vpce-1", "vpce-2"], "network_acl_ids": ["acl-1", "acl-2"], "instance_ids": ["i-1", "i-2", "i-3", "i-4", "i-5", "i-6"], "vpc_peering_connection_ids": ["pcx-1"], "image_id": "ami-1"},
        "roles": {},
        "lambda": {},
        "dynamodb": {},
        "ecs": {},
        "eks": {},
        "elbv2": {},
        "sqs": {},
        "sns": {},
        "kms": {},
        "step_functions": {}
    });
    let mut checks = Vec::new();
    let valid = regional::fixture::validate(&fixtures, &mut checks);
    assert!(!valid, "missing role ARNs should not validate");
    Ok(())
}

#[test]
fn fixture_validation_requires_kms_alias_target_resolution() -> TestResult {
    let fixtures = json!({
        "ec2": {"vpc_ids": ["vpc-1", "vpc-2"], "vpc_cidr_blocks": ["10.0.0.0/16", "10.1.0.0/16"], "subnet_ids": ["subnet-1", "subnet-2"], "security_group_ids": ["sg-1", "sg-2"], "network_interface_ids": ["eni-1", "eni-2"], "volume_ids": ["vol-1", "vol-2"], "route_table_ids": ["rt-1", "rt-2"], "internet_gateway_ids": ["igw-1", "igw-2"], "nat_gateway_ids": ["nat-1", "nat-2"], "vpc_endpoint_ids": ["vpce-1", "vpce-2"], "network_acl_ids": ["acl-1", "acl-2"], "instance_ids": ["i-1", "i-2", "i-3", "i-4", "i-5", "i-6"], "vpc_peering_connection_ids": ["pcx-1"], "image_id": "ami-1"},
        "roles": {"lambda": "arn:aws:iam::000000000000:role/lambda", "ecs": "arn:aws:iam::000000000000:role/ecs", "eks_cluster": "arn:aws:iam::000000000000:role/eks", "eks_node": "arn:aws:iam::000000000000:role/eks-node", "states": "arn:aws:iam::000000000000:role/states"},
        "lambda": {"function_arns": ["arn:aws:lambda:us-east-1:000000000000:function/f1", "arn:aws:lambda:us-east-1:000000000000:function/f2"]},
        "dynamodb": {"table_names": ["table-1", "table-2"]},
        "ecs": {"task_definition_arns": ["arn:aws:ecs:us-east-1:000000000000:task-definition/t1:1", "arn:aws:ecs:us-east-1:000000000000:task-definition/t2:1", "arn:aws:ecs:us-east-1:000000000000:task-definition/t3:1", "arn:aws:ecs:us-east-1:000000000000:task-definition/t4:1"], "clusters": [{"arn": "arn:aws:ecs:us-east-1:000000000000:cluster/c1", "service_arns": ["arn:aws:ecs:us-east-1:000000000000:service/c1/s1", "arn:aws:ecs:us-east-1:000000000000:service/c1/s2"], "task_arns": ["arn:aws:ecs:us-east-1:000000000000:task/c1/t1", "arn:aws:ecs:us-east-1:000000000000:task/c1/t2"]}, {"arn": "arn:aws:ecs:us-east-1:000000000000:cluster/c2", "service_arns": ["arn:aws:ecs:us-east-1:000000000000:service/c2/s1", "arn:aws:ecs:us-east-1:000000000000:service/c2/s2"], "task_arns": ["arn:aws:ecs:us-east-1:000000000000:task/c2/t1", "arn:aws:ecs:us-east-1:000000000000:task/c2/t2"]}]},
        "eks": {"clusters": [{"arn": "arn:aws:eks:us-east-1:000000000000:cluster/e1", "name": "e1", "nodegroups": ["ng1", "ng2"]}]},
        "elbv2": {"target_group_arns": ["arn:aws:elasticloadbalancing:us-east-1:000000000000:targetgroup/tg1/abc", "arn:aws:elasticloadbalancing:us-east-1:000000000000:targetgroup/tg2/def"], "load_balancers": [{"arn": "arn:aws:elasticloadbalancing:us-east-1:000000000000:loadbalancer/app/lb1/abc", "listeners": ["arn:aws:elasticloadbalancing:us-east-1:000000000000:listener/app/lb1/abc/1", "arn:aws:elasticloadbalancing:us-east-1:000000000000:listener/app/lb1/abc/2"], "rule_arns": ["arn:aws:elasticloadbalancing:us-east-1:000000000000:listener-rule/app/lb1/abc/1/r1", "arn:aws:elasticloadbalancing:us-east-1:000000000000:listener-rule/app/lb1/abc/1/r2", "arn:aws:elasticloadbalancing:us-east-1:000000000000:listener-rule/app/lb1/abc/2/r3", "arn:aws:elasticloadbalancing:us-east-1:000000000000:listener-rule/app/lb1/abc/2/r4"]}, {"arn": "arn:aws:elasticloadbalancing:us-east-1:000000000000:loadbalancer/app/lb2/def", "listeners": ["arn:aws:elasticloadbalancing:us-east-1:000000000000:listener/app/lb2/def/1", "arn:aws:elasticloadbalancing:us-east-1:000000000000:listener/app/lb2/def/2"], "rule_arns": ["arn:aws:elasticloadbalancing:us-east-1:000000000000:listener-rule/app/lb2/def/1/r5", "arn:aws:elasticloadbalancing:us-east-1:000000000000:listener-rule/app/lb2/def/1/r6", "arn:aws:elasticloadbalancing:us-east-1:000000000000:listener-rule/app/lb2/def/2/r7", "arn:aws:elasticloadbalancing:us-east-1:000000000000:listener-rule/app/lb2/def/2/r8"]}]}
    });
    let mut checks = Vec::new();
    let valid = regional::fixture::validate(&fixtures, &mut checks);
    assert!(!valid, "incomplete KMS alias targets should not validate");
    Ok(())
}

#[tokio::test]
async fn given_empty_database_when_asserting_regional_then_failures() -> TestResult {
    let temp = tempfile::tempdir()?;
    let db_path = temp.path().join("test.db");
    let options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true);
    let mut connection = SqliteConnection::connect_with(&options).await?;
    connection.execute("CREATE TABLE resource_observations (id INTEGER PRIMARY KEY, scan_id TEXT, source TEXT, resource_json TEXT)").await?;
    connection.execute("CREATE TABLE edge_observations (id INTEGER PRIMARY KEY, scan_id TEXT, source TEXT, edge_json TEXT)").await?;
    connection.execute("CREATE TABLE scans (scan_id TEXT PRIMARY KEY, status TEXT)").await?;
    connection.close().await?;

    let fixtures = json!({});
    let checks = regional::check(&root(), &db_path, "scan", Some(&fixtures), None).await?;

    assert!(has_failed_check(&checks), "empty database should produce failures");
    Ok(())
}