//! Durable per-batch graph acknowledgement log.
use super::*;

pub(super) struct ProjectionLog {
    pub(super) pool: SqlitePool,
    pub(super) destination: String,
}

impl ProjectionLog {
    pub(super) fn prepare(path: &Path, destination: String) -> Self {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Full)
            .busy_timeout(Duration::from_secs(5));
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_lazy_with(options);
        Self { pool, destination }
    }

    #[tracing::instrument(skip_all)]
    pub(super) async fn initialize(&self) -> Result<(), GraphError> {
        sqlx::query("CREATE TABLE IF NOT EXISTS projection_ack (destination TEXT NOT NULL, graph TEXT NOT NULL, scan_id TEXT NOT NULL, sequence INTEGER NOT NULL, digest TEXT NOT NULL, PRIMARY KEY(destination,graph,scan_id,sequence))")
            .execute(&self.pool).await?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub(super) async fn completed(
        &self,
        batch: &GraphBatch,
        digest: &str,
    ) -> Result<bool, GraphError> {
        let sequence = i64::try_from(batch.sequence)
            .map_err(|_| GraphError::Invalid("sequence overflow".into()))?;
        let found: Option<String> = sqlx::query_scalar("SELECT digest FROM projection_ack WHERE destination=?1 AND graph=?2 AND scan_id=?3 AND sequence=?4")
            .bind(&self.destination).bind(&batch.graph).bind(&batch.scan_id).bind(sequence).fetch_optional(&self.pool).await?;
        match found {
            Some(old) if old == digest => Ok(true),
            Some(_) => Err(GraphError::Invalid(
                "projection batch changed after checkpoint; use a new scan ID".into(),
            )),
            None => Ok(false),
        }
    }

    #[tracing::instrument(skip_all)]
    pub(super) async fn commit(&self, batch: &GraphBatch, digest: &str) -> Result<(), GraphError> {
        let sequence = i64::try_from(batch.sequence)
            .map_err(|_| GraphError::Invalid("sequence overflow".into()))?;
        sqlx::query("INSERT INTO projection_ack(destination,graph,scan_id,sequence,digest) VALUES (?1,?2,?3,?4,?5)")
            .bind(&self.destination).bind(&batch.graph).bind(&batch.scan_id).bind(sequence).bind(digest).execute(&self.pool).await?;
        Ok(())
    }
}
