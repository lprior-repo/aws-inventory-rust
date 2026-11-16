use super::TestResult;
use sqlx::{Row, SqlitePool};

#[derive(Debug, PartialEq, Eq)]
pub(super) struct DurableState {
    version: i64,
    schema: Vec<(String, String, String, Option<String>)>,
    tables: Vec<TableState>,
}

#[derive(Debug, PartialEq, Eq)]
struct TableState {
    name: String,
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
}

// Snapshot every persisted column, including new facet/candidate metadata, rather
// than silently omitting a table when the inventory schema gains another field.
pub(super) async fn read_durable_state(pool: &SqlitePool) -> TestResult<DurableState> {
    let version = sqlx::query_scalar("PRAGMA user_version").fetch_one(pool).await?;
    let schema = sqlx::query_as("SELECT type,name,tbl_name,sql FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%' ORDER BY type,name")
        .fetch_all(pool).await?;
    let names: Vec<String> = sqlx::query_scalar("SELECT name FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .fetch_all(pool).await?;
    let mut tables = Vec::with_capacity(names.len());
    for name in names {
        tables.push(read_table(pool, name).await?);
    }
    Ok(DurableState { version, schema, tables })
}

async fn read_table(pool: &SqlitePool, name: String) -> TestResult<TableState> {
    let columns: Vec<String> = sqlx::query_scalar("SELECT name FROM pragma_table_info(?1) ORDER BY cid")
        .bind(&name).fetch_all(pool).await?;
    let quoted = columns.iter().map(|column| quote_identifier(column)).collect::<Vec<_>>();
    let values = quoted.iter().map(|column| format!("quote({column})")).collect::<Vec<_>>().join(",");
    let mut query = sqlx::QueryBuilder::<sqlx::Sqlite>::new(format!("SELECT {values} FROM {} ORDER BY {}", quote_identifier(&name), quoted.join(",")));
    let rows = query.build().fetch_all(pool).await?.iter().map(|row| {
        (0..columns.len()).map(|index| row.try_get::<String, _>(index)).collect::<Result<Vec<_>, _>>()
    }).collect::<Result<Vec<_>, _>>()?;
    Ok(TableState { name, columns, rows })
}

fn quote_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}
