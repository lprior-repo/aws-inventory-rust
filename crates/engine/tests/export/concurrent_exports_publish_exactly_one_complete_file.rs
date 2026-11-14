use super::*;

#[tokio::test]
pub(super) async fn concurrent_exports_publish_exactly_one_complete_file() -> TestResult {
    let store = fixture(ScanStatus::Complete).await?;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("raced");
    let cancellation = CancellationToken::new();
    let (first, second) = tokio::join!(
        export_snapshot(&store, SCAN, &path, ExportOptions::default(), &cancellation),
        export_snapshot(&store, SCAN, &path, ExportOptions::default(), &cancellation),
    );
    assert_ne!(first.is_ok(), second.is_ok());
    assert_eq!(
        parse(&path, ExportFormat::Jsonl).await?.summary.resources,
        2
    );
    assert!(!directory.path().join("raced.partial").exists());
    store.close().await;
    Ok(())
}
