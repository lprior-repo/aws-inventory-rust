use super::*;

#[tokio::test]
pub(super) async fn existing_outputs_and_partials_are_never_clobbered_or_removed() -> TestResult {
    let store = fixture(ScanStatus::Complete).await?;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("owned-by-someone-else");
    tokio::fs::write(&path, b"preserve final").await?;
    assert!(
        export_snapshot(
            &store,
            SCAN,
            &path,
            ExportOptions::default(),
            &CancellationToken::new()
        )
        .await
        .is_err()
    );
    assert_eq!(tokio::fs::read(&path).await?, b"preserve final");
    let path = directory.path().join("unfinished");
    let partial = directory.path().join("unfinished.partial");
    tokio::fs::write(&partial, b"preserve foreign partial").await?;
    assert!(
        export_snapshot(
            &store,
            SCAN,
            &path,
            ExportOptions::default(),
            &CancellationToken::new()
        )
        .await
        .is_err()
    );
    assert_eq!(tokio::fs::read(partial).await?, b"preserve foreign partial");
    assert!(!path.exists());
    store.close().await;
    Ok(())
}
