use super::*;

pub(super) async fn run<S, F, G>(make: &F) -> TestResult
where
    S: InventoryStore,
    F: AsyncFn() -> TestResult<(G, S, Vec<u8>)>,
    G: OutcomeDetailStorageCapability,
{
    let (_fixture, store, _) = make().await?;
    given_failed_scope_when_finished_with_reason_then_reason_is_stored(&store, &_fixture).await?;
    let (_fixture, store, _) = make().await?;
    given_overlong_reason_when_finished_then_bound_is_enforced(&store, &_fixture).await?;
    let (_fixture, store, _) = make().await?;
    given_prior_reason_when_scope_finishes_without_detail_then_reason_is_cleared(&store, &_fixture)
        .await?;
    let (_fixture, store, _) = make().await?;
    given_prior_reason_when_last_page_commits_then_reason_is_cleared(&store, &_fixture).await
}

async fn lease<S: InventoryStore>(store: &S, sweep: &str) -> TestResult<WorkLease> {
    store.seed_works(&[work(sweep, "us-east-1")?]).await?;
    claim(store, sweep, 100).await
}

async fn given_failed_scope_when_finished_with_reason_then_reason_is_stored<
    S: InventoryStore,
    G: OutcomeDetailStorageCapability,
>(
    store: &S,
    fixture: &G,
) -> TestResult {
    let lease = lease(store, "detail-stored").await?;
    store
        .finish_scope(
            lease,
            ScopeOutcome::Failed,
            101,
            Some("decode: ListThingsOutput: unexpected member"),
        )
        .await?;
    assert_eq!(
        fixture.stored_outcomes("detail-stored").await?,
        [(
            "failed".to_owned(),
            Some("decode: ListThingsOutput: unexpected member".to_owned())
        )]
    );
    Ok(())
}

async fn given_overlong_reason_when_finished_then_bound_is_enforced<
    S: InventoryStore,
    G: OutcomeDetailStorageCapability,
>(
    store: &S,
    fixture: &G,
) -> TestResult {
    let lease = lease(store, "detail-bound").await?;
    let longest = "r".repeat(inventory_engine::inventory::MAX_OUTCOME_DETAIL_BYTES);
    let over = format!("{longest}r");
    assert!(matches!(
        store
            .finish_scope(lease.clone(), ScopeOutcome::Failed, 101, Some(&over))
            .await,
        Err(StoreError::Invalid(_))
    ));
    assert!(fixture.stored_outcomes("detail-bound").await?.is_empty());
    store
        .finish_scope(lease, ScopeOutcome::Failed, 102, Some(&longest))
        .await?;
    assert_eq!(
        fixture.stored_outcomes("detail-bound").await?,
        [("failed".to_owned(), Some(longest))]
    );
    Ok(())
}

async fn given_prior_reason_when_scope_finishes_without_detail_then_reason_is_cleared<
    S: InventoryStore,
    G: OutcomeDetailStorageCapability,
>(
    store: &S,
    fixture: &G,
) -> TestResult {
    let lease = lease(store, "detail-cleared").await?;
    fixture.seed_prior_outcome_detail("detail-cleared").await?;
    store
        .finish_scope(lease, ScopeOutcome::Partial, 101, None)
        .await?;
    assert_eq!(
        fixture.stored_outcomes("detail-cleared").await?,
        [("partial".to_owned(), None)]
    );
    Ok(())
}

async fn given_prior_reason_when_last_page_commits_then_reason_is_cleared<
    S: InventoryStore,
    G: OutcomeDetailStorageCapability,
>(
    store: &S,
    fixture: &G,
) -> TestResult {
    let lease = lease(store, "detail-page").await?;
    fixture.seed_prior_outcome_detail("detail-page").await?;
    store
        .commit_page(page(lease, Vec::new(), true, None, 101))
        .await?;
    assert_eq!(
        fixture.stored_outcomes("detail-page").await?,
        [("complete".to_owned(), None)]
    );
    Ok(())
}
