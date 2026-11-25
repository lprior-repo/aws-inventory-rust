use super::*;
use inventory_core::inventory::{OperationId, Region, ScopeKey, ServiceId};
use inventory_engine::inventory::WorkSpec;

pub(super) async fn run<S, F, G>(make: &F) -> TestResult
where
    S: InventoryStore,
    F: AsyncFn() -> TestResult<(G, S, Vec<u8>)>,
{
    let (_guard, store, _) = make().await?;
    given_two_regions_when_seeded_then_identity_is_preserved(&store).await?;
    let (_guard, store, _) = make().await?;
    given_expired_lease_when_reclaimed_then_stale_commit_is_fenced(&store).await?;
    let (_guard, store, _) = make().await?;
    given_token_cycle_when_committed_then_page_and_children_persist(&store).await
}

fn denied_work() -> TestResult<WorkSpec> {
    let denied = ScopeKey::new(
        AccountId::parse("123456789012")?,
        Region::parse("us-east-1")?,
        ServiceId::parse("example")?,
        OperationId::parse("DeleteThing")?,
        None,
    )?;
    let sweep = SweepId::new("sweep-a".to_owned())?;
    Ok(WorkSpec::new(sweep, denied, None, Vec::new())?)
}

async fn given_two_regions_when_seeded_then_identity_is_preserved<S: InventoryStore>(
    store: &S,
) -> TestResult {
    let works = [work("sweep-a", "us-east-1")?, work("sweep-a", "us-west-2")?];
    store.seed_works(&works).await?;
    assert!(store.seed_works(&[denied_work()?]).await.is_err());
    let first = claim(store, "sweep-a", 100).await?;
    let second = claim(store, "sweep-a", 100).await?;
    assert_eq!(first.scope().region().as_str(), "us-east-1");
    assert_eq!(second.scope().region().as_str(), "us-west-2");
    let sweep = SweepId::new("sweep-a".to_owned())?;
    assert!(store.claim(&sweep, 119, 20).await?.is_none());
    Ok(())
}

async fn given_expired_lease_when_reclaimed_then_stale_commit_is_fenced<S: InventoryStore>(
    store: &S,
) -> TestResult {
    let stale = seed_claim(store, "resume-id", 100).await?;
    let current = claim(store, "resume-id", 120).await?;
    let commit = page(stale, Vec::new(), true, None, 120);
    let rejected = store.commit_page(commit).await;
    assert!(matches!(rejected, Err(StoreError::StaleLease)));
    let commit = page(current, Vec::new(), false, Some(vec![9, 8, 7]), 121);
    store.commit_page(commit).await?;
    let resumed = claim(store, "resume-id", 122).await?;
    assert_eq!(resumed.token(), Some([9, 8, 7].as_slice()));
    Ok(())
}

async fn cycle_page<S: InventoryStore>(store: &S, token: u8, at: i64) -> TestResult {
    let lease = claim(store, "cycle", at).await?;
    let next = Some(vec![token]);
    let commit = page(lease, Vec::new(), false, next, at.saturating_add(1));
    store.commit_page(commit).await?;
    Ok(())
}

async fn repeat_with_child<S: InventoryStore>(store: &S) -> TestResult {
    let lease = claim(store, "cycle", 204).await?;
    let observations = vec![observation("cycle-resource", "persisted")?];
    let mut repeated = page(lease, observations, false, Some(vec![b'B']), 205);
    let child = child_work("cycle", &resource("parent")?)?;
    repeated.children.push(child);
    let result = store.commit_page(repeated).await?;
    assert!(result.repeated_token);
    assert_eq!(result.next_generation, None);
    Ok(())
}

async fn given_token_cycle_when_committed_then_page_and_children_persist<S: InventoryStore>(
    store: &S,
) -> TestResult {
    observe(store, "baseline", "unseen", "baseline").await?;
    store.seed_works(&[work("cycle", "us-east-1")?]).await?;
    cycle_page(store, b'B', 200).await?;
    cycle_page(store, b'A', 202).await?;
    repeat_with_child(store).await?;
    let summary = summary(store, "cycle").await?;
    assert_eq!(summary.partial_scopes, 1);
    assert_eq!(summary.work_pending, 1);
    assert!(has(store, "cycle-resource").await?);
    assert!(has(store, "unseen").await?);
    let child = claim(store, "cycle", 206).await?;
    assert_eq!(child.scope().parent(), Some(&resource("parent")?));
    Ok(())
}
