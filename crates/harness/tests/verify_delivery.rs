//! Delivery verification tests.

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use inventory_harness::verify_delivery;
use serde_json::json;
use tempfile::TempDir;

const SCAN: &str = "fixture-scan";

fn create_temp_dir(prefix: &str) -> TempDir {
    tempfile::Builder::new()
        .prefix(prefix)
        .tempdir()
        .expect("tempdir")
}

fn create_receipt_file(output: &std::path::Path, scan_id: &str, status: &str) -> serde_json::Value {
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
    std::fs::create_dir_all(&dir).expect("create dir");
    let path = dir.join("delivery.json");
    let content = serde_json::to_string_pretty(&receipt).expect("serialize");
    let mut file = File::create(path).expect("create file");
    file.write_all(content.as_bytes()).expect("write");

    // Write falkor plan
    let plan_path = output.join("falkor-plan.json");
    let plan_content =
        serde_json::to_string(&receipt["outputs"]["falkor"]["export"]).expect("serialize");
    let mut file = File::create(plan_path).expect("create file");
    file.write_all(plan_content.as_bytes()).expect("write");

    receipt
}

fn create_inventory_file(output: &std::path::Path) {
    let path = output.join("inventory.jsonl");
    let content = serde_json::to_string(&json!({
        "record": "header",
        "data": {
            "artifact": "aws-inventory.snapshot",
            "version": 1,
            "provenance": "all_observations",
            "scan": {
                "scan_id": SCAN,
                "status": "complete",
                "started_at": 100,
                "finished_at": 200
            }
        }
    }))
    .unwrap();
    let mut file = File::create(path).expect("create file");
    file.write_all(content.as_bytes()).expect("write");
}

#[test]
fn test_validate_receipt_ok() {
    let receipt = json!({
        "version": 1,
        "scan_id": SCAN,
        "status": "complete",
        "collection_status": "complete",
        "delivery_status": "complete"
    });
    let result = verify_delivery::validate_receipt(&receipt, SCAN, "complete");
    assert!(result.is_ok());
}

#[test]
fn test_validate_receipt_wrong_version() {
    let receipt = json!({
        "version": 2,
        "scan_id": SCAN,
        "status": "complete",
        "collection_status": "complete",
        "delivery_status": "complete"
    });
    let result = verify_delivery::validate_receipt(&receipt, SCAN, "complete");
    assert!(result.is_err());
}

#[test]
fn test_validate_receipt_wrong_scan_id() {
    let receipt = json!({
        "version": 1,
        "scan_id": "other",
        "status": "complete",
        "collection_status": "complete",
        "delivery_status": "complete"
    });
    let result = verify_delivery::validate_receipt(&receipt, SCAN, "complete");
    assert!(result.is_err());
}

#[test]
fn test_validate_receipt_wrong_status() {
    let receipt = json!({
        "version": 1,
        "scan_id": SCAN,
        "status": "partial",
        "collection_status": "complete",
        "delivery_status": "complete"
    });
    let result = verify_delivery::validate_receipt(&receipt, SCAN, "complete");
    assert!(result.is_err());
}

#[test]
fn test_validate_receipt_wrong_collection_status() {
    let receipt = json!({
        "version": 1,
        "scan_id": SCAN,
        "status": "complete",
        "collection_status": "partial",
        "delivery_status": "complete"
    });
    let result = verify_delivery::validate_receipt(&receipt, SCAN, "complete");
    assert!(result.is_err());
}

#[test]
fn test_validate_receipt_wrong_delivery_status() {
    let receipt = json!({
        "version": 1,
        "scan_id": SCAN,
        "status": "complete",
        "collection_status": "complete",
        "delivery_status": "failed"
    });
    let result = verify_delivery::validate_receipt(&receipt, SCAN, "complete");
    assert!(result.is_err());
}

#[test]
fn test_verify_receipt_file() {
    let temp = create_temp_dir("verify-receipt-file-");
    let output = temp.path().to_path_buf();
    let receipt = create_receipt_file(&output, SCAN, "complete");

    let result = verify_delivery::verify_receipt_file(&receipt, &output);
    match result {
        Ok(value) => {
            assert!(value["bytes"].is_i64());
            assert!(value["sha256"].is_string());
        }
        Err(e) => {
            eprintln!("receipt directory: {}", receipt["directory"]);
            eprintln!("output: {}", output.display());
            eprintln!("looking for: {}", output.join("delivery.json").display());
            eprintln!("file exists: {}", output.join("delivery.json").exists());
            panic!("verify_receipt_file failed: {}", e);
        }
    }

    drop(temp);
}

#[test]
fn test_verify_receipt_file_outside_directory() {
    let temp = create_temp_dir("verify-receipt-outside-");
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
}

#[test]
fn test_verify_receipt_file_missing_file() {
    let temp = create_temp_dir("verify-receipt-missing-");
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
}

#[test]
fn test_verify_push_first_complete() {
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

    let result = verify_delivery::verify_push(&receipt, &model, "test-graph", false);
    match result {
        Ok(_) => {}
        Err(e) => panic!("verify_push failed: {}", e),
    }
}

#[test]
fn test_verify_push_graph_mismatch() {
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
}

#[test]
fn test_verify_push_scan_id_mismatch() {
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
}

#[test]
fn test_verify_push_resource_count_mismatch() {
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
}

#[test]
fn test_verify_push_edge_count_mismatch() {
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
}

#[test]
fn test_verify_push_first_partial_with_replayed_batches() {
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
    let result = verify_delivery::verify_push(&receipt, &model, "test-graph", true);
    match result {
        Ok(_) => {}
        Err(e) => panic!("resume verify_push failed: {}", e),
    }
}

#[test]
fn test_verify_push_first_partial_without_replayed_batches() {
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
}

#[test]
fn test_verify_push_resume_complete() {
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

    let result = verify_delivery::verify_push(&receipt, &model, "test-graph", true);
    match result {
        Ok(_) => {}
        Err(e) => panic!("resume verify_push failed: {}", e),
    }
}

#[test]
fn test_verify_push_resume_incomplete() {
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
}

#[test]
fn test_sha256_hex() {
    let result = verify_delivery::sha256_hex(b"hello");
    assert_eq!(
        result,
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
}

#[test]
fn test_canonical_json() {
    let value = json!({"b": 2, "a": 1});
    let result = verify_delivery::canonical(&value);
    assert_eq!(result, "{\"a\":1,\"b\":2}");
}

#[test]
fn test_strict_json_ok() {
    let result = verify_delivery::strict_json("{\"a\":1}");
    assert!(result.is_ok());
    assert_eq!(result.unwrap()["a"], 1);
}

#[test]
fn test_strict_json_invalid() {
    let result = verify_delivery::strict_json("not json");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_build_expected() {
    let temp = create_temp_dir("verify-build-expected-");
    let db_path = temp.path().join("test.sqlite");
    // Create empty database file
    std::fs::write(&db_path, "").expect("create database file");
    let url = format!("file:{}", db_path.display());

    let pool = sqlx::SqlitePool::connect(&url).await.expect("connect pool");

    // Create schema
    sqlx::query(
        "CREATE TABLE scans(scan_id TEXT, status TEXT, started_at INTEGER, finished_at INTEGER)",
    )
    .execute(&pool)
    .await
    .expect("create scans");

    sqlx::query("CREATE TABLE scan_resources(scan_id TEXT, id TEXT, resource_json TEXT)")
        .execute(&pool)
        .await
        .expect("create scan_resources");

    sqlx::query(
        "CREATE TABLE scan_edges(scan_id TEXT, from_id TEXT, to_id TEXT, relation TEXT, source TEXT, edge_json TEXT)",
    )
    .execute(&pool)
    .await
    .expect("create scan_edges");

    sqlx::query(
        "CREATE TABLE scope_checkpoints(scan_id TEXT, scope_key TEXT, last_page INTEGER, complete INTEGER, failed INTEGER, account_id TEXT, region TEXT, collector TEXT, parent_id TEXT, source TEXT)",
    )
    .execute(&pool)
    .await
    .expect("create scope_checkpoints");

    // Seed data
    sqlx::query("INSERT INTO scans VALUES (?, 'complete', 100, 200)")
        .bind(SCAN)
        .execute(&pool)
        .await
        .expect("insert scan");

    sqlx::query("INSERT INTO scan_resources(scan_id, id, resource_json) VALUES (?, ?, ?)")
        .bind(SCAN)
        .bind("arn:aws:ec2:us-east-1:111111111111:instance/i-1")
        .bind("{\"evidence\":\"direct\"}")
        .execute(&pool)
        .await
        .expect("insert resource");

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
    .await
    .expect("insert edge");

    sqlx::query(
        "INSERT INTO scope_checkpoints(scan_id, scope_key, last_page, complete, failed, account_id, region, collector, parent_id, source) VALUES (?, ?, 1, 1, 0, '111111111111', 'us-east-1', 'direct', null, 'direct')",
    )
    .bind(SCAN)
    .bind("sc1")
    .execute(&pool)
    .await
    .expect("insert checkpoint");

    // Build expected model
    let model = verify_delivery::build_expected(&pool, SCAN)
        .await
        .expect("build expected");

    // Verify counts
    assert_eq!(model["counts"]["resources"], 1);
    assert_eq!(model["counts"]["edges"], 1);
    assert_eq!(model["counts"]["completed"], 1);
    assert_eq!(model["counts"]["failed"], 0);
    assert_eq!(model["scan"]["scan_id"], SCAN);
    assert_eq!(model["scan"]["status"], "complete");

    // Verify node rows contain the resource
    let node_rows = model["node_rows"].as_array().expect("node rows");
    assert_eq!(node_rows.len(), 2);

    // Verify edge rows
    let edge_rows = model["edge_rows"].as_array().expect("edge rows");
    assert_eq!(edge_rows.len(), 1);

    pool.close().await;
    drop(temp);
}

#[tokio::test]
async fn test_build_expected_partial_scan() {
    let temp = create_temp_dir("verify-build-partial-");
    let db_path = temp.path().join("test.sqlite");
    // Create empty database file
    std::fs::write(&db_path, "").expect("create database file");
    let url = format!("file:{}", db_path.display());

    let pool = sqlx::SqlitePool::connect(&url).await.expect("connect pool");

    // Create schema
    sqlx::query(
        "CREATE TABLE scans(scan_id TEXT, status TEXT, started_at INTEGER, finished_at INTEGER)",
    )
    .execute(&pool)
    .await
    .expect("create scans");

    sqlx::query("CREATE TABLE scan_resources(scan_id TEXT, id TEXT, resource_json TEXT)")
        .execute(&pool)
        .await
        .expect("create scan_resources");

    sqlx::query(
        "CREATE TABLE scan_edges(scan_id TEXT, from_id TEXT, to_id TEXT, relation TEXT, source TEXT, edge_json TEXT)",
    )
    .execute(&pool)
    .await
    .expect("create scan_edges");

    sqlx::query(
        "CREATE TABLE scope_checkpoints(scan_id TEXT, scope_key TEXT, last_page INTEGER, complete INTEGER, failed INTEGER, account_id TEXT, region TEXT, collector TEXT, parent_id TEXT, source TEXT)",
    )
    .execute(&pool)
    .await
    .expect("create scope_checkpoints");

    // Seed data with failed scope
    sqlx::query("INSERT INTO scans VALUES (?, 'partial', 100, 150)")
        .bind(SCAN)
        .execute(&pool)
        .await
        .expect("insert scan");

    sqlx::query(
        "INSERT INTO scope_checkpoints(scan_id, scope_key, last_page, complete, failed, account_id, region, collector, parent_id, source) VALUES (?, ?, 0, 0, 1, '111111111111', 'us-east-1', 'direct', null, 'direct')",
    )
    .bind(SCAN)
    .bind("sc1")
    .execute(&pool)
    .await
    .expect("insert failed checkpoint");

    let model = verify_delivery::build_expected(&pool, SCAN)
        .await
        .expect("build expected");

    assert_eq!(model["scan"]["status"], "partial");
    assert_eq!(model["counts"]["completed"], 0);
    assert_eq!(model["counts"]["failed"], 1);
    assert_eq!(model["observations"]["failure"], 1);

    pool.close().await;
    drop(temp);
}

#[tokio::test]
async fn test_build_expected_invalid_scan_status() {
    let temp = create_temp_dir("verify-build-invalid-");
    let db_path = temp.path().join("test.sqlite");
    // Create empty database file
    std::fs::write(&db_path, "").expect("create database file");
    let url = format!("file:{}", db_path.display());

    let pool = sqlx::SqlitePool::connect(&url).await.expect("connect pool");

    sqlx::query(
        "CREATE TABLE scans(scan_id TEXT, status TEXT, started_at INTEGER, finished_at INTEGER)",
    )
    .execute(&pool)
    .await
    .expect("create scans");

    sqlx::query("CREATE TABLE scan_resources(scan_id TEXT, id TEXT, resource_json TEXT)")
        .execute(&pool)
        .await
        .expect("create scan_resources");

    sqlx::query(
        "CREATE TABLE scan_edges(scan_id TEXT, from_id TEXT, to_id TEXT, relation TEXT, source TEXT, edge_json TEXT)",
    )
    .execute(&pool)
    .await
    .expect("create scan_edges");

    sqlx::query(
        "CREATE TABLE scope_checkpoints(scan_id TEXT, scope_key TEXT, last_page INTEGER, complete INTEGER, failed INTEGER, account_id TEXT, region TEXT, collector TEXT, parent_id TEXT, source TEXT)",
    )
    .execute(&pool)
    .await
    .expect("create scope_checkpoints");

    sqlx::query("INSERT INTO scans VALUES (?, 'running', 100, null)")
        .bind(SCAN)
        .execute(&pool)
        .await
        .expect("insert scan");

    let result = verify_delivery::build_expected(&pool, SCAN).await;
    assert!(result.is_err());

    pool.close().await;
    drop(temp);
}
