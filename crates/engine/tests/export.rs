//! Portable snapshot contracts through the public Store/export APIs.

use futures_util::StreamExt;
use inventory_core::{
    CollectionEvent, CollectionFailure, CollectionPage, CollectionScope, Edge, Resource,
    ScanStatus, Source,
};
use inventory_engine::export::{
    EXPORT_ARTIFACT, EXPORT_VERSION, ExportCounts, ExportError, ExportFormat, ExportHeader,
    ExportOptions, ExportRecord, MAX_EXPORT_BYTES, MAX_EXPORT_RECORDS, ProvenanceMode,
    export_snapshot,
};
use inventory_engine::store::Store;
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio_util::sync::CancellationToken;

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
const SCAN: &str = "portable-snapshot";

#[derive(Debug, Deserialize, PartialEq)]
struct Document {
    header: ExportHeader,
    records: Vec<ExportRecord>,
    summary: ExportCounts,
}

fn scope(source: Source) -> CollectionScope {
    CollectionScope {
        account_id: "111111111111".into(),
        region: "eu-west-1".into(),
        collector: format!("fixture_{}", source.as_str()),
        parent_id: Some("parent/世界".into()),
        source,
    }
}

fn resource(id: &str, source: Source, time: i64) -> Resource {
    Resource {
        id: id.into(),
        account_id: "111111111111".into(),
        region: "eu-west-1".into(),
        kind: "aws_vpc".into(),
        name: Some("quotes \" and newline\n日本語".into()),
        properties: json!({"null":null,"enabled":true,"counter":18446744073709551615_u64,
            "nested":{"source":source,"items":["line\n\t",3,false]},"binary_text":"\u{0000}"}),
        tags: BTreeMap::from([("purpose".into(), "JSON / JSONL \\ round trip".into())]),
        source,
        observed_at: time,
    }
}

fn edge(source: Source, time: i64) -> Edge {
    Edge {
        from_id: "z-resource".into(),
        to_id: "uncollected-target".into(),
        relation: "USES".into(),
        source,
        observed_at: time,
        properties: json!({"paths":["VpcId"],"owner_resolution":"observed","source":source}),
    }
}

fn fixture_pages() -> Vec<CollectionPage> {
    let mut pages: Vec<_> = [
        (Source::Direct, 100),
        (Source::CloudControl, 120),
        (Source::AwsConfig, 150),
        (Source::ResourceExplorer, 180),
    ]
    .into_iter()
    .map(|(source, time)| CollectionPage {
        scope: scope(source),
        page_number: 1,
        next_token: (source == Source::Direct).then(|| "opaque-next-page".into()),
        resources: vec![resource("z-resource", source, time)],
        edges: if source == Source::ResourceExplorer {
            Vec::new()
        } else {
            vec![edge(source, 100)]
        },
    })
    .collect();
    pages.push(CollectionPage {
        scope: scope(Source::Direct),
        page_number: 2,
        next_token: None,
        resources: vec![
            resource("z-resource", Source::Direct, 101),
            resource("a-resource", Source::Direct, 102),
        ],
        edges: vec![edge(Source::Direct, 101)],
    });
    pages
}

async fn fixture(status: ScanStatus) -> Result<Store, Box<dyn std::error::Error + Send + Sync>> {
    let store = Store::open(":memory:").await?;
    store.begin_scan(SCAN, 1).await?;
    for page in fixture_pages() {
        store.apply_page(SCAN, page).await?;
    }
    store.finish_scan(SCAN, status, 200).await?;
    Ok(store)
}

async fn parse(
    path: &Path,
    format: ExportFormat,
) -> Result<Document, Box<dyn std::error::Error + Send + Sync>> {
    let bytes = tokio::fs::read(path).await?;
    if format == ExportFormat::Json {
        return Ok(serde_json::from_slice(&bytes)?);
    }
    let text = std::str::from_utf8(&bytes)?;
    let mut lines = text.lines();
    let first = lines.next().ok_or("missing header")?;
    assert!(serde_json::from_str::<CollectionEvent>(first).is_err());
    let ExportRecord::Header(header) = serde_json::from_str(first)? else {
        return Err("first record is not a header".into());
    };
    let mut records = lines
        .map(serde_json::from_str)
        .collect::<Result<Vec<ExportRecord>, _>>()?;
    let Some(ExportRecord::Summary(summary)) = records.pop() else {
        return Err("last record is not a summary".into());
    };
    Ok(Document {
        header,
        records,
        summary,
    })
}

async fn export_above_query_cap() -> TestResult {
    const TOTAL: usize = 10_001;
    const PAGE_ROWS: usize = 500;
    let pages = TOTAL.div_ceil(PAGE_ROWS);
    let store = Store::open(":memory:").await?;
    store.begin_scan(SCAN, 1).await?;
    for page_index in 0..pages {
        let start = page_index.saturating_mul(PAGE_ROWS);
        let end = start.saturating_add(PAGE_ROWS).min(TOTAL);
        let resources = (start..end)
            .map(|index| Resource {
                id: format!("resource-{index:05}"),
                account_id: "111111111111".into(),
                region: "eu-west-1".into(),
                kind: "aws_test_resource".into(),
                name: None,
                properties: json!({"index": index}),
                tags: BTreeMap::new(),
                source: Source::Direct,
                observed_at: 1,
            })
            .collect();
        store
            .apply_page(
                SCAN,
                CollectionPage {
                    scope: scope(Source::Direct),
                    page_number: u32::try_from(page_index.saturating_add(1))?,
                    next_token: (end < TOTAL).then(|| format!("next-{end}")),
                    resources,
                    edges: Vec::new(),
                },
            )
            .await?;
    }
    store.finish_scan(SCAN, ScanStatus::Complete, 2).await?;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("over-query-cap.jsonl");
    let summary = export_snapshot(
        &store,
        SCAN,
        &path,
        ExportOptions::default(),
        &CancellationToken::new(),
    )
    .await?;
    assert_eq!(
        summary.counts,
        ExportCounts {
            resources: 10_001,
            edges: 0,
            scopes: 1,
            failures: 0,
            records: 10_002
        }
    );
    let mut reader = BufReader::new(tokio::fs::File::open(&path).await?);
    let mut line = Vec::with_capacity(16_384);
    let mut digest = Sha256::new();
    let mut byte_count = 0_u64;
    let mut resource_count = 0_usize;
    let mut scope_seen = false;
    let mut end_seen = false;
    let mut eof_seen = false;
    // This independent reader holds at most one bounded line. An accidentally
    // unframed whole-file JSON document cannot force a giant test allocation.
    for line_index in 0..TOTAL.saturating_add(4) {
        line.clear();
        let read = (&mut reader)
            .take(16_384)
            .read_until(b'\n', &mut line)
            .await?;
        if read == 0 {
            eof_seen = true;
            break;
        }
        assert!(!end_seen, "extra data after summary");
        assert_eq!(
            line.last(),
            Some(&b'\n'),
            "fixture line exceeded its read bound"
        );
        byte_count = byte_count
            .checked_add(u64::try_from(read)?)
            .ok_or("byte count overflow")?;
        digest.update(&line);
        match serde_json::from_slice::<ExportRecord>(&line)? {
            ExportRecord::Header(header) => {
                assert_eq!(line_index, 0);
                assert_eq!(header.artifact, EXPORT_ARTIFACT);
                assert_eq!(header.version, EXPORT_VERSION);
                assert_eq!(header.scan.summary.resources, 10_001);
                assert_eq!(header.scan.summary.edges, 0);
                assert_eq!(header.scan.summary.completed_scopes, 1);
                assert_eq!(header.scan.status, ScanStatus::Complete);
            }
            ExportRecord::Resource(row) => {
                assert!(line_index != 0 && !scope_seen);
                assert_eq!(row.resource.id, format!("resource-{resource_count:05}"));
                assert_eq!(row.resource.properties, json!({"index":resource_count}));
                assert_eq!(row.resource.source, Source::Direct);
                assert!(row.observation.is_none());
                resource_count = resource_count.saturating_add(1);
            }
            ExportRecord::Scope(row) => {
                assert_eq!(resource_count, TOTAL);
                assert!(!scope_seen);
                assert_eq!(row.last_page, u32::try_from(pages)?);
                assert!(row.complete && !row.failed);
                scope_seen = true;
            }
            ExportRecord::Summary(counts) => {
                assert_eq!(resource_count, TOTAL);
                assert!(scope_seen);
                assert_eq!(counts, summary.counts);
                end_seen = true;
            }
            _ => return Err("unexpected record kind in resource-only fixture".into()),
        }
    }
    assert!(eof_seen && end_seen);
    assert_eq!(resource_count, TOTAL);
    assert_eq!(byte_count, summary.bytes);
    assert_eq!(tokio::fs::metadata(&path).await?.len(), summary.bytes);
    assert_eq!(
        format!("{:x}", base16ct::HexDisplay(&digest.finalize())),
        summary.sha256
    );
    store.close().await;
    Ok(())
}
#[path = "export/all_observations_retain_every_source_page_and_original_value.rs"]
mod all_observations_retain_every_source_page_and_original_value;
#[path = "export/both_formats_round_trip_preferred_resources_edges_and_coverage.rs"]
mod both_formats_round_trip_preferred_resources_edges_and_coverage;
#[path = "export/cancellation_before_and_during_a_pending_read_does_not_publish.rs"]
mod cancellation_before_and_during_a_pending_read_does_not_publish;
#[path = "export/concurrent_exports_publish_exactly_one_complete_file.rs"]
mod concurrent_exports_publish_exactly_one_complete_file;
#[path = "export/exact_record_and_byte_budgets_succeed_and_overflow_cleans_owned_partial.rs"]
mod exact_record_and_byte_budgets_succeed_and_overflow_cleans_owned_partial;
#[path = "export/existing_outputs_and_partials_are_never_clobbered_or_removed.rs"]
mod existing_outputs_and_partials_are_never_clobbered_or_removed;
#[path = "export/failures_and_empty_scopes_survive_export_without_inventing_resources.rs"]
mod failures_and_empty_scopes_survive_export_without_inventing_resources;
#[path = "export/finalized_partial_and_cancelled_require_opt_in_and_keep_their_status.rs"]
mod finalized_partial_and_cancelled_require_opt_in_and_keep_their_status;
#[path = "export/invalid_budget_bounds_are_rejected_before_any_file_is_created.rs"]
mod invalid_budget_bounds_are_rejected_before_any_file_is_created;
#[path = "export/ordering_and_content_hash_are_deterministic_for_both_modes_and_formats.rs"]
mod ordering_and_content_hash_are_deterministic_for_both_modes_and_formats;
#[path = "export/running_and_unknown_scans_never_create_an_output_even_with_opt_in.rs"]
mod running_and_unknown_scans_never_create_an_output_even_with_opt_in;
#[path = "export/snapshot_export_can_exceed_the_read_only_query_row_cap.rs"]
mod snapshot_export_can_exceed_the_read_only_query_row_cap;
