use inventory_core::{CollectionPage, CollectionScope, Edge, Resource, ScanStatus, Source};
use inventory_engine::export::{
    ExportCounts, ExportFormat, ExportHeader, ExportOptions, ExportRecord, ProvenanceMode,
    export_snapshot,
};
use inventory_engine::store::Store;
use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed, TestCaseError, TestCaseResult};
use serde::Deserialize;
use serde_json::json;
use std::collections::BTreeMap;
use std::time::Duration;
use tokio::runtime::Builder;
use tokio_util::sync::CancellationToken;

fn failure(error: impl std::fmt::Display) -> TestCaseError {
    TestCaseError::fail(error.to_string())
}

#[derive(Deserialize)]
struct Document {
    header: ExportHeader,
    records: Vec<ExportRecord>,
    summary: ExportCounts,
}

fn decode(bytes: &[u8], format: ExportFormat) -> Result<Document, TestCaseError> {
    if format == ExportFormat::Json {
        return serde_json::from_slice(bytes).map_err(failure);
    }
    let mut entries = std::str::from_utf8(bytes).map_err(failure)?.lines();
    let first = entries.next().ok_or_else(|| failure("missing header"))?;
    let ExportRecord::Header(header) = serde_json::from_str(first).map_err(failure)? else {
        return Err(failure("expected header record"));
    };
    let mut records = entries
        .map(serde_json::from_str)
        .collect::<Result<Vec<ExportRecord>, _>>()
        .map_err(failure)?;
    let Some(ExportRecord::Summary(summary)) = records.pop() else {
        return Err(failure("missing summary"));
    };
    Ok(Document {
        header,
        records,
        summary,
    })
}

async fn round_trip(values: &[(u16, Vec<char>)], source: Source) -> TestCaseResult {
    let store = Store::open(":memory:").await.map_err(failure)?;
    store
        .begin_scan("generated-export", 0)
        .await
        .map_err(failure)?;
    let scope = CollectionScope {
        account_id: "123456789012".into(),
        region: "us-east-1".into(),
        collector: "generated".into(),
        parent_id: None,
        source,
    };
    let mut resources: Vec<_> = values.iter().enumerate().map(|(index, (number, characters))| {
        let text: String = characters.iter().collect();
        Resource {
            id: format!("generated-{index:03}"), account_id: "123456789012".into(), region: "us-east-1".into(),
            kind: "aws_test_resource".into(), name: Some(text.clone()), source, observed_at: i64::from(*number),
            properties: json!({"integer":number,"text":text,"list":[null,true,false,{"inner":text}]}),
            tags: BTreeMap::from([("text".into(), text)]),
        }
    }).collect();
    resources.sort_by(|left, right| left.id.cmp(&right.id));
    let edges: Vec<_> = resources
        .iter()
        .map(|resource| Edge {
            from_id: resource.id.clone(),
            to_id: "external-resource".into(),
            relation: "REFERENCES".into(),
            source,
            observed_at: resource.observed_at,
            properties: json!({"source_name":resource.name}),
        })
        .collect();
    store
        .apply_page(
            "generated-export",
            CollectionPage {
                scope: scope.clone(),
                page_number: 1,
                next_token: None,
                resources: resources.clone(),
                edges: edges.clone(),
            },
        )
        .await
        .map_err(failure)?;
    store
        .finish_scan("generated-export", ScanStatus::Complete, 65_536)
        .await
        .map_err(failure)?;
    let directory = tempfile::tempdir().map_err(failure)?;
    for provenance in [ProvenanceMode::Normalized, ProvenanceMode::AllObservations] {
        for format in [ExportFormat::Json, ExportFormat::Jsonl] {
            let path = directory.path().join(format!("{format:?}-{provenance:?}"));
            let options = ExportOptions {
                format,
                provenance,
                ..ExportOptions::default()
            };
            let receipt = export_snapshot(
                &store,
                "generated-export",
                &path,
                options,
                &CancellationToken::new(),
            )
            .await
            .map_err(failure)?;
            let document = decode(&tokio::fs::read(&path).await.map_err(failure)?, format)?;
            prop_assert_eq!(document.header.provenance, provenance);
            prop_assert_eq!(document.summary, receipt.counts);
            let actual_resources: Vec<_> = document
                .records
                .iter()
                .filter_map(|record| match record {
                    ExportRecord::Resource(row) => Some(row.resource.clone()),
                    _ => None,
                })
                .collect();
            let actual_edges: Vec<_> = document
                .records
                .iter()
                .filter_map(|record| match record {
                    ExportRecord::Edge(row) => Some(row.edge.clone()),
                    _ => None,
                })
                .collect();
            prop_assert_eq!(&actual_resources, &resources);
            prop_assert_eq!(&actual_edges, &edges);
            for record in document.records {
                let observation = match record {
                    ExportRecord::Resource(row) => row.observation,
                    ExportRecord::Edge(row) => row.observation,
                    _ => continue,
                };
                match provenance {
                    ProvenanceMode::Normalized => prop_assert!(observation.is_none()),
                    ProvenanceMode::AllObservations => {
                        let observation =
                            observation.ok_or_else(|| failure("missing observation provenance"))?;
                        prop_assert_eq!(&observation.scope, &scope);
                        prop_assert_eq!(observation.page_number, 1);
                    }
                }
            }
        }
    }
    store.close().await;
    Ok(())
}

proptest! {
    #![proptest_config(Config {
        cases: 16,
        max_shrink_iters: 128,
        rng_seed: RngSeed::Fixed(0x4558_504f_5254_5631),
        ..Config::default()
    })]

    #[test]
    fn generated_values_keep_exact_domain_records(
        values in prop::collection::vec((any::<u16>(), prop::collection::vec(prop::sample::select(vec!['a', 'Z', 'é', '中', '\\', '"', '\n', '\t']), 0..32)), 0..12),
        source_index in 0..4_u8,
    ) {
        let source = match source_index {
            0 => Source::Direct,
            1 => Source::CloudControl,
            2 => Source::AwsConfig,
            _ => Source::ResourceExplorer,
        };
        let runtime = Builder::new_current_thread().enable_all().build().map_err(failure)?;
        runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(10), round_trip(&values, source)).await
                .map_err(|_| failure("inventory export property exceeded its 10-second case deadline"))?
        })?;
    }
}
