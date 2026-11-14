#![allow(dead_code)]

use cmdb_view::Snapshot;
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub const ACCOUNT_A: &str = "111111111111";
pub const ACCOUNT_B: &str = "222222222222";
pub const INSTANCE_ONE: &str = "i-001";
pub const INSTANCE_TWO: &str = "i-002";
pub const BUCKET: &str = "b-001";
pub const ROLE: &str = "r-001";
pub const DANGLING: &str = "missing-role";

fn resource(
    id: &str,
    account: &str,
    region: &str,
    kind: &str,
    name: Option<&str>,
    tags: &[(&str, &str)],
) -> Value {
    let tags: BTreeMap<&str, &str> = tags.iter().copied().collect();
    json!({
        "id": id,
        "account_id": account,
        "region": region,
        "kind": kind,
        "name": name,
        "properties": {"k": "v"},
        "tags": tags,
        "source": "direct",
        "observed_at": 1
    })
}

fn edge(from: &str, to: &str, relation: &str) -> Value {
    json!({
        "record": "edge",
        "data": {
            "edge": {
                "from_id": from,
                "to_id": to,
                "relation": relation,
                "source": "direct",
                "observed_at": 2,
                "properties": {}
            }
        }
    })
}

fn header(scan_id: &str, resources: u64, edges: u64) -> Value {
    json!({
        "artifact": "aws-inventory.snapshot",
        "version": 1,
        "provenance": "normalized",
        "scan": {
            "scan_id": scan_id,
            "resources": resources,
            "edges": edges,
            "completed_scopes": 2,
            "failed_scopes": 1,
            "status": "complete",
            "started_at": 10,
            "finished_at": 20
        },
        "catalog_digest": "catalog-sha"
    })
}

pub fn document() -> Value {
    json!({
        "header": header("scan-fixture", 4, 5),
        "records": [
            {"record": "resource", "data": {"resource": resource(
                INSTANCE_ONE, ACCOUNT_A, "us-east-1", "aws_ec2_instance", Some("web-one"),
                &[("Name", "web-one"), ("env", "prod")])}},
            {"record": "resource", "data": {"resource": resource(
                INSTANCE_TWO, ACCOUNT_A, "us-east-1", "aws_ec2_instance", Some("Web-Two"),
                &[("Name", "Web-Two")])}},
            {"record": "resource", "data": {"resource": resource(
                BUCKET, ACCOUNT_B, "eu-west-1", "aws_s3_bucket", Some("bucket-one"),
                &[("team", "storage")])}},
            {"record": "resource", "data": {"resource": resource(
                ROLE, ACCOUNT_B, "us-east-1", "aws_iam_role", Some("role-one"), &[])}},
            edge(INSTANCE_ONE, INSTANCE_TWO, "contains"),
            edge(INSTANCE_TWO, INSTANCE_ONE, "references"),
            edge(INSTANCE_ONE, BUCKET, "references"),
            edge(BUCKET, DANGLING, "references"),
            edge(INSTANCE_TWO, ROLE, "references"),
            {"record": "scope", "data": {
                "scope": {"account_id": ACCOUNT_A, "region": "us-east-1",
                    "collector": "ec2", "parent_id": null, "source": "direct"},
                "last_page": 1, "complete": true, "failed": false}},
            {"record": "failure", "data": {
                "scope": {"account_id": ACCOUNT_B, "region": "eu-west-1",
                    "collector": "s3", "parent_id": null, "source": "direct"},
                "category": "access_denied", "message": "denied"}}
        ],
        "summary": {"resources": 4, "edges": 5, "scopes": 1, "failures": 1, "records": 11}
    })
}

pub fn bytes() -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(&document())
}

pub fn bytes_with_scan_id(scan_id: &str) -> Result<Vec<u8>, serde_json::Error> {
    let mut value = document();
    if let Some(target) = value
        .get_mut("header")
        .and_then(|header| header.get_mut("scan"))
        .and_then(|scan| scan.get_mut("scan_id"))
    {
        *target = json!(scan_id);
    }
    serde_json::to_vec(&value)
}

pub fn duplicate_bytes() -> Result<Vec<u8>, serde_json::Error> {
    let mut value = document();
    if let Some(provenance) = value
        .get_mut("header")
        .and_then(|header| header.get_mut("provenance"))
    {
        *provenance = json!("all_observations");
    }
    let originals: Vec<Value> = value
        .get("records")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if let Some(records) = value.get_mut("records").and_then(Value::as_array_mut) {
        records.extend(originals);
    }
    serde_json::to_vec(&value)
}

pub fn snapshot() -> Result<Snapshot, Box<dyn std::error::Error>> {
    Ok(Snapshot::parse(&bytes()?)?)
}
