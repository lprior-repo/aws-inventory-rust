use super::{DatabaseUrl, PgWriteConfig};
use crate::inventory::StoreError;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;

pub(super) async fn connect(
    url: &DatabaseUrl,
    config: PgWriteConfig,
) -> Result<PgPool, StoreError> {
    PgPoolOptions::new()
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
        .map_err(|_| StoreError::Invalid("PostgreSQL connection failed"))
}

pub(super) async fn check_version(pool: &PgPool) -> Result<(), StoreError> {
    let bytes = super::schema::metadata(pool, "schema_version").await?;
    let found = std::str::from_utf8(&bytes)
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .ok_or(StoreError::Invalid("invalid PostgreSQL schema version"))?;
    if found != 14 {
        return Err(StoreError::SchemaVersion { found });
    }
    Ok(())
}
