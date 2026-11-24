use super::*;

#[tokio::test]
pub(super) async fn given_a_partial_scope_and_two_complete_misses_then_only_the_second_complete_miss_deletes()
-> TestResult {
    let (directory, store, _) = fixture().await?;
    store.seed_work(work("seen", "us-east-1")?).await?;
    let seen = store
        .claim(&SweepId::new("seen".to_owned())?, 100, 20)
        .await?
        .ok_or("missing lease")?;
    store
        .commit_page(page(
            seen,
            vec![observation("thing-1", "alpha")?],
            true,
            None,
            101,
        ))
        .await?;

    store.seed_work(work("partial", "us-east-1")?).await?;
    let partial = store
        .claim(&SweepId::new("partial".to_owned())?, 200, 20)
        .await?
        .ok_or("missing partial lease")?;
    store
        .finish_scope(partial, ScopeOutcome::Inaccessible, 201, None)
        .await?;
    assert!(
        store
            .projection_resource(&resource("thing-1")?)
            .await?
            .is_some()
    );

    for sweep in ["complete-one", "complete-two"] {
        store.seed_work(work(sweep, "us-east-1")?).await?;
        let lease = store
            .claim(&SweepId::new(sweep.to_owned())?, 300, 20)
            .await?
            .ok_or("missing complete lease")?;
        store
            .commit_page(page(lease, Vec::new(), true, None, 301))
            .await?;
        let remains = store
            .projection_resource(&resource("thing-1")?)
            .await?
            .is_some();
        assert_eq!(remains, sweep == "complete-one");
    }
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?;
    let typed_rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM thing_table")
        .fetch_one(&pool)
        .await?;
    pool.close().await;
    assert_eq!(typed_rows, 0);
    Ok(())
}

async fn pending_store() -> TestResult<(tempfile::TempDir, BinaryStore)> {
    let (directory, store, _) = fixture().await?;
    let queued = work("rebuild-request", "us-east-1")?;
    store.seed_works(&[queued]).await?;
    let sweep = SweepId::new("rebuild-request".to_owned())?;
    let lease = store.claim(&sweep, 100, 20).await?.ok_or("missing lease")?;
    let observations = vec![observation("thing-1", "alpha")?];
    let commit = page(lease, observations, true, None, 101);
    store.commit_page(commit).await?;
    Ok((directory, store))
}

#[tokio::test]
async fn given_pending_projection_work_when_graph_rebuild_is_requested_then_one_full_rebuild_replaces_it()
-> TestResult {
    let (_directory, store) = pending_store().await?;
    assert!(!store.projection_state().await?.rebuild_required);
    store.request_graph_rebuild().await?;
    let state = store.projection_state().await?;
    assert_eq!(state.pending, 1);
    assert!(state.rebuild_required);
    let entries = store.drain_projection(10).await?;
    assert_eq!(entries.len(), 1);
    let marker = entries.first().ok_or("missing rebuild marker")?;
    assert!(marker.full_rebuild);
    let ack = inventory_engine::inventory::ProjectionAck::full_rebuild(marker.revision)?;
    store.ack_projection(ack).await?;
    let after = store.projection_state().await?;
    assert_eq!(after.pending, 0);
    assert!(!after.rebuild_required);
    Ok(())
}
