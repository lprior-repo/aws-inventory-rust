use inventory_core::{CollectionPage, CollectionScope, Resource, ScanStatus, Source};
use inventory_engine::store::Store;
use serde_json::{Value, json};
use sqlx::{Connection, SqliteConnection, sqlite::SqliteConnectOptions};
use std::{collections::BTreeSet, path::Path};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn added_views() -> TestResult<Vec<String>> {
    let legacy_document: Value = serde_json::to_value(toml::from_str::<toml::Table>(
        include_str!("../../../catalog/cloudcontrol-v04-types.toml"),
    )?)?;
    let legacy: BTreeSet<String> = serde_json::from_value(
        legacy_document
            .get("types")
            .cloned()
            .ok_or("v04 types missing")?,
    )?;
    let catalog: Value =
        serde_json::from_str(include_str!("../../../catalog/cloudcontrol-resources.json"))?;
    catalog
        .get("resource_types")
        .and_then(Value::as_array)
        .ok_or("types missing")?
        .iter()
        .filter(|item| {
            item.get("type_name")
                .and_then(Value::as_str)
                .is_some_and(|name| !legacy.contains(name))
        })
        .map(|item| {
            item.get("view_name")
                .and_then(Value::as_str)
                .filter(|name| {
                    name.starts_with("aws_cc_")
                        && name
                            .bytes()
                            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
                })
                .map(str::to_owned)
                .ok_or_else(|| "invalid view name".into())
        })
        .collect()
}

fn parent_views() -> TestResult<Vec<String>> {
    let catalog: Value = serde_json::to_value(toml::from_str::<toml::Table>(include_str!(
        "../../../catalog/cloudcontrol-parent-resources.toml"
    ))?)?;
    catalog
        .get("resource_types")
        .and_then(Value::as_array)
        .ok_or("parent types missing")?
        .iter()
        .map(|item| {
            item.get("view_name")
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| "parent view missing".into())
        })
        .collect()
}

async fn connect(path: &Path) -> TestResult<SqliteConnection> {
    Ok(SqliteConnection::connect_with(&SqliteConnectOptions::new().filename(path)).await?)
}

async fn version_four_fixture(path: &Path) -> TestResult {
    let store = Store::open(path).await?;
    store.begin_scan("preserve", 1).await?;
    store
        .apply_page(
            "preserve",
            CollectionPage {
                scope: CollectionScope {
                    account_id: "111111111111".into(),
                    region: "us-east-1".into(),
                    collector: "fixture".into(),
                    parent_id: None,
                    source: Source::Direct,
                },
                page_number: 1,
                next_token: None,
                resources: vec![Resource {
                    id: "arn:aws:ec2:us-east-1:111111111111:vpc/vpc-preserve".into(),
                    account_id: "111111111111".into(),
                    region: "us-east-1".into(),
                    kind: "aws_vpc".into(),
                    name: Some("preserve".into()),
                    properties: json!({"vpc_id":"vpc-preserve"}),
                    tags: Default::default(),
                    source: Source::Direct,
                    observed_at: 2,
                }],
                edges: vec![],
            },
        )
        .await?;
    store
        .finish_scan("preserve", ScanStatus::Complete, 3)
        .await?;
    store.close().await;
    let mut connection = connect(path).await?;

    for view in added_views()?.into_iter().chain(parent_views()?) {
        sqlx::query(sqlx::AssertSqlSafe(format!("DROP VIEW {view}").as_str()))
            .execute(&mut connection)
            .await?;
    }
    sqlx::raw_sql("CREATE TABLE user_data(value TEXT); INSERT INTO user_data VALUES('preserve'); CREATE VIEW user_vpc AS SELECT * FROM aws_vpc; PRAGMA user_version=4;")
        .execute(&mut connection).await?;
    connection.close().await?;
    Ok(())
}

#[tokio::test]
async fn upgrades_existing_v04_inventory_without_recollecting_or_replacing_user_objects()
-> TestResult {
    let temporary = tempfile::tempdir()?;
    let path = temporary.path().join("inventory.sqlite");
    version_four_fixture(&path).await?;
    assert_eq!(added_views()?.len(), 197);
    let store = Store::open(&path).await?;
    assert_eq!(store.scan_status("preserve").await?, ScanStatus::Complete);
    assert_eq!(store.summary("preserve").await?.resources, 1);
    assert_eq!(
        store.query("SELECT value FROM user_data", 1).await?,
        json!([{"value":"preserve"}])
    );
    assert_eq!(
        store.query("SELECT count(*) AS n FROM user_vpc", 1).await?,
        json!([{"n":1}])
    );
    assert_eq!(store.query("SELECT count(*) AS n FROM sqlite_schema WHERE type='view' AND name GLOB 'aws_cc_*'", 1).await?, json!([{"n":421}]));
    assert_eq!(
        store
            .query("SELECT user_version AS version FROM pragma_user_version", 1)
            .await?,
        json!([{"version":13}])
    );
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn view_name_conflict_rolls_back_added_views_and_schema_version() -> TestResult {
    let temporary = tempfile::tempdir()?;
    let path = temporary.path().join("inventory.sqlite");
    version_four_fixture(&path).await?;
    let views = added_views()?;
    let last = views.last().ok_or("expected expanded views")?;
    let mut connection = connect(&path).await?;
    sqlx::query(sqlx::AssertSqlSafe(
        format!("CREATE VIEW {last} AS SELECT 'caller-owned' AS value").as_str(),
    ))
    .execute(&mut connection)
    .await?;
    connection.close().await?;
    assert!(Store::open(&path).await.is_err());
    let mut connection = connect(&path).await?;
    let version: i64 = sqlx::query_scalar("PRAGMA user_version")
        .fetch_one(&mut connection)
        .await?;
    let resources: i64 = sqlx::query_scalar("SELECT count(*) FROM resource_observations")
        .fetch_one(&mut connection)
        .await?;
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM sqlite_schema WHERE type='view' AND name GLOB 'aws_cc_*'",
    )
    .fetch_one(&mut connection)
    .await?;
    let preserved: String = sqlx::query_scalar(sqlx::AssertSqlSafe(
        format!("SELECT value FROM {last}").as_str(),
    ))
    .fetch_one(&mut connection)
    .await?;
    assert_eq!((version, resources, count), (4, 1, 201));
    assert_eq!(preserved, "caller-owned");
    connection.close().await?;
    Ok(())
}
