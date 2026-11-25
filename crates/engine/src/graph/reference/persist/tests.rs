use super::{ReferenceConfig, persist_scan_owned};
use crate::export::{ExportError, ExportOptions, ReferenceMode, export_snapshot};
use crate::graph::reference::{RestoreOutcome, restore_synthetic};
use crate::store::Store;
use inventory_core::{CollectionPage, CollectionScope, Resource, ScanStatus, Source};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use tokio_util::sync::CancellationToken;

type TestResult = Result<(), Box<dyn std::error::Error>>;

const SCAN: &str = "interleave-scan";
const KEY: &str = "arn:aws:kms:us-east-1:111111111111:key/interleave";
const BUCKET: &str = "arn:aws:s3:::interleave-bucket";

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

async fn seeded(path: &std::path::Path) -> Result<Store, Box<dyn std::error::Error>> {
    let store = Store::open(path).await?;
    store.begin_scan(SCAN, 1).await?;
    let page = CollectionPage {
        scope: CollectionScope {
            account_id: "111111111111".to_owned(),
            region: "us-east-1".to_owned(),
            collector: "interleave_fixture".to_owned(),
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
                json!({"resource_identifier":"interleave"}),
            ),
        ],
        edges: Vec::new(),
    };
    store.apply_page(SCAN, page).await?;
    store.finish_scan(SCAN, ScanStatus::Complete, 2).await?;
    Ok(store)
}

fn derive() -> ExportOptions {
    ExportOptions {
        reference_mode: ReferenceMode::Derive,
        ..ExportOptions::default()
    }
}

#[tokio::test]
async fn given_export_b_persists_and_publishes_after_export_a_when_a_fails_then_a_rollback_is_superseded_and_the_edge_survives()
-> TestResult {
    let directory = tempfile::tempdir()?;
    let database = directory.path().join("inventory.sqlite");
    let first = seeded(&database).await?;
    let second = Store::open(&database).await?;
    let config = ReferenceConfig::default();

    let owned_by_a = persist_scan_owned(&first, SCAN, &config).await?;
    assert_eq!(first.edges(SCAN).await?.len(), 1);

    let published = directory.path().join("b.jsonl");
    let summary = export_snapshot(
        &second,
        SCAN,
        &published,
        derive(),
        &CancellationToken::new(),
    )
    .await?;
    assert_eq!(summary.counts.edges, 1);

    let outcome = restore_synthetic(&first, SCAN, &owned_by_a.rollback).await?;

    assert_eq!(outcome, RestoreOutcome::Superseded);
    assert_eq!(second.edges(SCAN).await?.len(), 1, "B's refs survive A");
    assert_eq!(first.edges(SCAN).await?.len(), 1);
    first.close().await;
    second.close().await;
    Ok(())
}

#[tokio::test]
async fn given_later_failed_export_when_it_rolls_back_then_it_restores_the_earlier_writers_state()
-> TestResult {
    let directory = tempfile::tempdir()?;
    let database = directory.path().join("inventory.sqlite");
    let first = seeded(&database).await?;
    let second = Store::open(&database).await?;
    let config = ReferenceConfig::default();

    let owned_by_a = persist_scan_owned(&first, SCAN, &config).await?;
    let owned_by_b = persist_scan_owned(&second, SCAN, &config).await?;

    assert_eq!(
        restore_synthetic(&second, SCAN, &owned_by_b.rollback).await?,
        RestoreOutcome::Restored
    );
    assert_eq!(
        first.edges(SCAN).await?.len(),
        1,
        "A's state is B's baseline"
    );
    assert_eq!(
        restore_synthetic(&first, SCAN, &owned_by_a.rollback).await?,
        RestoreOutcome::Superseded
    );
    assert_eq!(first.edges(SCAN).await?.len(), 1);
    first.close().await;
    second.close().await;
    Ok(())
}

#[tokio::test]
async fn given_sole_failed_export_when_it_rolls_back_then_the_pre_export_state_returns()
-> TestResult {
    let directory = tempfile::tempdir()?;
    let database = directory.path().join("inventory.sqlite");
    let store = seeded(&database).await?;
    let output = directory.path().join("failed.jsonl");
    let result = export_snapshot(
        &store,
        SCAN,
        &output,
        ExportOptions {
            max_bytes: 1,
            ..derive()
        },
        &CancellationToken::new(),
    )
    .await;
    assert!(matches!(result, Err(ExportError::Limit(_))));
    assert!(store.edges(SCAN).await?.is_empty());
    store.close().await;
    Ok(())
}
