use super::*;

use inventory_core::{
    catalog::runtime::{ContextProjectionPlan, RuntimeCatalog},
    inventory::{DiscoveryContext, DiscoveryFrame, OperationId, ResourceId, ServiceId},
    record::{FieldId, Key, Record, Value},
};
use inventory_engine::inventory::{ChangeStats, SchemaRegistry, WorkLease};
use std::collections::BTreeMap;

async fn work_only_fixture() -> TestResult<(tempfile::TempDir, BinaryStore)> {
    let directory = tempfile::tempdir()?;
    let bytes = branch_catalog_bytes(false)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let registry = SchemaRegistry::new(&bytes, catalog.schemas().to_vec())?;
    let store = BinaryStore::create(&directory.path().join("inventory.sqlite"), registry).await?;
    Ok((directory, store))
}

fn native_work(
    sweep: &SweepId,
    branch: &str,
    selector: &str,
    ordinal: u32,
    token: Option<Vec<u8>>,
) -> TestResult<WorkSpec> {
    let fixed = match branch {
        "DISCOVER" => "discovery-name",
        "EMIT" => "emitting-name",
        _ => return Err("unknown fixture branch".into()),
    };
    let selector_field = FieldId::new(3)?;
    let bindings = Record::encode(&Value::Map(BTreeMap::from([
        (Key::Field(FieldId::new(2)?), Value::Text(fixed.to_owned())),
        (Key::Field(selector_field), Value::Text(selector.to_owned())),
    ])))?;
    let projection = ContextProjectionPlan {
        selected: vec![vec![selector_field].into_boxed_slice()].into_boxed_slice(),
        ..ContextProjectionPlan::default()
    };
    let frame = DiscoveryFrame::project(
        ServiceId::parse("example")?,
        OperationId::parse("ListThings")?,
        ordinal,
        &projection,
        &bindings,
        &bindings,
        &bindings,
    )?;
    let context = DiscoveryContext::empty().push(frame)?;
    let scope = scope("us-east-1")?
        .with_branch(branch.to_owned())?
        .with_context(context.discriminator()?);
    Ok(
        WorkSpec::new(sweep.clone(), scope, token, bindings.into_bytes())?
            .with_context(context.encode()?)?,
    )
}

async fn claim_native(store: &BinaryStore, sweep: &SweepId, at: i64) -> TestResult<WorkLease> {
    store
        .claim(sweep, at, 20)
        .await?
        .ok_or_else(|| "native work was not scheduled".into())
}

async fn claim_native_work(store: &BinaryStore, work: WorkSpec, at: i64) -> TestResult<WorkLease> {
    let sweep = work.sweep().clone();
    store.seed_work(work).await?;
    claim_native(store, &sweep, at).await
}

async fn observe_native_member(
    store: &BinaryStore,
    sweep: &SweepId,
    list: &str,
    ordinal: u32,
    at: i64,
) -> TestResult {
    let listing = native_work(sweep, "DISCOVER", list, ordinal, None)?;
    let lease = claim_native_work(store, listing, at).await?;
    let mut describe = native_work(sweep, "EMIT", "listed-id", ordinal, None)?;
    describe.mark_scope_member(ResourceId::parse("listed-id")?);
    let describe_scope = describe.scope().clone();
    let mut commit = page(lease, Vec::new(), true, None, at.saturating_add(1));
    commit.children.push(describe);
    assert_eq!(store.commit_page(commit).await?.stats.resources_deleted, 0);

    let lease = claim_native(store, sweep, at.saturating_add(2)).await?;
    assert_eq!(lease.scope(), &describe_scope);
    let commit = page(
        lease,
        vec![observation("modeled-id", "described value")?.with_facet_precedence(100)],
        true,
        None,
        at.saturating_add(3),
    );
    assert_eq!(store.commit_page(commit).await?.stats.resources_deleted, 0);
    Ok(())
}

async fn finish_native_listing(
    store: &BinaryStore,
    listing: WorkSpec,
    outcome: ScopeOutcome,
    at: i64,
) -> TestResult<ChangeStats> {
    let lease = claim_native_work(store, listing, at).await?;
    Ok(store
        .finish_scope(lease, outcome, at.saturating_add(1), None)
        .await?)
}

#[tokio::test]
async fn given_a_listed_description_when_two_full_listings_miss_it_then_only_the_second_retires_it()
-> TestResult {
    let (_directory, store) = work_only_fixture().await?;
    let key = resource("modeled-id")?;
    let baseline = SweepId::new("listed".to_owned())?;
    observe_native_member(&store, &baseline, "primary", 0, 100).await?;
    assert!(store.projection_resource(&key).await?.is_some());

    let first = SweepId::new("first-miss".to_owned())?;
    let listing = native_work(&first, "DISCOVER", "primary", 1, None)?;
    let stats = finish_native_listing(&store, listing, ScopeOutcome::Complete, 200).await?;
    assert_eq!(stats.resources_deleted, 0);
    let unrelated = native_work(&first, "DISCOVER", "unrelated", 1, None)?;
    let stats = finish_native_listing(&store, unrelated, ScopeOutcome::Complete, 210).await?;
    assert_eq!(stats.resources_deleted, 0);
    assert!(store.projection_resource(&key).await?.is_some());

    let partial = SweepId::new("interrupted".to_owned())?;
    let listing = native_work(&partial, "DISCOVER", "primary", 2, None)?;
    let lease = claim_native_work(&store, listing, 300).await?;
    let started = page(lease, Vec::new(), false, Some(vec![b'A']), 301);
    assert_eq!(store.commit_page(started).await?.stats.resources_deleted, 0);
    let tail = claim_native(&store, &partial, 302).await?;
    let stats = store
        .finish_scope(tail, ScopeOutcome::Partial, 303, None)
        .await?;
    assert_eq!(stats.resources_deleted, 0);
    assert!(store.projection_resource(&key).await?.is_some());

    let tail_only = SweepId::new("tail-only".to_owned())?;
    let listing = native_work(&tail_only, "DISCOVER", "primary", 3, Some(vec![b'B']))?;
    let lease = claim_native_work(&store, listing, 400).await?;
    assert_eq!(lease.token(), Some(b"B".as_slice()));
    let stats = store
        .commit_page(page(lease, Vec::new(), true, None, 401))
        .await?
        .stats;
    assert_eq!(stats.resources_deleted, 0);
    assert!(store.projection_resource(&key).await?.is_some());

    let second = SweepId::new("second-miss".to_owned())?;
    let listing = native_work(&second, "DISCOVER", "primary", 4, None)?;
    let stats = finish_native_listing(&store, listing, ScopeOutcome::Complete, 500).await?;
    assert_eq!(stats.resources_deleted, 1);
    assert!(store.projection_resource(&key).await?.is_none());
    assert!(
        store
            .drain_projection(100)
            .await?
            .iter()
            .any(|entry| entry.deleted && entry.key.as_ref() == Some(&key))
    );
    Ok(())
}

async fn miss_both_native_listings(
    store: &BinaryStore,
    sweep: &str,
    at: i64,
) -> TestResult<ChangeStats> {
    let sweep = SweepId::new(sweep.to_owned())?;
    let first = native_work(&sweep, "DISCOVER", "primary", 0, None)?;
    let second = native_work(&sweep, "DISCOVER", "alternate", 1, None)?;
    let first = claim_native_work(store, first, at).await?;
    let second = claim_native_work(store, second, at).await?;
    let stats = store
        .finish_scope(first, ScopeOutcome::Complete, at.saturating_add(1), None)
        .await?;
    assert_eq!(stats.resources_deleted, 0);
    Ok(store
        .finish_scope(second, ScopeOutcome::Complete, at.saturating_add(2), None)
        .await?)
}

#[tokio::test]
async fn given_shared_native_describe_authority_when_the_last_live_memberships_expire_then_it_retires_once()
-> TestResult {
    let (_directory, store) = work_only_fixture().await?;
    let key = resource("modeled-id")?;
    let baseline = SweepId::new("shared-listed".to_owned())?;
    observe_native_member(&store, &baseline, "primary", 0, 100).await?;
    observe_native_member(&store, &baseline, "alternate", 1, 110).await?;

    for (name, at) in [("primary-miss-one", 200), ("primary-miss-two", 300)] {
        let sweep = SweepId::new(name.to_owned())?;
        let listing = native_work(&sweep, "DISCOVER", "primary", 0, None)?;
        let stats = finish_native_listing(&store, listing, ScopeOutcome::Complete, at).await?;
        assert_eq!(stats.resources_deleted, 0);
        assert!(store.projection_resource(&key).await?.is_some());
    }

    let reappeared = SweepId::new("shared-listed-again".to_owned())?;
    observe_native_member(&store, &reappeared, "primary", 0, 400).await?;
    assert_eq!(
        miss_both_native_listings(&store, "shared-miss-one", 500)
            .await?
            .resources_deleted,
        0
    );
    assert!(store.projection_resource(&key).await?.is_some());
    assert_eq!(
        miss_both_native_listings(&store, "shared-miss-two", 600)
            .await?
            .resources_deleted,
        1
    );
    assert!(store.projection_resource(&key).await?.is_none());
    Ok(())
}

#[tokio::test]
async fn given_an_independent_facet_when_the_listed_facet_expires_then_the_survivor_is_published()
-> TestResult {
    let (directory, store) = work_only_fixture().await?;
    let baseline = SweepId::new("preferred-description".to_owned())?;
    observe_native_member(&store, &baseline, "primary", 0, 100).await?;
    let fallback = SweepId::new("independent-description".to_owned())?;
    let work = native_work(&fallback, "EMIT", "independent-id", 0, None)?;
    let lease = claim_native_work(&store, work, 200).await?;
    let secondary = observation("modeled-id", "independent value")?.with_facet_precedence(0);
    store
        .commit_page(page(lease, vec![secondary], true, None, 201))
        .await?;
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?;
    let value = "SELECT f_1 FROM thing_table WHERE id='modeled-id'";
    let before: String = sqlx::query_scalar(value).fetch_one(&pool).await?;
    assert_eq!(before, "described value");

    let first = SweepId::new("preferred-miss-one".to_owned())?;
    let listing = native_work(&first, "DISCOVER", "primary", 0, None)?;
    let stats = finish_native_listing(&store, listing, ScopeOutcome::Complete, 300).await?;
    assert_eq!((stats.resources_deleted, stats.resources_changed), (0, 0));
    let second = SweepId::new("preferred-miss-two".to_owned())?;
    let listing = native_work(&second, "DISCOVER", "primary", 0, None)?;
    let stats = finish_native_listing(&store, listing, ScopeOutcome::Complete, 400).await?;
    assert_eq!((stats.resources_deleted, stats.resources_changed), (0, 1));

    let after: String = sqlx::query_scalar(value).fetch_one(&pool).await?;
    assert_eq!(after, "independent value");
    let retired: (i64, i64) = sqlx::query_as(
        "SELECT COUNT(*),max(last_seen) FROM resource_scopes WHERE id='modeled-id' AND missing_complete=2",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(retired, (1, 401));
    let facets: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM resource_facets WHERE id='modeled-id'")
            .fetch_one(&pool)
            .await?;
    assert_eq!(facets, 1);
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_a_fresh_describe_when_its_last_membership_expires_then_the_observation_survives()
-> TestResult {
    let (_directory, store) = work_only_fixture().await?;
    let baseline = SweepId::new("initial-description".to_owned())?;
    observe_native_member(&store, &baseline, "primary", 0, 100).await?;
    let first = SweepId::new("first-stale-membership".to_owned())?;
    let listing = native_work(&first, "DISCOVER", "primary", 0, None)?;
    let stats = finish_native_listing(&store, listing, ScopeOutcome::Complete, 200).await?;
    assert_eq!(stats.resources_deleted, 0);

    let second = SweepId::new("fresh-description".to_owned())?;
    let listing = native_work(&second, "DISCOVER", "primary", 0, None)?;
    let list = claim_native_work(&store, listing, 300).await?;
    let describe = native_work(&second, "EMIT", "listed-id", 1, None)?;
    let describe = claim_native_work(&store, describe, 300).await?;
    let fresh = observation("modeled-id", "fresh value")?;
    let hash = *fresh.content_hash().as_bytes();
    store
        .commit_page(page(describe, vec![fresh], true, None, 301))
        .await?;
    let stats = store
        .finish_scope(list, ScopeOutcome::Complete, 302, None)
        .await?;
    assert_eq!(stats.resources_deleted, 0);
    let projected = store
        .projection_resource(&resource("modeled-id")?)
        .await?
        .ok_or("fresh described resource disappeared")?;
    assert_eq!(projected.content_hash, hash);
    Ok(())
}
