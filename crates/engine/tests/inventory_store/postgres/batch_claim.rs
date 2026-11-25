use super::{DatabaseUrl, PgPool, PgStore, TestResult, run};
use inventory_core::inventory::{AccountId, OperationId, Region, ScopeKey, ServiceId};
use inventory_engine::inventory::{StoreError, SweepId, WorkSpec};
use std::collections::{BTreeMap, BTreeSet};

fn make_work(sweep: &str, region: &str) -> TestResult<WorkSpec> {
    let account = AccountId::parse("111111111111")?;
    let scope = ScopeKey::new(
        account,
        Region::parse(region)?,
        ServiceId::parse("example")?,
        OperationId::parse("ListThings")?,
        None,
    )?;
    Ok(WorkSpec::new(
        SweepId::new(sweep.to_owned())?,
        scope,
        None,
        Vec::new(),
    )?)
}

fn scope_region(lease: &inventory_engine::inventory::WorkLease) -> String {
    lease.scope().region().as_str().to_owned()
}

async fn check_partial_and_remainder(
    store: &PgStore,
    _pool: &PgPool,
    _url: &DatabaseUrl,
) -> TestResult {
    store
        .seed_works(&[
            make_work("sweep-partial", "us-east-1")?,
            make_work("sweep-partial", "us-west-2")?,
            make_work("sweep-partial", "eu-west-1")?,
        ])
        .await?;
    let sweep = SweepId::new("sweep-partial".to_owned())?;
    let first: Vec<String> = store
        .claim_batch(&sweep, 100, 10_000, 2)
        .await?
        .iter()
        .map(scope_region)
        .collect();
    assert_eq!(first, ["eu-west-1", "us-east-1"]);
    let second: Vec<String> = store
        .claim_batch(&sweep, 100, 10_000, 2)
        .await?
        .iter()
        .map(scope_region)
        .collect();
    assert_eq!(second, ["us-west-2"]);
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_three_scopes_when_claiming_batch_of_two_then_partial_and_remainder() -> TestResult {
    run(check_partial_and_remainder).await
}

async fn check_batch_sorted(store: &PgStore, _pool: &PgPool, _url: &DatabaseUrl) -> TestResult {
    store
        .seed_works(&[
            make_work("sweep-sort", "us-west-2")?,
            make_work("sweep-sort", "us-east-1")?,
            make_work("sweep-sort", "eu-west-1")?,
        ])
        .await?;
    let sweep = SweepId::new("sweep-sort".to_owned())?;
    let leases = store.claim_batch(&sweep, 100, 10_000, 3).await?;
    assert_eq!(leases.len(), 3);
    let scopes: Vec<_> = leases.iter().map(|l| l.scope()).collect();
    assert!(scopes.windows(2).all(|pair| pair.first() < pair.get(1)));
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_three_scopes_when_claiming_batch_then_sorted_by_scope() -> TestResult {
    run(check_batch_sorted).await
}

async fn check_expired_reclaim(store: &PgStore, _pool: &PgPool, _url: &DatabaseUrl) -> TestResult {
    store
        .seed_works(&[
            make_work("sweep-expired", "us-east-1")?,
            make_work("sweep-expired", "us-west-2")?,
        ])
        .await?;
    let sweep = SweepId::new("sweep-expired".to_owned())?;
    let first = store.claim_batch(&sweep, 100, 10, 2).await?;
    assert_eq!(first.len(), 2);
    let first_gens: BTreeMap<String, i64> = first
        .iter()
        .map(|l| (scope_region(l), l.generation()))
        .collect();
    let second = store.claim_batch(&sweep, 120, 10, 2).await?;
    assert_eq!(second.len(), 2);
    for lease in second {
        let region = scope_region(&lease);
        let first_gen = first_gens
            .get(&region)
            .ok_or("missing first generation for region")?;
        assert!(lease.generation() > *first_gen);
    }
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_expired_lease_when_claiming_batch_then_higher_generation() -> TestResult {
    run(check_expired_reclaim).await
}

async fn check_limit_zero(store: &PgStore, _pool: &PgPool, _url: &DatabaseUrl) -> TestResult {
    store
        .seed_works(&[make_work("sweep-zero", "us-east-1")?])
        .await?;
    let sweep = SweepId::new("sweep-zero".to_owned())?;
    let result = store.claim_batch(&sweep, 100, 10_000, 0).await;
    assert!(matches!(result, Err(StoreError::Invalid(_))));
    let leases = store.claim_batch(&sweep, 100, 10_000, 1).await?;
    assert_eq!(leases.len(), 1);
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_limit_zero_when_claiming_batch_then_refused_and_work_remains() -> TestResult {
    run(check_limit_zero).await
}

async fn check_all_leased(store: &PgStore, _pool: &PgPool, _url: &DatabaseUrl) -> TestResult {
    store
        .seed_works(&[make_work("sweep-empty", "us-east-1")?])
        .await?;
    let sweep = SweepId::new("sweep-empty".to_owned())?;
    let first = store.claim_batch(&sweep, 100, 10_000, 1).await?;
    assert_eq!(first.len(), 1);
    let second = store.claim_batch(&sweep, 100, 10_000, 1).await?;
    assert!(second.is_empty());
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_all_scopes_leased_when_claiming_batch_then_empty() -> TestResult {
    run(check_all_leased).await
}

async fn check_generation_overflow(
    store: &PgStore,
    pool: &PgPool,
    _url: &DatabaseUrl,
) -> TestResult {
    store
        .seed_works(&[
            make_work("sweep-overflow", "us-east-1")?,
            make_work("sweep-overflow", "us-west-2")?,
        ])
        .await?;
    sqlx::query(
        "UPDATE inventory.work_queue SET generation=9223372036854775807 WHERE sweep_id='sweep-overflow'",
    )
    .execute(pool)
    .await?;
    let sweep = SweepId::new("sweep-overflow".to_owned())?;
    let result = store.claim_batch(&sweep, 100, 10_000, 2).await;
    assert!(matches!(result, Err(StoreError::LeaseGenerationOverflow)));
    let count_row: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM inventory.work_queue WHERE sweep_id='sweep-overflow' AND state=1",
    )
    .fetch_one(pool)
    .await?;
    assert_eq!(count_row, 0);
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_generation_overflow_when_claiming_batch_then_rollback() -> TestResult {
    run(check_generation_overflow).await
}

async fn check_concurrent_claimers(
    store: &PgStore,
    _pool: &PgPool,
    url: &DatabaseUrl,
) -> TestResult {
    store
        .seed_works(&[
            make_work("sweep-concurrent", "us-east-1")?,
            make_work("sweep-concurrent", "us-west-2")?,
            make_work("sweep-concurrent", "eu-west-1")?,
            make_work("sweep-concurrent", "ap-southeast-1")?,
        ])
        .await?;
    let sweep = SweepId::new("sweep-concurrent".to_owned())?;
    let store_a = store;
    let store_b = PgStore::open(url, super::PgWriteConfig::default()).await?;
    let (result_a, result_b) = tokio::join!(
        store_a.claim_batch(&sweep, 100, 10_000, 3),
        store_b.claim_batch(&sweep, 100, 10_000, 3)
    );
    store_b.close().await;
    let leases_a = result_a?;
    let leases_b = result_b?;
    let regions_a: BTreeSet<String> = leases_a.iter().map(scope_region).collect();
    let regions_b: BTreeSet<String> = leases_b.iter().map(scope_region).collect();
    let mut all_regions = regions_a.clone();
    all_regions.extend(regions_b.clone());
    assert_eq!(all_regions.len(), 4);
    let intersection: BTreeSet<String> = regions_a.intersection(&regions_b).cloned().collect();
    assert!(intersection.is_empty());
    assert_eq!(all_regions.len(), 4);
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_concurrent_claimers_when_claiming_batch_then_no_overlap() -> TestResult {
    run(check_concurrent_claimers).await
}
