//! Behavior: ordering_and_content_hash_are_deterministic_for_both_modes_and_formats.
use super::*;

#[tokio::test]
pub(super) async fn ordering_and_content_hash_are_deterministic_for_both_modes_and_formats()
-> TestResult {
    let store = fixture(ScanStatus::Complete).await?;
    let directory = tempfile::tempdir()?;
    for provenance in [ProvenanceMode::Normalized, ProvenanceMode::AllObservations] {
        for format in [ExportFormat::Json, ExportFormat::Jsonl] {
            let options = ExportOptions {
                format,
                provenance,
                ..ExportOptions::default()
            };
            let a = directory
                .path()
                .join(format!("first-{provenance:?}-{format:?}"));
            let b = directory
                .path()
                .join(format!("second-{provenance:?}-{format:?}"));
            let first =
                export_snapshot(&store, SCAN, &a, options, &CancellationToken::new()).await?;
            let second =
                export_snapshot(&store, SCAN, &b, options, &CancellationToken::new()).await?;
            assert_eq!(first, second);
            assert_eq!(tokio::fs::read(a).await?, tokio::fs::read(b).await?);
        }
    }
    store.close().await;
    Ok(())
}
