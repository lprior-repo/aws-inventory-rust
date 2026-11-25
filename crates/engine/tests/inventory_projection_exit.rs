#[path = "inventory_store/support.rs"]
#[allow(
    dead_code,
    reason = "shared via #[path]; each test target uses a subset of the helpers"
)]
mod support;

use inventory_core::inventory::ResourceId;
use inventory_engine::inventory::{ProjectionAck, SweepId, run_projection};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::collections::BTreeMap;
use support::{TestResult, fixture, observation, page, work};

#[tokio::test]
async fn given_one_changed_resource_when_reprojecting_then_only_that_key_is_sent() -> TestResult {
    let (_directory, store, _) = fixture().await?;

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
                observation("thing-2", "beta")?,
            ],
            true,
            None,
            201,
        ))
        .await?;

    let pending = store.drain_projection(100).await?;
    assert_eq!(
        pending.len(),
        1,
        "only the changed resource should be pending"
    );
    let expected_id = ResourceId::parse("thing-1")?;
    let Some(entry) = pending.first() else {
        return Err("expected one pending entry".into());
    };
    let Some(key) = entry.key.as_ref() else {
        return Err("pending entry missing key".into());
    };
    assert_eq!(
        key.id, expected_id,
        "pending entry should be for the changed resource"
    );

    Ok(())
}

#[tokio::test]
async fn given_incremental_and_rebuilt_graphs_when_compared_then_they_are_equal() -> TestResult {
    let (directory, store, _) = fixture().await?;

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
    assert_eq!(
        incremental_keys.len(),
        3,
        "incremental projection delivered 3 resources"
    );

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?;
    sqlx::query("UPDATE metadata SET value=100000 WHERE key='projection_pending'")
        .execute(&pool)
        .await?;
    pool.close().await;

    store
        .seed_work(work("overflow-sweep", "us-east-1")?)
        .await?;
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

    let pending = store.drain_projection(100).await?;
    let marker = pending
        .iter()
        .find(|e| e.full_rebuild)
        .ok_or("missing rebuild marker after overflow")?;
    store
        .ack_projection(ProjectionAck::full_rebuild(marker.revision)?)
        .await?;

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

    assert_eq!(
        rebuild_keys.keys().collect::<Vec<_>>(),
        incremental_keys.keys().collect::<Vec<_>>(),
        "rebuild enumeration must match incremental resource set"
    );

    Ok(())
}

#[tokio::test]
async fn given_offline_falkordb_when_sweeping_then_the_queue_is_retained_for_later_delivery()
-> TestResult {
    let (_directory, store, _) = fixture().await?;

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

    let backend =
        inventory_graph::falkor::FalkorDb::connect("redis://127.0.0.1:1", "offline-test")?;
    let report = run_projection(&store, &backend, 10).await?;
    assert!(report.offline, "report should indicate FalkorDB is offline");
    assert_eq!(report.acknowledged, 0, "no acknowledgments when offline");

    let pending = store.drain_projection(100).await?;
    assert!(
        !pending.is_empty(),
        "queue should still have entries after offline attempt"
    );

    let deliverable = pending.iter().filter(|e| e.key.is_some()).count();
    assert!(
        deliverable > 0,
        "entries remain deliverable after FalkorDB returns"
    );

    Ok(())
}

#[tokio::test]
async fn given_outbox_overflow_when_rebuilt_then_graph_is_complete() -> TestResult {
    let (directory, store, _) = fixture().await?;

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

    for entry in store.drain_projection(100).await? {
        store
            .ack_projection(ProjectionAck::new(
                entry.key.clone().ok_or("missing key")?,
                entry.revision,
            )?)
            .await?;
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?;
    sqlx::query("UPDATE metadata SET value=100000 WHERE key='projection_pending'")
        .execute(&pool)
        .await?;
    pool.close().await;

    store
        .seed_work(work("overflow-sweep", "us-east-1")?)
        .await?;
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

    let pending = store.drain_projection(100).await?;
    let Some(first) = pending.first() else {
        return Err("no entries after overflow".into());
    };
    assert!(
        first.full_rebuild,
        "overflow should trigger rebuild marker first"
    );

    store
        .ack_projection(ProjectionAck::full_rebuild(first.revision)?)
        .await?;

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
