use super::{TestResult, native::NativeGraph};
use inventory_engine::inventory::{
    BinaryStore, ProjectionAck, ProjectionError, StoreError, run_projection,
};
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::path::Path;

async fn connect(database: &Path) -> TestResult<SqlitePool> {
    Ok(SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(database))
        .await?)
}

pub(super) async fn request_rebuild(database: &Path) -> TestResult {
    // There is no public request-rebuild API. This transparent fixture operation
    // supplies only the persisted control marker normally created at outbox
    // overflow; every resource, tag and edge still comes from the real collector.
    let pool = connect(database).await?;
    let mut transaction = pool.begin().await?;
    let pending: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM projection_outbox")
        .fetch_one(&mut *transaction)
        .await?;
    assert_eq!(
        pending, 0,
        "rebuild fixture must not discard pending deliveries"
    );
    let revision: i64 = sqlx::query_scalar("UPDATE metadata SET value=CAST(value AS INTEGER)+1 WHERE key='projection_revision' RETURNING value")
        .fetch_one(&mut *transaction).await?;
    sqlx::query("INSERT INTO projection_outbox(resource_key,revision,deleted,full_rebuild) VALUES(X'00',?1,0,1)")
        .bind(revision).execute(&mut *transaction).await?;
    sqlx::query("UPDATE metadata SET value=1 WHERE key='projection_pending'")
        .execute(&mut *transaction)
        .await?;
    sqlx::query("UPDATE metadata SET value=X'01' WHERE key='projection_rebuild'")
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await?;
    pool.close().await;
    Ok(())
}

pub(super) async fn deliver_without_ack(
    store: &BinaryStore,
    graph: &NativeGraph,
    database: &Path,
) -> TestResult {
    let pool = connect(database).await?;
    // Fault injection is at SQLite acknowledgement, never at the graph client:
    // run_projection has already performed its genuine native writes when this
    // trigger aborts deletion. Replaying the unacked revision must be idempotent.
    // Persistent only in this fresh owned database so the separate production
    // writer observes the injected fault; removed before the retry.
    sqlx::query("CREATE TRIGGER native_ack_failure BEFORE DELETE ON projection_outbox BEGIN SELECT RAISE(ABORT,'owned-native-ack-failure'); END")
        .execute(&pool).await?;
    let delivered = run_projection(store, graph.endpoint(), graph.name(), 1000).await;
    let cleanup = sqlx::query("DROP TRIGGER native_ack_failure")
        .execute(&pool)
        .await;
    pool.close().await;
    cleanup?;
    match delivered {
        Err(ProjectionError::Store(StoreError::Sqlite(sqlx::Error::Database(error)))) => {
            assert_eq!(error.message(), "owned-native-ack-failure");
        }
        other => {
            return Err(format!(
                "expected real delivery followed by SQLite ack rejection, got {other:?}"
            )
            .into());
        }
    }
    assert_eq!(
        store.projection_state().await?.pending,
        1,
        "failed acknowledgement must preserve exact pending revision"
    );
    Ok(())
}

pub(super) async fn old_ack(
    store: &BinaryStore,
    resource: &inventory_core::inventory::ResourceKey,
) -> TestResult<ProjectionAck> {
    let pending = store.drain_projection(1000).await?;
    let [entry] = pending.as_slice() else {
        return Err("expected exactly one real tag delivery".into());
    };
    assert_eq!(entry.key.as_ref(), Some(resource));
    assert!(!entry.deleted && !entry.full_rebuild);
    Ok(ProjectionAck::new(resource.clone(), entry.revision)?)
}

pub(super) async fn assert_superseded(database: &Path, old: ProjectionAck) -> TestResult {
    let reopened = BinaryStore::open(database).await?;
    let before = reopened.drain_projection(1000).await?;
    let [latest] = before.as_slice() else {
        return Err("newer revision not retained after reopen".into());
    };
    assert_eq!(latest.key, old.key);
    assert!(
        latest.revision > old.revision,
        "replacement must supersede delivered revision"
    );
    assert_eq!(
        reopened.ack_projection(old).await?,
        0,
        "superseded acknowledgement must acknowledge zero rows"
    );
    assert_eq!(
        reopened.drain_projection(1000).await?,
        before,
        "stale ack must not delete newer pending delivery"
    );
    Ok(())
}

pub(super) async fn offline(
    store: &BinaryStore,
    database: &Path,
    graph: &NativeGraph,
) -> TestResult {
    let before = store.drain_projection(1000).await?;
    assert!(
        !before.is_empty(),
        "offline test must contain real pending work"
    );
    let route = database.with_extension("owned-unavailable.sock");
    assert!(
        !route.exists(),
        "outage route must not touch a running/shared service"
    );
    let endpoint = format!("redis+unix://{}", route.display());
    let report = run_projection(store, &endpoint, graph.name(), 1000).await?;
    assert!(report.offline);
    assert_eq!(report.acknowledged, 0);
    assert_eq!(report.pending, u64::try_from(before.len())?);
    assert!(!report.rebuild_incomplete);
    let reopened = BinaryStore::open(database).await?;
    assert_eq!(
        reopened.drain_projection(1000).await?,
        before,
        "offline backlog must survive reopen unchanged"
    );
    Ok(())
}
