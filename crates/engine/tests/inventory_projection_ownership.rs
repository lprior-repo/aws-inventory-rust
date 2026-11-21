#[path = "inventory_store/support.rs"]
#[allow(
    dead_code,
    reason = "shared via #[path]; each test target uses a subset of the helpers"
)]
mod support;

use inventory_engine::inventory::{
    BinaryStore, ProjectionAck, ProjectionError, SweepId, run_projection,
};
use sqlx::{Connection, SqliteConnection, sqlite::SqliteConnectOptions};
use support::{TestResult, fixture, observation, page, work};

#[tokio::test]
async fn given_another_delivery_owner_when_any_handle_projects_then_it_fails_busy_without_blocking_resource_writers()
-> TestResult {
    // Given: an OS-managed SQLite delivery lock on the separate persistent file.
    let (directory, store, _) = fixture().await?;
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    let mut lock = SqliteConnection::connect_with(
        &SqliteConnectOptions::new()
            .filename(
                directory
                    .path()
                    .join("inventory.sqlite.projection-lock.sqlite"),
            )
            .create_if_missing(true),
    )
    .await?;
    sqlx::query("BEGIN IMMEDIATE").execute(&mut lock).await?;
    // When: an independent handle attempts projection, no network is needed.
    assert!(matches!(
        run_projection(&reopened, "redis://127.0.0.1:1", "ownership", 1).await,
        Err(ProjectionError::DeliveryBusy)
    ));
    // Then: ordinary resource commits still progress while delivery is owned.
    store
        .seed_work(work("writer-progress", "us-east-1")?)
        .await?;
    let lease = store
        .claim(&SweepId::new("writer-progress".into())?, 1, 100)
        .await?
        .ok_or("missing writer lease")?;
    store
        .commit_page(page(
            lease,
            vec![observation("writer-progress", "value")?],
            true,
            None,
            2,
        ))
        .await?;
    assert!(store.projection_state().await?.pending > 0);
    lock.close().await?;
    let report = run_projection(&reopened, "redis://127.0.0.1:1", "ownership", 1).await?;
    assert!(report.offline);
    assert_eq!(report.acknowledged, 0);
    Ok(())
}

#[tokio::test]
async fn given_superseded_and_replayed_acknowledgements_when_applied_then_only_removed_rows_are_counted()
-> TestResult {
    let (directory, store, _) = fixture().await?;
    store.seed_work(work("overflow", "us-east-1")?).await?;
    let lease = store
        .claim(&SweepId::new("overflow".into())?, 1, 50)
        .await?
        .ok_or("missing overflow lease")?;
    let mut raw = SqliteConnection::connect_with(
        &SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
    )
    .await?;
    sqlx::query("UPDATE metadata SET value=100000 WHERE key='projection_pending'")
        .execute(&mut raw)
        .await?;
    raw.close().await?;
    store
        .commit_page(page(
            lease,
            vec![observation("one", "seed")?],
            true,
            None,
            2,
        ))
        .await?;
    let marker = store
        .drain_projection(1000)
        .await?
        .into_iter()
        .find(|entry| entry.full_rebuild)
        .ok_or("missing overflow rebuild marker")?;
    assert_eq!(
        store
            .ack_projection(ProjectionAck::full_rebuild(marker.revision)?)
            .await?,
        1
    );
    assert_eq!(
        store
            .ack_projection(ProjectionAck::full_rebuild(marker.revision)?)
            .await?,
        0
    );
    let mut pending = Vec::new();
    for (sweep, value, at) in [("old", "alice", 1), ("new", "bob", 101)] {
        store.seed_work(work(sweep, "us-east-1")?).await?;
        let lease = store
            .claim(&SweepId::new(sweep.into())?, at, 50)
            .await?
            .ok_or("missing lease")?;
        store
            .commit_page(page(
                lease,
                vec![observation("one", value)?],
                true,
                None,
                at + 1,
            ))
            .await?;
        let entry = store
            .drain_projection(1000)
            .await?
            .into_iter()
            .find(|entry| entry.key.is_some())
            .ok_or("missing resource entry")?;
        pending.push(entry);
    }
    let [old, new] = pending.as_slice() else {
        return Err("wrong revision count".into());
    };
    let key = old.key.clone().ok_or("missing key")?;
    // When/Then: obsolete delivery is a no-op, newest delivery removes one row.
    assert_eq!(
        store
            .ack_projection(ProjectionAck::new(key.clone(), old.revision)?)
            .await?,
        0
    );
    assert_eq!(store.drain_projection(1000).await?, vec![new.clone()]);
    assert_eq!(
        store
            .ack_projection(ProjectionAck::new(key.clone(), new.revision)?)
            .await?,
        1
    );
    assert_eq!(
        store
            .ack_projection(ProjectionAck::new(key, new.revision)?)
            .await?,
        0
    );
    assert_eq!(store.projection_state().await?.pending, 0);
    Ok(())
}

#[tokio::test]
async fn given_delivery_ownership_in_another_process_when_projecting_then_it_is_busy() -> TestResult
{
    const CHILD_DATABASE: &str = "INVENTORY_PROJECTION_OWNERSHIP_CHILD_DATABASE";
    const TEST: &str =
        "given_delivery_ownership_in_another_process_when_projecting_then_it_is_busy";
    if let Some(path) = std::env::var_os(CHILD_DATABASE) {
        // Child process opens its own pools, never inheriting an in-memory lock.
        let store = BinaryStore::open(std::path::Path::new(&path)).await?;
        assert!(matches!(
            run_projection(&store, "redis://127.0.0.1:1", "ownership", 1).await,
            Err(ProjectionError::DeliveryBusy)
        ));
        return Ok(());
    }
    let (directory, _store, _) = fixture().await?;
    let mut lock = SqliteConnection::connect_with(
        &SqliteConnectOptions::new()
            .filename(
                directory
                    .path()
                    .join("inventory.sqlite.projection-lock.sqlite"),
            )
            .create_if_missing(true),
    )
    .await?;
    sqlx::query("BEGIN IMMEDIATE").execute(&mut lock).await?;
    let output = tokio::time::timeout(
        std::time::Duration::from_secs(20),
        tokio::process::Command::new(std::env::current_exe()?)
            .args(["--exact", TEST, "--nocapture"])
            .env(CHILD_DATABASE, directory.path().join("inventory.sqlite"))
            .kill_on_drop(true)
            .output(),
    )
    .await??;
    assert!(
        output.status.success(),
        "cross-process delivery exclusion failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("1 passed"),
        "child must execute the exact ownership test"
    );
    lock.close().await?;
    Ok(())
}

#[tokio::test]
async fn given_relative_inventory_path_when_cwd_changes_then_delivery_uses_its_original_sidecar()
-> TestResult {
    const CHILD: &str = "INVENTORY_PROJECTION_RELATIVE_PATH_CHILD";
    const TEST: &str =
        "given_relative_inventory_path_when_cwd_changes_then_delivery_uses_its_original_sidecar";
    if std::env::var_os(CHILD).is_some() {
        // Only this exact test runs in the child, so cwd mutation cannot race
        // another test. Open relative paths before moving into an unrelated dir.
        let store = BinaryStore::open(std::path::Path::new("inventory.sqlite")).await?;
        let mut lock = SqliteConnection::connect_with(
            &SqliteConnectOptions::new()
                .filename("inventory.sqlite.projection-lock.sqlite")
                .create_if_missing(true),
        )
        .await?;
        sqlx::query("BEGIN IMMEDIATE").execute(&mut lock).await?;
        let elsewhere = tempfile::tempdir()?;
        std::env::set_current_dir(elsewhere.path())?;
        assert!(matches!(
            run_projection(&store, "redis://127.0.0.1:1", "ownership", 1).await,
            Err(ProjectionError::DeliveryBusy)
        ));
        lock.close().await?;
        return Ok(());
    }
    let (directory, _store, _) = fixture().await?;
    let output = tokio::time::timeout(
        std::time::Duration::from_secs(20),
        tokio::process::Command::new(std::env::current_exe()?)
            .args(["--exact", TEST, "--nocapture"])
            .env(CHILD, "1")
            .current_dir(directory.path())
            .kill_on_drop(true)
            .output(),
    )
    .await??;
    assert!(
        output.status.success(),
        "relative-path delivery lock failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("1 passed"),
        "child must execute the exact relative-path test"
    );
    Ok(())
}
