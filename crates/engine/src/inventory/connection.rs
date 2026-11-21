use super::{BinaryStore, SchemaRegistry, StoreError};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{Row, SqlitePool};
use std::path::Path;
use std::time::Duration;

pub(super) async fn create(
    path: &Path,
    registry: SchemaRegistry,
) -> Result<BinaryStore, StoreError> {
    let writer = pool()
        .connect_with(options(path, true)?.journal_mode(SqliteJournalMode::Wal))
        .await?;
    let version: i64 = sqlx::query("PRAGMA user_version")
        .fetch_one(&writer)
        .await?
        .try_get(0)?;
    if version != 0 {
        writer.close().await;
        return Err(StoreError::SchemaVersion { found: version });
    }
    if let Err(error) = super::schema::initialize(&writer, &registry).await {
        writer.close().await;
        return Err(error);
    }
    let reader = reader_pool(path, &writer).await?;
    Ok(BinaryStore {
        writer,
        reader,
        registry,
    })
}

pub(super) async fn open(path: &Path) -> Result<BinaryStore, StoreError> {
    let writer = pool()
        .connect_with(options(path, false)?.journal_mode(SqliteJournalMode::Wal))
        .await?;
    let version: i64 = sqlx::query("PRAGMA user_version")
        .fetch_one(&writer)
        .await?
        .try_get(0)?;
    if !matches!(version, 14..=16) {
        writer.close().await;
        return Err(StoreError::SchemaVersion { found: version });
    }
    let registry = match super::schema::load_registry(&writer).await {
        Ok(registry) => registry,
        Err(error) => {
            writer.close().await;
            return Err(error);
        }
    };
    if let Err(error) = super::schema::migrate_inventory(&writer).await {
        writer.close().await;
        return Err(error);
    }
    let reader = reader_pool(path, &writer).await?;
    Ok(BinaryStore {
        writer,
        reader,
        registry,
    })
}

async fn reader_pool(path: &Path, writer: &SqlitePool) -> Result<SqlitePool, StoreError> {
    if path == Path::new(":memory:") {
        return Ok(writer.clone());
    }
    // WAL readers do not block each other; a retained projection snapshot must not starve
    // latest-state reads. The writer pool stays single-connection.
    Ok(pool()
        .max_connections(4)
        .connect_with(
            options(path, false)?
                .read_only(true)
                .pragma("query_only", "ON")
                .journal_mode(SqliteJournalMode::Wal),
        )
        .await?)
}

fn pool() -> SqlitePoolOptions {
    SqlitePoolOptions::new()
        .max_connections(1)
        .min_connections(1)
        .idle_timeout(None)
        .max_lifetime(None)
        .acquire_timeout(Duration::from_secs(5))
}

fn options(path: &Path, create: bool) -> Result<SqliteConnectOptions, StoreError> {
    let name = path
        .to_str()
        .ok_or(StoreError::Invalid("database path is not UTF-8"))?;
    if name.is_empty() || name.starts_with("file:") || name.contains('\0') {
        return Err(StoreError::Invalid("invalid database path"));
    }
    // Pool options are also the immutable delivery-lock path capability. Resolve
    // relative disk paths at open, but retain SQLite's special in-memory name.
    let options = SqliteConnectOptions::new();
    let options = if path == Path::new(":memory:") {
        options.filename(path)
    } else {
        options.filename(std::path::absolute(path).map_err(sqlx::Error::Io)?)
    };
    Ok(options
        .create_if_missing(create)
        .foreign_keys(true)
        .synchronous(SqliteSynchronous::Full)
        .busy_timeout(Duration::from_secs(5))
        .pragma("trusted_schema", "OFF")
        .pragma("recursive_triggers", "ON")
        .pragma("temp_store", "FILE"))
}
