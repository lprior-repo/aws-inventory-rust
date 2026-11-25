use super::{BinaryStore, SchemaRegistry, StoreError};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{Row, Sqlite, SqlitePool, Transaction};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, MutexGuard};

impl BinaryStore {
    /// Queue for SQLite outside the pool's connection-acquisition deadline.
    /// The caller keeps the FIFO turn until its transaction has been settled or dropped.
    pub(super) async fn begin_write(
        &self,
    ) -> Result<(MutexGuard<'_, ()>, Transaction<'static, Sqlite>), StoreError> {
        let turn = self.writer_turn.lock().await;
        Ok((turn, self.writer.begin().await?))
    }

    pub(super) async fn read_turn(&self) -> Option<MutexGuard<'_, ()>> {
        if self.reader_shares_writer {
            Some(self.writer_turn.lock().await)
        } else {
            None
        }
    }
}

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
        writer_turn: Arc::new(Mutex::new(())),
        reader_shares_writer: path == Path::new(":memory:"),
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
    if !matches!(version, 14..=17) {
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
        writer_turn: Arc::new(Mutex::new(())),
        reader_shares_writer: path == Path::new(":memory:"),
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
