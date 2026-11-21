use super::super::SchemaRegistry;
use super::{Permissions, StoreError};
use inventory_core::inventory::KindSchema;
use sqlx::{Row, Sqlite, SqlitePool, Transaction};
use std::{collections::BTreeSet, sync::Arc};

/// Each metadata value stays below the bundled SQLite `MAX_LENGTH` (8 MiB), so the reviewed
/// catalog is stored as a head row plus ordered `reviewed_catalog.NN` continuation rows.
const CHUNK_BYTES: usize = 4 * 1024 * 1024;
const MAX_CHUNKS: usize = super::super::MAX_CATALOG_BYTES.div_ceil(CHUNK_BYTES);
// Continuation keys carry two digits, so the head plus `.01`-`.99` is the hard ceiling.
const _: () = assert!(MAX_CHUNKS <= 100);

pub(in crate::inventory) async fn persist(
    tx: &mut Transaction<'_, Sqlite>,
    registry: &SchemaRegistry,
) -> Result<(), StoreError> {
    super::provenance::initialize(tx).await?;
    let Some(catalog) = registry.permissions.catalog() else {
        return Ok(());
    };
    let mut chunks = catalog.chunks(CHUNK_BYTES);
    let Some(head) = chunks.next() else {
        return Ok(());
    };
    // The head row is the commit marker: an existing head keeps the stored catalog untouched.
    let inserted = sqlx::query(
        "INSERT INTO metadata(key,value) VALUES('reviewed_catalog',?1) ON CONFLICT(key) DO NOTHING",
    )
    .bind(head)
    .execute(&mut **tx)
    .await?
    .rows_affected();
    if inserted == 0 {
        return Ok(());
    }
    sqlx::query("DELETE FROM metadata WHERE key GLOB 'reviewed_catalog.[0-9][0-9]'")
        .execute(&mut **tx)
        .await?;
    for (index, chunk) in chunks.enumerate() {
        sqlx::query("INSERT INTO metadata(key,value) VALUES(?1,?2)")
            .bind(format!("reviewed_catalog.{:02}", index.saturating_add(1)))
            .bind(chunk)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

pub(in crate::inventory) async fn load_registry(
    pool: &SqlitePool,
    digest: [u8; 32],
    schemas: Vec<Arc<KindSchema>>,
    operations: BTreeSet<(String, String)>,
) -> Result<SchemaRegistry, StoreError> {
    // Binary key order puts the head row first, then continuation rows in chunk order.
    let rows = sqlx::query("SELECT key,value FROM metadata WHERE key='reviewed_catalog' OR key GLOB 'reviewed_catalog.[0-9][0-9]' ORDER BY key LIMIT ?1")
        .bind(i64::try_from(MAX_CHUNKS).map_err(|_| StoreError::Invalid("catalog chunk bound"))?)
        .fetch_all(pool).await?;
    let stored = rows
        .first()
        .map(|row| row.try_get::<&str, _>(0))
        .transpose()?
        .is_some_and(|key| key == "reviewed_catalog");
    let registry = if stored {
        let mut bytes = Vec::new();
        for row in &rows {
            let chunk: &[u8] = row.try_get(1)?;
            if bytes.len().saturating_add(chunk.len()) > super::super::MAX_CATALOG_BYTES {
                return Err(StoreError::CatalogMismatch);
            }
            bytes.extend_from_slice(chunk);
        }
        let registry = SchemaRegistry::new(&bytes, schemas)?;
        if registry.digest() != &digest || registry.operations() != &operations {
            return Err(StoreError::CatalogMismatch);
        }
        registry
    } else {
        SchemaRegistry::stored(digest, schemas, operations, Permissions::Unavailable)?
    };
    Ok(registry)
}

pub(in crate::inventory) async fn open_with_catalog(
    path: &std::path::Path,
    catalog: &[u8],
) -> Result<super::super::BinaryStore, StoreError> {
    if catalog.is_empty() || catalog.len() > super::super::MAX_CATALOG_BYTES {
        return Err(StoreError::Invalid("catalog size exceeds storage bounds"));
    }
    let mut store = super::super::connection::open(path).await?;
    store.validate_catalog(catalog).await?;
    let registry = SchemaRegistry::new(catalog, store.registry.schemas().to_vec())?;
    if registry.operations() != store.registry.operations() {
        return Err(StoreError::CatalogMismatch);
    }
    let mut tx = store.writer.begin().await?;
    persist(&mut tx, &registry).await?;
    tx.commit().await?;
    store.registry = registry;
    Ok(store)
}
