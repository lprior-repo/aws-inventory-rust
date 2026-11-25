use super::*;

pub(super) async fn run<S, F, G>(make: &F) -> TestResult
where
    S: InventoryStore,
    F: AsyncFn() -> TestResult<(G, S, Vec<u8>)>,
{
    let (_guard, store, _) = make().await?;
    given_unchanged_record_when_reobserved_then_no_projection_write(&store).await?;
    let (_guard, store, _) = make().await?;
    given_old_ack_when_new_revision_exists_then_new_revision_remains(&store).await?;
    let (_guard, store, _) = make().await?;
    given_pending_projection_work_when_rebuild_requested_then_one_full_rebuild_replaces_it(&store)
        .await
}

async fn first_pending<S: InventoryStore>(
    store: &S,
) -> TestResult<inventory_engine::inventory::PendingProjection> {
    let pending = store.drain_projection(10).await?;
    assert_eq!(pending.len(), 1);
    Ok(pending
        .into_iter()
        .next()
        .ok_or("missing projection entry")?)
}

async fn acknowledge<S: InventoryStore>(store: &S) -> TestResult {
    let pending = first_pending(store).await?;
    let key = pending.key.ok_or("missing resource key")?;
    let ack = ProjectionAck::new(key, pending.revision)?;
    assert_eq!(store.ack_projection(ack).await?, 1);
    assert_eq!(store.projection_state().await?.pending, 0);
    Ok(())
}

async fn given_unchanged_record_when_reobserved_then_no_projection_write<S: InventoryStore>(
    store: &S,
) -> TestResult {
    observe(store, "first", "thing-1", "alpha").await?;
    acknowledge(store).await?;
    let lease = seed_claim(store, "second", 200).await?;
    let observations = vec![observation("thing-1", "alpha")?];
    let commit = page(lease, observations, true, None, 201);
    let result = store.commit_page(commit).await?;
    assert_eq!(result.stats.resources_inserted, 0);
    assert_eq!(result.stats.resources_changed, 0);
    assert!(store.drain_projection(10).await?.is_empty());
    Ok(())
}

async fn given_old_ack_when_new_revision_exists_then_new_revision_remains<S: InventoryStore>(
    store: &S,
) -> TestResult {
    observe(store, "revision-one", "thing-1", "alpha").await?;
    let old = first_pending(store).await?;
    let old_key = old.key.clone().ok_or("missing resource key")?;
    let ack = ProjectionAck::new(old_key.clone(), old.revision)?;
    assert_eq!(store.ack_projection(ack).await?, 1);
    observe(store, "revision-two", "thing-1", "beta").await?;
    let stale = ProjectionAck::new(old_key, old.revision)?;
    assert_eq!(store.ack_projection(stale).await?, 0);
    let current = first_pending(store).await?;
    assert!(current.revision > old.revision);
    assert!(!current.deleted);
    assert_eq!(current.key, old.key);
    Ok(())
}

async fn given_pending_projection_work_when_rebuild_requested_then_one_full_rebuild_replaces_it<
    S: InventoryStore,
>(
    store: &S,
) -> TestResult {
    observe(store, "rebuild-request", "thing-1", "alpha").await?;
    assert!(!store.projection_state().await?.rebuild_required);
    store.request_graph_rebuild().await?;
    let state = store.projection_state().await?;
    assert_eq!(state.pending, 1);
    assert!(state.rebuild_required);
    let entries = store.drain_projection(10).await?;
    assert_eq!(entries.len(), 1);
    let marker = entries.first().ok_or("missing rebuild marker")?;
    assert!(marker.full_rebuild);
    let ack = ProjectionAck::full_rebuild(marker.revision)?;
    store.ack_projection(ack).await?;
    let after = store.projection_state().await?;
    assert_eq!(after.pending, 0);
    assert!(!after.rebuild_required);
    Ok(())
}
