use super::*;

pub(super) async fn run<S, F, G>(make: &F) -> TestResult
where
    S: InventoryStore,
    F: AsyncFn() -> TestResult<(G, S, Vec<u8>)>,
{
    let (_guard, store, _) = make().await?;
    given_manifest_when_recreated_then_original_account_is_preserved(&store).await?;
    let (_guard, store, _) = make().await?;
    given_owned_continuation_when_released_then_resume_preserves_data_and_token(&store).await?;
    let (_guard, store, _) = make().await?;
    given_stale_owner_when_released_then_replacement_stays_leased(&store).await?;
    let (_guard, store, _) = make().await?;
    given_listener_claim_when_returning_then_depth_is_transactional(&store).await?;
    let (_guard, store, _) = make().await?;
    given_completed_scope_when_released_then_it_stays_complete(&store).await
}

async fn given_manifest_when_recreated_then_original_account_is_preserved<S: InventoryStore>(
    store: &S,
) -> TestResult {
    assert_eq!(store.discover_active_sweep().await?, None);
    let sweep = SweepId::new("manifest-control".to_owned())?;
    let account = AccountId::parse("123456789012")?;
    let other = AccountId::parse("111111111111")?;
    store.create_sweep_manifest(&sweep, &account).await?;
    store.create_sweep_manifest(&sweep, &other).await?;
    store
        .seed_works(&[work(sweep.as_str(), "us-east-1")?])
        .await?;
    assert_eq!(store.discover_active_sweep().await?, Some(sweep.clone()));
    assert!(matches!(store.resume_sweep(&sweep, &other).await,
        Err(StoreError::AccountMismatch(expected, Some(found)))
            if expected == other && found == account));
    store.resume_sweep(&sweep, &account).await?;
    Ok(())
}

async fn continuation<S: InventoryStore>(store: &S) -> TestResult<(SweepId, WorkLease, i64)> {
    let sweep = SweepId::new("released-continuation".to_owned())?;
    let account = AccountId::parse("123456789012")?;
    store.create_sweep_manifest(&sweep, &account).await?;
    let now = i64::try_from(std::time::UNIX_EPOCH.elapsed()?.as_millis())?;
    let first = seed_claim(store, sweep.as_str(), now).await?;
    let observations = vec![observation("released-resource", "preserved")?];
    let commit = page(first, observations, false, Some(vec![9, 8, 7]), now);
    store.commit_page(commit).await?;
    let lease = store
        .claim(&sweep, now, 60000)
        .await?
        .ok_or("missing continuation")?;
    Ok((sweep, lease, now))
}

async fn given_owned_continuation_when_released_then_resume_preserves_data_and_token<
    S: InventoryStore,
>(
    store: &S,
) -> TestResult {
    let (sweep, lease, now) = continuation(store).await?;
    release_preserving_state(store, &sweep, lease.clone()).await?;
    store
        .resume_sweep(&sweep, &AccountId::parse("123456789012")?)
        .await?;
    let current = claim(store, sweep.as_str(), now).await?;
    assert_eq!(current.token(), Some([9, 8, 7].as_slice()));
    let stale = page(lease, Vec::new(), true, None, now);
    assert!(matches!(
        store.commit_page(stale).await,
        Err(StoreError::StaleLease)
    ));
    assert!(has(store, "released-resource").await?);
    store
        .commit_page(page(current, Vec::new(), true, None, now))
        .await?;
    assert_eq!(store.discover_active_sweep().await?, None);
    assert_eq!(store.sweep_summary(&sweep).await?.complete_scopes, 1);
    Ok(())
}

async fn given_stale_owner_when_released_then_replacement_stays_leased<S: InventoryStore>(
    store: &S,
) -> TestResult {
    let stale = seed_claim(store, "stale-release", 100).await?;
    let current = claim(store, "stale-release", 120).await?;
    assert!(matches!(
        store.release_lease(stale).await,
        Err(StoreError::StaleLease)
    ));
    let sweep = SweepId::new("stale-release".to_owned())?;
    assert!(store.claim(&sweep, 121, 20).await?.is_none());
    store
        .commit_page(page(current, Vec::new(), true, None, 121))
        .await?;
    assert_eq!(store.sweep_summary(&sweep).await?.complete_scopes, 1);
    Ok(())
}

async fn given_completed_scope_when_released_then_it_stays_complete<S: InventoryStore>(
    store: &S,
) -> TestResult {
    let lease = seed_claim(store, "completed-release", 100).await?;
    store
        .commit_page(page(lease.clone(), Vec::new(), true, None, 101))
        .await?;
    assert!(matches!(
        store.release_lease(lease).await,
        Err(StoreError::StaleLease)
    ));
    let sweep = SweepId::new("completed-release".to_owned())?;
    assert!(store.claim(&sweep, 102, 20).await?.is_none());
    assert_eq!(store.sweep_summary(&sweep).await?.complete_scopes, 1);
    Ok(())
}

async fn release_preserving_state<S: InventoryStore>(
    store: &S,
    sweep: &SweepId,
    lease: WorkLease,
) -> TestResult {
    let before = store.sweep_summary(sweep).await?;
    let outbox = store.projection_state().await?;
    store.release_lease(lease).await?;
    assert_eq!(store.sweep_summary(sweep).await?, before);
    assert_eq!(store.projection_state().await?, outbox);
    Ok(())
}

async fn given_listener_claim_when_returning_then_depth_is_transactional<S: InventoryStore>(
    store: &S,
) -> TestResult {
    let sweep = SweepId::new("depth-control".to_owned())?;
    let works = [
        work(sweep.as_str(), "us-east-1")?,
        work(sweep.as_str(), "us-west-2")?,
    ];
    store.seed_works(&works).await?;
    verify_plain_claim(store, &sweep).await?;
    let first = claim_with_depth(store, &sweep, 100).await?;
    let second = claim_with_depth(store, &sweep, 100).await?;
    assert_eq!(first.pending_work(), Some(2));
    assert_eq!(second.pending_work(), Some(2));
    let commit = page(first, Vec::new(), true, None, 101);
    store.commit_page(commit).await?;
    store.release_lease(second).await?;
    let released = claim_with_depth(store, &sweep, 102).await?;
    assert_eq!(released.pending_work(), Some(1));
    let commit = page(released, Vec::new(), true, None, 103);
    store.commit_page(commit).await?;
    assert_eq!(store.discover_active_sweep().await?, None);
    Ok(())
}

async fn verify_plain_claim<S: InventoryStore>(store: &S, sweep: &SweepId) -> TestResult {
    let lease = claim(store, sweep.as_str(), 100).await?;
    assert_eq!(lease.pending_work(), None);
    store.release_lease(lease).await?;
    Ok(())
}

async fn claim_with_depth<S: InventoryStore>(
    store: &S,
    sweep: &SweepId,
    now: i64,
) -> TestResult<WorkLease> {
    Ok(store
        .claim_with_depth(sweep, now, 1_000)
        .await?
        .ok_or("depth claim absent")?)
}
