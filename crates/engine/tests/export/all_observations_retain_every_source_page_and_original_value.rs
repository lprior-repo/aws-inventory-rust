use super::*;

#[tokio::test]
pub(super) async fn all_observations_retain_every_source_page_and_original_value() -> TestResult {
    let store = fixture(ScanStatus::Complete).await?;
    let directory = tempfile::tempdir()?;
    let expected = fixture_pages();
    for format in [ExportFormat::Json, ExportFormat::Jsonl] {
        let options = ExportOptions {
            format,
            provenance: ProvenanceMode::AllObservations,
            ..ExportOptions::default()
        };
        let path = directory.path().join(format!("all-{format:?}"));
        let result =
            export_snapshot(&store, SCAN, &path, options, &CancellationToken::new()).await?;
        assert_eq!(
            result.counts,
            ExportCounts {
                resources: 6,
                edges: 4,
                scopes: 4,
                failures: 0,
                records: 14
            }
        );
        let document = parse(&path, format).await?;
        assert_eq!(document.header.scan.summary.resources, 2);
        assert_eq!(document.header.scan.summary.edges, 1);
        assert_eq!(document.header.provenance, ProvenanceMode::AllObservations);
        for page in &expected {
            for resource in &page.resources {
                let matches = document
                    .records
                    .iter()
                    .filter(|record| match record {
                        ExportRecord::Resource(row) => {
                            row.resource == *resource
                                && row.observation.as_ref().is_some_and(|observation| {
                                    observation.scope == page.scope
                                        && observation.page_number == page.page_number
                                })
                        }
                        _ => false,
                    })
                    .count();
                assert_eq!(matches, 1);
            }
            for edge in &page.edges {
                let matches = document
                    .records
                    .iter()
                    .filter(|record| match record {
                        ExportRecord::Edge(row) => {
                            row.edge == *edge
                                && row.observation.as_ref().is_some_and(|observation| {
                                    observation.scope == page.scope
                                        && observation.page_number == page.page_number
                                })
                        }
                        _ => false,
                    })
                    .count();
                assert_eq!(matches, 1);
            }
        }
        assert!(
            !tokio::fs::read_to_string(path)
                .await?
                .contains("opaque-next-page")
        );
    }
    store.close().await;
    Ok(())
}
