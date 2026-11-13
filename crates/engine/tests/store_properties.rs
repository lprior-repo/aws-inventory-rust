//! Generated page commands are checked against an independent committed ledger.
//! SQLx transactions, failed writes, finalization and database reopen are real.

use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::time::Duration;

use inventory_core::{
    CollectionFailure, CollectionPage, CollectionScope, Edge, Resource, ScanStatus, Source,
};
use inventory_engine::store::{Checkpoint, Store, StoreError};
use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed, TestCaseError, TestCaseResult};
use serde_json::json;
use tokio::runtime::{Builder, Runtime};

const SCAN: &str = "property-scan";

fn failure(error: impl std::fmt::Display) -> TestCaseError {
    TestCaseError::fail(error.to_string())
}

fn runtime() -> Result<Runtime, TestCaseError> {
    Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(failure)
}

async fn bounded_case(case: impl Future<Output = TestCaseResult>) -> TestCaseResult {
    tokio::time::timeout(Duration::from_secs(10), case)
        .await
        .map_err(|_| failure("durable store property exceeded its 10-second case deadline"))?
}

fn scope() -> CollectionScope {
    CollectionScope {
        account_id: "111111111111".into(),
        region: "us-east-1".into(),
        collector: "property_collector".into(),
        parent_id: None,
        source: Source::Direct,
    }
}

fn resource(page: u32, index: u16, value: u16) -> Resource {
    Resource {
        id: format!("fixture-{page}-{index}"),
        account_id: "111111111111".into(),
        region: "us-east-1".into(),
        kind: "aws_ec2_instance".into(),
        name: Some(format!("value-{value}")),
        properties: json!({"value":value,"revision":0}),
        tags: BTreeMap::from([("fixture".into(), value.to_string())]),
        source: Source::Direct,
        observed_at: i64::from(page),
    }
}

fn pages(values: &[u16]) -> Result<Vec<CollectionPage>, TestCaseError> {
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let page_number = u32::try_from(index.saturating_add(1)).map_err(failure)?;
            let resources: Vec<_> = (0..value
                .checked_rem(3)
                .ok_or_else(|| failure("invalid fixture count"))?
                .saturating_add(1))
                .map(|item| resource(page_number, item, *value))
                .collect();
            let edges = resources
                .iter()
                .map(|resource| Edge {
                    from_id: resource.id.clone(),
                    to_id: format!("external-{page_number}"),
                    relation: "IN_VPC".into(),
                    source: Source::Direct,
                    observed_at: i64::from(page_number),
                    properties: json!({"fixture_value":value}),
                })
                .collect();
            Ok(CollectionPage {
                scope: scope(),
                page_number,
                next_token: (index.saturating_add(1) < values.len())
                    .then(|| format!("next-{page_number}")),
                resources,
                edges,
            })
        })
        .collect()
}

#[derive(Default)]
struct Ledger {
    committed: Vec<CollectionPage>,
    failed: bool,
    finished: Option<ScanStatus>,
}

impl Ledger {
    fn accepts(&self, page: &CollectionPage) -> bool {
        if page.page_number == 0
            || page
                .resources
                .iter()
                .any(|row| row.source != Source::Direct)
        {
            return false;
        }
        if let Some(previous) = self
            .committed
            .iter()
            .find(|old| old.page_number == page.page_number)
        {
            return previous == page;
        }
        let resource_ids: BTreeSet<_> = page.resources.iter().map(|row| &row.id).collect();
        let edge_ids: BTreeSet<_> = page
            .edges
            .iter()
            .map(|row| (&row.from_id, &row.to_id, &row.relation))
            .collect();
        let expected = self
            .committed
            .last()
            .map_or(1, |last| last.page_number.saturating_add(1));
        let scope_done = self
            .committed
            .last()
            .is_some_and(|last| last.next_token.is_none());
        let repeated_cursor = page.next_token.as_ref().is_some_and(|token| {
            self.committed
                .iter()
                .any(|old| old.next_token.as_ref() == Some(token))
        });
        self.finished.is_none()
            && !scope_done
            && !repeated_cursor
            && page.page_number == expected
            && resource_ids.len() == page.resources.len()
            && edge_ids.len() == page.edges.len()
    }

    fn effective_status(&self, requested: ScanStatus) -> ScanStatus {
        if requested == ScanStatus::Complete
            && (self.failed
                || !self
                    .committed
                    .last()
                    .is_some_and(|last| last.next_token.is_none()))
        {
            ScanStatus::Partial
        } else {
            requested
        }
    }
}

async fn assert_ledger(store: &Store, ledger: &Ledger) -> TestCaseResult {
    let mut expected_resources: Vec<_> = ledger
        .committed
        .iter()
        .flat_map(|page| page.resources.iter().cloned())
        .collect();
    expected_resources.sort_by(|left, right| left.id.cmp(&right.id));
    let mut expected_edges: Vec<_> = ledger
        .committed
        .iter()
        .flat_map(|page| page.edges.iter().cloned())
        .collect();
    expected_edges.sort_by(|left, right| {
        (&left.from_id, &left.to_id, &left.relation).cmp(&(
            &right.from_id,
            &right.to_id,
            &right.relation,
        ))
    });
    prop_assert_eq!(
        store.resources(SCAN).await.map_err(failure)?,
        expected_resources.clone()
    );
    prop_assert_eq!(
        store.edges(SCAN).await.map_err(failure)?,
        expected_edges.clone()
    );
    let last = ledger
        .committed
        .last()
        .ok_or_else(|| failure("fixture ledger must start nonempty"))?;
    let expected_checkpoint = Checkpoint {
        last_page: last.page_number,
        next_token: last.next_token.clone(),
        complete: last.next_token.is_none(),
        failed: ledger.failed,
    };
    prop_assert_eq!(
        store.checkpoint(SCAN, &scope()).await.map_err(failure)?,
        Some(expected_checkpoint)
    );
    let summary = store.summary(SCAN).await.map_err(failure)?;
    prop_assert_eq!(
        summary.resources,
        u64::try_from(expected_resources.len()).map_err(failure)?
    );
    prop_assert_eq!(
        summary.edges,
        u64::try_from(expected_edges.len()).map_err(failure)?
    );
    prop_assert_eq!(
        summary.completed_scopes,
        u64::from(last.next_token.is_none() && !ledger.failed)
    );
    prop_assert_eq!(summary.failed_scopes, u64::from(ledger.failed));
    prop_assert_eq!(
        store.scan_status(SCAN).await.map_err(failure)?,
        ledger.finished.unwrap_or(ScanStatus::Running)
    );
    prop_assert_eq!(
        store
            .query("SELECT count(*) AS pages FROM collection_pages", 1)
            .await
            .map_err(failure)?,
        json!([{"pages":ledger.committed.len()}])
    );
    prop_assert_eq!(
        store
            .query("SELECT count(*) AS failures FROM collection_failures", 1)
            .await
            .map_err(failure)?,
        json!([{"failures":usize::from(ledger.failed)}])
    );
    Ok(())
}

async fn apply(store: &Store, ledger: &mut Ledger, page: CollectionPage) -> TestCaseResult {
    let accepted = ledger.accepts(&page);
    let result = store.apply_page(SCAN, page.clone()).await;
    prop_assert_eq!(
        result.is_ok(),
        accepted,
        "page {:?}; result {:?}",
        page,
        result
    );
    if accepted
        && !ledger
            .committed
            .iter()
            .any(|old| old.page_number == page.page_number)
    {
        ledger.committed.push(page);
    }
    assert_ledger(store, ledger).await
}

async fn finish(store: &Store, ledger: &mut Ledger, requested: ScanStatus) -> TestCaseResult {
    let effective = ledger.effective_status(requested);
    let accepted = ledger.finished.is_none_or(|status| status == effective);
    let result = store.finish_scan(SCAN, requested, 1_000).await;
    prop_assert_eq!(result.is_ok(), accepted);
    if accepted {
        ledger.finished = Some(effective);
    }
    assert_ledger(store, ledger).await
}

async fn record_failure(store: &Store, ledger: &mut Ledger) -> TestCaseResult {
    let result = store
        .record_failure(
            SCAN,
            &CollectionFailure {
                scope: scope(),
                category: "synthetic_failure".into(),
                message: "synthetic scoped failure".into(),
            },
        )
        .await;
    prop_assert_eq!(result.is_ok(), ledger.finished.is_none());
    if result.is_ok() {
        ledger.failed = true;
    }
    assert_ledger(store, ledger).await
}

fn alter(page: &mut CollectionPage, variant: u8) {
    match variant {
        1 => {
            if let Some(resource) = page.resources.first_mut() {
                resource.properties = json!({"revision":1});
            }
        }
        2 => {
            if let Some(resource) = page.resources.first().cloned() {
                page.resources.push(resource);
            }
        }
        3 => {
            if let Some(edge) = page.edges.first().cloned() {
                page.edges.push(edge);
            }
        }
        4 => page.next_token = Some("next-1".into()),
        5 => page.next_token = None,
        9 => {
            if let Some(resource) = page.resources.first_mut() {
                resource.source = Source::AwsConfig;
            }
        }
        10 => page.page_number = 0,
        _ => {}
    }
}

async fn commands(
    store: &Store,
    ledger: &mut Ledger,
    canonical: &[CollectionPage],
    actions: &[(u8, u8)],
) -> TestCaseResult {
    for (selector, variant) in actions.iter().take(16) {
        match variant {
            6 => finish(store, ledger, ScanStatus::Complete).await?,
            7 => finish(store, ledger, ScanStatus::Cancelled).await?,
            8 => record_failure(store, ledger).await?,
            _ => {
                let index = usize::from(*selector)
                    .checked_rem(canonical.len())
                    .ok_or_else(|| failure("empty canonical fixture"))?;
                let mut page = canonical
                    .get(index)
                    .cloned()
                    .ok_or_else(|| failure("missing fixture page"))?;
                alter(&mut page, *variant);
                apply(store, ledger, page).await?;
            }
        }
    }
    Ok(())
}

async fn final_commands(
    store: &Store,
    ledger: &mut Ledger,
    canonical: &[CollectionPage],
) -> TestCaseResult {
    for page in canonical.iter().take(5) {
        apply(store, ledger, page.clone()).await?;
    }
    finish(store, ledger, ScanStatus::Complete).await?;
    // Every accepted payload remains exactly replayable after finalization.
    // Changing that payload must remain a conflict and leave all tables intact.
    for page in ledger.committed.clone().into_iter().take(5) {
        store
            .apply_page(SCAN, page.clone())
            .await
            .map_err(failure)?;
        let mut changed = page;
        changed
            .resources
            .first_mut()
            .ok_or_else(|| failure("missing fixture resource"))?
            .observed_at = 999;
        prop_assert!(matches!(
            store.apply_page(SCAN, changed).await,
            Err(StoreError::Conflict(_))
        ));
        assert_ledger(store, ledger).await?;
    }
    Ok(())
}

async fn page_case(values: &[u16], actions: &[(u8, u8)], split: usize) -> TestCaseResult {
    let folder = tempfile::tempdir().map_err(failure)?;
    let path = folder.path().join("pages.sqlite");
    let canonical = pages(values)?;
    let first = canonical
        .first()
        .cloned()
        .ok_or_else(|| failure("empty page fixture"))?;
    let mut ledger = Ledger {
        committed: vec![first.clone()],
        ..Ledger::default()
    };
    let (before, after) = actions.split_at(split.min(actions.len()));
    let store = Store::open(&path).await.map_err(failure)?;
    let initial = async {
        store.begin_scan(SCAN, 1).await.map_err(failure)?;
        store.apply_page(SCAN, first).await.map_err(failure)?;
        assert_ledger(&store, &ledger).await?;
        commands(&store, &mut ledger, &canonical, before).await
    }
    .await;
    store.close().await;
    initial?;
    let reopened = Store::open(&path).await.map_err(failure)?;
    let result = async {
        assert_ledger(&reopened, &ledger).await?;
        commands(&reopened, &mut ledger, &canonical, after).await?;
        final_commands(&reopened, &mut ledger, &canonical).await
    }
    .await;
    reopened.close().await;
    result
}

async fn rollback_case(value: u16, duplicate_edge: bool) -> TestCaseResult {
    let canonical = pages(&[value, value.saturating_add(1)])?;
    let first = canonical
        .first()
        .cloned()
        .ok_or_else(|| failure("first page missing"))?;
    let second = canonical
        .get(1)
        .cloned()
        .ok_or_else(|| failure("second page missing"))?;
    let store = Store::open(":memory:").await.map_err(failure)?;
    let result = async {
        store.begin_scan(SCAN, 1).await.map_err(failure)?;
        let mut ledger = Ledger {
            committed: vec![first.clone()],
            ..Ledger::default()
        };
        store.apply_page(SCAN, first).await.map_err(failure)?;
        let mut broken = second.clone();
        alter(&mut broken, if duplicate_edge { 3 } else { 2 });
        apply(&store, &mut ledger, broken).await?;
        apply(&store, &mut ledger, second).await?;
        finish(&store, &mut ledger, ScanStatus::Complete).await
    }
    .await;
    store.close().await;
    result
}

proptest! {
    #![proptest_config(Config {
        cases: 24,
        max_shrink_iters: 96,
        max_shrink_time: 2_000,
        rng_seed: RngSeed::Fixed(0x4157_535f_5041_4745),
        ..Config::default()
    })]

    #[test]
    fn generated_page_replay_failure_and_finalization_commands_match_the_ledger(
        values in prop::collection::vec(any::<u16>(), 2..=5),
        actions in prop::collection::vec((any::<u8>(), 0_u8..=10), 1..=16),
        split in 0_usize..=16,
    ) {
        runtime()?.block_on(bounded_case(page_case(&values, &actions, split)))?;
    }

    #[test]
    fn generated_resource_and_edge_constraint_failures_roll_back_the_whole_page(
        value in any::<u16>(),
        duplicate_edge in any::<bool>(),
    ) {
        runtime()?.block_on(bounded_case(rollback_case(value, duplicate_edge)))?;
    }
}
