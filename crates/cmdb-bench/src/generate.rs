use crate::{BenchDatabase, USAGE};
use inventory_core::catalog::runtime::RuntimeCatalog;
use inventory_core::record::{Exclusions, KeyRef, RecordBuilder, ValueRef};
use inventory_engine::inventory::{DatabaseUrl, PgStore, PgWriteConfig, SchemaRegistry};
use sqlx::{Connection, PgConnection, Postgres, QueryBuilder};
use std::collections::BTreeMap;
use std::error::Error;
use std::path::PathBuf;

const BENCH_KINDS: [&str; 2] = ["AWS::IAM::Role", "AWS::EC2::Instance"];

fn benchmark_tables(
    registry: &SchemaRegistry,
) -> Result<BTreeMap<String, String>, Box<dyn Error>> {
    let tables: BTreeMap<String, String> = registry
        .schemas()
        .iter()
        .map(|schema| {
            (
                schema.kind().as_str().to_owned(),
                schema.table().as_str().to_owned(),
            )
        })
        .collect();
    if BENCH_KINDS.iter().any(|kind| !tables.contains_key(*kind)) {
        return Err("catalog does not define a benchmark kind".into());
    }
    Ok(tables)
}

async fn ensure_fresh_database(connection: &mut PgConnection) -> Result<(), Box<dyn Error>> {
    let occupied: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace WHERE c.relkind IN ('r','p','v','m','f','S') AND n.nspname NOT IN ('pg_catalog','information_schema') AND n.nspname NOT LIKE 'pg_toast%') OR EXISTS(SELECT 1 FROM pg_namespace WHERE nspname = 'inventory')",
    )
    .fetch_one(&mut *connection)
    .await?;
    if occupied {
        return Err("target PostgreSQL database is not fresh; refusing to modify it".into());
    }
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
    connection: &mut PgConnection,
    tables: &BTreeMap<String, String>,
    account: &str,
    index: usize,
) -> Result<(), Box<dyn Error>> {
    let (region, kind, id) = bench_shape(index);
    let table = tables
        .get(kind)
        .ok_or("catalog does not define a benchmark kind")?;
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
    insert_detail(connection, table, &row).await?;
    insert_summary(connection, &row).await
}

fn bench_shape(index: usize) -> (&'static str, &'static str, String) {
    match index % 3 {
        0 => ("global", "AWS::IAM::Role", format!("role-{index}")),
        1 => (
            "eu-west-1",
            "AWS::EC2::Instance",
            format!("i-{index:012x}"),
        ),
        _ => (
            "us-east-1",
            "AWS::EC2::Instance",
            format!("i-{index:012x}"),
        ),
    }
}

async fn insert_detail(
    connection: &mut PgConnection,
    table: &str,
    row: &BenchRow<'_>,
) -> Result<(), Box<dyn Error>> {
    let mut detail = QueryBuilder::<Postgres>::new("INSERT INTO inventory.\"");
    detail
        .push(table)
        .push("\"(account,region,id,content_hash,first_seen,last_changed,record) VALUES (");
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
    detail.build().execute(connection).await?;
    Ok(())
}

async fn insert_summary(
    connection: &mut PgConnection,
    row: &BenchRow<'_>,
) -> Result<(), Box<dyn Error>> {
    sqlx::query(
        "INSERT INTO inventory.resources (account,region,kind,id,name,content_hash,first_seen,last_changed) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
    )
    .bind(row.account)
    .bind(row.region)
    .bind(row.kind)
    .bind(&row.id)
    .bind(&row.name)
    .bind(row.hash.as_slice())
    .bind(row.time)
    .bind(row.time)
    .execute(connection)
    .await?;
    Ok(())
}

pub(crate) async fn generate(
    target: BenchDatabase,
    catalog: PathBuf,
    accounts: usize,
    per_account: usize,
) -> Result<(), Box<dyn Error>> {
    let total = accounts.checked_mul(per_account).ok_or(USAGE)?;
    let bytes = tokio::fs::read(&catalog).await?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let registry = SchemaRegistry::new(&bytes, runtime.schemas().to_vec())?;
    let tables = benchmark_tables(&registry)?;
    let mut connection = PgConnection::connect(target.dsn()).await?;
    ensure_fresh_database(&mut connection).await?;
    let url = DatabaseUrl::parse(target.dsn())?;
    let store = PgStore::create_fresh(&url, registry, PgWriteConfig::default()).await?;
    store.close().await;
    let mut transaction = connection.begin().await?;
    for account in 1..=accounts {
        let account = account_id(account)?;
        for index in 0..per_account {
            insert_resource(&mut transaction, &tables, &account, index).await?;
        }
    }
    transaction.commit().await?;
    connection.close().await?;
    println!("created_synthetic_reader_fixture resources={total} database={url}");
    Ok(())
}
