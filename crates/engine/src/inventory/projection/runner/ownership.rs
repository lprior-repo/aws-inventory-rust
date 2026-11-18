use super::{BinaryStore, ProjectionError, StoreError};
use sqlx::{
    Connection, SqliteConnection,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteLockingMode, SqliteSynchronous},
};
use std::time::Duration;

/// A separate SQLite file owns delivery, never the inventory writer transaction.
/// The lock covers every graph effect, snapshot and acknowledgement, including
/// rebuilds. EXCLUSIVE locking mode retains the lock after committing a durable
/// generation, until the connection/process dies; it has no TTL.
/// Never remove/reset this persistent sidecar while its inventory graph exists.
pub(super) struct DeliveryOwnership {
    connection: SqliteConnection,
    generation: i64,
}

impl DeliveryOwnership {
    pub(super) fn generation(&self) -> i64 {
        self.generation
    }

    pub(super) async fn release(self) -> Result<(), ProjectionError> {
        self.connection.close().await.map_err(StoreError::from)?;
        Ok(())
    }
}

pub(super) async fn acquire(store: &BinaryStore) -> Result<DeliveryOwnership, ProjectionError> {
    // Reading immutable pool options never acquires a reader connection;
    // an existing projector may retain that connection throughout its snapshot.
    let connection_options = store.reader.connect_options();
    let file = connection_options.get_filename();
    if file.as_os_str().is_empty() || file == std::path::Path::new(":memory:") {
        return Err(StoreError::Invalid("projection requires a file-backed inventory").into());
    }
    let canonical = tokio::fs::canonicalize(file).await?;
    let mut path = canonical.into_os_string();
    path.push(".projection-lock.sqlite");
    let options = SqliteConnectOptions::new()
        .filename(std::path::Path::new(&path))
        .create_if_missing(true)
        .busy_timeout(Duration::ZERO)
        .journal_mode(SqliteJournalMode::Delete)
        .synchronous(SqliteSynchronous::Full)
        .locking_mode(SqliteLockingMode::Exclusive);
    let mut connection = SqliteConnection::connect_with(&options)
        .await
        .map_err(delivery_error)?;
    let generation = allocate(&mut connection).await.map_err(delivery_error)?;
    Ok(DeliveryOwnership {
        connection,
        generation,
    })
}

async fn allocate(connection: &mut SqliteConnection) -> Result<i64, sqlx::Error> {
    // Commit before any Redis request, even if its caller subsequently cancels.
    // EXCLUSIVE locking_mode retains ownership continuously across this commit.
    sqlx::query("BEGIN EXCLUSIVE")
        .execute(&mut *connection)
        .await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS delivery_generation (id INTEGER PRIMARY KEY CHECK(id=1), generation INTEGER NOT NULL CHECK(generation>0)) STRICT")
        .execute(&mut *connection).await?;
    let generation = sqlx::query_scalar("INSERT INTO delivery_generation(id,generation) VALUES(1,1) ON CONFLICT(id) DO UPDATE SET generation=generation+1 RETURNING generation")
        .fetch_one(&mut *connection).await?;
    sqlx::query("COMMIT").execute(connection).await?;
    Ok(generation)
}

fn delivery_error(error: sqlx::Error) -> ProjectionError {
    if is_busy(&error) {
        ProjectionError::DeliveryBusy
    } else {
        StoreError::from(error).into()
    }
}

fn is_busy(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .is_some_and(|error| matches!(error.code().as_deref(), Some("5" | "6")))
}
