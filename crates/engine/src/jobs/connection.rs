use super::*;

impl JobStore {
    #[tracing::instrument(skip(path))]
    pub async fn open(path: impl AsRef<Path>) -> Result<Self, JobError> {
        let path = path.as_ref();
        if path.as_os_str().is_empty() {
            return Err(JobError::Invalid("empty queue path"));
        }
        let options = SqliteConnectOptions::new()
            .filename(path)
            .in_memory(path == Path::new(":memory:"))
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Full)
            .busy_timeout(Duration::from_secs(5))
            .disable_statement_logging();
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_secs(10))
            .connect_with(options)
            .await?;
        if let Err(error) = initialize(&pool).await {
            pool.close().await;
            return Err(error);
        }
        Ok(Self { pool })
    }

    #[tracing::instrument(skip(self))]
    pub async fn close(self) {
        self.pool.close().await;
    }
}
#[tracing::instrument(skip(pool))]
pub(super) async fn initialize(pool: &SqlitePool) -> Result<(), JobError> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let application: i64 = sqlx::query_scalar("PRAGMA application_id")
        .fetch_one(&mut *tx)
        .await?;
    let tables: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%'",
    )
    .fetch_one(&mut *tx)
    .await?;
    if application == 0 && tables == 0 {
        sqlx::raw_sql(include_str!("../../sql/jobs.sql"))
            .execute(&mut *tx)
            .await?;
    } else if application == APPLICATION_ID {
        let version: i64 =
            sqlx::query_scalar("SELECT version FROM inventory_job_schema WHERE singleton=1")
                .fetch_one(&mut *tx)
                .await?;
        if version != 1 {
            return Err(JobError::ForeignDatabase);
        }
    } else {
        return Err(JobError::ForeignDatabase);
    }
    tx.commit().await?;
    Ok(())
}
