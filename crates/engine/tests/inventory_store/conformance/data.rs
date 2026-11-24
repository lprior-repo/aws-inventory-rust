use super::*;

pub(super) async fn run<S, F, G>(make: &F) -> TestResult
where
    S: InventoryStore,
    F: AsyncFn() -> TestResult<(G, S, Vec<u8>)>,
{
    let (_guard, store, catalog) = make().await?;
    given_catalog_when_validated_then_only_matching_bytes_pass(&store, &catalog).await?;
    let (_guard, store, _) = make().await?;
    given_final_page_when_committed_then_completion_is_idempotent(&store).await?;
    let (_guard, store, _) = make().await?;
    given_partial_when_followed_by_two_misses_then_only_second_deletes(&store).await?;
    Ok(())
}

async fn given_catalog_when_validated_then_only_matching_bytes_pass<S: InventoryStore>(
    store: &S,
    catalog: &[u8],
) -> TestResult {
    store.validate_catalog(catalog).await?;
    assert!(matches!(
        store.validate_catalog(b"different catalog").await,
        Err(StoreError::CatalogMismatch)
    ));
    let schemas = store.registry().schemas();
    assert_eq!(schemas.len(), 1);
    assert_eq!(
        schemas.first().map(|s| s.kind().as_str()),
        Some("AWS::Example::Thing")
    );
    Ok(())
}

async fn given_final_page_when_committed_then_completion_is_idempotent<S: InventoryStore>(
    store: &S,
) -> TestResult {
    let lease = seed_claim(store, "finalize", 100).await?;
    let replay = lease.clone();
    let observations = vec![observation("thing-1", "alpha")?];
    let commit = page(lease, observations, true, None, 101);
    let result = store.commit_page(commit).await?;
    assert_eq!(result.stats.resources_inserted, 1);
    let dummy = SweepId::new("dummy".to_owned())?;
    let account = AccountId::parse("111111111111")?;
    store.resume_sweep(&dummy, &account).await?;
    let observed = store.projection_resource(&resource("thing-1")?).await?;
    assert_eq!(observed.map(|item| item.name), Some(None));
    let commit = page(replay, Vec::new(), true, None, 102);
    let repeated = store.commit_page(commit).await;
    assert!(matches!(repeated, Err(StoreError::StaleLease)));
    let summary = summary(store, "finalize").await?;
    assert_eq!(summary.complete_scopes, 1);
    assert_eq!(summary.resources_observed, 1);
    assert_eq!(summary.work_pending, 0);
    Ok(())
}

async fn complete_empty<S: InventoryStore>(store: &S, sweep: &str) -> TestResult<u64> {
    let lease = seed_claim(store, sweep, 300).await?;
    let commit = page(lease, Vec::new(), true, None, 301);
    Ok(store.commit_page(commit).await?.stats.resources_deleted)
}

async fn given_partial_when_followed_by_two_misses_then_only_second_deletes<S: InventoryStore>(
    store: &S,
) -> TestResult {
    observe(store, "seen", "thing-1", "alpha").await?;
    let partial = seed_claim(store, "partial", 200).await?;
    store
        .finish_scope(partial, ScopeOutcome::Inaccessible, 201, None)
        .await?;
    assert!(has(store, "thing-1").await?);
    assert_eq!(summary(store, "partial").await?.inaccessible_scopes, 1);
    assert_eq!(complete_empty(store, "complete-one").await?, 0);
    assert!(has(store, "thing-1").await?);
    assert_eq!(complete_empty(store, "complete-two").await?, 1);
    assert!(!has(store, "thing-1").await?);
    Ok(())
}
