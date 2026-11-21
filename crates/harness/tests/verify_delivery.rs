//! Delivery verification tests.

use std::fs::File;
use std::io::Write;

use inventory_harness::verify_delivery;
use serde_json::json;
use tempfile::TempDir;

const SCAN: &str = "fixture-scan";

fn create_temp_dir(prefix: &str) -> std::io::Result<TempDir> {
    tempfile::Builder::new().prefix(prefix).tempdir()
}

fn create_receipt_file(
    output: &std::path::Path,
    scan_id: &str,
    status: &str,
) -> std::io::Result<serde_json::Value> {
    let receipt = json!({
        "version": 1,
        "scan_id": scan_id,
        "status": status,
        "collection_status": status,
        "delivery_status": "complete",
        "directory": format!("{}/delivery-{}", output.display(), scan_id),
        "timestamp": 1234567890,
        "argv": ["test"],
        "workspace": "/tmp",
        "output": output.to_string_lossy(),
        "elapsed_seconds": 1.0,
        "status_code": 0,
        "outputs": {
            "inventory": {
                "path": format!("{}/inventory.jsonl", output.display()),
                "format": "jsonl",
                "version": 1,
                "records": 10,
                "bytes": 1024
            },
            "falkor": {
                "export": {
                    "graph": "test-graph",
                    "scan_id": scan_id,
                    "resources": 1,
                    "edges": 1,
                    "batches": 1,
                    "nodes": 2,
                    "bytes": 256,
                    "elapsed_seconds": 0.1
                },
                "push": {
                    "status": "complete",
                    "batches": 1,
                    "applied_batches": 1,
                    "skipped_batches": 0,
                    "elapsed_seconds": 0.1
                },
                "plan": format!("{}/falkor-plan.json", output.display())
            }
        }
    });

    // Write delivery.json
    let dir = output.join(format!("delivery-{}", scan_id));
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("delivery.json");
    let content = serde_json::to_string_pretty(&receipt)?;
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;

    // Write falkor plan
    let plan_path = output.join("falkor-plan.json");
    let plan_content = serde_json::to_string(&json!({
        "graph": "test-graph",
        "scan_id": scan_id,
        "resources": 1,
        "edges": 1,
        "batches": 1,
        "nodes": 2,
        "bytes": 256,
        "elapsed_seconds": 0.1
    }))?;
    let mut file = File::create(plan_path)?;
    file.write_all(plan_content.as_bytes())?;

    Ok(receipt)
}

#[test]
fn test_validate_receipt_ok() -> Result<(), Box<dyn std::error::Error>> {
    let receipt = json!({
        "version": 1,
        "scan_id": SCAN,
        "status": "complete",
        "collection_status": "complete",
        "delivery_status": "complete"
    });
    let result = verify_delivery::validate_receipt(&receipt, SCAN, "complete");
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_validate_receipt_wrong_version() -> Result<(), Box<dyn std::error::Error>> {
    let receipt = json!({
        "version": 2,
        "scan_id": SCAN,
        "status": "complete",
        "collection_status": "complete",
        "delivery_status": "complete"
    });
    let result = verify_delivery::validate_receipt(&receipt, SCAN, "complete");
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_validate_receipt_wrong_scan_id() -> Result<(), Box<dyn std::error::Error>> {
    let receipt = json!({
        "version": 1,
        "scan_id": "other",
        "status": "complete",
        "collection_status": "complete",
        "delivery_status": "complete"
    });
    let result = verify_delivery::validate_receipt(&receipt, SCAN, "complete");
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_validate_receipt_wrong_status() -> Result<(), Box<dyn std::error::Error>> {
    let receipt = json!({
        "version": 1,
        "scan_id": SCAN,
        "status": "partial",
        "collection_status": "complete",
        "delivery_status": "complete"
    });
    let result = verify_delivery::validate_receipt(&receipt, SCAN, "complete");
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_validate_receipt_wrong_collection_status() -> Result<(), Box<dyn std::error::Error>> {
    let receipt = json!({
        "version": 1,
        "scan_id": SCAN,
        "status": "complete",
        "collection_status": "partial",
        "delivery_status": "complete"
    });
    let result = verify_delivery::validate_receipt(&receipt, SCAN, "complete");
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_validate_receipt_wrong_delivery_status() -> Result<(), Box<dyn std::error::Error>> {
    let receipt = json!({
        "version": 1,
        "scan_id": SCAN,
        "status": "complete",
        "collection_status": "complete",
        "delivery_status": "failed"
    });
    let result = verify_delivery::validate_receipt(&receipt, SCAN, "complete");
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_verify_receipt_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp = create_temp_dir("verify-receipt-file-")?;
    let output = temp.path().to_path_buf();
    let receipt = create_receipt_file(&output, SCAN, "complete")?;

    let result = verify_delivery::verify_receipt_file(&receipt, &output)?;
    assert!(result.get("bytes").is_some());
    assert!(result.get("sha256").is_some());

    drop(temp);
    Ok(())
}

#[test]
fn test_verify_receipt_file_outside_directory() -> Result<(), Box<dyn std::error::Error>> {
    let temp = create_temp_dir("verify-receipt-outside-")?;
    let output = temp.path().to_path_buf();
    let receipt = json!({
        "version": 1,
        "scan_id": SCAN,
        "status": "complete",
        "collection_status": "complete",
        "delivery_status": "complete",
        "directory": "/tmp/other",
        "outputs": {
            "inventory": { "path": "/tmp/other/inventory.jsonl" },
            "falkor": {
                "plan": "/tmp/other/plan.json",
                "export": { "graph": "g" },
                "push": { "applied_batches": 0 }
            }
        }
    });

    let result = verify_delivery::verify_receipt_file(&receipt, &output);
    assert!(result.is_err());

    drop(temp);
    Ok(())
}

#[test]
fn test_verify_receipt_file_missing_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp = create_temp_dir("verify-receipt-missing-")?;
    let output = temp.path().to_path_buf();
    let receipt = json!({
        "version": 1,
        "scan_id": SCAN,
        "status": "complete",
        "collection_status": "complete",
        "delivery_status": "complete",
        "directory": output.to_string_lossy(),
        "outputs": {
            "inventory": { "path": "/tmp/other/inventory.jsonl" },
            "falkor": {
                "plan": "/tmp/other/plan.json",
                "export": { "graph": "g" },
                "push": { "applied_batches": 0 }
            }
        }
    });

    let result = verify_delivery::verify_receipt_file(&receipt, &output);
    assert!(result.is_err());

    drop(temp);
    Ok(())
}

#[test]
fn test_verify_push_first_complete() -> Result<(), Box<dyn std::error::Error>> {
    let receipt = json!({
        "outputs": {
            "falkor": {
                "export": {
                    "graph": "test-graph",
                    "scan_id": SCAN,
                    "resources": 1,
                    "edges": 1,
                    "batches": 5
                },
                "push": {
                    "status": "complete",
                    "batches": 5,
                    "applied_batches": 5
                }
            }
        }
    });
    let model = json!({
        "scan": { "scan_id": SCAN },
        "counts": { "resources": 1, "edges": 1 }
    });

    verify_delivery::verify_push(&receipt, &model, "test-graph", false)?;
    Ok(())
}

#[test]
fn test_verify_push_graph_mismatch() -> Result<(), Box<dyn std::error::Error>> {
    let receipt = json!({
        "outputs": {
            "falkor": {
                "export": {
                    "graph": "wrong-graph",
                    "scan_id": SCAN,
                    "resources": 1,
                    "edges": 1,
                    "batches": 1
                },
                "push": {
                    "status": "complete",
                    "batches": 1,
                    "applied_batches": 1
                }
            }
        }
    });
    let model = json!({
        "scan": { "scan_id": SCAN },
        "counts": { "resources": 1, "edges": 1 }
    });

    let result = verify_delivery::verify_push(&receipt, &model, "test-graph", false);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_verify_push_scan_id_mismatch() -> Result<(), Box<dyn std::error::Error>> {
    let receipt = json!({
        "outputs": {
            "falkor": {
                "export": {
                    "graph": "test-graph",
                    "scan_id": "other",
                    "resources": 1,
                    "edges": 1,
                    "batches": 1
                },
                "push": {
                    "status": "complete",
                    "batches": 1,
                    "applied_batches": 1
                }
            }
        }
    });
    let model = json!({
        "scan": { "scan_id": SCAN },
        "counts": { "resources": 1, "edges": 1 }
    });

    let result = verify_delivery::verify_push(&receipt, &model, "test-graph", false);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_verify_push_resource_count_mismatch() -> Result<(), Box<dyn std::error::Error>> {
    let receipt = json!({
        "outputs": {
            "falkor": {
                "export": {
                    "graph": "test-graph",
                    "scan_id": SCAN,
                    "resources": 5,
                    "edges": 1,
                    "batches": 1
                },
                "push": {
                    "status": "complete",
                    "batches": 1,
                    "applied_batches": 1
                }
            }
        }
    });
    let model = json!({
        "scan": { "scan_id": SCAN },
        "counts": { "resources": 1, "edges": 1 }
    });

    let result = verify_delivery::verify_push(&receipt, &model, "test-graph", false);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_verify_push_edge_count_mismatch() -> Result<(), Box<dyn std::error::Error>> {
    let receipt = json!({
        "outputs": {
            "falkor": {
                "export": {
                    "graph": "test-graph",
                    "scan_id": SCAN,
                    "resources": 1,
                    "edges": 5,
                    "batches": 1
                },
                "push": {
                    "status": "complete",
                    "batches": 1,
                    "applied_batches": 1
                }
            }
        }
    });
    let model = json!({
        "scan": { "scan_id": SCAN },
        "counts": { "resources": 1, "edges": 1 }
    });

    let result = verify_delivery::verify_push(&receipt, &model, "test-graph", false);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_verify_push_first_partial_with_replayed_batches() -> Result<(), Box<dyn std::error::Error>>
{
    let receipt = json!({
        "outputs": {
            "falkor": {
                "export": {
                    "graph": "test-graph",
                    "scan_id": SCAN,
                    "resources": 1,
                    "edges": 1,
                    "batches": 10
                },
                "push": {
                    "status": "partial",
                    "batches": 10,
                    "applied_batches": 5
                }
            }
        }
    });
    let model = json!({
        "scan": { "scan_id": SCAN },
        "counts": { "resources": 1, "edges": 1 }
    });

    // First delivery must apply all batches
    let result = verify_delivery::verify_push(&receipt, &model, "test-graph", false);
    assert!(result.is_err());

    // Resume delivery with complete status and correct batch counts passes
    let receipt = json!({
        "outputs": {
            "falkor": {
                "export": {
                    "graph": "test-graph",
                    "scan_id": SCAN,
                    "resources": 1,
                    "edges": 1,
                    "batches": 10
                },
                "push": {
                    "status": "complete",
                    "batches": 10,
                    "applied_batches": 10,
                    "skipped_batches": 0
                }
            }
        }
    });
    verify_delivery::verify_push(&receipt, &model, "test-graph", true)?;
    Ok(())
}

#[test]
fn test_verify_push_first_partial_without_replayed_batches()
-> Result<(), Box<dyn std::error::Error>> {
    let receipt = json!({
        "outputs": {
            "falkor": {
                "export": {
                    "graph": "test-graph",
                    "scan_id": SCAN,
                    "resources": 1,
                    "edges": 1,
                    "batches": 10
                },
                "push": {
                    "status": "partial",
                    "batches": 10,
                    "applied_batches": 0
                }
            }
        }
    });
    let model = json!({
        "scan": { "scan_id": SCAN },
        "counts": { "resources": 1, "edges": 1 }
    });

    // First delivery: partial with no applied_batches is a failure
    let result = verify_delivery::verify_push(&receipt, &model, "test-graph", false);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_verify_push_resume_complete() -> Result<(), Box<dyn std::error::Error>> {
    let receipt = json!({
        "outputs": {
            "falkor": {
                "export": {
                    "graph": "test-graph",
                    "scan_id": SCAN,
                    "resources": 1,
                    "edges": 1,
                    "batches": 10
                },
                "push": {
                    "status": "complete",
                    "batches": 10,
                    "applied_batches": 10,
                    "skipped_batches": 0
                }
            }
        }
    });
    let model = json!({
        "scan": { "scan_id": SCAN },
        "counts": { "resources": 1, "edges": 1 }
    });

    verify_delivery::verify_push(&receipt, &model, "test-graph", true)?;
    Ok(())
}

#[test]
fn test_verify_push_resume_incomplete() -> Result<(), Box<dyn std::error::Error>> {
    let receipt = json!({
        "outputs": {
            "falkor": {
                "export": {
                    "graph": "test-graph",
                    "scan_id": SCAN,
                    "resources": 1,
                    "edges": 1,
                    "batches": 10
                },
                "push": {
                    "status": "partial",
                    "batches": 10,
                    "applied_batches": 5
                }
            }
        }
    });
    let model = json!({
        "scan": { "scan_id": SCAN },
        "counts": { "resources": 1, "edges": 1 }
    });

    // Resume delivery must be complete
    let result = verify_delivery::verify_push(&receipt, &model, "test-graph", true);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_sha256_hex() -> Result<(), Box<dyn std::error::Error>> {
    let result = verify_delivery::sha256_hex(b"hello");
    assert_eq!(
        result,
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
    Ok(())
}

#[test]
fn test_canonical_json() -> Result<(), Box<dyn std::error::Error>> {
    let value = json!({"b": 2, "a": 1});
    let result = verify_delivery::canonical(&value);
    assert_eq!(result, "{\"a\":1,\"b\":2}");
    Ok(())
}

#[test]
fn test_strict_json_ok() -> Result<(), Box<dyn std::error::Error>> {
    let result = verify_delivery::strict_json("{\"a\":1}")?;
    assert_eq!(result.get("a"), Some(&json!(1)));
    Ok(())
}

#[test]
fn test_strict_json_invalid() -> Result<(), Box<dyn std::error::Error>> {
    let result = verify_delivery::strict_json("not json");
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_build_expected() -> Result<(), Box<dyn std::error::Error>> {
    let temp = create_temp_dir("verify-build-expected-")?;
    let db_path = temp.path().join("test.sqlite");
    // Create empty database file
    std::fs::write(&db_path, "")?;
    let url = format!("file:{}", db_path.display());

    let pool = sqlx::SqlitePool::connect(&url).await?;

    // Create schema
    sqlx::query(
        "CREATE TABLE scans(scan_id TEXT, status TEXT, started_at INTEGER, finished_at INTEGER)",
    )
    .execute(&pool)
    .await?;

    sqlx::query("CREATE TABLE scan_resources(scan_id TEXT, id TEXT, resource_json TEXT)")
        .execute(&pool)
        .await?;

    sqlx::query(
        "CREATE TABLE scan_edges(scan_id TEXT, from_id TEXT, to_id TEXT, relation TEXT, source TEXT, edge_json TEXT)",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE scope_checkpoints(scan_id TEXT, scope_key TEXT, last_page INTEGER, complete INTEGER, failed INTEGER, account_id TEXT, region TEXT, collector TEXT, parent_id TEXT, source TEXT)",
    )
    .execute(&pool)
    .await?;

    // Seed data
    sqlx::query("INSERT INTO scans VALUES (?, 'complete', 100, 200)")
        .bind(SCAN)
        .execute(&pool)
        .await?;

    sqlx::query("INSERT INTO scan_resources(scan_id, id, resource_json) VALUES (?, ?, ?)")
        .bind(SCAN)
        .bind("arn:aws:ec2:us-east-1:111111111111:instance/i-1")
        .bind("{\"evidence\":\"direct\"}")
        .execute(&pool)
        .await?;

    sqlx::query(
        "INSERT INTO scan_edges(scan_id, from_id, to_id, relation, source, edge_json) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(SCAN)
    .bind("arn:aws:ec2:us-east-1:111111111111:instance/i-1")
    .bind("arn:aws:ec2:us-east-1:111111111111:vpc/vpc-ref")
    .bind("IN_VPC")
    .bind("direct")
    .bind("{}")
    .execute(&pool)
    .await?;

    sqlx::query(
        "INSERT INTO scope_checkpoints(scan_id, scope_key, last_page, complete, failed, account_id, region, collector, parent_id, source) VALUES (?, ?, 1, 1, 0, '111111111111', 'us-east-1', 'direct', null, 'direct')",
    )
    .bind(SCAN)
    .bind("sc1")
    .execute(&pool)
    .await?;

    // Build expected model
    let model = verify_delivery::build_expected(&pool, SCAN).await?;

    // Verify counts
    assert_eq!(
        model.get("counts").and_then(|c| c.get("resources")),
        Some(&json!(1))
    );
    assert_eq!(
        model.get("counts").and_then(|c| c.get("edges")),
        Some(&json!(1))
    );
    assert_eq!(
        model.get("counts").and_then(|c| c.get("completed")),
        Some(&json!(1))
    );
    assert_eq!(
        model.get("counts").and_then(|c| c.get("failed")),
        Some(&json!(0))
    );
    assert_eq!(
        model.get("scan").and_then(|s| s.get("scan_id")),
        Some(&json!(SCAN))
    );
    assert_eq!(
        model.get("scan").and_then(|s| s.get("status")),
        Some(&json!("complete"))
    );

    // Verify node rows contain the resource
    let node_rows = model
        .get("node_rows")
        .and_then(serde_json::Value::as_array)
        .ok_or("missing node_rows")?;
    assert_eq!(node_rows.len(), 2);

    // Verify edge rows
    let edge_rows = model
        .get("edge_rows")
        .and_then(serde_json::Value::as_array)
        .ok_or("missing edge_rows")?;
    assert_eq!(edge_rows.len(), 1);

    pool.close().await;
    drop(temp);
    Ok(())
}

#[tokio::test]
async fn test_build_expected_partial_scan() -> Result<(), Box<dyn std::error::Error>> {
    let temp = create_temp_dir("verify-build-partial-")?;
    let db_path = temp.path().join("test.sqlite");
    // Create empty database file
    std::fs::write(&db_path, "")?;
    let url = format!("file:{}", db_path.display());

    let pool = sqlx::SqlitePool::connect(&url).await?;

    // Create schema
    sqlx::query(
        "CREATE TABLE scans(scan_id TEXT, status TEXT, started_at INTEGER, finished_at INTEGER)",
    )
    .execute(&pool)
    .await?;

    sqlx::query("CREATE TABLE scan_resources(scan_id TEXT, id TEXT, resource_json TEXT)")
        .execute(&pool)
        .await?;

    sqlx::query(
        "CREATE TABLE scan_edges(scan_id TEXT, from_id TEXT, to_id TEXT, relation TEXT, source TEXT, edge_json TEXT)",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE scope_checkpoints(scan_id TEXT, scope_key TEXT, last_page INTEGER, complete INTEGER, failed INTEGER, account_id TEXT, region TEXT, collector TEXT, parent_id TEXT, source TEXT)",
    )
    .execute(&pool)
    .await?;

    // Seed data with failed scope
    sqlx::query("INSERT INTO scans VALUES (?, 'partial', 100, 150)")
        .bind(SCAN)
        .execute(&pool)
        .await?;

    sqlx::query(
        "INSERT INTO scope_checkpoints(scan_id, scope_key, last_page, complete, failed, account_id, region, collector, parent_id, source) VALUES (?, ?, 0, 0, 1, '111111111111', 'us-east-1', 'direct', null, 'direct')",
    )
    .bind(SCAN)
    .bind("sc1")
    .execute(&pool)
    .await?;

    let model = verify_delivery::build_expected(&pool, SCAN).await?;

    assert_eq!(
        model.get("scan").and_then(|s| s.get("status")),
        Some(&json!("partial"))
    );
    assert_eq!(
        model.get("counts").and_then(|c| c.get("completed")),
        Some(&json!(0))
    );
    assert_eq!(
        model.get("counts").and_then(|c| c.get("failed")),
        Some(&json!(1))
    );
    assert_eq!(
        model.get("observations").and_then(|o| o.get("failure")),
        Some(&json!(1))
    );

    pool.close().await;
    drop(temp);
    Ok(())
}

#[tokio::test]
async fn test_build_expected_invalid_scan_status() -> Result<(), Box<dyn std::error::Error>> {
    let temp = create_temp_dir("verify-build-invalid-")?;
    let db_path = temp.path().join("test.sqlite");
    // Create empty database file
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

    sqlx::query(
        "CREATE TABLE scan_edges(scan_id TEXT, from_id TEXT, to_id TEXT, relation TEXT, source TEXT, edge_json TEXT)",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE scope_checkpoints(scan_id TEXT, scope_key TEXT, last_page INTEGER, complete INTEGER, failed INTEGER, account_id TEXT, region TEXT, collector TEXT, parent_id TEXT, source TEXT)",
    )
    .execute(&pool)
    .await?;

    sqlx::query("INSERT INTO scans VALUES (?, 'running', 100, null)")
        .bind(SCAN)
        .execute(&pool)
        .await?;

    let result = verify_delivery::build_expected(&pool, SCAN).await;
    assert!(result.is_err());

    pool.close().await;
    drop(temp);
    Ok(())
}

fn graph_model() -> serde_json::Value {
    json!({
        "scan": {"scan_id": "scan-a", "status": "complete"},
        "counts": {"resources": 2, "edges": 1, "nodes": 3, "completed": 1, "failed": 0},
        "node_rows": [
            ["r1", "scan-a", "observed", "scan-a::r1", "ec2", "000000000000", "us-east-1", "web", "{\"size\":\"small\"}", "[{\"tag_key\":\"env\",\"tag_value\":\"prod\"}]", "collector", 1, 12, 1],
            ["r2", "scan-a", "observed", "scan-a::r2", "s3", "000000000000", "us-east-1", "bucket", "{}", "[]", "collector", 1, 12, 1]
        ],
        "edge_rows": [
            ["r1", "r2", "RELATES_TO", "collector", "{}", "scan-a", "scan-a", 1, "{}", 4, "scan-a::r1", "scan-a::r2"]
        ],
        "tag_rows": [
            ["r1", "scan-a::r1", "000000000000", "us-east-1", "env", "prod", "inventory.tags"]
        ]
    })
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
    let model = graph_model();
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
    let model = graph_model();
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
