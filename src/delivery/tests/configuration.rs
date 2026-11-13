use super::*;

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
