mod commit;
mod completion;
mod config;
mod connection;
mod lease;
mod outbox;
mod references;
mod url;
mod work;

pub mod schema;

pub use config::PgWriteConfig;
pub use url::DatabaseUrl;

use crate::inventory::{MAX_CATALOG_BYTES, SchemaRegistry, StoreError};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

pub struct PgStore {
    pub(crate) pool: PgPool,
    pub(crate) registry: SchemaRegistry,
    pub(crate) config: PgWriteConfig,
}

impl PgStore {
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

    pub async fn close(self) {
        self.pool.close().await;
    }
}

async fn reopen(pool: &PgPool) -> Result<SchemaRegistry, StoreError> {
    connection::check_version(pool).await?;
    schema::load_registry(pool).await
}
