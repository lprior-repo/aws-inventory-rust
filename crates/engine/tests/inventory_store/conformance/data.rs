use super::*;
use inventory_engine::inventory::WorkSpec;

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
    let (_guard, store, _) = make().await?;
    given_vanishing_resource_when_two_complete_sweeps_miss_it_then_it_is_deleted(&store).await?;
    let (_guard, store, _) = make().await?;
    given_deleted_resource_with_edge_when_two_complete_sweeps_miss_it_then_edges_are_cleared(
        &store,
    )
    .await?;
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
    // First sweep: observe resource.
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

    // Both resources should be in the projection after draining.
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

    // Second sweep: complete, misses the resource (only "keep" observed).
    seed(store, work("s2", "us-east-1")?).await?;
    let sweep2 = SweepId::new("s2".to_owned())?;
    let lease2 = store
        .claim(&sweep2, 200, 20)
        .await?
        .ok_or("missing lease s2")?;
    let commit2 = page(lease2, vec![observation("keep", "beta")?], true, None, 201);
    let result2 = store.commit_page(commit2).await?;
    assert_eq!(result2.stats.resources_deleted, 0);
    // After first complete miss, resource is still stored.
    assert!(store.projection_resource(&gone_key).await?.is_some());

    // Third sweep: complete, misses the resource again.
    seed(store, work("s3", "us-east-1")?).await?;
    let sweep3 = SweepId::new("s3".to_owned())?;
    let lease3 = store
        .claim(&sweep3, 300, 20)
        .await?
        .ok_or("missing lease s3")?;
    let commit3 = page(lease3, vec![observation("keep", "beta")?], true, None, 301);
    let result3 = store.commit_page(commit3).await?;
    assert_eq!(result3.stats.resources_deleted, 1);

    // After two complete misses, the resource should be gone.
    assert!(store.projection_resource(&gone_key).await?.is_none());
    assert!(store.projection_resource(&keep_key).await?.is_some());

    // The deletion outbox should carry an entry for the removed resource.
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
    // Sweep 1: observe "gone" with unresolved reference to "keep".
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

    // Sweep 2: observe "keep" and "gone" again. The reference is now resolved
    // and an edge is created.
    observe_source_and_target(store, "s2", "gone", "keep").await?;
    assert!(
        !store
            .projection_edges_for(&gone_key, None, 10)
            .await?
            .edges
            .is_empty()
    );

    // Sweep 3: complete, misses "gone".
    let lease3 = seed_claim(store, "s3", 300).await?;
    let commit3 = page(lease3, vec![observation("keep", "keep")?], true, None, 301);
    let result3 = store.commit_page(commit3).await?;
    assert_eq!(result3.stats.resources_deleted, 0);
    assert!(store.projection_resource(&gone_key).await?.is_some());

    // Sweep 4: complete, misses "gone" again.
    let lease4 = seed_claim(store, "s4", 400).await?;
    let commit4 = page(lease4, vec![observation("keep", "keep")?], true, None, 401);
    let result4 = store.commit_page(commit4).await?;
    assert_eq!(result4.stats.resources_deleted, 1);

    // Drain the projection outbox: it must carry the deletion for "gone".
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

    // After two complete misses, the resource is gone and its edges are cleared.
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
