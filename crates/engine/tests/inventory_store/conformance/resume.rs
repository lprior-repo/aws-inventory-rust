use super::*;

pub(super) async fn run<S, F, G>(make: &F) -> TestResult
where
    S: InventoryStore,
    F: AsyncFn() -> TestResult<(G, S, Vec<u8>)>,
{
    let (_guard, store, _) = make().await?;
    given_wrong_account_when_resumed_then_queue_is_unchanged(&store).await?;
    let (_guard, store, _) = make().await?;
    given_missing_manifest_when_resumed_then_account_mismatch(&store).await?;
    let (_guard, store, _) = make().await?;
    given_live_lease_when_resumed_then_rejected(&store).await?;
    let (_guard, store, _) = make().await?;
    given_expired_lease_when_resumed_then_old_lease_is_fenced(&store).await
}

fn manifest(sweep: &str) -> TestResult<(SweepId, AccountId)> {
    let sweep = SweepId::new(sweep.to_owned())?;
    let account = AccountId::parse("111111111111")?;
    Ok((sweep, account))
}

async fn given_wrong_account_when_resumed_then_queue_is_unchanged<S: InventoryStore>(
    store: &S,
) -> TestResult {
    store.seed_works(&[work("work-1", "us-east-1")?]).await?;
    let (sweep, account_a) = manifest("work-1")?;
    let account_b = AccountId::parse("222222222222")?;
    match store.resume_sweep(&sweep, &account_b).await {
        Err(StoreError::AccountMismatch(expected, manifest)) => {
            assert_eq!(expected, account_b);
            assert_eq!(manifest, Some(account_a));
        }
        other => return Err(format!("expected AccountMismatch, got: {other:?}").into()),
    }
    assert!(store.claim(&sweep, 100, 20).await?.is_some());
    Ok(())
}

async fn given_missing_manifest_when_resumed_then_account_mismatch<S: InventoryStore>(
    store: &S,
) -> TestResult {
    let queued = work("no-manifest", "us-east-1")?;
    store.seed_works(&[queued]).await?;
    let sweep = SweepId::new("no-manifest".to_owned())?;
    let account = AccountId::parse("111111111111")?;
    match store.resume_sweep(&sweep, &account).await {
        Err(StoreError::AccountMismatch(expected, manifest)) => {
            assert_eq!(expected, account);
            assert_eq!(manifest, None);
        }
        other => return Err(format!("expected AccountMismatch, got: {other:?}").into()),
    }
    Ok(())
}

async fn given_live_lease_when_resumed_then_rejected<S: InventoryStore>(store: &S) -> TestResult {
    let queued = work("live-lease", "us-east-1")?;
    store.seed_works(&[queued]).await?;
    let (sweep, account) = manifest("live-lease")?;
    let now = i64::try_from(std::time::UNIX_EPOCH.elapsed()?.as_millis())?;
    assert!(store.claim(&sweep, now, 60000).await?.is_some());
    assert!(matches!(
        store.resume_sweep(&sweep, &account).await,
        Err(StoreError::LiveLease)
    ));
    Ok(())
}

async fn given_expired_lease_when_resumed_then_old_lease_is_fenced<S: InventoryStore>(
    store: &S,
) -> TestResult {
    let old = seed_claim(store, "takeover", 100).await?;
    let (sweep, account) = manifest("takeover")?;
    store.resume_sweep(&sweep, &account).await?;
    let current = claim(store, "takeover", 101).await?;
    let commit = page(old, Vec::new(), true, None, 102);
    assert!(matches!(
        store.commit_page(commit).await,
        Err(StoreError::StaleLease)
    ));
    let commit = page(current, Vec::new(), true, None, 103);
    store.commit_page(commit).await?;
    assert_eq!(store.sweep_summary(&sweep).await?.complete_scopes, 1);
    Ok(())
}
