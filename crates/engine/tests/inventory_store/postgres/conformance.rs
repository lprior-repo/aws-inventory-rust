use super::{PgStore, PgWriteConfig, TestDatabase, TestResult, admin_url, registry, support};
use inventory_engine::inventory::{DatabaseUrl, SweepId};
use std::num::NonZeroUsize;

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_native_backend_when_conformance_runs_then_every_store_contract_holds() -> TestResult
{
    super::super::conformance::run(async || fixture(PgWriteConfig::default()).await).await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn forced_staging_copy_preserves_every_store_contract() -> TestResult {
    let config = PgWriteConfig::default().with_copy_min_rows(NonZeroUsize::MIN);
    let (database, store, _) = fixture(config).await?;
    let pool = sqlx::PgPool::connect(database.url()).await?;
    let sweep_id = SweepId::new("copy-test".to_owned())?;
    store
        .create_sweep_manifest(&sweep_id, "111111111111")
        .await?;
    store.seed_works(&[super::work("copy-test", None)?]).await?;
    let lease = super::claim(&store, "copy-test", 100).await?;
    let page = support::page(
        lease,
        vec![
            support::observation("one", "first")?,
            support::observation("two", "second")?,
        ],
        true,
        None,
        101,
    );
    let result = store.commit_page(page).await?;
    assert_eq!(result.stats.resources_inserted, 2);
    assert_eq!(
        staging_tables(&pool).await?,
        ["stg_resources", "stg_thing_table"]
    );
    store.close().await;
    pool.close().await;
    database.finish().await?;
    super::super::conformance::run(async || fixture(config).await).await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn forced_direct_copy_preserves_every_store_contract() -> TestResult {
    let config = PgWriteConfig::default()
        .with_copy_min_rows(NonZeroUsize::MIN)
        .with_direct_copy()?;
    let (database, store, _) = fixture(config).await?;
    let pool = sqlx::PgPool::connect(database.url()).await?;
    install_direct_witness(&pool).await?;
    let sweep_id = SweepId::new("copy-test".to_owned())?;
    store
        .create_sweep_manifest(&sweep_id, "111111111111")
        .await?;
    store.seed_works(&[super::work("copy-test", None)?]).await?;
    let lease = super::claim(&store, "copy-test", 100).await?;
    let page = support::page(
        lease,
        vec![
            support::observation("one", "first")?,
            support::observation("two", "second")?,
        ],
        true,
        None,
        101,
    );
    let result = store.commit_page(page).await?;
    assert_eq!(result.stats.resources_inserted, 2);
    assert_eq!(direct_witness(&pool).await?, 1);
    assert!(staging_tables(&pool).await?.is_empty());
    store.close().await;
    pool.close().await;
    database.finish().await?;
    super::super::conformance::run(async || fixture(config).await).await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn direct_copy_with_existing_keys_uses_staging() -> TestResult {
    let config = PgWriteConfig::default()
        .with_copy_min_rows(NonZeroUsize::MIN)
        .with_direct_copy()?;
    let (database, store, _) = fixture(config).await?;
    let pool = sqlx::PgPool::connect(database.url()).await?;
    let result = direct_existing_key(&store, &pool).await;
    store.close().await;
    pool.close().await;
    database.finish().await?;
    result
}

async fn direct_existing_key(store: &PgStore, pool: &sqlx::PgPool) -> TestResult {
    store.seed_works(&[super::work("work-1", None)?]).await?;
    let first = support::page(
        super::claim(store, "work-1", 100).await?,
        vec![support::observation("existing", "first")?],
        false,
        Some(b"next".to_vec()),
        101,
    );
    let (result, lease) = store.commit_page_keeping(first, 10_000).await?;
    assert_eq!(result.stats.resources_inserted, 1);
    assert!(staging_tables(pool).await?.is_empty());
    let second = support::page(
        lease.ok_or("first direct page did not keep its lease")?,
        vec![support::observation("existing", "changed")?],
        true,
        None,
        102,
    );
    assert_eq!(store.commit_page(second).await?.stats.resources_changed, 1);
    assert_eq!(
        staging_tables(pool).await?,
        ["stg_resources", "stg_thing_table"]
    );
    Ok(())
}
#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn direct_unique_violation_rolls_back_only_its_page_and_retries_staging() -> TestResult {
    let config = PgWriteConfig::default()
        .with_copy_min_rows(NonZeroUsize::MIN)
        .with_direct_copy()?;
    let (database, store, _) = fixture(config).await?;
    let pool = sqlx::PgPool::connect(database.url()).await?;
    let result = unique_violation_fallback(&store, &pool).await;
    store.close().await;
    pool.close().await;
    database.finish().await?;
    result
}

async fn unique_violation_fallback(store: &PgStore, pool: &sqlx::PgPool) -> TestResult {
    install_direct_witness(pool).await?;
    let sweep = SweepId::new("copy-race".to_owned())?;
    store.create_sweep_manifest(&sweep, "111111111111").await?;
    store.seed_works(&[super::work("copy-race", None)?]).await?;
    let first = support::page(
        super::claim(store, "copy-race", 100).await?,
        vec![support::observation("prior", "earlier page")?],
        false,
        Some(b"next".to_vec()),
        101,
    );
    let (prior, lease) = store.commit_page_keeping(first, 10_000).await?;
    assert_eq!(prior.stats.resources_inserted, 1);
    assert_eq!(direct_witness(pool).await?, 1);
    collide_on_direct_copy(
        store,
        pool,
        lease.ok_or("first page did not retain its lease")?,
    )
    .await
}

async fn collide_on_direct_copy(
    store: &PgStore,
    pool: &sqlx::PgPool,
    lease: inventory_engine::inventory::WorkLease,
) -> TestResult {
    use std::time::Duration;

    let page = support::page(
        lease,
        vec![support::observation("collision", "winner")?],
        true,
        None,
        102,
    );
    install_lookup_barrier(pool).await?;
    let mut barrier = pool.acquire().await?;
    sqlx::query("SELECT pg_advisory_lock(123456789,987654321)")
        .execute(&mut *barrier)
        .await?;
    let commit = store.commit_page(page);
    tokio::pin!(commit);
    let observed = tokio::select! {
        result = &mut commit => {
            release_lookup_barrier(&mut barrier).await?;
            result?;
            return Err("page completed before the lookup barrier".into());
        }
        observed = tokio::time::timeout(
            Duration::from_secs(10),
            wait_for_resource_lookup(pool),
        ) => match observed {
            Ok(Ok(found)) => found,
            Ok(Err(error)) => {
                release_lookup_barrier(&mut barrier).await?;
                return Err(error);
            }
            Err(error) => {
                release_lookup_barrier(&mut barrier).await?;
                return Err(error.into());
            }
        }
    };
    if !observed {
        release_lookup_barrier(&mut barrier).await?;
        return Err("resource lookup did not reach the test barrier".into());
    }
    let inserted = insert_competing_resource(pool).await;
    release_lookup_barrier(&mut barrier).await?;
    inserted?;
    let committed = commit.await?;
    verify_collision(pool, committed.stats.resources_inserted).await
}

async fn install_lookup_barrier(pool: &sqlx::PgPool) -> TestResult {
    sqlx::query("CREATE TABLE inventory.copy_direct_test_marker(label text PRIMARY KEY, witness bigint NOT NULL)")
        .execute(pool)
        .await?;
    sqlx::query("CREATE SEQUENCE inventory.copy_direct_test_marker_seq")
        .execute(pool)
        .await?;
    sqlx::query("CREATE FUNCTION inventory.copy_direct_test_barrier() RETURNS trigger LANGUAGE plpgsql AS $body$ BEGIN IF NOT EXISTS (SELECT 1 FROM inventory.copy_direct_test_marker WHERE label='collision') THEN INSERT INTO inventory.copy_direct_test_marker(label,witness) VALUES ('collision',COALESCE(pg_sequence_last_value('inventory.copy_direct_test_witness'),0)); PERFORM nextval('inventory.copy_direct_test_marker_seq'); END IF; PERFORM pg_advisory_xact_lock(123456789,987654321); RETURN NULL; END; $body$")
        .execute(pool)
        .await?;
    sqlx::query("CREATE TRIGGER copy_direct_test_barrier AFTER INSERT ON inventory.resource_scopes FOR EACH STATEMENT EXECUTE FUNCTION inventory.copy_direct_test_barrier()")
        .execute(pool)
        .await?;
    Ok(())
}

async fn insert_competing_resource(pool: &sqlx::PgPool) -> TestResult {
    let competing = support::observation("collision", "other scope")?;
    let key = competing.key();
    sqlx::query("INSERT INTO inventory.resources(account,region,kind,id,name,content_hash,first_seen,last_changed) VALUES($1,$2,$3,$4,$5,$6,100,100)")
        .bind(key.account.as_str())
        .bind(key.region.as_str())
        .bind(key.kind.as_str())
        .bind(key.id.as_str())
        .bind("other scope")
        .bind([1_u8; 32].as_slice())
        .execute(pool)
        .await?;
    Ok(())
}

async fn release_lookup_barrier(connection: &mut sqlx::PgConnection) -> TestResult {
    sqlx::query("SELECT pg_advisory_unlock(123456789,987654321)")
        .execute(connection)
        .await?;
    Ok(())
}

async fn verify_collision(pool: &sqlx::PgPool, inserted: u64) -> TestResult {
    assert_eq!(inserted, 1);
    assert_eq!(direct_witness(pool).await?, 2);
    verify_marker(pool).await?;
    assert_eq!(
        sqlx::query_scalar::<_, String>(
            "SELECT f_1 FROM inventory.thing_table WHERE id='collision'"
        )
        .fetch_one(pool)
        .await?,
        "winner"
    );
    assert!(
        sqlx::query_scalar::<_, bool>(
            "SELECT content_hash<>$1 FROM inventory.resources WHERE id='collision'"
        )
        .bind([1_u8; 32].as_slice())
        .fetch_one(pool)
        .await?
    );
    assert!(
        sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM inventory.resources WHERE id='prior')"
        )
        .fetch_one(pool)
        .await?
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM inventory.resources WHERE id IN ('prior','collision')"
        )
        .fetch_one(pool)
        .await?,
        2
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM inventory.thing_table WHERE id IN ('prior','collision')"
        )
        .fetch_one(pool)
        .await?,
        2
    );
    assert_eq!(
        staging_tables(pool).await?,
        ["stg_resources", "stg_thing_table"]
    );
    Ok(())
}

async fn wait_for_resource_lookup(pool: &sqlx::PgPool) -> TestResult<bool> {
    for _ in 0..100 {
        let active = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE datname=current_database() AND query LIKE '%resource_scopes%' AND wait_event_type='Lock')",
        )
        .fetch_one(pool)
        .await?;
        if active {
            return Ok(true);
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    Ok(false)
}

async fn fixture(config: PgWriteConfig) -> TestResult<(TestDatabase, PgStore, Vec<u8>)> {
    let database = TestDatabase::create(&admin_url()?).await?;
    let url = DatabaseUrl::parse(database.url())?;
    let store = PgStore::create(&url, registry()?, config).await?;
    for id in ["work-1", "live-lease", "takeover", "dummy"] {
        store
            .create_sweep_manifest(&SweepId::new(id.to_owned())?, "111111111111")
            .await?;
    }
    Ok((database, store, support::catalog_bytes()?))
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn a_page_at_the_threshold_creates_staging_tables_but_a_smaller_page_does_not() -> TestResult
{
    let threshold = NonZeroUsize::new(2).ok_or("threshold must be nonzero")?;
    let (database, store, _) =
        fixture(PgWriteConfig::default().with_copy_min_rows(threshold)).await?;
    let pool = sqlx::PgPool::connect(database.url()).await?;
    let result = check_stage_threshold(&store, &pool).await;
    store.close().await;
    pool.close().await;
    database.finish().await?;
    result
}

async fn check_stage_threshold(store: &PgStore, pool: &sqlx::PgPool) -> TestResult {
    store.seed_works(&[super::work("work-1", None)?]).await?;
    let first = support::page(
        super::claim(store, "work-1", 100).await?,
        vec![support::observation("one", "first")?],
        false,
        Some(b"next".to_vec()),
        101,
    );
    let (result, lease) = store.commit_page_keeping(first, 10_000).await?;
    assert_eq!(result.stats.resources_inserted, 1);
    assert!(staging_tables(pool).await?.is_empty());
    let second = support::page(
        lease.ok_or("first page did not keep its lease")?,
        vec![
            support::observation("two", "second")?,
            support::observation("three", "third")?,
        ],
        true,
        None,
        102,
    );
    let result = store.commit_page(second).await?;
    assert_eq!(result.stats.resources_inserted, 2);
    assert_eq!(
        staging_tables(pool).await?,
        ["stg_resources", "stg_thing_table"]
    );
    Ok(())
}

async fn staging_tables(pool: &sqlx::PgPool) -> TestResult<Vec<String>> {
    Ok(sqlx::query_scalar(
        "SELECT relname::text FROM pg_class WHERE relpersistence = 't' AND relname IN ('stg_resources','stg_thing_table') ORDER BY relname",
    )
    .fetch_all(pool)
    .await?)
}

async fn install_direct_witness(pool: &sqlx::PgPool) -> TestResult {
    sqlx::query("CREATE SEQUENCE inventory.copy_direct_test_witness")
        .execute(pool)
        .await?;
    sqlx::query("CREATE FUNCTION inventory.copy_direct_test_witness_fn() RETURNS trigger LANGUAGE plpgsql AS $body$ BEGIN IF current_query() LIKE 'COPY inventory.resources%' THEN PERFORM nextval('inventory.copy_direct_test_witness'); END IF; RETURN NULL; END; $body$")
        .execute(pool)
        .await?;
    sqlx::query("CREATE TRIGGER copy_direct_test_witness BEFORE INSERT ON inventory.resources FOR EACH STATEMENT EXECUTE FUNCTION inventory.copy_direct_test_witness_fn()")
        .execute(pool)
        .await?;
    Ok(())
}

async fn direct_witness(pool: &sqlx::PgPool) -> TestResult<i64> {
    Ok(sqlx::query_scalar(
        "SELECT COALESCE(pg_sequence_last_value('inventory.copy_direct_test_witness'),0)",
    )
    .fetch_one(pool)
    .await?)
}

async fn verify_marker(pool: &sqlx::PgPool) -> TestResult {
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM inventory.copy_direct_test_marker")
            .fetch_one(pool)
            .await?,
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT witness FROM inventory.copy_direct_test_marker WHERE label='collision'"
        )
        .fetch_one(pool)
        .await?,
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COALESCE(pg_sequence_last_value('inventory.copy_direct_test_marker_seq'),0)"
        )
        .fetch_one(pool)
        .await?,
        1
    );
    Ok(())
}
impl support::OutcomeDetailStorageCapability for TestDatabase {
    async fn stored_outcomes(
        &self,
        sweep: &str,
    ) -> support::TestResult<Vec<(String, Option<String>)>> {
        let pool = sqlx::PgPool::connect(self.url()).await?;
        let rows = sqlx::query_as(
            "SELECT outcome,detail FROM inventory.scope_outcomes WHERE sweep_id=$1 ORDER BY scope_key",
        )
        .bind(sweep)
        .fetch_all(&pool)
        .await?;
        pool.close().await;
        Ok(rows)
    }

    async fn seed_prior_outcome_detail(&self, sweep: &str) -> support::TestResult {
        let pool = sqlx::PgPool::connect(self.url()).await?;
        sqlx::query("INSERT INTO inventory.scope_outcomes(sweep_id,scope_key,outcome,observed_at,detail) SELECT sweep_id,scope_key,'failed',90,'transport: earlier' FROM inventory.work_queue WHERE sweep_id=$1")
            .bind(sweep)
            .execute(&pool)
            .await?;
        pool.close().await;
        Ok(())
    }
}

impl support::UnobservedScopesCapability for PgStore {
    async fn unobserved_scopes(
        &self,
        sweep: &SweepId,
        now_ms: i64,
    ) -> Result<
        Vec<inventory_engine::inventory::UnobservedScope>,
        inventory_engine::inventory::StoreError,
    > {
        PgStore::unobserved_scopes(self, sweep, now_ms).await
    }
}
