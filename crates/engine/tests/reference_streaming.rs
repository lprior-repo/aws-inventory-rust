//! Behavior: reference resolution streams resources and therefore succeeds on
//! scans whose serialized resources exceed the bounded export snapshot budget.
use inventory_core::{CollectionPage, CollectionScope, Resource, ScanStatus, Source};
use inventory_engine::{
    graph::reference::{self, ReferenceConfig},
    store::{Store, StoreError},
};
use serde_json::json;
use std::collections::BTreeMap;

type TestResult = Result<(), Box<dyn std::error::Error>>;

const SCAN: &str = "streaming-scan";
const ACCOUNT: &str = "111111111111";
const REGION: &str = "us-east-1";
/// Ten resources per page at ~690 KB each; ten pages exceed the 64 MiB export bound.
const PADDING_BYTES: usize = 690_000;
const PER_PAGE: usize = 10;
const PAGES: usize = 10;

fn bulk_resource(page: usize, offset: usize) -> Resource {
    Resource {
        id: format!("arn:aws:ec2:{REGION}:{ACCOUNT}:instance/i-{page}-{offset}"),
        account_id: ACCOUNT.to_owned(),
        region: REGION.to_owned(),
        kind: "aws_ec2_instance".to_owned(),
        name: None,
        properties: json!({"padding": "x".repeat(PADDING_BYTES)}),
        tags: BTreeMap::new(),
        source: Source::Direct,
        observed_at: 1,
    }
}

#[tokio::test]
async fn resolution_and_persistence_stream_past_the_export_byte_budget() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan(SCAN, 1).await?;
    for page in 0..PAGES {
        store
            .apply_page(
                SCAN,
                CollectionPage {
                    scope: CollectionScope {
                        account_id: ACCOUNT.to_owned(),
                        region: REGION.to_owned(),
                        collector: format!("bulk-{page}"),
                        parent_id: None,
                        source: Source::Direct,
                    },
                    page_number: 1,
                    next_token: None,
                    resources: (0..PER_PAGE)
                        .map(|offset| bulk_resource(page, offset))
                        .collect(),
                    edges: Vec::new(),
                },
            )
            .await?;
    }
    store.finish_scan(SCAN, ScanStatus::Complete, 2).await?;

    // The bounded snapshot collector still refuses this scan; the resolver must
    // not depend on it.
    assert!(matches!(
        store.resources(SCAN).await,
        Err(StoreError::Limit(_))
    ));
    let resolved = reference::resolve_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    assert_eq!(resolved.summary.candidates_scanned, 0);
    assert!(resolved.edges.is_empty());

    let persisted = reference::persist_scan(&store, SCAN, &ReferenceConfig::default()).await?;
    assert_eq!(persisted.edges_emitted, 0);
    assert!(store.edges(SCAN).await?.is_empty());
    store.close().await;
    Ok(())
}
