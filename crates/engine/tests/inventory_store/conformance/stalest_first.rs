use super::*;
use inventory_engine::inventory::{InventoryStore, WorkSpec};

const AGES: [(&str, i64); 3] = [("us-east-1", 300), ("us-east-2", 100), ("eu-west-1", 200)];

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
    given_earlier_outcomes_when_claiming_then_the_stalest_scope_goes_first(&store).await?;
    let (_guard, store, _) = make().await?;
    given_no_earlier_outcomes_when_claiming_then_scope_key_order_is_kept(&store).await?;
    let (_guard, store, _) = make().await?;
    given_only_an_incomplete_outcome_when_claiming_then_the_scope_is_still_never_completed(&store)
        .await?;
    let (_guard, store, _) = make().await?;
    given_expired_leases_and_ready_work_when_claiming_then_the_stalest_of_both_goes_first(&store)
        .await?;
    let (_guard, store, _) = make().await?;
    given_scope_completed_at_time_zero_when_claiming_then_it_sorts_after_never_completed(&store)
        .await?;
    let (_guard, store, _) = make().await?;
    given_migrated_database_when_claiming_then_stalest_first_ordering_is_preserved(&store).await
}

async fn completed_first_sweep<S: InventoryStore>(store: &S) -> TestResult {
    let works: Vec<WorkSpec> = AGES
        .iter()
        .map(|(region, _)| work("first", region))
        .collect::<TestResult<Vec<_>>>()?;
    store.seed_works(&works).await?;
    let sweep = SweepId::new("first".to_owned())?;
    for _ in AGES {
        let lease = store
            .claim(&sweep, 10, 1_000)
            .await?
            .ok_or("missing lease")?;
        let observed = AGES
            .iter()
            .find(|(region, _)| *region == lease.scope().region().as_str())
            .map(|(_, at)| *at)
            .ok_or("unexpected region")?;
        store
            .finish_scope(lease, ScopeOutcome::Complete, observed, None)
            .await?;
    }
    assert!(store.claim(&sweep, 20, 1_000).await?.is_none());
    Ok(())
}

async fn claim_regions<S: InventoryStore>(
    store: &S,
    sweep: &SweepId,
    now: i64,
    lease_ms: i64,
    count: usize,
) -> TestResult<Vec<String>> {
    let mut regions = Vec::new();
    for _ in 0..count {
        let lease = store
            .claim(sweep, now, lease_ms)
            .await?
            .ok_or("missing lease")?;
        regions.push(lease.scope().region().as_str().to_owned());
    }
    Ok(regions)
}

async fn given_earlier_outcomes_when_claiming_then_the_stalest_scope_goes_first<
    S: InventoryStore,
>(
    store: &S,
) -> TestResult {
    completed_first_sweep(store).await?;

    let works: Vec<WorkSpec> = ["us-east-1", "ap-south-1", "eu-west-1", "us-east-2"]
        .iter()
        .map(|region| work("second", region))
        .collect::<TestResult<Vec<_>>>()?;
    store.seed_works(&works).await?;

    let sweep = SweepId::new("second".to_owned())?;
    assert_eq!(
        claim_regions(store, &sweep, 1_000, 1_000, 4).await?,
        ["ap-south-1", "us-east-2", "eu-west-1", "us-east-1"]
    );
    assert!(store.claim(&sweep, 1_000, 1_000).await?.is_none());
    Ok(())
}

async fn given_no_earlier_outcomes_when_claiming_then_scope_key_order_is_kept<S: InventoryStore>(
    store: &S,
) -> TestResult {
    for (region, _) in AGES {
        seed(store, work("second", region)?).await?;
    }
    let sweep = SweepId::new("second".to_owned())?;
    assert_eq!(
        claim_regions(store, &sweep, 1_000, 1_000, 3).await?,
        ["eu-west-1", "us-east-1", "us-east-2"]
    );
    Ok(())
}

async fn given_only_an_incomplete_outcome_when_claiming_then_the_scope_is_still_never_completed<
    S: InventoryStore,
>(
    store: &S,
) -> TestResult {
    completed_first_sweep(store).await?;
    seed(store, work("partial", "us-east-2")?).await?;
    let partial = SweepId::new("partial".to_owned())?;
    let lease = store
        .claim(&partial, 800, 1_000)
        .await?
        .ok_or("missing lease")?;
    store
        .finish_scope(lease, ScopeOutcome::Inaccessible, 900, None)
        .await?;

    let works: Vec<WorkSpec> = AGES
        .iter()
        .map(|(region, _)| work("third", region))
        .collect::<TestResult<Vec<_>>>()?;
    store.seed_works(&works).await?;
    let third = SweepId::new("third".to_owned())?;
    assert_eq!(
        claim_regions(store, &third, 1_000, 1_000, 3).await?,
        ["us-east-2", "eu-west-1", "us-east-1"]
    );
    Ok(())
}

async fn given_expired_leases_and_ready_work_when_claiming_then_the_stalest_of_both_goes_first<
    S: InventoryStore,
>(
    store: &S,
) -> TestResult {
    completed_first_sweep(store).await?;

    for (region, _) in AGES {
        seed(store, work("second", region)?).await?;
    }
    let sweep = SweepId::new("second".to_owned())?;
    assert_eq!(
        claim_regions(store, &sweep, 1_000, 10, 3).await?,
        ["us-east-2", "eu-west-1", "us-east-1"]
    );
    seed(store, work("second", "ap-south-1")?).await?;
    assert_eq!(
        claim_regions(store, &sweep, 2_000, 1_000, 4).await?,
        ["ap-south-1", "us-east-2", "eu-west-1", "us-east-1"]
    );
    assert!(store.claim(&sweep, 2_000, 1_000).await?.is_none());
    Ok(())
}

async fn given_scope_completed_at_time_zero_when_claiming_then_it_sorts_after_never_completed<
    S: InventoryStore,
>(
    store: &S,
) -> TestResult {
    seed(store, work("zero", "ap-south-1")?).await?;
    let sweep = SweepId::new("zero".to_owned())?;
    let lease = store
        .claim(&sweep, 0, 1_000)
        .await?
        .ok_or("missing lease")?;
    store
        .finish_scope(lease, ScopeOutcome::Complete, 0, None)
        .await?;

    seed(store, work("second", "ap-south-1")?).await?;
    seed(store, work("second", "us-east-1")?).await?;
    let second = SweepId::new("second".to_owned())?;
    assert_eq!(
        claim_regions(store, &second, 1_000, 1_000, 2).await?,
        ["us-east-1", "ap-south-1"]
    );
    Ok(())
}

async fn given_migrated_database_when_claiming_then_stalest_first_ordering_is_preserved<
    S: InventoryStore,
>(
    store: &S,
) -> TestResult {
    seed(store, work("migrate", "us-east-1")?).await?;
    seed(store, work("migrate", "us-east-2")?).await?;
    let first = SweepId::new("migrate".to_owned())?;
    {
        let lease = store
            .claim(&first, 100, 1_000)
            .await?
            .ok_or("missing lease")?;
        store
            .finish_scope(lease, ScopeOutcome::Complete, 100, None)
            .await?;
    }
    {
        let lease = store
            .claim(&first, 200, 1_000)
            .await?
            .ok_or("missing lease")?;
        store
            .finish_scope(lease, ScopeOutcome::Complete, 200, None)
            .await?;
    }

    seed(store, work("second", "us-east-1")?).await?;
    seed(store, work("second", "us-east-2")?).await?;
    seed(store, work("second", "eu-west-1")?).await?;
    let second = SweepId::new("second".to_owned())?;
    assert_eq!(
        claim_regions(store, &second, 1_000, 1_000, 3).await?,
        ["eu-west-1", "us-east-1", "us-east-2"]
    );
    Ok(())
}
