use super::*;

#[tokio::test]
pub(super) async fn finalized_partial_and_cancelled_require_opt_in_and_keep_their_status()
-> TestResult {
    let directory = tempfile::tempdir()?;
    for status in [ScanStatus::Partial, ScanStatus::Cancelled] {
        let store = fixture(status).await?;
        let path = directory.path().join(status.as_str());
        assert!(matches!(
            export_snapshot(
                &store,
                SCAN,
                &path,
                ExportOptions::default(),
                &CancellationToken::new()
            )
            .await,
            Err(ExportError::Invalid(_))
        ));
        assert!(!path.exists());
        let options = ExportOptions {
            allow_partial: true,
            ..ExportOptions::default()
        };
        let result =
            export_snapshot(&store, SCAN, &path, options, &CancellationToken::new()).await?;
        assert_eq!(result.status, status);
        assert_eq!(
            parse(&path, ExportFormat::Jsonl).await?.header.scan.status,
            status
        );
        store.close().await;
    }
    Ok(())
}
