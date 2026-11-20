use super::*;

async fn sweep_once(
    store: &BinaryStore,
    sweep: &str,
    regions: &[(&str, &[(&str, &str)])],
    base: i64,
    edges: Vec<Edge>,
) -> TestResult<inventory_engine::inventory::ChangeStats> {
    let works = regions
        .iter()
        .map(|(region, _)| relationship_work(sweep, region, None, Some("beta")))
        .collect::<TestResult<Vec<_>>>()?;
    store.seed_works(&works).await?;
    let id = SweepId::new(sweep.to_owned())?;
    let mut total = inventory_engine::inventory::ChangeStats::default();
    let mut edges = Some(edges);
    for offset in 0..i64::try_from(regions.len())? {
        let claimed_at = base.saturating_add(offset);
        let committed_at = claimed_at.saturating_add(1);
        let lease = store
            .claim(&id, claimed_at, 20)
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
        commit.edges = edges.take().unwrap_or_default();
        let result = store.commit_page(commit).await?;
        total.resources_deleted = total
            .resources_deleted
            .saturating_add(result.stats.resources_deleted);
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

use inventory_core::inventory::Observation;
use inventory_core::record::Exclusions;

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
    let both = [
        ("us-east-1", &[("east", "beta")][..]),
        ("us-west-2", &[("west", "beta")][..]),
    ];
    sweep_once(&store, "n1", &both, 100, Vec::new()).await?;
    let narrowed = [("us-east-1", &[("east", "beta")][..])];
    let mut deleted = 0;
    for (index, sweep) in ["n2", "n3", "n4"].into_iter().enumerate() {
        let base = 1_000 * i64::try_from(index + 1)?;
        deleted += sweep_once(&store, sweep, &narrowed, base, Vec::new())
            .await?
            .resources_deleted;
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
