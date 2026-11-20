//! M6.1 Exit Evidence Tests
//!
//! Four exit-evidence sentences, each mapped to a test:
//! 1. After a re-sweep with one change, only that key is sent
//! 2. A rebuilt graph equals the incrementally updated one (resource-set
//!    equality at the store snapshot; graph CONTENT equality needs live FalkorDB)
//! 3. With FalkorDB stopped a sweep completes and the queue is retained
//!    (the drain-after-return half needs live FalkorDB; see `graph_retry.rs`)
//! 4. Overflowing the queue ends in a full rebuild enumeration with nothing lost
#[path = "inventory_store/support.rs"]
mod support;

use inventory_engine::inventory::{ProjectionAck, SweepId, run_projection};
use inventory_core::inventory::ResourceId;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::collections::BTreeMap;
use support::{
    TestResult, fixture, observation, page, work,
};

/// Test 1: After a re-sweep with one change, only that key is sent.
#[tokio::test]
async fn given_one_changed_resource_when_reprojecting_then_only_that_key_is_sent() -> TestResult {
    let (_directory, store, _) = fixture().await?;

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

    // Drain and ack all initial projections
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
    assert_eq!(delivered.len(), 2, "initial sweep delivers 2 resources");

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
    let expected_id = ResourceId::parse("thing-1")?;
    let Some(entry) = pending.first() else {
        return Err("expected one pending entry".into());
    };
    let Some(key) = entry.key.as_ref() else {
        return Err("pending entry missing key".into());
    };
    assert_eq!(
        key.id,
        expected_id,
        "pending entry should be for the changed resource"
    );

    Ok(())
}

/// Test 2: A rebuilt graph equals the incrementally updated one.
#[tokio::test]
async fn given_incremental_and_rebuilt_graphs_when_compared_then_they_are_equal() -> TestResult {
    let (directory, store, _) = fixture().await?;

    // Add three resources incrementally
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
                observation("inc-3", "c")?,
            ],
            true,
            None,
            101,
        ))
        .await?;

    // Track what incremental projection would deliver
    let mut incremental_keys = BTreeMap::new();
    for entry in store.drain_projection(100).await? {
        if let Some(key) = &entry.key {
            incremental_keys.insert(key.id.to_string(), !entry.deleted);
        }
        store
            .ack_projection(ProjectionAck::new(
                entry.key.clone().ok_or("missing key")?,
                entry.revision,
            )?)
            .await?;
    }
    assert_eq!(incremental_keys.len(), 3, "incremental projection delivered 3 resources");

    // Now force overflow BEFORE making a change, so the change triggers a rebuild marker
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

    // Make a change that should trigger a rebuild marker due to overflow
    store.seed_work(work("overflow-sweep", "us-east-1")?).await?;
    let lease = store
        .claim(&SweepId::new("overflow-sweep".to_owned())?, 200, 20)
        .await?
        .ok_or("missing overflow-sweep lease")?;
    store
        .commit_page(page(
            lease,
            vec![observation("inc-1", "changed")?],
            true,
            None,
            201,
        ))
        .await?;

    // The rebuild marker is now in the outbox
    let pending = store.drain_projection(100).await?;
    let marker = pending
        .iter()
        .find(|e| e.full_rebuild)
        .ok_or("missing rebuild marker after overflow")?;
    store
        .ack_projection(ProjectionAck::full_rebuild(marker.revision)?)
        .await?;

    // Enumerate live resources via projection snapshot (same source as rebuild)
    let mut snapshot = store.begin_projection_snapshot().await?;
    let mut rebuild_keys = BTreeMap::new();
    let mut after = None;
    for _ in 0..100 {
        let page = snapshot.resources(after.take(), 1000).await?;
        for r in &page {
            rebuild_keys.insert(r.key.id.to_string(), true);
        }
        if page.len() < 1000 {
            break;
        }
    }

    // The rebuilt graph (via snapshot) must contain exactly the same resources
    // that the incremental graph would have produced.
    assert_eq!(
        rebuild_keys.keys().collect::<Vec<_>>(),
        incremental_keys.keys().collect::<Vec<_>>(),
        "rebuild enumeration must match incremental resource set"
    );

    Ok(())
}

/// Test 3: With FalkorDB stopped a sweep completes and the queue is retained.
#[tokio::test]
async fn given_offline_falkordb_when_sweeping_then_the_queue_is_retained_for_later_delivery() -> TestResult {
    let (_directory, store, _) = fixture().await?;

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

    // Verify entries are still deliverable (not corrupted by offline attempt)
    let deliverable = pending
        .iter()
        .filter(|e| e.key.is_some())
        .count();
    assert!(
        deliverable > 0,
        "entries remain deliverable after FalkorDB returns"
    );

    Ok(())
}

/// Test 4: Overflowing the queue ends in a graph equal to a rebuild.
#[tokio::test]
async fn given_outbox_overflow_when_rebuilt_then_graph_is_complete() -> TestResult {
    let (directory, store, _) = fixture().await?;

    // Add multiple resources
    store.seed_work(work("overflow", "us-east-1")?).await?;
    let lease = store
        .claim(&SweepId::new("overflow".to_owned())?, 100, 20)
        .await?
        .ok_or("missing lease")?;
    store
        .commit_page(page(
            lease,
            vec![
                observation("ov-1", "a")?,
                observation("ov-2", "b")?,
                observation("ov-3", "c")?,
            ],
            true,
            None,
            101,
        ))
        .await?;

    // Ack the incremental projections to simulate normal delivery
    for entry in store.drain_projection(100).await? {
        store
            .ack_projection(ProjectionAck::new(
                entry.key.clone().ok_or("missing key")?,
                entry.revision,
            )?)
            .await?;
    }

    // Force outbox overflow by triggering a change while outbox is over limit
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

    // Make a change that should trigger a rebuild marker
    store.seed_work(work("overflow-sweep", "us-east-1")?).await?;
    let lease = store
        .claim(&SweepId::new("overflow-sweep".to_owned())?, 200, 20)
        .await?
        .ok_or("missing overflow-sweep lease")?;
    store
        .commit_page(page(
            lease,
            vec![observation("ov-1", "changed")?],
            true,
            None,
            201,
        ))
        .await?;

    // Draining should yield a rebuild marker first
    let pending = store.drain_projection(100).await?;
    let Some(first) = pending.first() else {
        return Err("no entries after overflow".into());
    };
    assert!(
        first.full_rebuild,
        "overflow should trigger rebuild marker first"
    );

    // Acknowledge the rebuild marker
    store
        .ack_projection(ProjectionAck::full_rebuild(first.revision)?)
        .await?;

    // Verify all original resources are still present via projection snapshot
    let mut snapshot = store.begin_projection_snapshot().await?;
    let mut rebuild_keys = BTreeMap::new();
    let mut after = None;
    for _ in 0..100 {
        let page = snapshot.resources(after.take(), 1000).await?;
        for r in &page {
            rebuild_keys.insert(r.key.id.to_string(), true);
        }
        if page.len() < 1000 {
            break;
        }
    }

    for expected in ["ov-1", "ov-2", "ov-3"] {
        assert!(
            rebuild_keys.contains_key(expected),
            "rebuild must include resource {}",
            expected
        );
    }

    assert!(
        !store.projection_state().await?.rebuild_required,
        "rebuild should not be required after acknowledging marker"
    );

    Ok(())
}
