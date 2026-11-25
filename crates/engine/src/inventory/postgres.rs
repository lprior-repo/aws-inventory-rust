mod commit;
mod completion;
mod config;
mod connection;
mod copy;
mod lease;
mod outbox;
mod projection_api;
mod projection_delivery;
mod projection_read;
mod projection_state;
mod references;
mod summary;
mod url;
mod work;

pub mod schema;

pub use config::PgWriteConfig;
pub use projection_delivery::PgDelivery;
pub use projection_read::PgProjectionSnapshot;
pub use url::DatabaseUrl;

use crate::inventory::{
    MAX_CATALOG_BYTES, SchemaRegistry, StoreError, SweepId, UnobservedScope, decode_scope_key,
};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};

pub struct PgStore {
    pub(crate) pool: PgPool,
    pub(crate) registry: SchemaRegistry,
    pub(crate) config: PgWriteConfig,
}

impl PgStore {
    pub async fn exists(url: &DatabaseUrl, config: PgWriteConfig) -> Result<bool, StoreError> {
        let pool = connection::connect(url, config).await?;
        let result = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM pg_namespace WHERE nspname='inventory')",
        )
        .fetch_one(&pool)
        .await;
        pool.close().await;
        result.map_err(StoreError::from)
    }

    pub async fn create(
        url: &DatabaseUrl,
        registry: SchemaRegistry,
        config: PgWriteConfig,
    ) -> Result<Self, StoreError> {
        let pool = connection::connect(url, config).await?;
        match schema::initialize(&pool, &registry).await {
            Ok(()) => Ok(Self {
                pool,
                registry,
                config,
            }),
            Err(error) => {
                pool.close().await;
                Err(error)
            }
        }
    }

    pub async fn open(url: &DatabaseUrl, config: PgWriteConfig) -> Result<Self, StoreError> {
        let pool = connection::connect(url, config).await?;
        let registry = match reopen(&pool).await {
            Ok(registry) => registry,
            Err(error) => {
                pool.close().await;
                return Err(error);
            }
        };
        Ok(Self {
            pool,
            registry,
            config,
        })
    }

    pub fn registry(&self) -> &SchemaRegistry {
        &self.registry
    }

    pub fn config(&self) -> PgWriteConfig {
        self.config
    }

    pub async fn validate_catalog(&self, catalog: &[u8]) -> Result<(), StoreError> {
        if catalog.len() > MAX_CATALOG_BYTES {
            return Err(StoreError::Invalid("catalog size exceeds bounds"));
        }
        if Sha256::digest(catalog).as_slice() != self.registry.digest() {
            return Err(StoreError::CatalogMismatch);
        }
        Ok(())
    }
    pub async fn unobserved_scopes(
        &self,
        sweep: &SweepId,
        now_ms: i64,
    ) -> Result<Vec<UnobservedScope>, StoreError> {
        let rows = sqlx::query("SELECT scope_key,observed_at FROM inventory.scope_outcomes WHERE sweep_id=$1 AND outcome='unobserved' ORDER BY scope_key")
            .bind(sweep.as_str())
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter()
            .map(|row| {
                let bytes: Vec<u8> = row.try_get("scope_key")?;
                let last_observed_ms: i64 = row.try_get("observed_at")?;
                Ok(UnobservedScope {
                    scope: decode_scope_key(&bytes)?,
                    last_observed_ms,
                    age_ms: now_ms.saturating_sub(last_observed_ms).max(0),
                })
            })
            .collect()
    }

    pub async fn close(self) {
        self.pool.close().await;
    }
}

async fn reopen(pool: &PgPool) -> Result<SchemaRegistry, StoreError> {
    connection::check_version(pool).await?;
    schema::load_registry(pool).await
}
