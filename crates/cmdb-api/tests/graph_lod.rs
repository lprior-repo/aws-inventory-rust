//! Graph depth/caps, LOD bounds, coverage and cross-endpoint fingerprint parity.

mod support;

use axum::http::StatusCode;
use serde_json::Value;

#[tokio::test]
async fn graph_depth_two_reaches_dangling_targets() -> Result<(), Box<dyn std::error::Error>> {
    let app = support::app()?;
    let (status, body) = support::get(&app, "/api/v1/graph/i-001?depth=2").await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.get("root").and_then(Value::as_str), Some("i-001"));
    assert_eq!(body.get("depth").and_then(Value::as_u64), Some(2));
    let nodes = support::array(&body, "nodes").ok_or("missing nodes")?;
    let edges = support::array(&body, "edges").ok_or("missing edges")?;
    assert_eq!(nodes.len(), 5);
    assert_eq!(edges.len(), 5);
    assert_eq!(support::first_id(&body, "nodes"), Some("b-001"));
    let dangling = nodes
        .iter()
        .find(|node| node.get("id").and_then(Value::as_str) == Some(support::DANGLING))
        .ok_or("dangling node missing")?;
    assert_eq!(
        dangling.get("present").and_then(Value::as_bool),
        Some(false)
    );
    assert_eq!(
        body.get("truncated_nodes").and_then(Value::as_bool),
        Some(false)
    );
    assert_eq!(
        body.get("truncated_depth").and_then(Value::as_bool),
        Some(false)
    );
    Ok(())
}

#[tokio::test]
async fn graph_caps_and_depth_are_enforced() -> Result<(), Box<dyn std::error::Error>> {
    let app = support::app()?;
    let (status, capped) =
        support::get(&app, "/api/v1/graph/i-001?depth=2&max_nodes=1&max_edges=1").await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        support::array(&capped, "nodes").map(Vec::len),
        Some(1),
        "the root always fits the node cap"
    );
    assert_eq!(
        capped.get("truncated_nodes").and_then(Value::as_bool),
        Some(true)
    );

    let (status, shallow) = support::get(&app, "/api/v1/graph/i-001?depth=1").await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        shallow.get("truncated_depth").and_then(Value::as_bool),
        Some(true)
    );

    let (status, body) = support::get(&app, "/api/v1/graph/nope").await?;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(support::error_code(&body), Some("resource_not_found"));
    Ok(())
}

#[tokio::test]
async fn lod_bounds_kinds_accounts_and_top_degree() -> Result<(), Box<dyn std::error::Error>> {
    let app = support::app()?;
    let (status, body) = support::get(&app, "/api/v1/lod?top=1&buckets=1").await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.get("total_resources").and_then(Value::as_u64), Some(4));
    assert_eq!(support::array(&body, "kinds").map(Vec::len), Some(1));
    assert_eq!(support::array(&body, "accounts").map(Vec::len), Some(1));
    assert_eq!(support::array(&body, "top_degree").map(Vec::len), Some(1));
    assert_eq!(
        body.get("truncated_kinds").and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        body.get("truncated_accounts").and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(body.get("max_top_degree").and_then(Value::as_u64), Some(1));
    Ok(())
}

#[tokio::test]
async fn coverage_reports_scopes_and_failures() -> Result<(), Box<dyn std::error::Error>> {
    let app = support::app()?;
    let (status, body) = support::get(&app, "/api/v1/coverage").await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(support::array(&body, "scopes").map(Vec::len), Some(1));
    let failures = support::array(&body, "failures").ok_or("missing failures")?;
    assert_eq!(failures.len(), 1);
    assert_eq!(
        failures
            .first()
            .and_then(|failure| failure.get("category"))
            .and_then(Value::as_str),
        Some("access_denied")
    );
    Ok(())
}

#[tokio::test]
async fn every_endpoint_returns_the_same_fingerprint() -> Result<(), Box<dyn std::error::Error>> {
    let app = support::app()?;
    let expected = support::fingerprint()?;
    let uris = [
        "/health",
        "/api/v1/meta",
        "/api/v1/resources",
        "/api/v1/resources/i-001",
        "/api/v1/graph/i-001",
        "/api/v1/lod",
        "/api/v1/coverage",
    ];
    for uri in uris {
        let (status, body) = support::get(&app, uri).await?;
        assert_eq!(status, StatusCode::OK, "{uri}");
        assert_eq!(
            support::body_fingerprint(&body),
            Some(expected.as_str()),
            "{uri}"
        );
    }
    Ok(())
}
