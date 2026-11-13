use super::*;

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
