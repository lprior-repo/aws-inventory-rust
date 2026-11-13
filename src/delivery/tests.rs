use super::*;
use clap::Parser;
use proptest::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};

fn options(directory: &Path) -> DeliveryArgs {
    DeliveryArgs {
        output_dir: Some(directory.to_owned()),
        ..DeliveryArgs::default()
    }
}

#[test]
fn persisted_v04_scan_jobs_deserialize_without_delivery_side_effects() -> Result<()> {
    let parsed = crate::cli::Cli::try_parse_from(["aws-inventory", "scan", "--service", "ec2"])?;
    let crate::cli::Command::Scan(scan) = parsed.command else {
        anyhow::bail!("not a scan");
    };
    let mut legacy = serde_json::to_value(scan)?;
    legacy
        .as_object_mut()
        .context("scan payload")?
        .remove("delivery");
    let restored: crate::cli::ScanArgs = serde_json::from_value(legacy)?;
    assert!(restored.delivery.output_dir.is_none());
    assert!(!restored.delivery.falkor);
    crate::jobs::validate_scan(&restored)?;
    Ok(())
}

#[test]
fn cli_requires_output_directory_before_admitting_falkor_delivery() {
    assert!(crate::cli::Cli::try_parse_from(["aws-inventory", "scan", "--falkor"]).is_err());
    assert!(
        crate::cli::Cli::try_parse_from(["aws-inventory", "deliver", "--scan-id", "x", "--falkor"])
            .is_err()
    );
    assert!(
        crate::cli::Cli::try_parse_from([
            "aws-inventory",
            "scan",
            "--output-dir",
            "outputs",
            "--falkor"
        ])
        .is_ok()
    );
}

#[tokio::test]
async fn repeated_offline_delivery_publishes_separate_complete_artifacts() -> Result<()> {
    let temporary = tempfile::tempdir()?;
    let database = temporary.path().join("inventory.sqlite");
    let demo = crate::demo::create(database.clone()).await?;
    let scan = demo
        .get("scan_id")
        .and_then(Value::as_str)
        .context("demo ID")?;
    let options = options(&temporary.path().join("outputs"));
    let token = CancellationToken::new();
    let first = deliver_one(&database, scan, &options, &token).await?;
    let second = deliver_one(&database, scan, &options, &token).await?;
    assert_eq!(first.get("delivery_status"), Some(&json!("complete")));
    assert_eq!(second.get("delivery_status"), Some(&json!("complete")));
    assert_ne!(first.get("directory"), second.get("directory"));
    let path = first
        .pointer("/outputs/inventory/path")
        .and_then(Value::as_str)
        .context("export path")?;
    let content = tokio::fs::read_to_string(path).await?;
    assert!(content.contains("arn:aws:ec2:us-east-1:111122223333:instance/i-demo"));
    assert!(content.contains("IN_VPC"));
    let directory = first
        .get("directory")
        .and_then(Value::as_str)
        .context("receipt directory")?;
    let receipt: Value = serde_json::from_slice(
        &tokio::fs::read(Path::new(directory).join("delivery.json")).await?,
    )?;
    assert_eq!(receipt, first);
    assert!(
        !Path::new(directory)
            .join("inventory.jsonl.partial")
            .exists()
    );
    Ok(())
}

#[tokio::test]
async fn partial_delivery_requires_explicit_opt_in_and_retains_scan_status() -> Result<()> {
    let temporary = tempfile::tempdir()?;
    let database = temporary.path().join("inventory.sqlite");
    let store = Store::open(&database).await?;
    store.begin_scan("partial-fixture", 1).await?;
    store
        .finish_scan("partial-fixture", ScanStatus::Partial, 2)
        .await?;
    store.close().await;
    let mut options = options(&temporary.path().join("outputs"));
    let token = CancellationToken::new();
    let skipped = deliver_one(&database, "partial-fixture", &options, &token).await?;
    assert_eq!(skipped.get("delivery_status"), Some(&json!("skipped")));
    assert!(!temporary.path().join("outputs").exists());
    options.allow_partial = true;
    options.export_format = ExportFormat::Json;
    let exported = deliver_one(&database, "partial-fixture", &options, &token).await?;
    assert_eq!(exported.get("delivery_status"), Some(&json!("complete")));
    assert_eq!(exported.get("status"), Some(&json!("partial")));
    let file = exported
        .pointer("/outputs/inventory/path")
        .and_then(Value::as_str)
        .context("inventory path")?;
    let document: Value = serde_json::from_slice(&tokio::fs::read(file).await?)?;
    assert!(document.is_object());
    assert!(document.to_string().contains("partial"));
    Ok(())
}

#[tokio::test]
async fn cancellation_stops_delivery_admission_without_creating_database_or_outputs() -> Result<()>
{
    let temporary = tempfile::tempdir()?;
    let database = temporary.path().join("must-not-create.sqlite");
    let token = CancellationToken::new();
    token.cancel();
    let report = deliver_one(
        &database,
        "cancelled-fixture",
        &options(&temporary.path().join("outputs")),
        &token,
    )
    .await?;
    assert_eq!(report.get("status"), Some(&json!("cancelled")));
    assert!(!database.exists());
    assert!(!temporary.path().join("outputs").exists());
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn delivery_deadline_cancels_drains_and_reports_timeout_distinctly() -> Result<()> {
    let token = CancellationToken::new();
    let drained = AtomicBool::new(false);
    let work = async {
        token.cancelled().await;
        drained.store(true, Ordering::SeqCst);
        anyhow::bail!("cancelled and cleaned up")
    };
    let outcome = bounded_delivery(work, &token, 1).await;
    assert!(drained.load(Ordering::SeqCst));
    let report = delivery_receipt(
        "test",
        ScanStatus::Complete,
        Path::new("outputs/test"),
        outcome,
        json!({}),
        false,
    );
    assert_eq!(report.get("status"), Some(&json!("partial")));
    assert_eq!(report.get("delivery_status"), Some(&json!("timed_out")));
    assert_eq!(report.get("cleanup_complete"), Some(&json!(true)));
    Ok(())
}

#[tokio::test]
async fn delivery_failure_does_not_relabel_or_mutate_collected_snapshot() -> Result<()> {
    let temporary = tempfile::tempdir()?;
    let database = temporary.path().join("inventory.sqlite");
    let demo = crate::demo::create(database.clone()).await?;
    let scan = demo
        .get("scan_id")
        .and_then(Value::as_str)
        .context("demo scan")?;
    let obstruction = temporary.path().join("file-not-directory");
    tokio::fs::write(&obstruction, b"preserve me").await?;
    let input = json!({"status":"complete","scans":[demo.clone()]});
    let result = after_scan(
        &database,
        &options(&obstruction),
        input,
        &CancellationToken::new(),
    )
    .await?;
    assert_eq!(result.get("status"), Some(&json!("partial")));
    assert_eq!(result.get("collection_status"), Some(&json!("complete")));
    let store = Store::open(&database).await?;
    assert_eq!(store.scan_status(scan).await?, ScanStatus::Complete);
    assert_eq!(store.summary(scan).await?.resources, 7);
    store.close().await;
    assert_eq!(tokio::fs::read(&obstruction).await?, b"preserve me");
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 64, ..ProptestConfig::default() })]
    #[test]
    fn collection_and_delivery_outcomes_never_erase_partial_or_cancelled_evidence(
        collection in prop::sample::select(vec!["complete", "partial", "cancelled"]),
        deliveries in prop::collection::vec(any::<bool>(), 0..17),
        cancelled in any::<bool>(),
    ) {
        let mut result = json!({"status":collection,"preserved":"data"});
        let entries = deliveries.iter().map(|succeeded| json!({"delivery_status":if *succeeded {"complete"} else {"failed"}})).collect();
        let merged = merge_delivery_result(&mut result, entries, cancelled);
        prop_assert!(merged.is_ok());
        let expected = if cancelled { "cancelled" } else if collection == "complete" && deliveries.contains(&false) { "partial" } else { collection };
        prop_assert_eq!(result.get("status"), Some(&json!(expected)));
        prop_assert_eq!(result.get("collection_status"), Some(&json!(collection)));
        prop_assert_eq!(result.get("preserved"), Some(&json!("data")));
    }
}

#[tokio::test]
async fn missing_falkor_environment_retains_completed_inventory_and_graph_plan_receipts()
-> Result<()> {
    let temporary = tempfile::tempdir()?;
    let database = temporary.path().join("inventory.sqlite");
    let demo = crate::demo::create(database.clone()).await?;
    let scan = demo
        .get("scan_id")
        .and_then(Value::as_str)
        .context("demo scan ID")?;
    let mut options = options(&temporary.path().join("outputs"));
    options.falkor = true;
    options.falkor_url_env = format!("AWS_INVENTORY_MISSING_{}", uuid::Uuid::new_v4().simple());
    assert!(std::env::var_os(&options.falkor_url_env).is_none());
    // Bypass preflight deliberately: exercise a late configuration failure after
    // both local artifacts exist. No environment mutation or network is needed.
    let result = after_scan(
        &database,
        &options,
        json!({"status":"complete","scans":[demo.clone()]}),
        &CancellationToken::new(),
    )
    .await?;
    assert_eq!(result.get("status"), Some(&json!("partial")));
    assert_eq!(result.get("collection_status"), Some(&json!("complete")));
    let receipt = result
        .pointer("/deliveries/0")
        .context("delivery receipt")?;
    assert_eq!(receipt.get("delivery_status"), Some(&json!("failed")));
    assert!(
        receipt
            .get("error")
            .and_then(Value::as_str)
            .is_some_and(|error| error.contains(&options.falkor_url_env))
    );
    let inventory = receipt
        .pointer("/outputs/inventory")
        .context("retained inventory output")?;
    let inventory_path = inventory
        .get("path")
        .and_then(Value::as_str)
        .context("inventory path")?;
    let bytes = tokio::fs::read(inventory_path).await?;
    let records = std::str::from_utf8(&bytes)?
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<Vec<export::ExportRecord>, _>>()?;
    let resources = records
        .iter()
        .filter(|row| matches!(row, export::ExportRecord::Resource(_)))
        .count();
    let edges = records
        .iter()
        .filter(|row| matches!(row, export::ExportRecord::Edge(_)))
        .count();
    assert_eq!(
        inventory
            .pointer("/summary/resources")
            .and_then(Value::as_u64),
        Some(u64::try_from(resources)?)
    );
    assert_eq!(
        inventory.pointer("/summary/edges").and_then(Value::as_u64),
        Some(u64::try_from(edges)?)
    );
    assert_eq!(
        inventory.pointer("/summary/bytes").and_then(Value::as_u64),
        Some(u64::try_from(bytes.len())?)
    );
    let hash = inventory
        .pointer("/summary/sha256")
        .and_then(Value::as_str)
        .context("retained inventory content hash")?;
    assert_eq!(hash.len(), 64);
    assert!(hash.bytes().all(|byte| byte.is_ascii_hexdigit()));
    let graph = receipt
        .pointer("/outputs/falkor")
        .context("retained graph plan")?;
    let graph_path = graph
        .get("plan")
        .and_then(Value::as_str)
        .context("graph path")?;
    let plan = tokio::fs::read_to_string(graph_path)
        .await?
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<Vec<graph::GraphBatch>, _>>()?;
    assert_eq!(
        graph.pointer("/export/batches").and_then(Value::as_u64),
        Some(u64::try_from(plan.len())?)
    );
    assert_eq!(
        graph.pointer("/export/resources").and_then(Value::as_u64),
        Some(u64::try_from(resources)?)
    );
    assert_eq!(
        graph.pointer("/export/edges").and_then(Value::as_u64),
        Some(u64::try_from(edges)?)
    );
    assert!(graph.get("push").is_none());
    let checkpoint = graph
        .get("checkpoint")
        .and_then(Value::as_str)
        .context("checkpoint path")?;
    assert!(!Path::new(checkpoint).exists());
    let directory = receipt
        .get("directory")
        .and_then(Value::as_str)
        .context("attempt directory")?;
    let persisted: Value = serde_json::from_slice(
        &tokio::fs::read(Path::new(directory).join("delivery.json")).await?,
    )?;
    assert_eq!(&persisted, receipt);
    let store = Store::open(&database).await?;
    assert_eq!(store.scan_status(scan).await?, ScanStatus::Complete);
    assert_eq!(
        store.summary(scan).await?.resources,
        u64::try_from(resources)?
    );
    store.close().await;
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn timeout_distinguishes_drained_work_from_failed_cleanup_and_keeps_completed_outputs()
-> Result<()> {
    let token = CancellationToken::new();
    let work = async {
        token.cancelled().await;
        Err::<(), _>(anyhow::Error::from(export::ExportError::Cleanup {
            cause: Box::new(export::ExportError::Cancelled),
            drain: None,
            removal: Some(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "owned partial cleanup refused",
            )),
        }))
        .context("delivery stage cleanup")
    };
    let outcome = bounded_delivery(work, &token, 1).await;
    assert!(token.is_cancelled());
    let outputs =
        json!({"inventory":{"path":"completed-inventory.jsonl","summary":{"resources":7}}});
    let report = delivery_receipt(
        "test",
        ScanStatus::Complete,
        Path::new("outputs/test"),
        outcome,
        outputs.clone(),
        false,
    );
    assert_eq!(report.get("delivery_status"), Some(&json!("timed_out")));
    assert_eq!(report.get("status"), Some(&json!("partial")));
    assert_eq!(report.get("collection_status"), Some(&json!("complete")));
    assert_eq!(report.get("drain_complete"), Some(&json!(true)));
    assert_eq!(report.get("cleanup_complete"), Some(&json!(false)));
    assert_eq!(report.get("outputs"), Some(&outputs));
    let detail = report
        .get("cleanup_error")
        .and_then(Value::as_str)
        .context("retained cleanup failure")?;
    assert!(detail.contains("delivery stage cleanup"));
    assert!(detail.contains("owned partial cleanup refused"));
    Ok(())
}

#[tokio::test]
async fn receipt_publication_keeps_existing_receipts_and_foreign_partials() -> Result<()> {
    let temporary = tempfile::tempdir()?;
    let directory = temporary.path().join("owned-attempt");
    tokio::fs::create_dir(&directory).await?;
    let first = json!({"delivery_status":"complete","outputs":{"inventory":{"path":"original"}}});
    persist_receipt(&directory, &first).await?;
    let path = directory.join("delivery.json");
    let original = tokio::fs::read(&path).await?;
    assert!(
        persist_receipt(&directory, &json!({"replacement":true}))
            .await
            .is_err()
    );
    assert_eq!(tokio::fs::read(&path).await?, original);
    assert!(!directory.join("delivery.json.partial").exists());
    let blocked = temporary.path().join("blocked-attempt");
    tokio::fs::create_dir(&blocked).await?;
    tokio::fs::write(
        blocked.join("delivery.json.partial"),
        b"foreign pending receipt",
    )
    .await?;
    assert!(persist_receipt(&blocked, &first).await.is_err());
    assert_eq!(
        tokio::fs::read(blocked.join("delivery.json.partial")).await?,
        b"foreign pending receipt"
    );
    assert!(!blocked.join("delivery.json").exists());
    Ok(())
}
