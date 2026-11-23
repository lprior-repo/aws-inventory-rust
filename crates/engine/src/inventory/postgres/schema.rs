mod candidates;
mod kinds;
mod provenance;
mod shared;

use crate::inventory::{SchemaRegistry, StoreError, schema::decode_schema};
use futures_util::{TryStreamExt, stream};
use inventory_core::inventory::KindSchema;
use sqlx::{PgPool, Postgres, Row, Transaction, postgres::PgRow};
use std::sync::Arc;

pub async fn initialize(pool: &PgPool, registry: &SchemaRegistry) -> Result<(), StoreError> {
    initialize_shared(pool, registry).await?;
    stream::iter(registry.schemas().chunks(200).map(Ok))
        .try_for_each(|batch| kinds::create_batch(pool, batch))
        .await?;
    sqlx::query("INSERT INTO inventory.metadata(key,value) VALUES('schema_version',$1)")
        .bind(b"14".as_slice())
        .execute(pool)
        .await?;
    Ok(())
}

async fn initialize_shared(pool: &PgPool, registry: &SchemaRegistry) -> Result<(), StoreError> {
    let mut tx = pool.begin().await?;
    sqlx::query("CREATE SCHEMA inventory")
        .execute(&mut *tx)
        .await?;
    shared::create(&mut tx).await?;
    provenance::create(&mut tx).await?;
    candidates::create(&mut tx).await?;
    persist_registry(&mut tx, registry).await?;
    tx.commit().await?;
    Ok(())
}

async fn persist_registry(
    tx: &mut Transaction<'_, Postgres>,
    registry: &SchemaRegistry,
) -> Result<(), StoreError> {
    let catalog = registry
        .permissions
        .catalog()
        .ok_or(StoreError::Invalid("reviewed catalog missing"))?;
    sqlx::query("INSERT INTO inventory.metadata(key,value) VALUES('catalog_digest',$1),('reviewed_catalog',$2)")
        .bind(registry.digest().as_slice())
        .bind(catalog)
        .execute(&mut **tx)
        .await?;
    persist_operations(tx, registry).await
}

async fn persist_operations(
    tx: &mut Transaction<'_, Postgres>,
    registry: &SchemaRegistry,
) -> Result<(), StoreError> {
    stream::iter(registry.operations().iter().map(Ok))
        .try_fold(tx, |tx, (service, operation)| async move {
            sqlx::query(
                "INSERT INTO inventory.catalog_operations(service,operation) VALUES($1,$2)",
            )
            .bind(service)
            .bind(operation)
            .execute(&mut **tx)
            .await?;
            Ok::<_, StoreError>(tx)
        })
        .await?;
    Ok(())
}

pub async fn load_registry(pool: &PgPool) -> Result<SchemaRegistry, StoreError> {
    let catalog = metadata(pool, "reviewed_catalog").await?;
    if catalog.len() > super::super::MAX_CATALOG_BYTES {
        return Err(StoreError::Invalid("stored catalog exceeds bounds"));
    }
    let registry = SchemaRegistry::new(&catalog, load_schemas(pool).await?)?;
    if registry.digest().as_slice() != metadata(pool, "catalog_digest").await? {
        return Err(StoreError::CatalogMismatch);
    }
    Ok(registry)
}

async fn load_schemas(pool: &PgPool) -> Result<Vec<Arc<KindSchema>>, StoreError> {
    let rows = sqlx::query(
        "SELECT kind,table_name,definition FROM inventory.kind_schemas ORDER BY kind LIMIT 65537",
    )
    .fetch_all(pool)
    .await?;
    if rows.len() > 65_536 {
        return Err(StoreError::Invalid("stored schema count exceeds bounds"));
    }
    rows.into_iter().map(decode_row).collect()
}

fn decode_row(row: PgRow) -> Result<Arc<KindSchema>, StoreError> {
    let kind: String = row.try_get("kind")?;
    let table: String = row.try_get("table_name")?;
    let bytes: Vec<u8> = row.try_get("definition")?;
    decode_schema(&kind, &table, &bytes).map(Arc::new)
}

pub(super) async fn metadata(pool: &PgPool, key: &str) -> Result<Vec<u8>, StoreError> {
    Ok(
        sqlx::query(
            "SELECT value FROM inventory.metadata WHERE key=$1 AND octet_length(value)<=$2",
        )
        .bind(key)
        .bind(
            i64::try_from(super::super::MAX_CATALOG_BYTES)
                .map_err(|_| StoreError::Invalid("catalog size bound overflow"))?,
        )
        .fetch_one(pool)
        .await?
        .try_get("value")?,
    )
}
