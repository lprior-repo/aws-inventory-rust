use super::*;

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
