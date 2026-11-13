//! Synthetic export fixture plus router-level request helpers.
//!
//! Every test binary includes this module; unused helpers in one binary only
//! produce dead-code warnings, never failures.

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::response::Response;
use cmdb_api::{AppState, build_router};
use cmdb_view::Snapshot;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::error::Error;
use tower::ServiceExt;

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
        "data": {"edge": {
            "from_id": from,
            "to_id": to,
            "relation": relation,
            "source": "direct",
            "observed_at": 2,
            "properties": {}
        }}
    })
}

fn header(scan_id: &str) -> Value {
    json!({
        "artifact": "aws-inventory.snapshot",
        "version": 1,
        "provenance": "normalized",
        "scan": {
            "scan_id": scan_id,
            "resources": 4,
            "edges": 5,
            "completed_scopes": 2,
            "failed_scopes": 1,
            "status": "complete",
            "started_at": 10,
            "finished_at": 20
        },
        "catalog_digest": "catalog-sha"
    })
}

/// The canonical fixture document.
pub fn document() -> Value {
    json!({
        "header": header("scan-fixture"),
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

/// Compact export bytes as read from disk.
pub fn bytes() -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(&document())
}

/// Parsed canonical snapshot.
pub fn snapshot() -> Result<Snapshot, Box<dyn Error>> {
    Ok(Snapshot::parse(&bytes()?)?)
}

/// Router over the canonical fixture.
pub fn app() -> Result<Router, Box<dyn Error>> {
    Ok(build_router(AppState::new(snapshot()?)))
}

/// Fingerprint of the canonical snapshot.
pub fn fingerprint() -> Result<String, Box<dyn Error>> {
    Ok(snapshot()?.fingerprint().to_owned())
}

/// Send any method with optional headers and an empty body.
pub async fn send(
    app: &Router,
    method: Method,
    uri: &str,
    headers: &[(&str, &str)],
) -> Result<Response, Box<dyn Error>> {
    let mut builder = Request::builder().method(method).uri(uri);
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }
    let request = builder.body(Body::empty())?;
    Ok(app.clone().oneshot(request).await?)
}

/// GET and decode the JSON body.
pub async fn get(app: &Router, uri: &str) -> Result<(StatusCode, Value), Box<dyn Error>> {
    let response = send(app, Method::GET, uri, &[]).await?;
    let status = response.status();
    Ok((status, body_json(response).await?))
}

/// Decode a response body as JSON.
pub async fn body_json(response: Response) -> Result<Value, Box<dyn Error>> {
    let bytes = axum::body::to_bytes(response.into_body(), 1_048_576).await?;
    Ok(serde_json::from_slice(&bytes)?)
}

/// The typed error code from a failure body.
pub fn error_code(body: &Value) -> Option<&str> {
    body.get("error")?.get("code")?.as_str()
}

/// The fingerprint from any response body.
pub fn body_fingerprint(body: &Value) -> Option<&str> {
    body.get("fingerprint")?.as_str()
}

/// The `resources` array of a page or graph response.
pub fn array<'a>(body: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    body.get(key)?.as_array()
}

/// Stable identity of the first element of an array field.
pub fn first_id<'a>(body: &'a Value, key: &str) -> Option<&'a str> {
    array(body, key)?.first()?.get("id")?.as_str()
}
