//! A page committed with `commit_page_keeping` keeps its lease for the next page.

use super::*;
use inventory_engine::inventory::{InventoryStore, MAX_OUTCOME_DETAIL_BYTES, WorkLease};

pub(super) async fn run<S, F, G>(make: &F) -> TestResult
where
    S: InventoryStore,
    F: AsyncFn() -> TestResult<(G, S, Vec<u8>)>,
{
    let (_guard, store, _) = make().await?;
    given_next_page_when_kept_then_lease_continues_to_completion(&store).await?;
    let (_guard, store, _) = make().await?;
    given_kept_lease_when_it_expires_then_reclaim_restarts_from_the_token(&store).await?;
    let (_guard, store, _) = make().await?;
    given_kept_lease_when_resumed_then_reclaim_restarts_from_the_token(&store).await?;
    let (_guard, store, _) = make().await?;
    given_repeated_token_when_kept_then_no_lease_and_scope_is_partial(&store).await?;
    let (_guard, store, _) = make().await?;
    given_zero_interval_when_kept_then_nothing_is_committed(&store).await?;
    let (_guard, store, _) = make().await?;
    given_kept_deadline_equal_to_the_old_one_when_kept_then_the_old_copy_is_stale(&store).await?;
    let (_guard, store, _) = make().await?;
    given_unrepresentable_deadline_when_the_page_is_final_then_the_scope_completes(&store).await?;
    let (_guard, store, _) = make().await?;
    given_unrepresentable_deadline_when_the_token_repeats_then_the_scope_is_partial(&store).await?;
    let (_guard, store, _) = make().await?;
    given_unrepresentable_deadline_when_the_page_continues_then_nothing_is_committed(&store)
        .await?;
    let (_guard, store, _) = make().await?;
    given_a_reason_over_the_bound_when_finished_then_it_is_refused_and_the_bound_is_accepted(&store)
        .await
}

/// Claims at 100 (deadline 120), then keeps one page at 110 for 50 (deadline 160).
async fn keep_one<S: InventoryStore>(
    store: &S,
    sweep: &str,
    token: u8,
) -> TestResult<(WorkLease, WorkLease)> {
    let first = seed_claim(store, sweep, 100).await?;
    let old = first.clone();
    let commit = page(first, Vec::new(), false, Some(vec![token]), 110);
    let (result, kept) = store.commit_page_keeping(commit, 50).await?;
    assert!(!result.repeated_token);
    Ok((old, kept.ok_or("the lease was not kept")?))
}

async fn given_next_page_when_kept_then_lease_continues_to_completion<S: InventoryStore>(
    store: &S,
) -> TestResult {
    let (old, kept) = keep_one(store, "kept", 1).await?;
    assert_eq!(kept.token(), Some([1].as_slice()));
    assert_eq!(kept.generation(), old.generation());
    let sweep = SweepId::new("kept".to_owned())?;
    assert!(store.claim(&sweep, 159, 20).await?.is_none());
    let stale = store
        .commit_page(page(old, Vec::new(), true, None, 111))
        .await;
    assert!(matches!(stale, Err(StoreError::StaleLease)));
    let last = page(
        kept,
        vec![observation("kept-last", "two")?],
        true,
        None,
        150,
    );
    let (result, none) = store.commit_page_keeping(last, 50).await?;
    assert!(!result.repeated_token);
    assert!(none.is_none());
    let summary = summary(store, "kept").await?;
    assert_eq!(summary.complete_scopes, 1);
    assert_eq!(summary.work_pending, 0);
    assert!(has(store, "kept-last").await?);
    Ok(())
}

async fn given_kept_lease_when_it_expires_then_reclaim_restarts_from_the_token<
    S: InventoryStore,
>(
    store: &S,
) -> TestResult {
    let (_, kept) = keep_one(store, "kept-expiry", 3).await?;
    let current = claim(store, "kept-expiry", 160).await?;
    assert_eq!(current.token(), Some([3].as_slice()));
    assert!(current.generation() > kept.generation());
    let stale = store
        .commit_page(page(kept, Vec::new(), true, None, 161))
        .await;
    assert!(matches!(stale, Err(StoreError::StaleLease)));
    store
        .commit_page(page(current, Vec::new(), true, None, 162))
        .await?;
    assert_eq!(summary(store, "kept-expiry").await?.complete_scopes, 1);
    Ok(())
}

async fn given_kept_lease_when_resumed_then_reclaim_restarts_from_the_token<S: InventoryStore>(
    store: &S,
) -> TestResult {
    let (_, kept) = keep_one(store, "takeover", 5).await?;
    let sweep = SweepId::new("takeover".to_owned())?;
    let account = AccountId::parse("111111111111")?;
    store.resume_sweep(&sweep, &account).await?;
    let current = claim(store, "takeover", 111).await?;
    assert_eq!(current.token(), Some([5].as_slice()));
    assert!(current.generation() > kept.generation());
    let stale = store
        .commit_page_keeping(page(kept, Vec::new(), true, None, 112), 50)
        .await;
    assert!(matches!(stale, Err(StoreError::StaleLease)));
    Ok(())
}

async fn given_repeated_token_when_kept_then_no_lease_and_scope_is_partial<S: InventoryStore>(
    store: &S,
) -> TestResult {
    let (_, kept) = keep_one(store, "kept-cycle", 7).await?;
    let again = page(kept, Vec::new(), false, Some(vec![7]), 120);
    let (result, none) = store.commit_page_keeping(again, 50).await?;
    assert!(result.repeated_token);
    assert!(none.is_none());
    let summary = summary(store, "kept-cycle").await?;
    assert_eq!(summary.partial_scopes, 1);
    assert_eq!(summary.work_pending, 0);
    Ok(())
}

async fn given_zero_interval_when_kept_then_nothing_is_committed<S: InventoryStore>(
    store: &S,
) -> TestResult {
    let lease = seed_claim(store, "kept-zero", 100).await?;
    let unused = lease.clone();
    let commit = page(
        lease,
        vec![observation("kept-zero", "one")?],
        false,
        Some(vec![9]),
        110,
    );
    let rejected = store.commit_page_keeping(commit, 0).await;
    assert!(matches!(rejected, Err(StoreError::Invalid(_))));
    assert!(!has(store, "kept-zero").await?);
    store
        .commit_page(page(unused, Vec::new(), true, None, 111))
        .await?;
    assert_eq!(summary(store, "kept-zero").await?.complete_scopes, 1);
    Ok(())
}

/// Claims at 100 (deadline 120) and keeps one page at 110 for 10, which would
/// also end at 120: the kept lease must not share the old deadline.
async fn given_kept_deadline_equal_to_the_old_one_when_kept_then_the_old_copy_is_stale<
    S: InventoryStore,
>(
    store: &S,
) -> TestResult {
    let first = seed_claim(store, "kept-equal", 100).await?;
    let old = first.clone();
    let commit = page(first, Vec::new(), false, Some(vec![11]), 110);
    let (_, kept) = store.commit_page_keeping(commit, 10).await?;
    let kept = kept.ok_or("the lease was not kept")?;
    let stale = store
        .commit_page(page(old, Vec::new(), true, None, 111))
        .await;
    assert!(matches!(stale, Err(StoreError::StaleLease)));
    store
        .commit_page(page(kept, Vec::new(), true, None, 120))
        .await?;
    assert_eq!(summary(store, "kept-equal").await?.complete_scopes, 1);
    Ok(())
}

/// A page committed at 110 with an interval of `i64::MAX` has no
/// representable deadline. A final page keeps no lease, so it does not need one.
async fn given_unrepresentable_deadline_when_the_page_is_final_then_the_scope_completes<
    S: InventoryStore,
>(
    store: &S,
) -> TestResult {
    let lease = seed_claim(store, "kept-final", 100).await?;
    let last = page(
        lease,
        vec![observation("kept-final", "one")?],
        true,
        None,
        110,
    );
    let (result, none) = store.commit_page_keeping(last, i64::MAX).await?;
    assert!(!result.repeated_token);
    assert!(none.is_none());
    assert!(has(store, "kept-final").await?);
    let summary = summary(store, "kept-final").await?;
    assert_eq!(summary.complete_scopes, 1);
    assert_eq!(summary.work_pending, 0);
    Ok(())
}

async fn given_unrepresentable_deadline_when_the_token_repeats_then_the_scope_is_partial<
    S: InventoryStore,
>(
    store: &S,
) -> TestResult {
    let (_, kept) = keep_one(store, "kept-wide-cycle", 13).await?;
    let again = page(kept, Vec::new(), false, Some(vec![13]), 120);
    let (result, none) = store.commit_page_keeping(again, i64::MAX).await?;
    assert!(result.repeated_token);
    assert!(none.is_none());
    let summary = summary(store, "kept-wide-cycle").await?;
    assert_eq!(summary.partial_scopes, 1);
    assert_eq!(summary.work_pending, 0);
    Ok(())
}

/// A page that continues its scope needs the deadline, so the commit is
/// rejected whole and the lease it came with still commits.
async fn given_unrepresentable_deadline_when_the_page_continues_then_nothing_is_committed<
    S: InventoryStore,
>(
    store: &S,
) -> TestResult {
    let lease = seed_claim(store, "kept-wide", 100).await?;
    let unused = lease.clone();
    let commit = page(
        lease,
        vec![observation("kept-wide", "one")?],
        false,
        Some(vec![15]),
        110,
    );
    let rejected = store.commit_page_keeping(commit, i64::MAX).await;
    assert!(matches!(rejected, Err(StoreError::Invalid(_))));
    assert!(!has(store, "kept-wide").await?);
    let retry = page(unused, Vec::new(), false, Some(vec![15]), 111);
    let (result, kept) = store.commit_page_keeping(retry, 50).await?;
    assert!(!result.repeated_token);
    assert_eq!(
        kept.ok_or("the lease was not kept")?.token(),
        Some([15].as_slice())
    );
    Ok(())
}

async fn given_a_reason_over_the_bound_when_finished_then_it_is_refused_and_the_bound_is_accepted<
    S: InventoryStore,
>(
    store: &S,
) -> TestResult {
    let lease = seed_claim(store, "bound", 100).await?;
    let longest = "r".repeat(MAX_OUTCOME_DETAIL_BYTES);
    let over = format!("{longest}r");

    let refused = store
        .finish_scope(lease.clone(), ScopeOutcome::Failed, 101, Some(&over))
        .await;
    assert!(matches!(refused, Err(StoreError::Invalid(_))));

    // The refused call should not have stored the outcome.
    let s = summary(store, "bound").await?;
    assert_eq!(s.complete_scopes, 0);
    assert_eq!(s.failed_scopes, 0);

    store
        .finish_scope(lease, ScopeOutcome::Failed, 102, Some(&longest))
        .await?;
    Ok(())
}
