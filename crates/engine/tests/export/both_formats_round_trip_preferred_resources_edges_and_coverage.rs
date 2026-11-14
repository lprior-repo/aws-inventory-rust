use super::*;

#[tokio::test]
pub(super) async fn both_formats_round_trip_preferred_resources_edges_and_coverage() -> TestResult {
    let store = fixture(ScanStatus::Complete).await?;
    let directory = tempfile::tempdir()?;
    let mut parsed = Vec::new();
    for format in [ExportFormat::Json, ExportFormat::Jsonl] {
        let path = directory.path().join(format!("normalized-{format:?}"));
        let options = ExportOptions {
            format,
            ..ExportOptions::default()
        };
        let result =
            export_snapshot(&store, SCAN, &path, options, &CancellationToken::new()).await?;
        let document = parse(&path, format).await?;
        assert_eq!(document.header.artifact, EXPORT_ARTIFACT);
        assert_eq!(document.header.version, EXPORT_VERSION);
        assert_eq!(document.header.scan.summary, store.summary(SCAN).await?);
        assert_eq!(document.header.scan.started_at, 1);
        assert_eq!(document.header.scan.finished_at, 200);
        assert_eq!(document.header.scan.status, ScanStatus::Complete);
        assert_eq!(document.summary, result.counts);
        assert_eq!(
            result.counts,
            ExportCounts {
                resources: 2,
                edges: 1,
                scopes: 4,
                failures: 0,
                records: 7
            }
        );
        let resources: Vec<_> = document
            .records
            .iter()
            .filter_map(|record| match record {
                ExportRecord::Resource(row) => {
                    assert!(row.observation.is_none());
                    Some(row.resource.clone())
                }
                _ => None,
            })
            .collect();
        let edges: Vec<_> = document
            .records
            .iter()
            .filter_map(|record| match record {
                ExportRecord::Edge(row) => {
                    assert!(row.observation.is_none());
                    Some(row.edge.clone())
                }
                _ => None,
            })
            .collect();
        assert_eq!(resources, store.resources(SCAN).await?);
        assert_eq!(edges, store.edges(SCAN).await?);
        assert_eq!(
            resources.last().map(|row| (row.source, row.observed_at)),
            Some((Source::Direct, 101))
        );
        let bytes = tokio::fs::read(&path).await?;
        assert_eq!(result.bytes, u64::try_from(bytes.len())?);
        assert_eq!(
            result.sha256,
            format!("{:x}", base16ct::HexDisplay(&Sha256::digest(&bytes)))
        );
        parsed.push(document);
    }
    assert_eq!(parsed.first(), parsed.last());
    store.close().await;
    Ok(())
}
