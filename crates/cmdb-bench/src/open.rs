use futures_util::{StreamExt, TryStreamExt, stream};
use inventory_core::catalog::runtime::RuntimeCatalog;
use inventory_engine::inventory::{DatabaseUrl, PgStore, PgWriteConfig, SchemaRegistry};
use inventory_harness::postgres::{TestDatabase, admin_url};
use sqlx::{AssertSqlSafe, Connection, Executor, PgConnection, SqlSafeStr};
use std::{collections::BTreeMap, path::PathBuf, time::Instant};
#[derive(Debug)]
pub(crate) struct OpenArgs {
    pub catalog: PathBuf,
}
pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> Result<OpenArgs, String> {
    match (args.next(), args.next(), args.next()) {
        (Some(flag), Some(path), None)
            if flag == "--catalog" && !path.is_empty() && !path.starts_with("--") =>
        {
            Ok(OpenArgs {
                catalog: path.into(),
            })
        }
        _ => Err("open requires exactly --catalog <file>".into()),
    }
}
pub(crate) async fn run(args: &OpenArgs) -> Result<(), String> {
    let bytes = tokio::fs::read(&args.catalog).await.map_err(text)?;
    let started = Instant::now();
    let runtime = RuntimeCatalog::from_bytes(&bytes).map_err(text)?;
    let decode = started.elapsed();
    let registry = SchemaRegistry::new(&bytes, runtime.schemas().to_vec()).map_err(text)?;
    let database = TestDatabase::create(&admin_url().map_err(text)?)
        .await
        .map_err(text)?;
    let measured = measure(database.url(), registry).await;
    let finished = database.finish().await.map_err(text);
    let (create, reopen, prepare, tables, columns) = measured?;
    finished?;
    println!("P11.catalog_decode_us\t{}", decode.as_micros());
    println!(
        "P11.schema_create_us\t{create}\nP11.reopen_us\t{reopen}\nP11.prepare_inserts_us\t{prepare}\nP11.tables\t{tables}\nP11.columns\t{columns}"
    );
    Ok(())
}
async fn measure(
    url: &str,
    registry: SchemaRegistry,
) -> Result<(u128, u128, u128, usize, usize), String> {
    let database = DatabaseUrl::parse(url).map_err(text)?;
    let config = PgWriteConfig::default();
    let started = Instant::now();
    let store = PgStore::create(&database, registry, config)
        .await
        .map_err(text)?;
    let create = started.elapsed().as_micros();
    store.close().await;
    let started = Instant::now();
    let store = PgStore::open(&database, config).await.map_err(text)?;
    let reopen = started.elapsed().as_micros();
    store.close().await;
    let mut connection = PgConnection::connect(url).await.map_err(text)?;
    let rows: Vec<(String, String)> = sqlx::query_as("SELECT c.table_name::text, c.column_name::text FROM information_schema.columns c JOIN inventory.kinds k ON k.table_name = c.table_name::text WHERE c.table_schema = 'inventory' ORDER BY c.table_name, c.ordinal_position")
        .fetch_all(&mut connection).await.map_err(text)?;
    let kinds = rows.into_iter().fold(
        BTreeMap::<String, Vec<String>>::new(),
        |mut kinds, (table, column)| {
            kinds.entry(table).or_default().push(column);
            kinds
        },
    );
    let started = Instant::now();
    stream::iter(kinds.iter())
        .map(Ok)
        .try_fold(&mut connection, |connection, (table, columns)| async move {
            let sql = insert_sql(table, columns);
            connection
                .prepare(AssertSqlSafe(sql).into_sql_str())
                .await
                .map_err(text)?;
            Ok::<_, String>(connection)
        })
        .await?;
    let prepare = started.elapsed().as_micros();
    connection.close().await.map_err(text)?;
    Ok((
        create,
        reopen,
        prepare,
        kinds.len(),
        kinds.values().map(Vec::len).sum(),
    ))
}
fn insert_sql(table: &str, columns: &[String]) -> String {
    let names = columns
        .iter()
        .map(|name| quote(name))
        .collect::<Vec<_>>()
        .join(",");
    let values = (1..=columns.len())
        .map(|index| format!("${index}"))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "INSERT INTO \"inventory\".{} ({names}) VALUES ({values})",
        quote(table)
    )
}
fn quote(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}
fn text(error: impl std::fmt::Display) -> String {
    error.to_string()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_catalog_argument_and_quoted_prepared_insert() -> Result<(), String> {
        assert_eq!(
            parse(&mut ["--catalog", "a"].map(str::to_owned).into_iter())?.catalog,
            PathBuf::from("a")
        );
        assert!(parse(&mut ["--catalog"].map(str::to_owned).into_iter()).is_err());
        assert_eq!(
            insert_sql("a\"b", &["id".into(), "x".into()]),
            "INSERT INTO \"inventory\".\"a\"\"b\" (\"id\",\"x\") VALUES ($1,$2)"
        );
        Ok(())
    }
    #[tokio::test]
    #[ignore = "requires INVENTORY_POSTGRES_URL"]
    async fn fixture_catalog_creates_reopens_and_prepares_a_real_kind_table()
    -> Result<(), Box<dyn std::error::Error>> {
        let bytes = crate::pipeline_test_catalog::bytes().map_err(text)?;
        let runtime = RuntimeCatalog::from_bytes(&bytes)?;
        let registry = SchemaRegistry::new(&bytes, runtime.schemas().to_vec())?;
        let database = TestDatabase::create(&admin_url()?).await?;
        let measured = measure(database.url(), registry).await;
        let cleanup = database.finish().await;
        let (_, _, _, tables, columns) = measured?;
        cleanup?;
        assert_eq!(tables, 1);
        assert_eq!(columns, 8);
        Ok(())
    }
}
