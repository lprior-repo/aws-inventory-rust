use super::{BinaryStore, SchemaRegistry, StoreError};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{Row, Sqlite, SqlitePool, Transaction};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, MutexGuard};

impl BinaryStore {
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

/// What a database path holds, as far as choosing between `open` and `create`.
///
/// `schema::initialize` publishes its work atomically (one transaction ending in
/// `PRAGMA user_version=17`), so a file that exists but reports version 0 with no
/// tables is an interrupted or never-started initialization that `create` may
/// safely resume; it must never be handed to `open`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreState {
    /// No file, or a zero-length file.
    Absent,
    /// A SQLite file with `user_version = 0` and no tables.
    Uninitialized,
    /// A file with a non-zero schema version; `open` decides whether it is supported.
    Initialized,
}

pub(super) async fn probe(path: &Path) -> Result<StoreState, StoreError> {
    if !std::fs::metadata(path).is_ok_and(|metadata| metadata.len() > 0) {
        return Ok(StoreState::Absent);
    }
    let pool = pool().connect_with(options(path, false)?).await?;
    let result = probe_open(&pool).await;
    pool.close().await;
    result
}

async fn probe_open(pool: &SqlitePool) -> Result<StoreState, StoreError> {
    let version: i64 = sqlx::query_scalar("PRAGMA user_version")
        .fetch_one(pool)
        .await?;
    if version != 0 {
        return Ok(StoreState::Initialized);
    }
    let objects: i64 = sqlx::query_scalar("SELECT count(*) FROM sqlite_master")
        .fetch_one(pool)
        .await?;
    if objects == 0 {
        Ok(StoreState::Uninitialized)
    } else {
        Err(StoreError::SchemaVersion { found: 0 })
    }
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
