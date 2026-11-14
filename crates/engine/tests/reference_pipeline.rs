use inventory_core::{CollectionPage, CollectionScope, Resource, ScanStatus, Source};
use inventory_engine::{
    export::{ExportError, ExportOptions, ExportRecord, ReferenceMode, export_snapshot},
    store::Store,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use tokio_util::sync::CancellationToken;

type TestResult = Result<(), Box<dyn std::error::Error>>;

const SCAN: &str = "pipeline-scan";
const KEY: &str = "arn:aws:kms:us-east-1:111111111111:key/pipeline";
const BUCKET: &str = "arn:aws:s3:::pipeline-bucket";

fn resource(id: &str, kind: &str, properties: Value) -> Resource {
    Resource {
        id: id.to_owned(),
        account_id: "111111111111".to_owned(),
        region: "us-east-1".to_owned(),
        kind: kind.to_owned(),
        name: None,
        properties,
        tags: BTreeMap::new(),
        source: Source::Direct,
        observed_at: 1,
    }
}

async fn fixture(status: ScanStatus) -> Result<Store, Box<dyn std::error::Error>> {
    let store = Store::open(":memory:").await?;
    store.begin_scan(SCAN, 1).await?;
    store
        .apply_page(
            SCAN,
            CollectionPage {
                scope: CollectionScope {
                    account_id: "111111111111".to_owned(),
                    region: "us-east-1".to_owned(),
                    collector: "pipeline_fixture".to_owned(),
                    parent_id: None,
                    source: Source::Direct,
                },
                page_number: 1,
                next_token: None,
                resources: vec![
                    resource(
                        BUCKET,
                        "aws_s3_bucket",
                        json!({"encryption":{"kms_key":KEY}}),
                    ),
                    resource(
                        KEY,
                        "aws_kms_key",
                        json!({"resource_identifier":"pipeline"}),
                    ),
                ],
                edges: Vec::new(),
            },
        )
        .await?;
    store.finish_scan(SCAN, status, 2).await?;
    Ok(store)
}

#[tokio::test]
async fn export_pipeline_persists_and_replays_universal_reference_edges() -> TestResult {
    let store = fixture(ScanStatus::Complete).await?;
    let directory = tempfile::tempdir()?;
    let plain = directory.path().join("plain.jsonl");
    let base = export_snapshot(
        &store,
        SCAN,
        &plain,
        ExportOptions::default(),
        &CancellationToken::new(),
    )
    .await?;
    assert_eq!(base.counts.edges, 0);
    assert!(store.edges(SCAN).await?.is_empty());

    let analyzed = directory.path().join("analyzed.jsonl");
    let enabled = ExportOptions {
        reference_mode: ReferenceMode::Derive,
        ..ExportOptions::default()
    };
    let first =
        export_snapshot(&store, SCAN, &analyzed, enabled, &CancellationToken::new()).await?;
    assert_eq!(first.counts.edges, 1);
    let persisted = store.edges(SCAN).await?;
    let edge = persisted
        .iter()
        .find(|edge| edge.relation == "REFERENCES")
        .ok_or("missing persisted edge")?;
    assert_eq!(edge.from_id, BUCKET);
    assert_eq!(edge.to_id, KEY);
    assert_eq!(
        edge.properties.get("source").and_then(Value::as_str),
        Some("universal_reference")
    );

    let text = tokio::fs::read_to_string(&analyzed).await?;
    let records = text
        .lines()
        .map(serde_json::from_str::<ExportRecord>)
        .collect::<Result<Vec<_>, _>>()?;
    assert!(records.iter().any(|record| matches!(
        record,
        ExportRecord::Edge(exported)
            if exported.edge.relation == "REFERENCES" && exported.edge.to_id == KEY
    )));

    let replay = directory.path().join("replay.jsonl");
    let second = export_snapshot(&store, SCAN, &replay, enabled, &CancellationToken::new()).await?;
    assert_eq!(second.counts.edges, first.counts.edges);
    assert_eq!(store.edges(SCAN).await?, persisted);

    let all = directory.path().join("all.jsonl");
    let provenance_aware = ExportOptions {
        provenance: inventory_engine::export::ProvenanceMode::AllObservations,
        reference_mode: ReferenceMode::Derive,
        ..ExportOptions::default()
    };
    let third = export_snapshot(
        &store,
        SCAN,
        &all,
        provenance_aware,
        &CancellationToken::new(),
    )
    .await?;
    assert_eq!(third.counts.edges, first.counts.edges);
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn illegal_export_never_persists_reference_edges() -> TestResult {
    let store = fixture(ScanStatus::Partial).await?;
    let directory = tempfile::tempdir()?;
    let output = directory.path().join("partial.jsonl");
    let result = export_snapshot(
        &store,
        SCAN,
        &output,
        ExportOptions {
            reference_mode: ReferenceMode::Derive,
            ..ExportOptions::default()
        },
        &CancellationToken::new(),
    )
    .await;
    assert!(matches!(result, Err(ExportError::Invalid(_))));
    assert!(!output.exists());
    assert!(store.edges(SCAN).await?.is_empty());
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn failed_publication_rolls_back_derived_edges() -> TestResult {
    let store = fixture(ScanStatus::Complete).await?;
    let directory = tempfile::tempdir()?;

    let output = directory.path().join("failed.jsonl");
    let result = export_snapshot(
        &store,
        SCAN,
        &output,
        ExportOptions {
            reference_mode: ReferenceMode::Derive,
            max_bytes: 1,
            ..ExportOptions::default()
        },
        &CancellationToken::new(),
    )
    .await;
    assert!(matches!(result, Err(ExportError::Limit(_))));
    assert!(!output.exists());
    assert!(!directory.path().join("failed.jsonl.partial").exists());
    assert!(store.edges(SCAN).await?.is_empty());
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn failed_publication_restores_previously_persisted_edges() -> TestResult {
    let store = fixture(ScanStatus::Complete).await?;
    let directory = tempfile::tempdir()?;
    let enabled = ExportOptions {
        reference_mode: ReferenceMode::Derive,
        ..ExportOptions::default()
    };
    let published = directory.path().join("published.jsonl");
    export_snapshot(&store, SCAN, &published, enabled, &CancellationToken::new()).await?;
    let persisted = store.edges(SCAN).await?;
    assert_eq!(persisted.len(), 1);

    let failed = directory.path().join("replay-failed.jsonl");
    let result = export_snapshot(
        &store,
        SCAN,
        &failed,
        ExportOptions {
            max_bytes: 1,
            ..enabled
        },
        &CancellationToken::new(),
    )
    .await;
    assert!(matches!(result, Err(ExportError::Limit(_))));
    assert!(!failed.exists());
    let restored = store.edges(SCAN).await?;
    assert_eq!(restored, persisted);
    assert_eq!(
        restored
            .first()
            .and_then(|edge| edge.properties.get("source"))
            .and_then(Value::as_str),
        Some("universal_reference")
    );
    store.close().await;
    Ok(())
}
