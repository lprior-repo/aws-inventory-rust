#[path = "../projection_degree/support.rs"]
mod support;

use super::{BinaryStore, LIVE_DATABASE, Path, env, run_projection};
use futures_util::{TryStreamExt, stream};
use inventory_graph::{EdgeRow, GraphBackend, NodeRow, falkor::FalkorDb};
use support::{seed_edges, sort_edges};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

async fn copy_fixture(database: &str, destination: &Path) -> TestResult {
    let pool = sqlx::SqlitePool::connect_with(
        sqlx::sqlite::SqliteConnectOptions::new().filename(database),
    )
    .await?;
    sqlx::query("VACUUM INTO ?1")
        .bind(destination.to_str().ok_or("non-UTF8 fixture path")?)
        .execute(&pool)
        .await?;
    pool.close().await;
    Ok(())
}

#[tokio::test]
#[ignore = "requires AWS_INVENTORY_PROJECTION_DATABASE SQLite fixture with pending resource and an explicit connected FalkorDB"]
async fn live_falkor_projects_all_2001_incident_edges_before_acknowledging() -> TestResult {
    let _database_owner = LIVE_DATABASE.lock().await;
    // Given: an isolated copy of a real pending SQLite inventory and a fresh graph.
    let database = env::var("AWS_INVENTORY_PROJECTION_DATABASE")?;
    let endpoint = env::var("AWS_INVENTORY_PROJECTION_URL")?;
    let graph = format!(
        "{}_degree_{}",
        env::var("AWS_INVENTORY_PROJECTION_GRAPH")?,
        std::process::id()
    );
    let directory = tempfile::tempdir()?;
    let destination = directory.path().join("degree.sqlite");
    copy_fixture(&database, &destination).await?;
    let store = BinaryStore::open(&destination).await?;
    let pending = store.drain_projection(1).await?;
    let entry = pending
        .first()
        .ok_or("live degree test requires a pending resource")?;
    let owner = entry
        .key
        .as_ref()
        .ok_or("live degree test requires a resource, not a rebuild marker")?;
    if entry.deleted {
        return Err("live degree test requires a non-deleted pending resource".into());
    }
    let pool = sqlx::SqlitePool::connect_with(
        sqlx::sqlite::SqliteConnectOptions::new().filename(&destination),
    )
    .await?;
    sqlx::query("DELETE FROM edges").execute(&pool).await?;
    pool.close().await;
    let expected = seed_edges(&destination, owner, 2_001).await?;
    let backend = FalkorDb::connect(&endpoint, &graph)?;
    backend.claim_projection(0).await?;
    backend.clear().await?;
    // When/Then: projection must deliver both chunks before durable acknowledgement.
    let result = project_and_compare(&store, &backend, expected).await;
    backend.clear().await?;
    result
}

async fn project_and_compare(
    store: &BinaryStore,
    backend: &FalkorDb,
    mut expected: Vec<EdgeRow>,
) -> TestResult {
    stream::iter(expected.iter().map(Ok::<_, Box<dyn std::error::Error>>))
        .try_for_each(|edge| async move {
            backend
                .upsert_node(&NodeRow {
                    key: edge.to.clone(),
                    name: None,
                    content_hash: [0; 32],
                })
                .await?;
            Ok(())
        })
        .await?;
    let pending = store.projection_state().await?.pending;
    let report = run_projection(store, backend, 1).await?;
    assert_eq!(report.acknowledged, 1);
    assert_eq!(
        report.pending,
        pending.checked_sub(1).ok_or("missing pending revision")?
    );
    assert_eq!((report.offline, report.rebuild_incomplete), (false, false));
    let mut delivered = backend.edge_keys(None, 2_000).await?;
    assert_eq!(delivered.len(), 2_000);
    let tail = backend.edge_keys(delivered.last(), 2_000).await?;
    assert_eq!(tail.len(), 1);
    delivered.extend(tail);
    sort_edges(&mut delivered);
    sort_edges(&mut expected);
    assert_eq!(delivered, expected);
    Ok(())
}
