use super::{BinaryStore, ProjectionDelivery, StoreError};
use sqlx::{
    Connection, SqliteConnection,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteLockingMode, SqliteSynchronous},
};
use std::time::Duration;

pub struct BinaryDelivery {
    connection: SqliteConnection,
    generation: i64,
}

impl ProjectionDelivery for BinaryDelivery {
    fn generation(&self) -> i64 {
        self.generation
    }

    async fn release(self) -> Result<(), StoreError> {
        self.connection.close().await?;
        Ok(())
    }
}

pub(super) async fn acquire(store: &BinaryStore) -> Result<BinaryDelivery, StoreError> {
    let options = connection_options(store).await?;
    let mut connection = SqliteConnection::connect_with(&options)
        .await
        .map_err(delivery_error)?;
    let generation = allocate(&mut connection).await.map_err(delivery_error)?;
    Ok(BinaryDelivery {
        connection,
        generation,
    })
}

async fn connection_options(store: &BinaryStore) -> Result<SqliteConnectOptions, StoreError> {
    let options = store.reader.connect_options();
    let file = options.get_filename();
    if file.as_os_str().is_empty() || file == std::path::Path::new(":memory:") {
        return Err(StoreError::Invalid(
            "projection requires a file-backed inventory",
        ));
    }
    let mut path = tokio::fs::canonicalize(file).await?.into_os_string();
    path.push(".projection-lock.sqlite");
    Ok(SqliteConnectOptions::new()
        .filename(std::path::Path::new(&path))
        .create_if_missing(true)
        .busy_timeout(Duration::ZERO)
        .journal_mode(SqliteJournalMode::Delete)
        .synchronous(SqliteSynchronous::Full)
        .locking_mode(SqliteLockingMode::Exclusive))
}

async fn allocate(connection: &mut SqliteConnection) -> Result<i64, sqlx::Error> {
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

fn delivery_error(error: sqlx::Error) -> StoreError {
    if error
        .as_database_error()
        .is_some_and(|error| matches!(error.code().as_deref(), Some("5" | "6")))
    {
        StoreError::DeliveryBusy
    } else {
        StoreError::from(error)
    }
}
