//! `claim_batch` ordering and limit behavior. The trait's default issues one
//! `claim` per lease; a store may implement it as a single statement.

use super::*;
use inventory_engine::inventory::{InventoryStore, WorkSpec};

async fn seed<S: InventoryStore>(store: &S, spec: WorkSpec) -> TestResult {
    store.seed_works(&[spec]).await?;
    Ok(())
}

pub(super) async fn run<S, F, G>(make: &F) -> TestResult
where
    S: InventoryStore,
    F: AsyncFn() -> TestResult<(G, S, Vec<u8>)>,
{
    let (_guard, store, _) = make().await?;
    given_three_ready_scopes_when_claiming_two_then_the_smaller_keys_come_first_and_one_remains(
        &store,
    )
    .await?;
    let (_guard, store, _) = make().await?;
    given_scopes_completed_at_different_times_when_claiming_batch_then_stalest_come_first(&store)
        .await?;
    let (_guard, store, _) = make().await?;
    given_a_zero_limit_when_claiming_a_batch_then_it_is_refused_and_the_work_stays_ready(&store)
        .await
}

async fn batch_regions<S: InventoryStore>(
    store: &S,
    sweep: &SweepId,
    limit: u16,
) -> TestResult<Vec<String>> {
    let leases = store.claim_batch(sweep, 100, 10_000, limit).await?;
    Ok(leases
        .iter()
        .map(|lease| lease.scope().region().as_str().to_owned())
        .collect())
}

async fn given_three_ready_scopes_when_claiming_two_then_the_smaller_keys_come_first_and_one_remains<
    S: InventoryStore,
>(
    store: &S,
) -> TestResult {
    // Three never-completed scopes: claim order is by scope key.
    seed(store, work("tie", "us-west-2")?).await?;
    seed(store, work("tie", "eu-west-1")?).await?;
    seed(store, work("tie", "us-east-1")?).await?;
    let sweep = SweepId::new("tie".to_owned())?;

    let first = batch_regions(store, &sweep, 2).await?;
    assert_eq!(first, ["eu-west-1", "us-east-1"]);

    let second = batch_regions(store, &sweep, 2).await?;
    assert_eq!(second, ["us-west-2"]);

    let none = batch_regions(store, &sweep, 2).await?;
    assert!(none.is_empty());
    Ok(())
}

async fn given_scopes_completed_at_different_times_when_claiming_batch_then_stalest_come_first<
    S: InventoryStore,
>(
    store: &S,
) -> TestResult {
    // First sweep: claim all three scopes, then complete them at staggered times.
    // All claims happen before any completion, so claim time < completion time always.
    seed(store, work("stale-s1", "us-west-2")?).await?;
    seed(store, work("stale-s1", "us-east-2")?).await?;
    seed(store, work("stale-s1", "eu-west-1")?).await?;
    let s1 = SweepId::new("stale-s1".to_owned())?;

    let l1 = store.claim(&s1, 100, 10_000).await?.ok_or("lease 1")?;
    let l2 = store.claim(&s1, 110, 10_000).await?.ok_or("lease 2")?;
    let l3 = store.claim(&s1, 120, 10_000).await?.ok_or("lease 3")?;

    // Complete in a different order from claims, each after its claim time.
    // l1 (eu-west-1) completes first -> stalest.
    store
        .finish_scope(l1, ScopeOutcome::Complete, 200, None)
        .await?;
    // l3 (us-west-2) completes second.
    store
        .finish_scope(l3, ScopeOutcome::Complete, 300, None)
        .await?;
    // l2 (us-east-2) completes last -> freshest.
    store
        .finish_scope(l2, ScopeOutcome::Complete, 400, None)
        .await?;

    // Completion times: eu-west-1@200 (oldest), us-west-2@300, us-east-2@400 (newest).
    // Stalest-first should pick eu-west-1 and us-west-2.

    // Second sweep: seed and claim batch of 2.
    seed(store, work("stale-s2", "us-west-2")?).await?;
    seed(store, work("stale-s2", "us-east-2")?).await?;
    seed(store, work("stale-s2", "eu-west-1")?).await?;
    let s2 = SweepId::new("stale-s2".to_owned())?;

    let batch = store.claim_batch(&s2, 1_000, 1_000, 2).await?;
    assert_eq!(batch.len(), 2);
    let regions: Vec<String> = batch
        .iter()
        .map(|l| l.scope().region().as_str().to_owned())
        .collect();
    // Stalest two are eu-west-1 and us-west-2; returned sorted by key.
    // Key order alone would give [eu-west-1, us-east-2].
    assert_eq!(regions, ["eu-west-1", "us-west-2"]);

    // Remaining scope is the newest: us-east-2.
    let remaining = batch_regions(store, &s2, 2).await?;
    assert_eq!(remaining, ["us-east-2"]);
    Ok(())
}

async fn given_a_zero_limit_when_claiming_a_batch_then_it_is_refused_and_the_work_stays_ready<
    S: InventoryStore,
>(
    store: &S,
) -> TestResult {
    seed(store, work("zero", "us-east-1")?).await?;
    let sweep = SweepId::new("zero".to_owned())?;

    let refused = store.claim_batch(&sweep, 100, 10_000, 0).await;
    assert!(matches!(refused, Err(StoreError::Invalid(_))));
    assert_eq!(batch_regions(store, &sweep, 1).await?, ["us-east-1"]);
    Ok(())
}
