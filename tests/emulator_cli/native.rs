use super::*;

#[test]
#[ignore = "requires the explicitly prepared local AWS Emulator and FalkorDB fixtures"]
fn emulator_cli_native_snapshot_reports_and_graph_delivery() -> Result<()> {
    let snapshot = fixture()?;
    let expected = case(&snapshot, "native")?;
    let directory = tempfile::tempdir()?;
    let database = text_path(&directory.path().join("inventory.sqlite"))?;
    let output_directory = text_path(&directory.path().join("outputs"))?;
    let graph_name = format!("inventory_cli_{}", uuid::Uuid::new_v4().simple());
    let url = std::env::var("FALKORDB_TEST_URL")?;
    let port = url
        .strip_prefix("redis://127.0.0.1:")
        .context("FalkorDB fixture must use loopback")?
        .parse::<u16>()?;
    ensure!(port != 0, "FalkorDB fixture port is not bound");
    let mut graph = OwnedGraph {
        address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port),
        name: Some(graph_name.clone()),
    };
    let before = now()?;
    let collected = cli(
        &[
            "scan",
            "--db",
            &database,
            "--region",
            "us-east-1",
            "--service",
            "all",
            "--attempts",
            "2",
            "--max-pages",
            "100",
            "--page-size",
            "1",
            "--concurrency",
            "32",
            "--per-bucket-concurrency",
            "4",
            "--rps",
            "100",
            "--output-dir",
            &output_directory,
            "--falkor",
            "--falkor-url-env",
            "FALKORDB_TEST_URL",
            "--graph",
            &graph_name,
        ],
        0,
    )?;
    let after = now()?;
    let id = scan_id(&collected)?;
    assert_eq!(collected.get("status"), expected.get("status"));
    assert_eq!(collected.pointer("/api/in_flight"), Some(&json!(0)));
    assert_eq!(collected.pointer("/api/attempts"), Some(&json!(212)));
    assert_eq!(collected.pointer("/api/successes"), Some(&json!(212)));
    assert_eq!(collected.get("collection_status"), Some(&json!("complete")));
    assert_eq!(
        collected.pointer("/deliveries/0/delivery_status"),
        Some(&json!("complete"))
    );
    assert_eq!(
        collected.pointer("/deliveries/0/outputs/inventory/summary/resources"),
        Some(&json!(150))
    );
    assert_eq!(
        collected.pointer("/deliveries/0/outputs/inventory/summary/edges"),
        Some(&json!(340))
    );
    assert_eq!(
        collected.pointer("/deliveries/0/outputs/falkor/push/applied_batches"),
        Some(&json!(6))
    );
    observations(&database, id, expected, before, after)?;
    let report = cli(&["report", "--db", &database, "--scan-id", id], 0)?;
    assert_eq!(report.pointer("/summary/resources"), Some(&json!(150)));
    assert_eq!(report.pointer("/summary/edges"), Some(&json!(340)));
    let architecture = cli(
        &[
            "architecture",
            "--db",
            &database,
            "--scan-id",
            id,
            "--limit",
            "1000",
        ],
        0,
    )?;
    assert_eq!(architecture.get("status"), Some(&json!("complete")));
    let replay = cli(
        &[
            "deliver",
            "--db",
            &database,
            "--scan-id",
            id,
            "--output-dir",
            &output_directory,
            "--falkor",
            "--falkor-url-env",
            "FALKORDB_TEST_URL",
            "--graph",
            &graph_name,
        ],
        0,
    )?;
    assert_eq!(
        replay.pointer("/outputs/falkor/push/applied_batches"),
        Some(&json!(0))
    );
    assert_eq!(
        replay.pointer("/outputs/falkor/push/skipped_batches"),
        Some(&json!(6))
    );
    let output = text_path(&directory.path().join("graph.jsonl"))?;
    let exported = cli(
        &[
            "graph-export",
            "--db",
            &database,
            "--scan-id",
            id,
            "--graph",
            &graph_name,
            "--output",
            &output,
        ],
        0,
    )?;
    assert_eq!(exported.get("resources"), Some(&json!(150)));
    assert_eq!(exported.get("edges"), Some(&json!(340)));
    assert_eq!(exported.get("batches"), Some(&json!(6)));
    let checkpoint = text_path(&directory.path().join("ack.sqlite"))?;
    let push = [
        "graph-push",
        "--input",
        &output,
        "--checkpoint",
        &checkpoint,
        "--url-env",
        "FALKORDB_TEST_URL",
    ];
    assert_eq!(cli(&push, 0)?.get("applied_batches"), Some(&json!(6)));
    assert_eq!(cli(&push, 0)?.get("applied_batches"), Some(&json!(0)));
    graph.remove()?;
    Ok(())
}
