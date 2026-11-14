use super::*;

#[test]
fn partial_and_cancelled_snapshots_preserve_exit_status_and_evidence() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let database_path = directory.path().join("inventory.sqlite");
    let database = path(&database_path)?;
    let input = fixture("architecture-events.jsonl")?;
    let imported = json_command(
        &arguments(&[
            "ingest",
            "--db",
            &database,
            "--scan-id",
            "partial-fixture",
            "--input",
            &input,
        ]),
        2,
    )?;
    assert_eq!(imported.get("status"), Some(&json!("partial")));
    assert_eq!(imported.pointer("/summary/resources"), Some(&json!(19)));
    assert_eq!(imported.pointer("/summary/edges"), Some(&json!(40)));
    assert_eq!(imported.pointer("/summary/failed_scopes"), Some(&json!(1)));

    let report = json_command(
        &arguments(&["report", "--db", &database, "--scan-id", "partial-fixture"]),
        2,
    )?;
    assert_eq!(report.get("status"), Some(&json!("partial")));
    assert_eq!(report.get("summary"), imported.get("summary"));
    let failures = report
        .get("failures")
        .and_then(Value::as_array)
        .context("partial report failures missing")?;
    assert_eq!(failures.len(), 1);
    assert_eq!(
        failures.first().and_then(|failure| failure.get("category")),
        Some(&json!("access_denied"))
    );

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        let store = inventory_engine::store::Store::open(&database_path).await?;
        let result: Result<()> = async {
            store.begin_scan("cancelled-fixture", 1).await?;
            store
                .finish_scan(
                    "cancelled-fixture",
                    inventory_core::ScanStatus::Cancelled,
                    2,
                )
                .await?;
            Ok(())
        }
        .await;
        store.close().await;
        result
    })?;
    drop(runtime);

    let cancelled = json_command(
        &arguments(&[
            "report",
            "--db",
            &database,
            "--scan-id",
            "cancelled-fixture",
        ]),
        2,
    )?;
    assert_eq!(cancelled.get("status"), Some(&json!("cancelled")));
    assert_eq!(cancelled.pointer("/summary/resources"), Some(&json!(0)));
    assert_eq!(cancelled.get("failures"), Some(&json!([])));
    let states = json_command(
        &arguments(&[
            "query",
            "--db",
            &database,
            "SELECT scan_id,status FROM scans ORDER BY scan_id",
        ]),
        0,
    )?;
    assert_eq!(
        states,
        json!([
            {"scan_id":"cancelled-fixture","status":"cancelled"},
            {"scan_id":"partial-fixture","status":"partial"}
        ])
    );
    Ok(())
}
