use super::*;

#[tokio::test]
pub(super) async fn exact_record_and_byte_budgets_succeed_and_overflow_cleans_owned_partial()
-> TestResult {
    let store = fixture(ScanStatus::Complete).await?;
    let directory = tempfile::tempdir()?;
    let baseline = directory.path().join("baseline");
    let receipt = export_snapshot(
        &store,
        SCAN,
        &baseline,
        ExportOptions::default(),
        &CancellationToken::new(),
    )
    .await?;
    let exact = ExportOptions {
        max_records: usize::try_from(receipt.counts.records)?,
        max_bytes: receipt.bytes,
        ..ExportOptions::default()
    };
    let path = directory.path().join("exact");
    assert_eq!(
        export_snapshot(&store, SCAN, &path, exact, &CancellationToken::new()).await?,
        receipt
    );
    for (name, options) in [
        (
            "short-byte",
            ExportOptions {
                max_bytes: receipt.bytes.saturating_sub(1),
                ..exact
            },
        ),
        (
            "short-record",
            ExportOptions {
                max_records: exact.max_records.saturating_sub(1),
                ..exact
            },
        ),
        (
            "one-byte",
            ExportOptions {
                max_bytes: 1,
                ..exact
            },
        ),
    ] {
        let path = directory.path().join(name);
        assert!(matches!(
            export_snapshot(&store, SCAN, &path, options, &CancellationToken::new()).await,
            Err(ExportError::Limit(_))
        ));
        assert!(!path.exists());
        assert!(!directory.path().join(format!("{name}.partial")).exists());
    }
    store.close().await;
    Ok(())
}
