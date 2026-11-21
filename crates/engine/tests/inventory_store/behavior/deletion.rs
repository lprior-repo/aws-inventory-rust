use super::*;

fn sweep_works(
    sweep: &str,
    regions: &[(&str, &[(&str, &str)])],
) -> TestResult<Vec<WorkSpec>> {
    regions
        .iter()
        .map(|(region, _)| relationship_work(sweep, region, None, Some("beta")))
        .collect()
}

async fn process_region(
    store: &BinaryStore,
    sweep: &SweepId,
    regions: &[(&str, &[(&str, &str)])],
    claimed_at: i64,
    committed_at: i64,
    edges: Option<Vec<Edge>>,
) -> TestResult<inventory_engine::inventory::ChangeStats> {
    let lease = store
        .claim(sweep, claimed_at, 20)
        .await?
        .ok_or("missing sweep lease")?;
    let region = lease.scope().region().as_str().to_owned();
    let ids = regions
        .iter()
        .find(|(candidate, _)| *candidate == region)
        .map(|(_, ids)| *ids)
        .ok_or("lease for unknown region")?;
    let observations = ids
        .iter()
        .map(|(id, value)| regional(id, value, &region))
        .collect::<TestResult<Vec<_>>>()?;
    let mut commit = page(lease, observations, true, None, committed_at);
    commit.edges = match edges {
        Some(list) => list,
        None => Vec::new(),
    };
    let result = store.commit_page(commit).await?;
    Ok(result.stats)
}

async fn sweep_once(
    store: &BinaryStore,
    sweep: &str,
    regions: &[(&str, &[(&str, &str)])],
    base: i64,
    edges: Vec<Edge>,
) -> TestResult<inventory_engine::inventory::ChangeStats> {
    let works = sweep_works(sweep, regions)?;
    store.seed_works(&works).await?;
    let id = SweepId::new(sweep.to_owned())?;
    let mut total = inventory_engine::inventory::ChangeStats::default();
    let mut edge_opt = Some(edges);
    for offset in 0..i64::try_from(regions.len())? {
        let claimed_at = base.saturating_add(offset);
        let committed_at = claimed_at.saturating_add(1);
        let stats = process_region(store, &id, regions, claimed_at, committed_at, edge_opt.take()).await?;
        total.resources_deleted = total.resources_deleted.saturating_add(stats.resources_deleted);
    }
    Ok(total)
}

fn regional(id: &str, value: &str, region: &str) -> TestResult<Observation> {
    let mut observed = observation(id, value)?;
    if region != "us-east-1" {
        let key = ResourceKey {
            region: Region::parse(region)?,
            ..resource(id)?
        };
        let record = observed.record().clone();
        observed = Observation::new(key, record, &Exclusions::none(), None)?;
    }
    Ok(observed)
}

fn account_observation(id: &str, value: &str, account: &str, region: &str) -> TestResult<Observation> {
    let key = ResourceKey {
        account: AccountId::parse(account)?,
        region: Region::parse(region)?,
        ..resource(id)?
    };
    let record = observation(id, value)?.record().clone();
    Ok(Observation::new(key, record, &Exclusions::none(), None)?)
}

use inventory_core::inventory::Observation;
use inventory_core::record::{Exclusions, FieldId, Record, Value};

fn account_scope(account: &str, region: &str) -> TestResult<ScopeKey> {
    Ok(ScopeKey::new(
        AccountId::parse(account)?,
        Region::parse(region)?,
        ServiceId::parse("example")?,
        OperationId::parse("ListThings")?,
        None,
    )?)
}

fn account_work(sweep: &str, account: &str, region: &str) -> TestResult<WorkSpec> {
    WorkSpec::new(
        SweepId::new(sweep.to_owned())?,
        account_scope(account, region)?,
        None,
        vec![1, 2],
    )
    .map_err(Into::into)
}

async fn get_scope_key_bytes(
    pool: &sqlx::SqlitePool,
    sweep: &str,
    _region: &str,
) -> TestResult<Vec<u8>> {
    let bytes: Vec<u8> = sqlx::query_scalar("SELECT scope_key FROM scope_outcomes WHERE sweep_id=?1 AND outcome='complete' LIMIT 1")
        .bind(sweep)
        .fetch_one(pool)
        .await?;
    Ok(bytes)
}

#[tokio::test]
async fn given_a_resource_that_vanishes_when_two_complete_sweeps_miss_it_then_row_edges_and_projection_are_gone()
-> TestResult {
    let (directory, store, _) = fixture().await?;
    let east = "us-east-1";
    let edge = Edge {
        from: resource("gone")?,
        to: resource("keep")?,
        relation: Relation::parse("depends-on")?,
        source: EdgeSource::Model,
    };
    sweep_once(
        &store,
        "s1",
        &[(east, &[("keep", "beta"), ("gone", "alpha")][..])],
        100,
        vec![edge],
    )
    .await?;
    for key in ["keep", "gone"] {
        assert!(store.projection_resource(&resource(key)?).await?.is_some());
    }
    // Drain the inserts so only deletion work remains in the outbox.
    for pending in store.drain_projection(100).await? {
        store
            .ack_projection(match pending.key {
                Some(key) => {
                    inventory_engine::inventory::ProjectionAck::new(key, pending.revision)?
                }
                None => inventory_engine::inventory::ProjectionAck::full_rebuild(pending.revision)?,
            })
            .await?;
    }

    // First complete sweep that misses it: still stored.
    let first = sweep_once(
        &store,
        "s2",
        &[(east, &[("keep", "beta")][..])],
        200,
        Vec::new(),
    )
    .await?;
    assert_eq!(first.resources_deleted, 0);
    assert!(
        store
            .projection_resource(&resource("gone")?)
            .await?
            .is_some()
    );
    assert_eq!(
        store
            .projection_edges_for(&resource("gone")?, None, 10)
            .await?
            .edges
            .len(),
        1
    );

    // Second complete sweep that misses it: deleted everywhere.
    let second = sweep_once(
        &store,
        "s3",
        &[(east, &[("keep", "beta")][..])],
        300,
        Vec::new(),
    )
    .await?;
    assert_eq!(second.resources_deleted, 1);
    assert!(
        store
            .projection_resource(&resource("gone")?)
            .await?
            .is_none()
    );
    assert!(
        store
            .projection_resource(&resource("keep")?)
            .await?
            .is_some()
    );
    assert!(
        store
            .projection_edges_for(&resource("keep")?, None, 10)
            .await?
            .edges
            .is_empty()
    );
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?;
    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM resources WHERE id='gone'")
        .fetch_one(&pool)
        .await?;
    let edges: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM edges")
        .fetch_one(&pool)
        .await?;
    let typed: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM thing_table")
        .fetch_one(&pool)
        .await?;
    pool.close().await;
    assert_eq!(
        (rows, edges, typed),
        (0, 0, 1),
        "only keep remains in the typed table"
    );
    // The graph is told: the outbox carries a deletion for the removed key.
    let gone = resource("gone")?;
    let pending = store.drain_projection(100).await?;
    assert!(
        pending
            .iter()
            .any(|entry| entry.deleted && entry.key.as_ref() == Some(&gone)),
        "outbox must carry the deletion for the graph projection: {pending:?}"
    );
    Ok(())
}

#[tokio::test]
async fn given_narrowed_regions_when_sweeps_complete_then_nothing_is_deleted_and_skipped_scope_is_unobserved_with_age()
-> TestResult {
    let (_directory, store, _) = fixture().await?;
    let both = [("us-east-1", &[("east", "beta")][..]), ("us-west-2", &[("west", "beta")][..])];
    sweep_once(&store, "n1", &both, 100, Vec::new()).await?;
    let narrowed = [("us-east-1", &[("east", "beta")][..])];
    let mut deleted: u64 = 0;
    for (index, sweep) in ["n2", "n3", "n4"].into_iter().enumerate() {
        let offset = i64::try_from(index)?;
        let result = sweep_once(&store, sweep, &narrowed, 1_000_i64.saturating_add(offset.saturating_mul(100_i64)), Vec::new()).await?;
        deleted = deleted.saturating_add(result.resources_deleted);
    }
    assert_eq!(deleted, 0);
    let west = ResourceKey {
        region: Region::parse("us-west-2")?,
        ..resource("west")?
    };
    assert!(store.projection_resource(&west).await?.is_some());

    let last = SweepId::new("n4".to_owned())?;
    assert_eq!(store.sweep_summary(&last).await?.unobserved_scopes, 1);
    let skipped = store.unobserved_scopes(&last, 5_000).await?;
    let [only] = skipped.as_slice() else {
        return Err("expected exactly one unobserved scope".into());
    };
    assert_eq!(only.scope, scope("us-west-2")?);
    assert_eq!(only.last_observed_ms, 102);
    assert_eq!(only.age_ms, 4_898);
    // A full-width sweep observes it again and clears the report.
    sweep_once(&store, "n5", &both, 6_000, Vec::new()).await?;
    let again = SweepId::new("n5".to_owned())?;
    assert!(store.unobserved_scopes(&again, 7_000).await?.is_empty());
    assert_eq!(store.sweep_summary(&again).await?.unobserved_scopes, 0);
    Ok(())
}

#[tokio::test]
async fn given_failed_outcome_when_scope_becomes_absent_then_age_measured_from_last_complete()
-> TestResult {
    let (directory, store, _) = fixture().await?;
    // S1: complete observation at t=100
    sweep_once(&store, "s1", &[("us-east-1", &[("x", "v1")][..])], 100, Vec::new()).await?;
    // Manually record a failed outcome at t=200 (simulating a failed sweep)
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?;
    let scope_key_bytes = get_scope_key_bytes(&pool, "s1", "us-east-1").await?;
    sqlx::query("INSERT INTO scope_outcomes(sweep_id,scope_key,outcome,observed_at) VALUES('s2',?1,'failed',200) ON CONFLICT(sweep_id,scope_key) DO UPDATE SET outcome='failed',observed_at=200")
        .bind(&scope_key_bytes)
        .execute(&pool)
        .await?;
    pool.close().await;
    // S3: scope absent (different scope entirely); should report unobserved with last_observed=101 (from complete), not 200 (from failed)
    sweep_once(&store, "s3", &[("us-west-2", &[("y", "v2")][..])], 300, Vec::new()).await?;
    let s3 = SweepId::new("s3".to_owned())?;
    let unobserved = store.unobserved_scopes(&s3, 1_000).await?;
    let [only] = unobserved.as_slice() else {
        return Err("expected exactly one unobserved scope".into());
    };
    assert_eq!(only.scope, scope("us-east-1")?);
    assert_eq!(only.last_observed_ms, 101, "age measured from last complete, not failed outcome");
    Ok(())
}

#[tokio::test]
async fn given_scope_with_non_complete_last_outcome_when_scope_becomes_absent_then_not_reported_unobserved()
-> TestResult {
    let (directory, store, _) = fixture().await?;
    // S1: complete observation
    sweep_once(&store, "s1", &[("us-east-1", &[("x", "v1")][..])], 100, Vec::new()).await?;
    // Manually record a partial outcome at t=200 (last outcome is not complete)
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?;
    let scope_key_bytes = get_scope_key_bytes(&pool, "s1", "us-east-1").await?;
    sqlx::query("INSERT INTO scope_outcomes(sweep_id,scope_key,outcome,observed_at) VALUES('s2',?1,'partial',200) ON CONFLICT(sweep_id,scope_key) DO UPDATE SET outcome='partial',observed_at=200")
        .bind(&scope_key_bytes)
        .execute(&pool)
        .await?;
    pool.close().await;
    // S3: scope absent; should NOT report as unobserved because last outcome was not complete
    sweep_once(&store, "s3", &[("us-east-1", &[("y", "v2")][..])], 300, Vec::new()).await?;
    let s3 = SweepId::new("s3".to_owned())?;
    let unobserved = store.unobserved_scopes(&s3, 1_000).await?;
    // The us-east-1 scope should not be reported because its last outcome was 'partial', not 'complete'
    // (the history query only considers 'complete' outcomes)
    let reported: Vec<_> = unobserved
        .iter()
        .filter(|u| u.scope.region().as_str() == "us-east-1")
        .collect();
    assert!(
        reported.is_empty(),
        "scope with last non-complete outcome should not be reported unobserved"
    );
    Ok(())
}

#[tokio::test]
async fn given_removed_account_profile_when_later_sweep_runs_only_other_account_then_removed_account_scopes_are_unobserved()
-> TestResult {
    let (_directory, store, _) = fixture().await?;
    // S1: account A complete
    let work_a = account_work("s1", "111111111111", "us-east-1")?;
    store.seed_work(work_a).await?;
    store.create_sweep_manifest(&SweepId::new("s1".to_owned())?, "111111111111").await?;
    let lease_a = store.claim(&SweepId::new("s1".to_owned())?, 100, 20).await?.ok_or("missing lease a")?;
    store.commit_page(page(lease_a, vec![account_observation("a", "va", "111111111111", "us-east-1")?], true, None, 101)).await?;
    // S2: account B complete
    let work_b = account_work("s2", "222222222222", "us-west-2")?;
    store.seed_work(work_b).await?;
    store.create_sweep_manifest(&SweepId::new("s2".to_owned())?, "222222222222").await?;
    let lease_b = store.claim(&SweepId::new("s2".to_owned())?, 200, 20).await?.ok_or("missing lease b")?;
    store.commit_page(page(lease_b, vec![account_observation("b", "vb", "222222222222", "us-west-2")?], true, None, 201)).await?;
    // S3: only account A runs again; account B should be reported unobserved
    let work_a3 = account_work("s3", "111111111111", "us-east-1")?;
    store.seed_work(work_a3).await?;
    store.create_sweep_manifest(&SweepId::new("s3".to_owned())?, "111111111111").await?;
    let lease_a3 = store.claim(&SweepId::new("s3".to_owned())?, 300, 20).await?.ok_or("missing lease a3")?;
    store.commit_page(page(lease_a3, vec![account_observation("a2", "va2", "111111111111", "us-east-1")?], true, None, 301)).await?;
    // Verify account B's scope is reported unobserved
    let s3 = SweepId::new("s3".to_owned())?;
    let unobserved = store.unobserved_scopes(&s3, 1_000).await?;
    let b_unobserved = unobserved
        .iter()
        .find(|u| u.scope.account().as_str() == "222222222222");
    assert!(
        b_unobserved.is_some(),
        "account B's scope should be reported unobserved when account B is not run in current sweep"
    );
    let b_scope = b_unobserved.ok_or("account B scope not found")?;
    assert_eq!(
        b_scope.last_observed_ms,
        201,
        "age measured from account B's last complete observation"
    );
    Ok(())
}