mod postgres_support;
#[path = "postgres_work/success.rs"]
mod success;

use inventory_core::inventory::{
    AccountId, OperationId, Region, ScopeKey, ScopeOutcome, ServiceId,
};
use inventory_engine::inventory::{
    DatabaseUrl, PgStore, PgWriteConfig, StoreError, SweepId, WorkLease, WorkSpec,
};
use inventory_harness::postgres::{TestDatabase, admin_url};
use postgres_support::{TestResult, registry};
use sqlx::PgPool;

async fn run(check: impl AsyncFn(&PgStore, &PgPool, &DatabaseUrl) -> TestResult) -> TestResult {
    let database = TestDatabase::create(&admin_url()?).await?;
    let url = DatabaseUrl::parse(database.url())?;
    let pool = PgPool::connect(database.url()).await?;
    let (registry, _) = registry(1, None)?;
    let store = PgStore::create(&url, registry, PgWriteConfig::default()).await?;
    let result = check(&store, &pool, &url).await;
    store.close().await;
    pool.close().await;
    database.finish().await?;
    result
}

fn work(sweep: &str, region: &str) -> TestResult<WorkSpec> {
    let scope = ScopeKey::new(
        AccountId::parse("123456789012")?,
        Region::parse(region)?,
        ServiceId::parse("example")?,
        OperationId::parse("ListThings0")?,
        None,
    )?;
    Ok(WorkSpec::new(
        SweepId::new(sweep.to_owned())?,
        scope,
        None,
        Vec::new(),
    )?)
}

async fn claim(store: &PgStore, sweep: &str, at: i64) -> TestResult<WorkLease> {
    Ok(store
        .claim(&SweepId::new(sweep.to_owned())?, at, 20)
        .await?
        .ok_or("missing lease")?)
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_work_given_two_regions_when_claimed_then_fenced() -> TestResult {
    run(identity).await
}

async fn identity(store: &PgStore, _: &PgPool, _: &DatabaseUrl) -> TestResult {
    store
        .seed_works(&[work("s", "us-east-1")?, work("s", "us-west-2")?])
        .await?;
    let first = claim(store, "s", 100).await?;
    let second = claim(store, "s", 100).await?;
    assert_eq!(first.scope().region().as_str(), "us-east-1");
    assert_eq!(second.scope().region().as_str(), "us-west-2");
    assert!(store.claim(first.sweep(), 119, 20).await?.is_none());
    let reclaimed = claim(store, "s", 120).await?;
    assert!(matches!(
        store
            .finish_scope(first, ScopeOutcome::Partial, 121, None)
            .await,
        Err(StoreError::StaleLease)
    ));
    store
        .finish_scope(reclaimed, ScopeOutcome::Partial, 121, None)
        .await?;
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_work_given_two_connections_when_claimed_then_scopes_are_distinct() -> TestResult {
    run(concurrent_claim).await
}

async fn concurrent_claim(store: &PgStore, _: &PgPool, url: &DatabaseUrl) -> TestResult {
    store
        .seed_works(&[work("s", "us-east-1")?, work("s", "us-west-2")?])
        .await?;
    let other = PgStore::open(url, PgWriteConfig::default()).await?;
    let (left, right) = tokio::join!(claim(store, "s", 100), claim(&other, "s", 100));
    assert_ne!(left?.scope(), right?.scope());
    other.close().await;
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_work_given_two_sweeps_when_seeded_then_one_identity_wins() -> TestResult {
    run(concurrent_seed).await
}

async fn concurrent_seed(store: &PgStore, _: &PgPool, url: &DatabaseUrl) -> TestResult {
    let other = PgStore::open(url, PgWriteConfig::default()).await?;
    let a = [work("a", "us-east-1")?];
    let b = [work("b", "us-east-1")?];
    let (a, b) = tokio::join!(store.seed_works(&a), other.seed_works(&b));
    assert!(matches!(
        (a, b),
        (Ok(()), Err(StoreError::ActiveSweepConflict))
            | (Err(StoreError::ActiveSweepConflict), Ok(()))
    ));
    other.close().await;
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_work_given_manifests_when_resumed_then_checked() -> TestResult {
    run(resume).await
}

async fn resume(store: &PgStore, _: &PgPool, _: &DatabaseUrl) -> TestResult {
    let account = AccountId::parse("111111111111")?;
    let sweep = SweepId::new("s".to_owned())?;
    store.seed_works(&[work("s", "us-east-1")?]).await?;
    check_manifest(store, &sweep, &account).await?;
    check_expired(store, &sweep, &account).await?;
    let now = i64::try_from(std::time::UNIX_EPOCH.elapsed()?.as_millis())?;
    let lease = claim(store, "s", now).await?;
    assert!(matches!(
        store.resume_sweep(&sweep, &account).await,
        Err(StoreError::LiveLease)
    ));
    let at = now.checked_add(1).ok_or("overflow")?;
    store
        .finish_scope(lease, ScopeOutcome::Complete, at, None)
        .await?;
    Ok(())
}

async fn check_manifest(store: &PgStore, sweep: &SweepId, account: &AccountId) -> TestResult {
    assert!(matches!(
        store.resume_sweep(sweep, account).await,
        Err(StoreError::AccountMismatch(_, None))
    ));
    store.create_sweep_manifest(sweep, account.as_str()).await?;
    let wrong = AccountId::parse("222222222222")?;
    assert!(matches!(
        store.resume_sweep(sweep, &wrong).await,
        Err(StoreError::AccountMismatch(_, Some(_)))
    ));
    Ok(())
}

async fn check_expired(store: &PgStore, sweep: &SweepId, account: &AccountId) -> TestResult {
    let old = claim(store, "s", 100).await?;
    store.resume_sweep(sweep, account).await?;
    assert!(matches!(
        store
            .finish_scope(old, ScopeOutcome::Partial, 101, None)
            .await,
        Err(StoreError::StaleLease)
    ));
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_work_given_max_generation_when_claimed_then_overflow_is_atomic() -> TestResult {
    run(overflow).await
}

async fn overflow(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    store.seed_works(&[work("s", "us-east-1")?]).await?;
    sqlx::query("UPDATE inventory.work_queue SET generation=9223372036854775807")
        .execute(pool)
        .await?;
    let sweep = SweepId::new("s".to_owned())?;
    assert!(matches!(
        store.claim(&sweep, 100, 20).await,
        Err(StoreError::LeaseGenerationOverflow)
    ));
    let state: i64 = sqlx::query_scalar("SELECT state FROM inventory.work_queue")
        .fetch_one(pool)
        .await?;
    assert_eq!(state, 0);
    assert!(store.claim(&sweep, i64::MAX, 1).await.is_err());
    assert!(store.claim(&sweep, -1, 1).await.is_err());
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_work_given_5001_scopes_when_seeded_then_all_batches_persist() -> TestResult {
    run(batches).await
}

async fn batches(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    let works = (0..5_001)
        .map(|n| work("s", &format!("us-east-{n}")))
        .collect::<TestResult<Vec<_>>>()?;
    store.seed_works(&works).await?;
    store.seed_works(&works).await?;
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM inventory.work_queue")
        .fetch_one(pool)
        .await?;
    assert_eq!(count, 5_001);
    let start = std::time::Instant::now();
    for _ in 0..100 {
        let lease = claim(store, "s", 100).await?;
        store
            .finish_scope(lease, ScopeOutcome::Complete, 101, None)
            .await?;
    }
    eprintln!(
        "P9 claims=100 elapsed_us={} workload=empty_scopes",
        start.elapsed().as_micros()
    );
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_work_given_invalid_context_when_seeded_then_batch_is_atomic() -> TestResult {
    run(invalid_batch).await
}

async fn invalid_batch(store: &PgStore, _: &PgPool, _: &DatabaseUrl) -> TestResult {
    let good = work("s", "us-east-1")?;
    let bad = good.clone().with_context(vec![0])?;
    assert!(store.seed_works(&[good, bad]).await.is_err());
    assert!(store.discover_active_sweep().await?.is_none());
    Ok(())
}
