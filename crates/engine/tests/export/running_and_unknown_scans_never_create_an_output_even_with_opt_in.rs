use super::*;

#[tokio::test]
pub(super) async fn running_and_unknown_scans_never_create_an_output_even_with_opt_in() -> TestResult
{
    let store = Store::open(":memory:").await?;
    store.begin_scan(SCAN, 1).await?;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("must-not-exist");
    let options = ExportOptions {
        allow_partial: true,
        ..ExportOptions::default()
    };
    assert!(matches!(
        export_snapshot(&store, SCAN, &path, options, &CancellationToken::new()).await,
        Err(ExportError::Invalid(_))
    ));
    assert!(matches!(
        export_snapshot(&store, "unknown", &path, options, &CancellationToken::new()).await,
        Err(ExportError::Store(_))
    ));
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
