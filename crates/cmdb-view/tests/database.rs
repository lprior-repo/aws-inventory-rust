#[path = "database/cursors.rs"]
mod cursors;
#[path = "database/graph_live.rs"]
mod graph_live;
#[path = "database/graph_offline.rs"]
mod graph_offline;
#[path = "database/snapshots.rs"]
mod snapshots;
#[path = "database/support.rs"]
mod support;
#[cfg(feature = "native-graph-integration")]
#[path = "database/tag_graph_live.rs"]
mod tag_graph_live;
#[path = "database/tags.rs"]
mod tags;
use cmdb_view::{Database, PageRequest, ResourceFilter};
use inventory_core::record::{KeyRef, RecordBuilder, ValueRef};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{SqlitePool, query};
use std::error::Error;
use std::path::Path;
use tempfile::tempdir;

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

struct Seed<'a> {
    table: &'a str,
    account: &'a str,
    region: &'a str,
    kind: &'a str,
    id: &'a str,
}

async fn writer(path: &Path) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true);
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
}

async fn seed(pool: &SqlitePool, seed: Seed<'_>, record: &[u8]) -> Result<(), sqlx::Error> {
    let hash = [7_u8; 32];
    let mut detail = sqlx::QueryBuilder::<sqlx::Sqlite>::new("INSERT INTO \"");
    detail
        .push(seed.table)
        .push("\" (account,region,id,content_hash,first_seen,last_changed,record) VALUES (")
        .push_bind(seed.account)
        .push(",")
        .push_bind(seed.region)
        .push(",")
        .push_bind(seed.id)
        .push(",")
        .push_bind(hash.as_slice())
        .push(",1,1,")
        .push_bind(record)
        .push(")");
    query("INSERT OR IGNORE INTO kinds (kind,table_name) VALUES (?,?)")
        .bind(seed.kind)
        .bind(seed.table)
        .execute(pool)
        .await?;
    detail.build().execute(pool).await?;
    query("INSERT INTO resources (account,region,kind,id,name,content_hash,first_seen,last_changed) VALUES (?,?,?,?,?,?,1,1)")
        .bind(seed.account).bind(seed.region).bind(seed.kind).bind(seed.id).bind(seed.id).bind(hash.as_slice()).execute(pool).await?;
    Ok(())
}

fn record_bytes() -> Result<Vec<u8>, Box<dyn Error>> {
    let mut builder = RecordBuilder::new(1024, 8, 4)?;
    builder.begin_map(None)?;
    builder.scalar(Some(KeyRef::Name("name")), ValueRef::Text("stored"))?;
    builder.end()?;
    Ok(builder.finish()?.into_bytes())
}

async fn schema15(path: &Path) -> Result<SqlitePool, Box<dyn Error>> {
    let pool = writer(path).await?;
    query("PRAGMA foreign_keys=ON").execute(&pool).await?;
    query("PRAGMA journal_mode=WAL").execute(&pool).await?;
    query("PRAGMA user_version=15").execute(&pool).await?;
    query("CREATE TABLE metadata(key TEXT PRIMARY KEY,value BLOB NOT NULL) WITHOUT ROWID")
        .execute(&pool)
        .await?;
    query(
        "CREATE TABLE kinds(kind TEXT PRIMARY KEY,table_name TEXT NOT NULL UNIQUE) WITHOUT ROWID",
    )
    .execute(&pool)
    .await?;
    query("CREATE TABLE resources(account TEXT,region TEXT,kind TEXT,id TEXT,name TEXT NULL,content_hash BLOB NOT NULL,first_seen INTEGER NOT NULL,last_changed INTEGER NOT NULL,PRIMARY KEY(account,region,kind,id)) WITHOUT ROWID").execute(&pool).await?;
    query("CREATE TABLE bucket_records(account TEXT,region TEXT,id TEXT,content_hash BLOB NOT NULL,first_seen INTEGER NOT NULL,last_changed INTEGER NOT NULL,record BLOB NOT NULL,PRIMARY KEY(account,region,id)) WITHOUT ROWID").execute(&pool).await?;
    query("CREATE TABLE instance_records(account TEXT,region TEXT,id TEXT,content_hash BLOB NOT NULL,first_seen INTEGER NOT NULL,last_changed INTEGER NOT NULL,record BLOB NOT NULL,PRIMARY KEY(account,region,id)) WITHOUT ROWID").execute(&pool).await?;
    query("CREATE TABLE edges(from_account TEXT,from_region TEXT,from_kind TEXT,from_id TEXT,to_account TEXT,to_region TEXT,to_kind TEXT,to_id TEXT,relation TEXT,source TEXT,PRIMARY KEY(from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source),FOREIGN KEY(from_account,from_region,from_kind,from_id) REFERENCES resources(account,region,kind,id) ON DELETE CASCADE,FOREIGN KEY(to_account,to_region,to_kind,to_id) REFERENCES resources(account,region,kind,id) ON DELETE CASCADE) WITHOUT ROWID").execute(&pool).await?;
    query("CREATE TABLE tags(account TEXT NOT NULL,region TEXT NOT NULL,kind TEXT NOT NULL,id TEXT NOT NULL,tag_key TEXT NOT NULL,tag_value TEXT NOT NULL,PRIMARY KEY(account,region,kind,id,tag_key),FOREIGN KEY(account,region,kind,id) REFERENCES resources(account,region,kind,id) ON DELETE CASCADE) WITHOUT ROWID").execute(&pool).await?;
    query("CREATE TABLE tag_observations(account TEXT NOT NULL,region TEXT NOT NULL,kind TEXT NOT NULL,id TEXT NOT NULL,coverage TEXT NOT NULL,observed_at INTEGER NOT NULL,last_complete_at INTEGER NULL,PRIMARY KEY(account,region,kind,id),FOREIGN KEY(account,region,kind,id) REFERENCES resources(account,region,kind,id) ON DELETE CASCADE) WITHOUT ROWID").execute(&pool).await?;
    Ok(pool)
}

async fn seed_edge(pool: &SqlitePool, edge: &cmdb_view::GraphEdge) -> Result<(), sqlx::Error> {
    query("INSERT INTO edges VALUES (?,?,?,?,?,?,?,?,?,?)")
        .bind(&edge.from.account)
        .bind(&edge.from.region)
        .bind(&edge.from.kind)
        .bind(&edge.from.id)
        .bind(&edge.to.account)
        .bind(&edge.to.region)
        .bind(&edge.to.kind)
        .bind(&edge.to.id)
        .bind(&edge.relation)
        .bind(&edge.provenance)
        .execute(pool)
        .await?;
    Ok(())
}

async fn populated(path: &Path) -> Result<Database, Box<dyn Error>> {
    let pool = schema15(path).await?;
    let bytes = record_bytes()?;
    seed(
        &pool,
        Seed {
            table: "bucket_records",
            account: "111111111111",
            region: "global",
            kind: "AWS::S3::Bucket",
            id: "shared",
        },
        &bytes,
    )
    .await?;
    seed(
        &pool,
        Seed {
            table: "bucket_records",
            account: "111111111111",
            region: "us-east-1",
            kind: "AWS::S3::Bucket",
            id: "shared",
        },
        &bytes,
    )
    .await?;
    seed(
        &pool,
        Seed {
            table: "bucket_records",
            account: "222222222222",
            region: "eu-west-1",
            kind: "AWS::S3::Bucket",
            id: "shared",
        },
        &bytes,
    )
    .await?;
    seed(
        &pool,
        Seed {
            table: "instance_records",
            account: "222222222222",
            region: "us-west-2",
            kind: "AWS::EC2::Instance",
            id: "shared",
        },
        &bytes,
    )
    .await?;
    let root = cmdb_view::GraphNode {
        account: "111111111111".to_owned(),
        region: "global".to_owned(),
        kind: "AWS::S3::Bucket".to_owned(),
        id: "shared".to_owned(),
    };
    seed_edge(
        &pool,
        &cmdb_view::GraphEdge {
            from: root.clone(),
            to: cmdb_view::GraphNode {
                account: "111111111111".to_owned(),
                region: "us-east-1".to_owned(),
                kind: "AWS::S3::Bucket".to_owned(),
                id: "shared".to_owned(),
            },
            relation: "replica".to_owned(),
            provenance: "catalog".to_owned(),
        },
    )
    .await?;
    seed_edge(
        &pool,
        &cmdb_view::GraphEdge {
            from: root,
            to: cmdb_view::GraphNode {
                account: "222222222222".to_owned(),
                region: "eu-west-1".to_owned(),
                kind: "AWS::S3::Bucket".to_owned(),
                id: "shared".to_owned(),
            },
            relation: "replica".to_owned(),
            provenance: "catalog".to_owned(),
        },
    )
    .await?;
    pool.close().await;
    Ok(Database::open(path).await?)
}

#[tokio::test]
async fn keyset_pages_preserve_full_resource_identity_and_global_order() -> TestResult {
    let dir = tempdir()?;
    let db = populated(&dir.path().join("inventory.sqlite")).await?;
    let first = db
        .page(
            &ResourceFilter::default(),
            PageRequest {
                limit: 2,
                after: None,
            },
        )
        .await?;
    let next = first.next.clone().ok_or("expected next page")?;
    let second = db
        .page(
            &ResourceFilter::default(),
            PageRequest {
                limit: 2,
                after: Some(next),
            },
        )
        .await?;
    assert_eq!(first.resources.len(), 2);
    assert_eq!(second.resources.len(), 2);
    assert_eq!(first.total_matching, 4);
    assert_eq!(
        first.resources.first().map(|item| item.key.region.as_str()),
        Some("global")
    );
    assert_eq!(
        second
            .resources
            .first()
            .map(|item| item.key.account.as_str()),
        Some("222222222222")
    );
    assert_eq!(
        second.resources.get(1).map(|item| item.key.kind.as_str()),
        Some("AWS::EC2::Instance")
    );
    assert!(second.next.is_none());
    Ok(())
}

#[tokio::test]
async fn filters_combine_identity_and_case_insensitive_text() -> TestResult {
    let dir = tempdir()?;
    let db = populated(&dir.path().join("inventory.sqlite")).await?;
    let filter = ResourceFilter {
        account: Some("222222222222".to_owned()),
        region: Some("eu-west-1".to_owned()),
        kind: Some("AWS::S3::Bucket".to_owned()),
        text: Some("SHARED".to_owned()),
        tag: None,
    };
    let page = db
        .page(
            &filter,
            PageRequest {
                limit: 10,
                after: None,
            },
        )
        .await?;
    assert_eq!(page.total_matching, 1);
    assert_eq!(
        page.resources.first().map(|item| item.key.region.as_str()),
        Some("eu-west-1")
    );
    Ok(())
}

#[tokio::test]
async fn counts_are_database_derived_and_version_mismatch_fails_closed() -> TestResult {
    let dir = tempdir()?;
    let path = dir.path().join("inventory.sqlite");
    let pool = schema15(&path).await?;
    pool.close().await;
    let db = Database::open(&path).await?;
    assert_eq!(db.counts().await?.resources, 0);
    let wrong_path = dir.path().join("wrong.sqlite");
    let wrong = writer(&wrong_path).await?;
    query("PRAGMA user_version=13").execute(&wrong).await?;
    wrong.close().await;
    assert!(Database::open(&wrong_path).await.is_err());
    Ok(())
}
