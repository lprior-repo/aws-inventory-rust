use super::*;

#[tokio::test]
pub(super) async fn invalid_budget_bounds_are_rejected_before_any_file_is_created() -> TestResult {
    let store = fixture(ScanStatus::Complete).await?;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("invalid");
    for options in [
        ExportOptions {
            max_records: 0,
            ..ExportOptions::default()
        },
        ExportOptions {
            max_records: MAX_EXPORT_RECORDS.saturating_add(1),
            ..ExportOptions::default()
        },
        ExportOptions {
            max_bytes: 0,
            ..ExportOptions::default()
        },
        ExportOptions {
            max_bytes: MAX_EXPORT_BYTES.saturating_add(1),
            ..ExportOptions::default()
        },
    ] {
        assert!(matches!(
            export_snapshot(&store, SCAN, &path, options, &CancellationToken::new()).await,
            Err(ExportError::Invalid(_))
        ));
    }
    assert!(
        tokio::fs::read_dir(directory.path())
            .await?
            .next_entry()
            .await?
            .is_none()
    );
    store.close().await;
    Ok(())
}
