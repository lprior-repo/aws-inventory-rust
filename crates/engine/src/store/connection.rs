//! Connection lifecycle and bounded SQLite pools.
use super::*;

impl Store {
    /// Open/create the SQLite inventory and migrate its supported schema.
    /// File-backed stores use WAL/FULL synchronization; `:memory:` uses separate
    /// reader and writer pools sharing one isolated in-memory database.
    /// # Errors
    /// Rejects empty paths, unsupported schemas or any database setup/migration failure.
    #[tracing::instrument(skip(path))]
    pub async fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let (options, memory) = connection_options(path.as_ref())?;
        let writer = pool_options()
            .connect_with(
                options
                    .clone()
                    .create_if_missing(true)
                    .journal_mode(SqliteJournalMode::Wal),
            )
            .await?;
        if let Err(error) = initialize_schema(&writer).await {
            writer.close().await;
            return Err(error);
        }
        let reader = pool_options()
            .test_before_acquire(false)
            .before_acquire(|connection, _| {
                Box::pin(async move {
                    connection.lock_handle().await?.remove_progress_handler();
                    Ok(true)
                })
            })
            .connect_with(options.read_only(!memory).pragma("query_only", "ON"))
            .await;
        match reader {
            Ok(reader) => Ok(Self {
                writer,
                reader,
                preparations: TaskTracker::new(),
                preparation_permits: Arc::new(Semaphore::new(1)),
            }),
            Err(error) => {
                writer.close().await;
                Err(error.into())
            }
        }
    }

    /// Consuming shutdown prevents simultaneous borrowers and drains preparation.
    #[tracing::instrument(skip(self))]
    pub async fn close(self) {
        self.preparation_permits.close();
        self.preparations.close();
        self.preparations.wait().await;
        self.reader.close().await;
        self.writer.close().await;
    }
}
pub(super) fn pool_options() -> SqlitePoolOptions {
    SqlitePoolOptions::new()
        .max_connections(1)
        .min_connections(1)
        .idle_timeout(None)
        .max_lifetime(None)
        .acquire_timeout(Duration::from_secs(5))
}

pub(super) fn connection_options(path: &Path) -> Result<(SqliteConnectOptions, bool), StoreError> {
    let name = path
        .to_str()
        .ok_or(StoreError::InvalidInput("database path is not UTF-8"))?;
    if name.is_empty() || name.starts_with("file:") || name.contains('\0') {
        return Err(StoreError::InvalidInput(
            "use a filesystem path or :memory:, not a SQLite URI",
        ));
    }
    let memory = name == ":memory:";
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(false)
        .foreign_keys(true)
        .synchronous(SqliteSynchronous::Full)
        .busy_timeout(Duration::from_secs(5))
        .command_buffer_size(8)
        .row_buffer_size(1)
        .pragma("trusted_schema", "OFF")
        .pragma("temp_store", "FILE")
        .pragma("cache_size", "-8192")
        .disable_statement_logging();
    if memory {
        let id = MEMORY_DATABASE_ID
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| StoreError::Limit("memory database identities"))?;
        Ok((
            options
                .filename(format!(
                    "file:inventory-memory-{}-{id}?mode=memory&cache=shared",
                    std::process::id()
                ))
                .in_memory(true)
                .shared_cache(true),
            true,
        ))
    } else {
        Ok((options, false))
    }
}
