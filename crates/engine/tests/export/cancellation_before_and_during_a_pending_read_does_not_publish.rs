//! Behavior: cancellation_before_and_during_a_pending_read_does_not_publish.
use super::*;

#[tokio::test]
pub(super) async fn cancellation_before_and_during_a_pending_read_does_not_publish() -> TestResult {
    let store = fixture(ScanStatus::Complete).await?;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("cancelled");
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    assert!(matches!(
        export_snapshot(&store, SCAN, &path, ExportOptions::default(), &cancellation).await,
        Err(ExportError::Cancelled)
    ));
    let cancellation = CancellationToken::new();
    let mut held_reader = store.resource_stream(SCAN);
    assert!(held_reader.next().await.transpose()?.is_some());
    let mut export = Box::pin(export_snapshot(
        &store,
        SCAN,
        &path,
        ExportOptions::default(),
        &cancellation,
    ));
    assert!(futures_util::poll!(&mut export).is_pending());
    cancellation.cancel();
    assert!(matches!(export.await, Err(ExportError::Cancelled)));
    drop(held_reader);
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
