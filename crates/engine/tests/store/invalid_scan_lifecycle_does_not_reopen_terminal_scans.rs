//! Behavior: invalid_scan_lifecycle_does_not_reopen_terminal_scans.
use super::*;

#[tokio::test]
pub(super) async fn invalid_scan_lifecycle_does_not_reopen_terminal_scans() -> TestResult {
    let store = Store::open(":memory:").await?;
    assert!(matches!(
        store.summary("unknown").await,
        Err(StoreError::MissingScan(_))
    ));
    assert!(store.begin_scan("one", -1).await.is_err());
    store.begin_scan("one", 10).await?;
    store.begin_scan("one", 10).await?;
    assert!(store.begin_scan("one", 11).await.is_err());
    assert!(
        store
            .finish_scan("one", ScanStatus::Running, 20)
            .await
            .is_err()
    );
    assert!(
        store
            .finish_scan("one", ScanStatus::Cancelled, 9)
            .await
            .is_err()
    );
    store.finish_scan("one", ScanStatus::Cancelled, 20).await?;
    assert_eq!(store.scan_status("one").await?, ScanStatus::Cancelled);
    assert!(store.begin_scan("one", 10).await.is_err());
    assert!(
        store
            .apply_page("one", page("ec2", Source::Direct, Vec::new()))
            .await
            .is_err()
    );
    Ok(())
}
