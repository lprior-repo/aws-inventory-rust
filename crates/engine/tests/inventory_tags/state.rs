use super::{BinaryStore, SweepId, TestResult, support};
use inventory_core::inventory::{Observation, ResourceKey, TagObservation};
use inventory_engine::inventory::{CommitResult, ProjectionTag, WorkSpec};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};

pub(super) async fn pool(directory: &tempfile::TempDir) -> TestResult<SqlitePool> {
    Ok(SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?)
}

pub(super) async fn commit_tags(
    store: &BinaryStore,
    sweep: &str,
    tags: Vec<TagObservation>,
    at: i64,
) -> TestResult<CommitResult> {
    // A parent-owned tag fetch does not certify an account-wide resource enumeration.
    let target = tags.first().ok_or("tag fetch needs a target")?.resource();
    store.seed_work(support::child_work(sweep, target)?).await?;
    let lease = store
        .claim(&SweepId::new(sweep.to_owned())?, at.saturating_sub(1), 100)
        .await?
        .ok_or("tag fetch lease missing")?;
    let mut page = support::page(lease, vec![], true, None, at);
    page.tags = tags;
    Ok(store.commit_page(page).await?)
}

pub(super) async fn commit_resources(
    store: &BinaryStore,
    sweep: &str,
    observations: Vec<Observation>,
    tags: Vec<TagObservation>,
    at: i64,
) -> TestResult<CommitResult> {
    store.seed_work(support::work(sweep, "us-east-1")?).await?;
    let lease = store
        .claim(&SweepId::new(sweep.to_owned())?, at.saturating_sub(1), 100)
        .await?
        .ok_or("resource lease missing")?;
    let mut page = support::page(lease, observations, true, None, at);
    page.tags = tags;
    Ok(store.commit_page(page).await?)
}

pub(super) async fn read_tags(
    store: &BinaryStore,
    key: &ResourceKey,
) -> TestResult<Vec<ProjectionTag>> {
    let mut view = store.begin_projection_snapshot().await?;
    let page = view.tags_for(key, None, 1000).await?;
    assert_eq!(page.next, None);
    view.finish().await?;
    Ok(page.tags)
}

pub(super) fn pair(key: &str, value: &str) -> ProjectionTag {
    ProjectionTag {
        key: key.to_owned(),
        value: value.to_owned(),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct DatabaseState {
    version: i64,
    schema: Vec<(String, String, String, Option<String>)>,
    tables: Vec<(String, Vec<Vec<String>>)>,
}

pub(super) async fn database_state(pool: &SqlitePool) -> TestResult<DatabaseState> {
    let version = sqlx::query_scalar("PRAGMA user_version")
        .fetch_one(pool)
        .await?;
    let schema: Vec<(String, String, String, Option<String>)> = sqlx::query_as(
        "SELECT type,name,tbl_name,sql FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%' ORDER BY type,name",
    ).fetch_all(pool).await?;
    let mut tables = Vec::new();
    for (_, name, _, _) in schema.iter().filter(|(kind, _, _, _)| kind == "table") {
        let quoted = quote(name);
        let columns = sqlx::query(sqlx::AssertSqlSafe(format!("PRAGMA table_info({quoted})")))
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(|row| row.try_get::<String, _>("name"))
            .collect::<Result<Vec<_>, _>>()?;
        let selection = columns
            .iter()
            .map(|column| format!("quote({})", quote(column)))
            .collect::<Vec<_>>()
            .join(",");
        let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT {selection} FROM {quoted}"
        )))
        .fetch_all(pool)
        .await?;
        let mut values = rows
            .into_iter()
            .map(|row| {
                (0..columns.len())
                    .map(|index| row.try_get::<String, _>(index))
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;
        values.sort();
        tables.push((name.clone(), values));
    }
    Ok(DatabaseState {
        version,
        schema,
        tables,
    })
}

fn quote(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

pub(super) async fn resource_image(pool: &SqlitePool) -> TestResult<Vec<(String, String)>> {
    // SQL quote is byte-exact for records/hashes (BLOB hex), and distinguishes NULL.
    Ok(sqlx::query_as(
        "SELECT 'resource:'||account||':'||region||':'||kind||':'||id,quote(name)||'|'||quote(content_hash)||'|'||quote(first_seen)||'|'||quote(last_changed) FROM resources UNION ALL SELECT 'typed:'||account||':'||region||':'||id,quote(content_hash)||'|'||quote(first_seen)||'|'||quote(last_changed)||'|'||quote(record)||'|'||quote(f_1) FROM thing_table ORDER BY 1",
    ).fetch_all(pool).await?)
}

pub(super) async fn scoped_work(
    store: &BinaryStore,
    work: WorkSpec,
    at: i64,
) -> TestResult<inventory_engine::inventory::WorkLease> {
    let sweep = work.sweep().clone();
    store.seed_work(work).await?;
    Ok(store
        .claim(&sweep, at.saturating_sub(1), 100)
        .await?
        .ok_or("scoped lease missing")?)
}
