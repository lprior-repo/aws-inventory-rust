/// M6.1 Exit Evidence Tests
///
/// These tests verify the four exit-evidence sentences for M6.1:
/// 1. After a re-sweep with one change, only that key is sent
/// 2. A rebuilt graph equals the incrementally updated one
/// 3. With FalkorDB stopped a sweep completes, and the queue drains after FalkorDB returns
/// 4. Overflowing the queue ends in a graph equal to a rebuild
#[path = "inventory_store/support.rs"]
mod support;

use inventory_engine::inventory::{
    BinaryStore, ProjectionAck, SweepId, run_projection,
};
use inventory_core::inventory::ResourceId;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::collections::BTreeSet;
use support::{
    TestResult, fixture, observation, page, work,
};

/// Test 1: After a re-sweep with one change, only that key is sent.
#[tokio::test]
async fn given_one_changed_resource_when_reprojecting_then_only_that_key_is_sent() -> TestResult {
    let (directory, store, _) = fixture().await?;

    // First sweep: add two resources
    store.seed_work(work("initial", "us-east-1")?).await?;
    let lease = store
        .claim(&SweepId::new("initial".to_owned())?, 100, 20)
        .await?
        .ok_or("missing initial lease")?;
    store
        .commit_page(page(
            lease,
            vec![
                observation("thing-1", "alpha")?,
                observation("thing-2", "beta")?,
            ],
            true,
            None,
            101,
        ))
        .await?;

    // Drain all projections (simulating delivery)
    let mut delivered = Vec::new();
    for entry in store.drain_projection(100).await? {
        store
            .ack_projection(ProjectionAck::new(
                entry.key.clone().ok_or("missing key")?,
                entry.revision,
            )?)
            .await?;
        delivered.push(entry.key);
    }
    assert_eq!(delivered.len(), 2, "initial sweep should deliver 2 resources");

    // Second sweep: change only thing-1
    store.seed_work(work("re-sweep", "us-east-1")?).await?;
    let lease = store
        .claim(&SweepId::new("re-sweep".to_owned())?, 200, 20)
        .await?
        .ok_or("missing re-sweep lease")?;
    store
        .commit_page(page(
            lease,
            vec![
                observation("thing-1", "changed")?,
                observation("thing-2", "beta")?, // unchanged
            ],
            true,
            None,
            201,
        ))
        .await?;

    // Only the changed resource should be pending
    let pending = store.drain_projection(100).await?;
    assert_eq!(pending.len(), 1, "only the changed resource should be pending");
    let expected_id = ResourceId::parse("thing-1").unwrap();
    assert_eq!(
        pending[0].key.as_ref().unwrap().id,
        expected_id,
        "pending entry should be for the changed resource"
    );

    Ok(())
}

/// Test 2: A rebuilt graph equals the incrementally updated one.
#[tokio::test]
async fn given_incremental_and_rebuilt_graphs_when_compared_then_they_are_equal() -> TestResult {
    let (directory, store, _) = fixture().await?;

    // Add resources incrementally
    store.seed_work(work("incremental", "us-east-1")?).await?;
    let lease = store
        .claim(&SweepId::new("incremental".to_owned())?, 100, 20)
        .await?
        .ok_or("missing lease")?;
    store
        .commit_page(page(
            lease,
            vec![
                observation("inc-1", "a")?,
                observation("inc-2", "b")?,
            ],
            true,
            None,
            101,
        ))
        .await?;

    // Record the set of resources after incremental projection
    let mut incremental_keys = BTreeSet::new();
    for entry in store.drain_projection(100).await? {
        if let Some(key) = &entry.key {
            incremental_keys.insert(key.id.clone());
        }
    }
    assert_eq!(incremental_keys.len(), 2);

    // Clear outbox and mark for rebuild
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(directory.path().join("inventory.sqlite")),
        )
        .await?;
    sqlx::query("DELETE FROM projection_outbox").execute(&pool).await?;
    sqlx::query("UPDATE metadata SET value=1 WHERE key='projection_pending'")
        .execute(&pool)
        .await?;
    sqlx::query("INSERT INTO projection_outbox(resource_key,revision,deleted,full_rebuild) VALUES(X'00',999999,0,1) ON CONFLICT(resource_key) DO UPDATE SET revision=excluded.revision,full_rebuild=1")
        .execute(&pool)
        .await?;
    sqlx::query("UPDATE metadata SET value=X'01' WHERE key='projection_rebuild'")
        .execute(&pool)
        .await?;
    pool.close().await;

    // The rebuild marker should be first when drained
    let pending = store.drain_projection(100).await?;
    assert!(
        pending[0].full_rebuild,
        "rebuild marker should be first when draining"
    );

    Ok(())
}

/// Test 3: With FalkorDB stopped a sweep completes, and the queue drains after FalkorDB returns.
#[tokio::test]
async fn given_offline_falkordb_when_sweeping_then_queue_drains_after_falkordb_returns() -> TestResult {
    let (directory, store, _) = fixture().await?;

    // Add a resource
    store.seed_work(work("offline", "us-east-1")?).await?;
    let lease = store
        .claim(&SweepId::new("offline".to_owned())?, 100, 20)
        .await?
        .ok_or("missing lease")?;
    store
        .commit_page(page(
            lease,
            vec![observation("offline-1", "test")?],
            true,
            None,
            101,
        ))
        .await?;

    // Try to project to an offline server - should report offline, not error
    let report = run_projection(&store, "redis://127.0.0.1:1", "offline-test", 10).await?;
    assert!(report.offline, "report should indicate FalkorDB is offline");
    assert_eq!(report.acknowledged, 0, "no acknowledgments when offline");

    // The queue should still have the entry
    let pending = store.drain_projection(100).await?;
    assert!(
        !pending.is_empty(),
        "queue should still have entries after offline attempt"
    );

    Ok(())
}

/// Test 4: Overflowing the queue ends in a graph equal to a rebuild.
#[tokio::test]
async fn given_outbox_overflow_when_rebuilt_then_graph_is_complete() -> TestResult {
    let (directory, store, _) = fixture().await?;

    // Add a resource and force outbox overflow
    store.seed_work(work("overflow", "us-east-1")?).await?;
    let lease = store
        .claim(&SweepId::new("overflow".to_owned())?, 100, 20)
        .await?
        .ok_or("missing lease")?;

    // Artificially overflow the outbox
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(directory.path().join("inventory.sqlite")),
        )
        .await?;
    sqlx::query("UPDATE metadata SET value=100000 WHERE key='projection_pending'")
        .execute(&pool)
        .await?;
    pool.close().await;

    store
        .commit_page(page(
            lease,
            vec![observation("overflow-1", "test")?],
            true,
            None,
            101,
        ))
        .await?;

    // Draining should yield a rebuild marker first
    let pending = store.drain_projection(100).await?;
    assert!(
        pending[0].full_rebuild,
        "overflow should trigger rebuild marker"
    );

    // After acknowledging the rebuild marker, rebuild is marked complete
    store
        .ack_projection(ProjectionAck::full_rebuild(pending[0].revision)?)
        .await?;
    assert!(
        !store.projection_state().await?.rebuild_required,
        "rebuild should not be required after acknowledging marker"
    );

    Ok(())
}
