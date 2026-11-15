use super::{Permissions, StoreError};
use super::super::SchemaRegistry;
use inventory_core::inventory::KindSchema;
use sqlx::{Row, Sqlite, SqlitePool, Transaction};
use std::{collections::BTreeSet, sync::Arc};

pub(in crate::inventory) async fn persist(tx: &mut Transaction<'_, Sqlite>, registry: &SchemaRegistry) -> Result<(), StoreError> {
    super::provenance::initialize(tx).await?;
    if let Some(catalog) = registry.permissions.catalog() {
        sqlx::query("INSERT INTO metadata(key,value) VALUES('reviewed_catalog',?1) ON CONFLICT(key) DO NOTHING")
            .bind(catalog).execute(&mut **tx).await?;
    }
    Ok(())
}

pub(in crate::inventory) async fn load_registry(pool: &SqlitePool, digest: [u8; 32], schemas: Vec<Arc<KindSchema>>,
    operations: BTreeSet<(String, String)>) -> Result<SchemaRegistry, StoreError> {
    let row = sqlx::query("SELECT value FROM metadata WHERE key='reviewed_catalog'").fetch_optional(pool).await?;
    let registry = match row {
        Some(row) => {
            let bytes: Vec<u8> = row.try_get(0)?;
            let registry = SchemaRegistry::new(&bytes, schemas)?;
            if registry.digest() != &digest || registry.operations() != &operations { return Err(StoreError::CatalogMismatch); }
            registry
        }
        None => SchemaRegistry::stored(digest, schemas, operations, Permissions::Unavailable)?,
    };
    initialize(pool).await?;
    Ok(registry)
}

async fn initialize(pool: &SqlitePool) -> Result<(), StoreError> {
    let mut tx = pool.begin().await?;
    super::provenance::initialize(&mut tx).await?;
    tx.commit().await?;
    Ok(())
}

pub(in crate::inventory) async fn open_with_catalog(path: &std::path::Path, catalog: &[u8]) -> Result<super::super::BinaryStore, StoreError> {
    if catalog.is_empty() || catalog.len() > super::super::MAX_CATALOG_BYTES {
        return Err(StoreError::Invalid("catalog size exceeds storage bounds"));
    }
    let mut store = super::super::connection::open(path).await?;
    store.validate_catalog(catalog).await?;
    let registry = SchemaRegistry::new(catalog, store.registry.schemas().to_vec())?;
    if registry.operations() != store.registry.operations() { return Err(StoreError::CatalogMismatch); }
    let mut tx = store.writer.begin().await?;
    persist(&mut tx, &registry).await?;
    tx.commit().await?;
    store.registry = registry;
    Ok(store)
}
