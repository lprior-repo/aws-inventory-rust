use super::*;
use inventory_core::inventory::{
    AccountId, Observation, OperationId, Region, ResourceKey, ScopeKey, ScopeOutcome, ServiceId,
};
use inventory_core::record::{Exclusions, Record, Value};
use inventory_engine::inventory::WorkSpec;
use std::collections::BTreeMap;

pub(super) async fn run<S, F, G>(make: &F) -> TestResult
where
    S: InventoryStore + UnobservedScopesCapability,
    F: AsyncFn() -> TestResult<(G, S, Vec<u8>)>,
{
    let (_guard, store, catalog) = make().await?;
    given_catalog_when_validated_then_only_matching_bytes_pass(&store, &catalog).await?;
    let (_guard, store, _) = make().await?;
    given_final_page_when_committed_then_completion_is_idempotent(&store).await?;
    let (_guard, store, _) = make().await?;
    given_partial_when_followed_by_two_misses_then_only_second_deletes(&store).await?;
    let (_guard, store, _) = make().await?;
    given_vanishing_resource_when_two_complete_sweeps_miss_it_then_it_is_deleted(&store).await?;
    let (_guard, store, _) = make().await?;
    given_deleted_resource_with_edge_when_two_complete_sweeps_miss_it_then_edges_are_cleared(
        &store,
    )
    .await?;
    let (_guard, store, _) = make().await?;
    given_failed_scope_when_absent_then_age_uses_last_complete_observation(&store).await?;
    let (_guard, store, _) = make().await?;
    given_partial_scope_when_current_then_not_reported_unobserved(&store).await?;
    let (_guard, store, _) = make().await?;
    given_narrowed_regions_when_complete_then_skipped_scope_is_reported(&store).await?;
    let (_guard, store, _) = make().await?;
    given_removed_account_when_sweep_runs_for_other_account_then_its_scopes_are_unobserved(&store)
        .await
}

async fn observe_account_scope<S: InventoryStore>(
    store: &S,
    sweep: &str,
    account: &str,
    region: &str,
    id: &str,
    value: &str,
    at: i64,
) -> TestResult {
    let sweep_id = SweepId::new(sweep.to_owned())?;
    let account_id = AccountId::parse(account)?;
    let scope_key = ScopeKey::new(
        account_id.clone(),
        Region::parse(region)?,
        ServiceId::parse("example")?,
        OperationId::parse("ListThings")?,
        None,
    )?;
    let bindings = Record::encode(&Value::Map(BTreeMap::new()))?.into_bytes();
    let work = WorkSpec::new(sweep_id.clone(), scope_key, None, bindings)?;
    store.seed_works(&[work]).await?;
    store.create_sweep_manifest(&sweep_id, &account_id).await?;
    let lease = claim(store, sweep, at).await?;
    let key = ResourceKey {
        account: account_id,
        region: Region::parse(region)?,
        ..resource(id)?
    };
    let observed = Observation::new(
        key,
        observation(id, value)?.record().clone(),
        &Exclusions::none(),
        None,
    )?;
    store
        .commit_page(page(
            lease,
            vec![observed],
            true,
            None,
            at.saturating_add(1),
        ))
        .await?;
    Ok(())
}

async fn given_removed_account_when_sweep_runs_for_other_account_then_its_scopes_are_unobserved<
    S: InventoryStore + UnobservedScopesCapability,
>(
    store: &S,
) -> TestResult {
    observe_account_scope(
        store,
        "account-a-1",
        "111111111111",
        "us-east-1",
        "a",
        "va",
        100,
    )
    .await?;
    observe_account_scope(
        store,
        "account-b",
        "222222222222",
        "us-west-2",
        "b",
        "vb",
        200,
    )
    .await?;
    observe_account_scope(
        store,
        "account-a-2",
        "111111111111",
        "us-east-1",
        "a2",
        "va2",
        300,
    )
    .await?;
    let sweep = SweepId::new("account-a-2".to_owned())?;
    let unobserved = store.unobserved_scopes(&sweep, 1_000).await?;
    let account_b = unobserved
        .iter()
        .find(|item| item.scope.account().as_str() == "222222222222")
        .ok_or("account B scope should be unobserved")?;
    assert_eq!(account_b.last_observed_ms, 201);
    Ok(())
}

async fn complete_region<S: InventoryStore>(
    store: &S,
    sweep: &str,
    region: &str,
    id: &str,
    at: i64,
) -> TestResult<u64> {
    store.seed_works(&[work(sweep, region)?]).await?;
    let lease = claim(store, sweep, at).await?;
    let observed = vec![observation_for_region(id, "value", region)?];
    let commit = page(lease, observed, true, None, at.saturating_add(1));
    Ok(store.commit_page(commit).await?.stats.resources_deleted)
}

fn observation_for_region(id: &str, value: &str, region: &str) -> TestResult<Observation> {
    let key = ResourceKey {
        region: Region::parse(region)?,
        ..resource(id)?
    };
    Ok(Observation::new(
        key,
        observation(id, value)?.record().clone(),
        &Exclusions::none(),
        None,
    )?)
}

async fn given_partial_scope_when_current_then_not_reported_unobserved<
    S: InventoryStore + UnobservedScopesCapability,
>(
    store: &S,
) -> TestResult {
    complete_one(store, "observed-east", "us-east-1", "x", "v1", 100).await?;
    store
        .seed_works(&[work("partial-east", "us-east-1")?])
        .await?;
    let partial = claim(store, "partial-east", 200).await?;
    store
        .finish_scope(partial, ScopeOutcome::Partial, 201, None)
        .await?;
    let absent = store
        .unobserved_scopes(&SweepId::new("partial-east".to_owned())?, 1_000)
        .await?;
    assert!(
        absent.is_empty(),
        "partial scope must not be reported as unobserved"
    );
    Ok(())
}
async fn given_narrowed_regions_when_complete_then_skipped_scope_is_reported<
    S: InventoryStore + UnobservedScopesCapability,
>(
    store: &S,
) -> TestResult {
    complete_region(store, "wide-1", "us-east-1", "east", 100).await?;
    complete_region(store, "wide-1", "us-west-2", "west", 101).await?;
    assert_eq!(
        complete_region(store, "narrow-1", "us-east-1", "east", 1_000).await?,
        0
    );
    assert_eq!(
        complete_region(store, "narrow-2", "us-east-1", "east", 1_100).await?,
        0
    );
    assert_eq!(
        complete_region(store, "narrow-3", "us-east-1", "east", 1_200).await?,
        0
    );
    let west = ResourceKey {
        region: Region::parse("us-west-2")?,
        ..resource("west")?
    };
    assert!(store.projection_resource(&west).await?.is_some());
    let sweep = SweepId::new("narrow-3".to_owned())?;
    assert_eq!(summary(store, "narrow-3").await?.unobserved_scopes, 1);
    let unobserved = store.unobserved_scopes(&sweep, 5_000).await?;
    let [only] = unobserved.as_slice() else {
        return Err("expected one unobserved western scope".into());
    };
    assert_eq!(only.scope, scope("us-west-2")?);
    assert_eq!(only.last_observed_ms, 102);
    assert_eq!(only.age_ms, 4_898);
    complete_region(store, "wide-2", "us-east-1", "east", 6_000).await?;
    complete_region(store, "wide-2", "us-west-2", "west", 6_001).await?;
    assert!(
        store
            .unobserved_scopes(&SweepId::new("wide-2".to_owned())?, 7_000)
            .await?
            .is_empty()
    );
    Ok(())
}

async fn complete_one<S: InventoryStore>(
    store: &S,
    sweep: &str,
    region: &str,
    id: &str,
    value: &str,
    at: i64,
) -> TestResult {
    store.seed_works(&[work(sweep, region)?]).await?;
    let lease = claim(store, sweep, at).await?;
    let observed = observation_for_region(id, value, region)?;
    let result = store
        .commit_page(page(
            lease,
            vec![observed],
            true,
            None,
            at.saturating_add(1),
        ))
        .await?;
    assert_eq!(result.stats.resources_deleted, 0);
    Ok(())
}

async fn given_failed_scope_when_absent_then_age_uses_last_complete_observation<
    S: InventoryStore + UnobservedScopesCapability,
>(
    store: &S,
) -> TestResult {
    complete_one(store, "observed-east", "us-east-1", "x", "v1", 100).await?;
    store
        .seed_works(&[work("failed-east", "us-east-1")?])
        .await?;
    let failed = claim(store, "failed-east", 200).await?;
    store
        .finish_scope(failed, ScopeOutcome::Failed, 201, None)
        .await?;
    complete_one(store, "observed-west", "us-west-2", "y", "v2", 300).await?;
    let sweep = SweepId::new("observed-west".to_owned())?;
    let unobserved = store.unobserved_scopes(&sweep, 1_000).await?;
    let [only] = unobserved.as_slice() else {
        return Err("expected the previously observed east scope to be unobserved".into());
    };
    assert_eq!(only.scope, scope("us-east-1")?);
    assert_eq!(only.last_observed_ms, 101);
    assert_eq!(only.age_ms, 899);
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

async fn given_vanishing_resource_when_two_complete_sweeps_miss_it_then_it_is_deleted<
    S: InventoryStore,
>(
    store: &S,
) -> TestResult {
    seed(store, work("s1", "us-east-1")?).await?;
    let sweep1 = SweepId::new("s1".to_owned())?;
    let lease1 = store
        .claim(&sweep1, 100, 20)
        .await?
        .ok_or("missing lease s1")?;
    let commit1 = page(
        lease1,
        vec![observation("gone", "alpha")?, observation("keep", "beta")?],
        true,
        None,
        101,
    );
    let result1 = store.commit_page(commit1).await?;
    assert_eq!(result1.stats.resources_inserted, 2);

    let gone_key = resource("gone")?;
    let keep_key = resource("keep")?;
    let first_drain = store.drain_projection(100).await?;
    for pending in first_drain {
        let key = pending.key.ok_or("missing projection key")?;
        store
            .ack_projection(ProjectionAck::new(key, pending.revision)?)
            .await?;
    }
    assert!(store.projection_resource(&gone_key).await?.is_some());
    assert!(store.projection_resource(&keep_key).await?.is_some());

    seed(store, work("s2", "us-east-1")?).await?;
    let sweep2 = SweepId::new("s2".to_owned())?;
    let lease2 = store
        .claim(&sweep2, 200, 20)
        .await?
        .ok_or("missing lease s2")?;
    let commit2 = page(lease2, vec![observation("keep", "beta")?], true, None, 201);
    let result2 = store.commit_page(commit2).await?;
    assert_eq!(result2.stats.resources_deleted, 0);
    assert!(store.projection_resource(&gone_key).await?.is_some());

    seed(store, work("s3", "us-east-1")?).await?;
    let sweep3 = SweepId::new("s3".to_owned())?;
    let lease3 = store
        .claim(&sweep3, 300, 20)
        .await?
        .ok_or("missing lease s3")?;
    let commit3 = page(lease3, vec![observation("keep", "beta")?], true, None, 301);
    let result3 = store.commit_page(commit3).await?;
    assert_eq!(result3.stats.resources_deleted, 1);

    assert!(store.projection_resource(&gone_key).await?.is_none());
    assert!(store.projection_resource(&keep_key).await?.is_some());

    let second_drain = store.drain_projection(100).await?;
    let mut found_deletion = false;
    for pending in second_drain {
        if let Some(key) = pending.key {
            store
                .ack_projection(ProjectionAck::new(key.clone(), pending.revision)?)
                .await?;
            if key == gone_key && pending.deleted {
                found_deletion = true;
            }
        }
    }
    assert!(
        found_deletion,
        "outbox must carry the deletion for the graph projection"
    );
    Ok(())
}

async fn given_deleted_resource_with_edge_when_two_complete_sweeps_miss_it_then_edges_are_cleared<
    S: InventoryStore,
>(
    store: &S,
) -> TestResult {
    seed_source_with_target_name(store, "gone", "keep").await?;

    let gone_key = resource("gone")?;
    assert!(store.projection_resource(&gone_key).await?.is_some());
    assert!(
        store
            .projection_edges_for(&gone_key, None, 10)
            .await?
            .edges
            .is_empty()
    );

    observe_source_and_target(store, "s2", "gone", "keep").await?;
    assert!(
        !store
            .projection_edges_for(&gone_key, None, 10)
            .await?
            .edges
            .is_empty()
    );

    let lease3 = seed_claim(store, "s3", 300).await?;
    let commit3 = page(lease3, vec![observation("keep", "keep")?], true, None, 301);
    let result3 = store.commit_page(commit3).await?;
    assert_eq!(result3.stats.resources_deleted, 0);
    assert!(store.projection_resource(&gone_key).await?.is_some());

    let lease4 = seed_claim(store, "s4", 400).await?;
    let commit4 = page(lease4, vec![observation("keep", "keep")?], true, None, 401);
    let result4 = store.commit_page(commit4).await?;
    assert_eq!(result4.stats.resources_deleted, 1);

    let drain = store.drain_projection(100).await?;
    let mut gone_deleted = false;
    for pending in drain {
        if let Some(key) = pending.key {
            store
                .ack_projection(ProjectionAck::new(key.clone(), pending.revision)?)
                .await?;
            if key == gone_key && pending.deleted {
                gone_deleted = true;
            }
        }
    }
    assert!(
        gone_deleted,
        "outbox must carry the deletion event for the removed resource"
    );

    assert!(!has(store, "gone").await?);
    assert!(store.projection_resource(&gone_key).await?.is_none());
    assert!(
        store
            .projection_edges_for(&gone_key, None, 10)
            .await?
            .edges
            .is_empty()
    );
    Ok(())
}

async fn seed_source_with_target_name<S: InventoryStore>(
    store: &S,
    source_id: &str,
    target_name: &str,
) -> TestResult {
    let work = relationship_work("s1", "us-east-1", Some(target_name), None)?;
    store.seed_works(&[work]).await?;
    let lease = claim(store, "s1", 100).await?;
    let commit = page(
        lease,
        vec![observation(source_id, "alpha")?],
        true,
        None,
        101,
    );
    store.commit_page(commit).await?;
    Ok(())
}

async fn observe_source_and_target<S: InventoryStore>(
    store: &S,
    sweep: &str,
    source_id: &str,
    target_id: &str,
) -> TestResult {
    let work = relationship_work(sweep, "us-east-1", Some(target_id), None)?;
    store.seed_works(&[work]).await?;
    let lease = claim(store, sweep, 200).await?;
    let observations = vec![
        observation(source_id, "alpha")?,
        observation(target_id, target_id)?,
    ];
    let mut commit = page(lease, observations, true, None, 201);
    commit.unresolved.push(unresolved(source_id, target_id)?);
    store.commit_page(commit).await?;
    Ok(())
}

async fn seed<S: InventoryStore>(store: &S, spec: WorkSpec) -> TestResult {
    store.seed_works(&[spec]).await?;
    Ok(())
}
