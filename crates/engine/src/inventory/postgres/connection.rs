use super::{DatabaseUrl, PgWriteConfig};
use crate::inventory::StoreError;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;

pub(super) async fn connect(
    url: &DatabaseUrl,
    config: PgWriteConfig,
) -> Result<PgPool, StoreError> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections())
        .acquire_timeout(Duration::from_secs(5))
        .after_connect(|connection, _| {
            Box::pin(async move {
                sqlx::query("SET search_path = inventory")
                    .execute(&mut *connection)
                    .await?;
                sqlx::query("SET application_name = 'aws-inventory'")
                    .execute(connection)
                    .await?;
                Ok(())
            })
        })
        .connect_with(url.options()?)
        .await
        .map_err(|_| StoreError::Invalid("PostgreSQL connection failed"))?;
    let version = pool.acquire().await?.server_version_num();
    if !crate::postgres_version::supported(version) {
        pool.close().await;
        return Err(StoreError::UnsupportedPostgresVersion);
    }
    Ok(pool)
}

