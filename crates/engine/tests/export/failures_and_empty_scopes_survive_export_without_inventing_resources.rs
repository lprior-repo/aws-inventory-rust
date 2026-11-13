//! Behavior: failures_and_empty_scopes_survive_export_without_inventing_resources.
use super::*;

#[tokio::test]
pub(super) async fn failures_and_empty_scopes_survive_export_without_inventing_resources()
-> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan(SCAN, 1).await?;
    store
        .apply_page(
            SCAN,
            CollectionPage {
                scope: scope(Source::Direct),
                page_number: 1,
                next_token: None,
                resources: Vec::new(),
                edges: Vec::new(),
            },
        )
        .await?;
    let failure = CollectionFailure {
        scope: scope(Source::AwsConfig),
        category: "access_denied".into(),
        message: "Configured recorder unavailable\nwith quoted \"detail\"".into(),
    };
    store.record_failure(SCAN, &failure).await?;
    store.finish_scan(SCAN, ScanStatus::Complete, 2).await?;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("failed.json");
    let options = ExportOptions {
        format: ExportFormat::Json,
        allow_partial: true,
        ..ExportOptions::default()
    };
    let result = export_snapshot(&store, SCAN, &path, options, &CancellationToken::new()).await?;
    assert_eq!(result.status, ScanStatus::Partial);
    assert_eq!(
        result.counts,
        ExportCounts {
            resources: 0,
            edges: 0,
            scopes: 2,
            failures: 1,
            records: 3
        }
    );
    let document = parse(&path, ExportFormat::Json).await?;
    assert_eq!(
        document.records.last(),
        Some(&ExportRecord::Failure(failure))
    );
    assert_eq!(document.header.scan.summary.completed_scopes, 1);
    assert_eq!(document.header.scan.summary.failed_scopes, 1);
    store.close().await;
    Ok(())
}
