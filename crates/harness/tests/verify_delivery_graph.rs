use inventory_harness::verify_delivery;
use serde_json::json;

mod verify_delivery_support;

use verify_delivery_support::{SCAN, create_temp_dir};

fn graph_model() -> serde_json::Value {
    json!({
        "scan": {"scan_id": "scan-a", "status": "complete"},
        "counts": {"resources": 2, "edges": 1, "nodes": 3, "completed": 1, "failed": 0},
        "node_rows": [
            ["r1", "scan-a", "observed", "scan-a::r1", "ec2", "000000000000", "us-east-1", "web", "{\"size\":\"small\"}", "{\"env\":\"prod\"}", "direct", 1, 12, 1],
            ["r2", "scan-a", "observed", "scan-a::r2", "s3", "000000000000", "us-east-1", "bucket", "{}", "{}", "direct", 1, 12, 1]
        ],
        "edge_rows": [
            ["r1", "r2", "RELATES_TO", "relation", "direct", "scan-a", "scan-a", 1, "{}", 4, "scan-a::r1", "scan-a::r2"]
        ],
        "tag_rows": []
    })
}
fn graph_model_with_has_tag() -> serde_json::Value {
    let mut model = graph_model();
    if let Some(slot) = model.get_mut("tag_rows") {
        *slot = json!([[
            "r1",
            "scan-a::r1",
            "000000000000",
            "us-east-1",
            "env",
            "prod",
            "inventory.tags"
        ]]);
    }
    model
}

struct Observed {
    nodes: serde_json::Value,
    edges: serde_json::Value,
    tags: serde_json::Value,
    marker: serde_json::Value,
    total: serde_json::Value,
}

fn observed_from(model: &serde_json::Value) -> Observed {
    Observed {
        nodes: model["node_rows"].clone(),
        edges: model["edge_rows"].clone(),
        tags: model["tag_rows"].clone(),
        marker: json!([["scan-a", "complete", 2, 1, 4, 1]]),
        total: json!([[3]]),
    }
}

fn compare(
    model: &serde_json::Value,
    o: &Observed,
) -> Result<verify_delivery::GraphComparison, inventory_harness::HarnessError> {
    verify_delivery::compare_graph_snapshot(model, &o.nodes, &o.edges, &o.tags, &o.marker, &o.total)
}

#[test]
fn graph_comparison_matches_identical_generation() -> Result<(), Box<dyn std::error::Error>> {
    let model = graph_model();
    let o = observed_from(&model);
    assert_eq!(
        compare(&model, &o)?,
        verify_delivery::GraphComparison::Matched
    );
    Ok(())
}

#[test]
fn graph_comparison_fails_stale_property() -> Result<(), Box<dyn std::error::Error>> {
    let model = graph_model();
    let mut o = observed_from(&model);
    *o.nodes
        .get_mut(0)
        .and_then(|row| row.get_mut(8))
        .ok_or("node row cell missing")? = json!("{\"size\":\"large\"}");
    let verdict = compare(&model, &o);
    assert!(
        verdict.is_err(),
        "stale property must fail, got {verdict:?}"
    );
    Ok(())
}

#[test]
fn graph_comparison_names_node_when_property_diverges() -> Result<(), Box<dyn std::error::Error>> {
    let model = graph_model();
    let mut o = observed_from(&model);
    *o.nodes
        .get_mut(0)
        .and_then(|row| row.get_mut(8))
        .ok_or("node row cell missing")? = json!("{\"size\":\"large\"}");
    let message = match compare(&model, &o) {
        Err(error) => error.to_string(),
        other => return Err(format!("property divergence must fail, got {other:?}").into()),
    };
    assert!(
        message.contains("scan-a::r1"),
        "message must name node: {message}"
    );
    assert!(
        message.contains("properties_json"),
        "message must name property: {message}"
    );
    Ok(())
}

#[test]
fn graph_comparison_fails_extra_edge() -> Result<(), Box<dyn std::error::Error>> {
    let model = graph_model();
    let mut o = observed_from(&model);
    o.edges
        .as_array_mut()
        .ok_or("edges not an array")?
        .push(json!([
            "r2",
            "r1",
            "RELATES_TO",
            "collector",
            "{}",
            "scan-a",
            "scan-a",
            1,
            "{}",
            4,
            "scan-a::r2",
            "scan-a::r1"
        ]));
    let verdict = compare(&model, &o);
    assert!(verdict.is_err(), "extra edge must fail, got {verdict:?}");
    Ok(())
}

#[test]
fn graph_comparison_fails_missing_edge() -> Result<(), Box<dyn std::error::Error>> {
    let model = graph_model();
    let mut o = observed_from(&model);
    o.edges = json!([]);
    let verdict = compare(&model, &o);
    assert!(verdict.is_err(), "missing edge must fail, got {verdict:?}");
    Ok(())
}

#[test]
fn graph_comparison_fails_missing_tag_membership() -> Result<(), Box<dyn std::error::Error>> {
    let model = graph_model();
    let mut o = observed_from(&model);
    *o.nodes
        .get_mut(0)
        .and_then(|row| row.get_mut(9))
        .ok_or("node row cell missing")? = json!("[]");
    let verdict = compare(&model, &o);
    assert!(
        verdict.is_err(),
        "missing tag membership must fail, got {verdict:?}"
    );
    Ok(())
}

#[test]
fn graph_comparison_fails_has_tag_edge_divergence_naming_resource_and_tag()
-> Result<(), Box<dyn std::error::Error>> {
    let model = graph_model_with_has_tag();
    let mut o = observed_from(&model);
    *o.tags
        .get_mut(0)
        .and_then(|row| row.get_mut(5))
        .ok_or("tag row cell missing")? = json!("staging");
    let message = match compare(&model, &o) {
        Err(error) => error.to_string(),
        other => return Err(format!("HAS_TAG divergence must fail, got {other:?}").into()),
    };
    assert!(
        message.contains("scan-a::r1"),
        "message must name node: {message}"
    );
    assert!(
        message.contains("tag"),
        "message must name HAS_TAG: {message}"
    );
    Ok(())
}

#[test]
fn graph_comparison_fails_missing_has_tag_edge() -> Result<(), Box<dyn std::error::Error>> {
    let model = graph_model_with_has_tag();
    let mut o = observed_from(&model);
    o.tags = json!([]);
    let message = match compare(&model, &o) {
        Err(error) => error.to_string(),
        other => return Err(format!("missing HAS_TAG must fail, got {other:?}").into()),
    };
    assert!(
        message.contains("scan-a::r1"),
        "message must name node: {message}"
    );
    Ok(())
}

#[test]
fn graph_comparison_is_inconclusive_for_other_scan_generation()
-> Result<(), Box<dyn std::error::Error>> {
    let model = graph_model();
    let mut o = observed_from(&model);
    o.marker = json!([["scan-b", "complete", 2, 1, 4, 1]]);
    let verdict = compare(&model, &o);
    assert!(
        matches!(
            verdict,
            Ok(verify_delivery::GraphComparison::Inconclusive(_))
        ),
        "generation mismatch must be inconclusive, got {verdict:?}"
    );
    Ok(())
}

#[test]
fn graph_comparison_is_inconclusive_while_sweep_is_writing()
-> Result<(), Box<dyn std::error::Error>> {
    let model = graph_model();
    let mut o = observed_from(&model);
    o.marker = json!([["scan-a", "writing", 2, 1, 4, 1]]);
    let verdict = compare(&model, &o);
    assert!(
        matches!(
            verdict,
            Ok(verify_delivery::GraphComparison::Inconclusive(_))
        ),
        "in-progress sweep must be inconclusive, got {verdict:?}"
    );
    Ok(())
}

#[test]
fn graph_comparison_is_inconclusive_without_marker() -> Result<(), Box<dyn std::error::Error>> {
    let model = graph_model();
    let mut o = observed_from(&model);
    o.marker = json!([]);
    let verdict = compare(&model, &o);
    assert!(
        matches!(
            verdict,
            Ok(verify_delivery::GraphComparison::Inconclusive(_))
        ),
        "absent marker must be inconclusive, got {verdict:?}"
    );
    Ok(())
}

#[tokio::test]
async fn test_build_expected_clean_verify_with_tags_and_properties()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = create_temp_dir("verify-clean-")?;
    let db_path = temp.path().join("test.sqlite");
    std::fs::write(&db_path, "")?;
    let url = format!("file:{}", db_path.display());
    let pool = sqlx::SqlitePool::connect(&url).await?;

    sqlx::query(
        "CREATE TABLE scans(scan_id TEXT, status TEXT, started_at INTEGER, finished_at INTEGER)",
    )
    .execute(&pool)
    .await?;
    sqlx::query("CREATE TABLE scan_resources(scan_id TEXT, id TEXT, resource_json TEXT)")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE TABLE scan_edges(scan_id TEXT, from_id TEXT, to_id TEXT, relation TEXT, source TEXT, edge_json TEXT)")
        .execute(&pool).await?;
    sqlx::query("CREATE TABLE scope_checkpoints(scan_id TEXT, scope_key TEXT, last_page INTEGER, complete INTEGER, failed INTEGER, account_id TEXT, region TEXT, collector TEXT, parent_id TEXT, source TEXT)")
        .execute(&pool).await?;

    sqlx::query("INSERT INTO scans VALUES (?, 'complete', 100, 200)")
        .bind(SCAN)
        .execute(&pool)
        .await?;

    let node_id = "arn:aws:ec2:us-east-1:111111111111:instance/i-1";
    let vpc_id = "arn:aws:ec2:us-east-1:111111111111:vpc/vpc-1";
    let node_json = r#"{"id":"arn:aws:ec2:us-east-1:111111111111:instance/i-1","account_id":"111111111111","region":"us-east-1","kind":"ec2","name":"web","properties":{"size":"small"},"tags":{"env":"prod"},"source":"direct","observed_at":100}"#;
    let vpc_json = r#"{"id":"arn:aws:ec2:us-east-1:111111111111:vpc/vpc-1","account_id":"111111111111","region":"us-east-1","kind":"vpc","name":null,"properties":{},"tags":{},"source":"direct","observed_at":100}"#;
    for (id, body) in [(node_id, node_json), (vpc_id, vpc_json)] {
        sqlx::query("INSERT INTO scan_resources(scan_id, id, resource_json) VALUES (?, ?, ?)")
            .bind(SCAN)
            .bind(id)
            .bind(body)
            .execute(&pool)
            .await?;
    }
    let edge_json = r#"{"from_id":"arn:aws:ec2:us-east-1:111111111111:instance/i-1","to_id":"arn:aws:ec2:us-east-1:111111111111:vpc/vpc-1","relation":"IN_VPC","source":"direct","observed_at":100,"properties":{}}"#;
    sqlx::query("INSERT INTO scan_edges(scan_id, from_id, to_id, relation, source, edge_json) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(SCAN).bind(node_id).bind(vpc_id).bind("IN_VPC").bind("direct").bind(edge_json)
        .execute(&pool).await?;
    sqlx::query("INSERT INTO scope_checkpoints(scan_id, scope_key, last_page, complete, failed, account_id, region, collector, parent_id, source) VALUES (?, 'sc1', 1, 1, 0, '111111111111', 'us-east-1', 'direct', null, 'direct')")
        .bind(SCAN).execute(&pool).await?;

    let model = verify_delivery::build_expected(&pool, SCAN).await?;

    let node_rows = json!([
        [
            node_id,
            SCAN,
            "observed",
            format!("{SCAN}::{node_id}"),
            "ec2",
            "111111111111",
            "us-east-1",
            "web",
            "{\"size\":\"small\"}",
            "{\"env\":\"prod\"}",
            "direct",
            100,
            12,
            1
        ],
        [
            vpc_id,
            SCAN,
            "observed",
            format!("{SCAN}::{vpc_id}"),
            "vpc",
            "111111111111",
            "us-east-1",
            "",
            "{}",
            "{}",
            "direct",
            100,
            12,
            1
        ]
    ]);
    let edge_rows = json!([[
        node_id,
        vpc_id,
        "RELATES_TO",
        "IN_VPC",
        "direct",
        SCAN,
        SCAN,
        100,
        "{}",
        4,
        format!("{SCAN}::{node_id}"),
        format!("{SCAN}::{vpc_id}")
    ]]);
    let tag_rows = json!([]);
    let marker = json!([[SCAN, "complete", 2, 1, 4, 1]]);
    let total = json!([[3]]);

    let verdict = verify_delivery::compare_graph_snapshot(
        &model, &node_rows, &edge_rows, &tag_rows, &marker, &total,
    )?;
    assert_eq!(
        verdict,
        verify_delivery::GraphComparison::Matched,
        "writer-shaped rows must verify clean"
    );

    let mut tampered = node_rows.clone();
    *tampered
        .get_mut(0)
        .and_then(|row| row.get_mut(8))
        .ok_or("node row cell missing")? = json!("{\"size\":\"large\"}");
    let message = match verify_delivery::compare_graph_snapshot(
        &model, &tampered, &edge_rows, &tag_rows, &marker, &total,
    ) {
        Err(error) => error.to_string(),
        other => return Err(format!("tampered property must fail, got {other:?}").into()),
    };
    assert!(
        message.contains(node_id),
        "message must name node: {message}"
    );

    pool.close().await;
    drop(temp);
    Ok(())
}
