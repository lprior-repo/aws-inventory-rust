use crate::USAGE;
use futures_util::{StreamExt, TryStreamExt, stream};
use inventory_core::record::{Exclusions, KeyRef, RecordBuilder, ValueRef};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{SqlitePool, query};
use std::error::Error;
use std::path::{Path, PathBuf};
async fn connect_writer(path: &std::path::Path) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true);
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
}

pub(crate) fn ensure_fresh(path: &Path) -> Result<(), Box<dyn Error>> {
    match std::fs::metadata(path) {
        Ok(metadata) if metadata.is_file() && metadata.len() == 0 => Ok(()),
        Ok(_) => Err(USAGE.into()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

async fn execute_schema(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    query("PRAGMA foreign_keys=ON").execute(pool).await?;
    query("PRAGMA journal_mode=WAL").execute(pool).await?;
    query("PRAGMA user_version=15").execute(pool).await?;
    query("CREATE TABLE metadata(key TEXT PRIMARY KEY,value BLOB NOT NULL) WITHOUT ROWID")
        .execute(pool)
        .await?;
    query(
        "CREATE TABLE kinds(kind TEXT PRIMARY KEY,table_name TEXT NOT NULL UNIQUE) WITHOUT ROWID",
    )
    .execute(pool)
    .await?;
    query("CREATE TABLE resources(account TEXT,region TEXT,kind TEXT,id TEXT,name TEXT NULL,content_hash BLOB NOT NULL,first_seen INTEGER NOT NULL,last_changed INTEGER NOT NULL,PRIMARY KEY(account,region,kind,id)) WITHOUT ROWID").execute(pool).await?;
    query("CREATE TABLE tags(account TEXT NOT NULL,region TEXT NOT NULL,kind TEXT NOT NULL,id TEXT NOT NULL,tag_key TEXT NOT NULL,tag_value TEXT NOT NULL,PRIMARY KEY(account,region,kind,id,tag_key),FOREIGN KEY(account,region,kind,id) REFERENCES resources(account,region,kind,id) ON DELETE CASCADE) WITHOUT ROWID").execute(pool).await?;
    query("CREATE TABLE tag_observations(account TEXT NOT NULL,region TEXT NOT NULL,kind TEXT NOT NULL,id TEXT NOT NULL,coverage TEXT NOT NULL CHECK(coverage IN('complete','partial','failed','unobserved')),observed_at INTEGER NOT NULL CHECK(observed_at>=0),last_complete_at INTEGER CHECK(last_complete_at>=0),PRIMARY KEY(account,region,kind,id),FOREIGN KEY(account,region,kind,id) REFERENCES resources(account,region,kind,id) ON DELETE CASCADE) WITHOUT ROWID").execute(pool).await?;
    query("CREATE TABLE ec2_instances(account TEXT,region TEXT,id TEXT,content_hash BLOB NOT NULL,first_seen INTEGER NOT NULL,last_changed INTEGER NOT NULL,record BLOB NOT NULL,PRIMARY KEY(account,region,id)) WITHOUT ROWID").execute(pool).await?;
    query("CREATE TABLE iam_roles(account TEXT,region TEXT,id TEXT,content_hash BLOB NOT NULL,first_seen INTEGER NOT NULL,last_changed INTEGER NOT NULL,record BLOB NOT NULL,PRIMARY KEY(account,region,id)) WITHOUT ROWID").execute(pool).await?;
    query("CREATE TABLE edges(from_account TEXT,from_region TEXT,from_kind TEXT,from_id TEXT,to_account TEXT,to_region TEXT,to_kind TEXT,to_id TEXT,relation TEXT,source TEXT,PRIMARY KEY(from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source),FOREIGN KEY(from_account,from_region,from_kind,from_id) REFERENCES resources(account,region,kind,id) ON DELETE CASCADE,FOREIGN KEY(to_account,to_region,to_kind,to_id) REFERENCES resources(account,region,kind,id) ON DELETE CASCADE) WITHOUT ROWID").execute(pool).await?;
    query("INSERT INTO kinds VALUES ('AWS::EC2::Instance','ec2_instances'),('AWS::IAM::Role','iam_roles')").execute(pool).await?;
    Ok(())
}

fn encode_record(index: usize, name: &str) -> Result<(Vec<u8>, [u8; 32]), Box<dyn Error>> {
    let mut builder = RecordBuilder::new(4096, 16, 4)?;
    builder.begin_map(None)?;
    builder.scalar(Some(KeyRef::Name("name")), ValueRef::Text(name))?;
    builder.scalar(
        Some(KeyRef::Name("sequence")),
        ValueRef::Int(i64::try_from(index)?),
    )?;
    builder.end()?;
    let record = builder.finish()?;
    let hash = record.content_hash(&Exclusions::none())?;
    Ok((record.into_bytes(), *hash.as_bytes()))
}

fn account_id(account: usize) -> Result<String, Box<dyn Error>> {
    if account > 999_999_999_999 {
        return Err(USAGE.into());
    }
    Ok(format!("{account:012}"))
}

struct BenchRow<'a> {
    account: &'a str,
    region: &'static str,
    kind: &'static str,
    id: String,
    name: String,
    hash: [u8; 32],
    record: Vec<u8>,
    time: i64,
}

async fn insert_resource(
    pool: &SqlitePool,
    account: &str,
    index: usize,
) -> Result<(), Box<dyn Error>> {
    let (region, kind, table, id) = bench_shape(index);
    let name = format!("resource-{index}");
    let (record, hash) = encode_record(index, &name)?;
    let time = i64::try_from(index)?;
    let row = BenchRow {
        account,
        region,
        kind,
        id,
        name,
        hash,
        record,
        time,
    };
    insert_detail(pool, table, &row).await?;
    insert_summary(pool, &row).await
}

fn bench_shape(index: usize) -> (&'static str, &'static str, &'static str, String) {
    match index % 3 {
        0 => (
            "global",
            "AWS::IAM::Role",
            "iam_roles",
            format!("role-{index}"),
        ),
        1 => (
            "eu-west-1",
            "AWS::EC2::Instance",
            "ec2_instances",
            format!("i-{index:012x}"),
        ),
        _ => (
            "us-east-1",
            "AWS::EC2::Instance",
            "ec2_instances",
            format!("i-{index:012x}"),
        ),
    }
}

async fn insert_detail(
    pool: &SqlitePool,
    table: &str,
    row: &BenchRow<'_>,
) -> Result<(), Box<dyn Error>> {
    let mut detail = sqlx::QueryBuilder::<sqlx::Sqlite>::new("INSERT INTO \"");
    detail.push(table).push("\" VALUES (");
    let mut values = detail.separated(",");
    values
        .push_bind(row.account)
        .push_bind(row.region)
        .push_bind(&row.id);
    values
        .push_bind(row.hash.as_slice())
        .push_bind(row.time)
        .push_bind(row.time);
    values.push_bind(&row.record);
    detail.push(")");
    detail.build().execute(pool).await?;
    Ok(())
}

async fn insert_summary(pool: &SqlitePool, row: &BenchRow<'_>) -> Result<(), Box<dyn Error>> {
    query("INSERT INTO resources VALUES (?,?,?,?,?,?,?,?)")
        .bind(row.account)
        .bind(row.region)
        .bind(row.kind)
        .bind(&row.id)
        .bind(&row.name)
        .bind(row.hash.as_slice())
        .bind(row.time)
        .bind(row.time)
        .execute(pool)
        .await?;
    Ok(())
}

pub(crate) async fn generate(
    path: PathBuf,
    accounts: usize,
    per_account: usize,
) -> Result<(), Box<dyn Error>> {
    ensure_fresh(&path)?;
    let pool = connect_writer(&path).await?;
    execute_schema(&pool).await?;
    let pool_ref = &pool;
    stream::iter(0..accounts)
        .map(Ok)
        .try_for_each(|account| async move {
            let account = account_id(account.checked_add(1).ok_or(USAGE)?)?;
            stream::iter(0..per_account)
                .map(Ok)
                .try_for_each(|index| insert_resource(pool_ref, &account, index))
                .await
        })
        .await?;
    pool.close().await;
    let total = accounts.checked_mul(per_account).ok_or(USAGE)?;
    println!(
        "created_synthetic_reader_fixture schema_version=15 resources={total} database={}",
        path.display()
    );
    Ok(())
}
