//! Router-level tests: health/meta, filtering, pagination and typed errors.

mod support;

use axum::http::{Method, StatusCode, header};
use serde_json::Value;

#[tokio::test]
async fn health_and_meta_share_fingerprint() -> Result<(), Box<dyn std::error::Error>> {
    let app = support::app()?;
    let (health_status, health) = support::get(&app, "/health").await?;
    let (meta_status, meta) = support::get(&app, "/api/v1/meta").await?;
    assert_eq!(health_status, StatusCode::OK);
    assert_eq!(meta_status, StatusCode::OK);
    assert_eq!(health.get("status").and_then(Value::as_str), Some("ok"));
    let fingerprint = support::body_fingerprint(&meta);
    assert_eq!(support::body_fingerprint(&health), fingerprint);
    assert_eq!(
        meta.get("parsed_resources").and_then(Value::as_u64),
        Some(4)
    );
    assert_eq!(meta.get("parsed_edges").and_then(Value::as_u64), Some(5));
    assert_eq!(
        meta.get("scan_id").and_then(Value::as_str),
        Some("scan-fixture")
    );
    assert_eq!(
        meta.get("provenance").and_then(Value::as_str),
        Some("normalized")
    );
    Ok(())
}

#[tokio::test]
async fn filters_paginate_and_stay_deterministic() -> Result<(), Box<dyn std::error::Error>> {
    let app = support::app()?;
    let uri = "/api/v1/resources?account=111111111111&kind=aws_ec2_instance&contains=web&limit=1";
    let (status, first) = support::get(&app, uri).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(first.get("total_matching").and_then(Value::as_u64), Some(2));
    assert_eq!(first.get("limit").and_then(Value::as_u64), Some(1));
    assert_eq!(first.get("next_cursor").and_then(Value::as_str), Some("1"));
    assert_eq!(
        support::first_id(&first, "resources"),
        Some(support::INSTANCE_ONE)
    );
    let fp = support::fingerprint()?;
    assert_eq!(support::body_fingerprint(&first), Some(fp.as_str()));

    let (_, repeat) = support::get(&app, uri).await?;
    assert_eq!(
        first, repeat,
        "identical requests must return identical pages"
    );

    let second_uri = "/api/v1/resources?account=111111111111&kind=aws_ec2_instance&contains=web&limit=1&cursor=1";
    let (status, second) = support::get(&app, second_uri).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        support::first_id(&second, "resources"),
        Some(support::INSTANCE_TWO)
    );
    assert_eq!(second.get("offset").and_then(Value::as_u64), Some(1));
    assert_eq!(second.get("next_cursor").and_then(Value::as_str), None);
    Ok(())
}

#[tokio::test]
async fn resource_detail_carries_identity_and_fingerprint() -> Result<(), Box<dyn std::error::Error>>
{
    let app = support::app()?;
    let (status, body) = support::get(&app, "/api/v1/resources/i-001").await?;
    assert_eq!(status, StatusCode::OK);
    let fp = support::fingerprint()?;
    assert_eq!(support::body_fingerprint(&body), Some(fp.as_str()));
    let resource = body.get("resource").ok_or("missing resource")?;
    assert_eq!(resource.get("id").and_then(Value::as_str), Some("i-001"));
    assert_eq!(
        resource.get("account_id").and_then(Value::as_str),
        Some(support::ACCOUNT_A)
    );
    assert_eq!(
        resource
            .get("tags")
            .and_then(|tags| tags.get("env"))
            .and_then(Value::as_str),
        Some("prod")
    );
    Ok(())
}

#[tokio::test]
async fn unknown_resource_and_route_are_typed_404s() -> Result<(), Box<dyn std::error::Error>> {
    let app = support::app()?;
    let (status, body) = support::get(&app, "/api/v1/resources/nope").await?;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(support::error_code(&body), Some("resource_not_found"));

    let (status, body) = support::get(&app, "/api/v1/resources/").await?;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(support::error_code(&body), Some("not_found"));

    let (status, body) = support::get(&app, "/api/v1/nope").await?;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(support::error_code(&body), Some("not_found"));
    Ok(())
}

#[tokio::test]
async fn malformed_and_out_of_range_queries_are_typed_400s()
-> Result<(), Box<dyn std::error::Error>> {
    let app = support::app()?;
    let cases = [
        ("/api/v1/resources?limit=abc", "malformed_query"),
        ("/api/v1/resources?limit=0", "invalid_page_limit"),
        ("/api/v1/resources?limit=10001", "invalid_page_limit"),
        ("/api/v1/resources?cursor=", "invalid_cursor"),
        ("/api/v1/graph/i-001?depth=3", "invalid_graph_depth"),
        ("/api/v1/graph/i-001?max_nodes=10001", "invalid_graph_limit"),
        ("/api/v1/graph/i-001?max_edges=20001", "invalid_graph_limit"),
        ("/api/v1/lod?top=1001", "invalid_lod_limit"),
        ("/api/v1/lod?buckets=1025", "invalid_lod_limit"),
    ];
    for (uri, code) in cases {
        let (status, body) = support::get(&app, uri).await?;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{uri}");
        assert_eq!(support::error_code(&body), Some(code), "{uri}");
    }
    let (status, _) = support::get(&app, "/api/v1/resources?limit=10000").await?;
    assert_eq!(status, StatusCode::OK);
    Ok(())
}

#[tokio::test]
async fn query_and_body_bounds_are_enforced() -> Result<(), Box<dyn std::error::Error>> {
    let app = support::app()?;
    let uri = format!("/api/v1/resources?contains={}", "a".repeat(4096));
    let (status, body) = support::get(&app, &uri).await?;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(support::error_code(&body), Some("query_too_long"));

    let response = support::send(
        &app,
        Method::GET,
        "/api/v1/resources",
        &[("content-length", "2048")],
    )
    .await?;
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    let body = support::body_json(response).await?;
    assert_eq!(support::error_code(&body), Some("payload_too_large"));
    Ok(())
}

#[tokio::test]
async fn cors_is_get_only_and_writes_are_refused() -> Result<(), Box<dyn std::error::Error>> {
    let app = support::app()?;
    let preflight = support::send(
        &app,
        Method::OPTIONS,
        "/api/v1/resources",
        &[
            ("origin", "http://localhost:3000"),
            ("access-control-request-method", "GET"),
        ],
    )
    .await?;
    assert_eq!(preflight.status(), StatusCode::OK);
    assert!(
        preflight
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_some()
    );
    let methods = preflight
        .headers()
        .get(header::ACCESS_CONTROL_ALLOW_METHODS)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    assert!(methods.contains("GET"), "{methods}");
    assert!(!methods.contains("POST"), "{methods}");

    let get = support::send(&app, Method::GET, "/health", &[]).await?;
    assert!(
        get.headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_some()
    );

    let post = support::send(&app, Method::POST, "/api/v1/resources", &[]).await?;
    assert_eq!(post.status(), StatusCode::METHOD_NOT_ALLOWED);
    let body = support::body_json(post).await?;
    assert_eq!(support::error_code(&body), Some("method_not_allowed"));
    Ok(())
}
