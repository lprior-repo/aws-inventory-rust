//! Upgrade from the real v5 boundary without changing observations or user data.
use inventory_core::{CollectionPage, CollectionScope, Resource, ScanStatus, Source};
use inventory_engine::store::Store;
use serde_json::{Value, json};
use sqlx::{Connection, Row, SqliteConnection, sqlite::SqliteConnectOptions};
use std::{collections::BTreeMap, path::Path};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

async fn connect(path: &Path) -> TestResult<SqliteConnection> {
    Ok(SqliteConnection::connect_with(&SqliteConnectOptions::new().filename(path)).await?)
}

fn legacy_parent_views() -> Vec<&'static str> {
    include_str!("../sql/006_cloudcontrol_parents.sql")
        .lines()
        .filter_map(|line| line.strip_prefix("CREATE VIEW "))
        .filter_map(|line| line.strip_suffix(" AS"))
        .collect()
}

fn parent_views() -> Vec<&'static str> {
    let additions = include_str!("../sql/013_cloudcontrol_parent_additions.sql");
    legacy_parent_views()
        .into_iter()
        .chain(additions.lines().filter_map(|line| line.strip_prefix("CREATE VIEW ")))
        .map(|line| line.strip_suffix(" AS").unwrap_or(line))
        .collect()
}

async fn version_five_fixture(path: &Path) -> TestResult {
    let store = Store::open(path).await?;
    store.begin_scan("immutable", 1).await?;
    store.apply_page("immutable", CollectionPage {
        scope: CollectionScope {
            account_id: "111111111111".into(), region: "us-east-1".into(),
            collector: "fixture".into(), parent_id: None, source: Source::Direct,
        },
        page_number: 1, next_token: None, edges: vec![],
        resources: vec![Resource {
            id: "arn:aws:eks:us-east-1:111111111111:cluster/preserve".into(),
            account_id: "111111111111".into(), region: "us-east-1".into(),
            kind: "aws_eks_cluster".into(), name: Some("preserve".into()),
            properties: json!({"cluster_name":"preserve","nested":{"retained":[true,null,3]}}),
            tags: BTreeMap::from([("owner".into(), "caller".into())]),
            source: Source::Direct, observed_at: 2,
        }],
    }).await?;
    store
        .finish_scan("immutable", ScanStatus::Complete, 3)
        .await?;
    store.close().await;
    let mut connection = connect(path).await?;
    let views = parent_views();
    assert_eq!(views.len(), 24);
    for name in views {
        assert!(name.starts_with("aws_cc_"));
        assert!(
            name.bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        );
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!("DROP VIEW {name}")))
            .execute(&mut connection)
            .await?;
    }
    sqlx::raw_sql("CREATE TABLE owner_data(note TEXT); INSERT INTO owner_data VALUES('retain'); CREATE VIEW owner_snapshot AS SELECT * FROM scan_resources; PRAGMA user_version=5;")
        .execute(&mut connection).await?;
    connection.close().await?;
    Ok(())
}

async fn frozen_rows(connection: &mut SqliteConnection) -> TestResult<Value> {
    let payloads: Vec<String> = sqlx::query_scalar(
        "SELECT resource_json FROM resource_observations ORDER BY scan_id,resource_id,source",
    )
    .fetch_all(&mut *connection)
    .await?;
    let fingerprints: Vec<Vec<u8>> = sqlx::query_scalar(
        "SELECT fingerprint FROM collection_pages ORDER BY scan_id,scope_key,page_number",
    )
    .fetch_all(&mut *connection)
    .await?;
    let scans =
        sqlx::query("SELECT scan_id,started_at,finished_at,status FROM scans ORDER BY scan_id")
            .fetch_all(&mut *connection)
            .await?
            .into_iter()
            .map(|row| {
                Ok(json!({"id":row.try_get::<String,_>("scan_id")?,
                "started_at":row.try_get::<i64,_>("started_at")?,
                "finished_at":row.try_get::<i64,_>("finished_at")?,
                "status":row.try_get::<String,_>("status")?}))
            })
            .collect::<Result<Vec<Value>, sqlx::Error>>()?;
    Ok(json!({"resources":payloads,"fingerprints":fingerprints,"scans":scans}))
}

#[tokio::test]
async fn schema_thirteen_preserves_every_old_view_and_immutable_payload() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("v5.sqlite");
    version_five_fixture(&path).await?;
    let mut before = connect(&path).await?;
    let frozen = frozen_rows(&mut before).await?;
    let views: Vec<(String, String)> =
        sqlx::query_as("SELECT name,sql FROM sqlite_schema WHERE type='view' ORDER BY name")
            .fetch_all(&mut before)
            .await?;
    before.close().await?;
    let store = Store::open(&path).await?;
    assert_eq!(store.scan_status("immutable").await?, ScanStatus::Complete);
    assert_eq!(
        store.query("SELECT note FROM owner_data", 1).await?,
        json!([{"note":"retain"}])
    );
    assert_eq!(
        store
            .query("SELECT count(*) AS n FROM owner_snapshot", 1)
            .await?,
        json!([{"n":1}])
    );
    assert_eq!(
        store
            .query("SELECT user_version AS version FROM pragma_user_version", 1)
            .await?,
        json!([{"version":13}])
    );
    assert_eq!(store.query("SELECT count(*) AS n FROM sqlite_schema WHERE type='view' AND name GLOB 'aws_cc_*'",1).await?, json!([{"n":421}]));
    store.close().await;
    let mut after = connect(&path).await?;
    assert_eq!(frozen_rows(&mut after).await?, frozen);
    for (name, sql) in views {
        let actual: String =
            sqlx::query_scalar("SELECT sql FROM sqlite_schema WHERE type='view' AND name=?")
                .bind(name)
                .fetch_one(&mut after)
                .await?;
        assert_eq!(actual, sql);
    }
    after.close().await?;
    // Reopening an already migrated database never recreates or changes views.
    Store::open(&path).await?.close().await;
    Ok(())
}

#[tokio::test]
async fn parent_view_conflict_rolls_back_the_entire_upgrade() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("conflict.sqlite");
    version_five_fixture(&path).await?;
    let mut connection = connect(&path).await?;
    let frozen = frozen_rows(&mut connection).await?;
    let views = parent_views();
    let last = views.last().ok_or("missing parent view")?;
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "CREATE TABLE {last}(note TEXT); INSERT INTO {last} VALUES('owned');"
    )))
    .execute(&mut connection)
    .await?;
    connection.close().await?;
    assert!(Store::open(&path).await.is_err());
    let mut connection = connect(&path).await?;
    assert_eq!(frozen_rows(&mut connection).await?, frozen);
    let version: i64 = sqlx::query_scalar("PRAGMA user_version")
        .fetch_one(&mut connection)
        .await?;
    let view_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM sqlite_schema WHERE type='view' AND name GLOB 'aws_cc_*'",
    )
    .fetch_one(&mut connection)
    .await?;
    let note: String = sqlx::query_scalar(sqlx::AssertSqlSafe(
        format!("SELECT note FROM {last}").as_str(),
    ))
    .fetch_one(&mut connection)
    .await?;
    assert_eq!((version, view_count, note), (5, 397, "owned".into()));
    connection.close().await?;
    Ok(())
}
