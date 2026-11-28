mod candidates;
#[cfg(test)]
mod claim_plans;
mod kinds;
mod provenance;
mod shared;

use crate::inventory::{SchemaRegistry, StoreError, schema::decode_schema};
use futures_util::{TryStreamExt, stream};
use inventory_core::inventory::KindSchema;
use sqlx::{PgPool, Postgres, Row, Transaction, postgres::PgRow};
use sha2::{Digest, Sha256};
use std::sync::Arc;

/// Publication state of the `inventory` schema.
///
/// `initialize` creates the shared tables first, then the per-kind tables in
/// bounded batches, and writes `schema_version` last in its own transaction. The
/// `schema_version` row is therefore the atomic "ready" record: a namespace
/// without it is an interrupted initialization that `initialize` resumes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchemaState {
    Absent,
    Incomplete,
    Ready,
}

#[derive(Debug, Eq, PartialEq)]
enum Plan {
    Fresh,
    Resume,
}

fn plan(
    state: SchemaState,
    stored_digest: Option<&[u8]>,
    expected: &[u8; 32],
) -> Result<Plan, StoreError> {
    match (state, stored_digest) {
        (SchemaState::Absent, _) => Ok(Plan::Fresh),
        (SchemaState::Ready, _) => Err(StoreError::Invalid("inventory schema already exists")),
        (SchemaState::Incomplete, Some(stored)) if stored == expected.as_slice() => {
            Ok(Plan::Resume)
        }
        (SchemaState::Incomplete, Some(_)) => Err(StoreError::CatalogMismatch),
        (SchemaState::Incomplete, None) => Err(StoreError::Invalid(
            "incomplete inventory schema has no catalog digest",
        )),
    }
}

pub async fn state(pool: &PgPool) -> Result<SchemaState, StoreError> {
    let namespace: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_namespace WHERE nspname='inventory')")
            .fetch_one(pool)
            .await?;
    if !namespace {
        return Ok(SchemaState::Absent);
    }
    let metadata: bool = sqlx::query_scalar("SELECT to_regclass('inventory.metadata') IS NOT NULL")
        .fetch_one(pool)
        .await?;
    if !metadata {
        return Err(StoreError::Invalid(
            "inventory schema exists without metadata",
        ));
    }
    let ready: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM inventory.metadata WHERE key='schema_version')",
    )
    .fetch_one(pool)
    .await?;
    Ok(if ready {
        SchemaState::Ready
    } else {
        SchemaState::Incomplete
    })
}

pub async fn initialize(pool: &PgPool, registry: &SchemaRegistry) -> Result<(), StoreError> {
    let state = state(pool).await?;
    let stored = match state {
        SchemaState::Incomplete => Some(metadata(pool, "catalog_digest").await?),
        SchemaState::Absent | SchemaState::Ready => None,
    };
    if plan(state, stored.as_deref(), registry.digest())? == Plan::Fresh {
        initialize_shared(pool, registry).await?;
    }
    populate(pool, registry).await
}

pub async fn create_fresh(pool: &PgPool, registry: &SchemaRegistry) -> Result<(), StoreError> {
    if state(pool).await? != SchemaState::Absent {
        return Err(StoreError::Invalid("inventory schema already exists"));
    }
    initialize_shared(pool, registry).await?;
    populate(pool, registry).await
}

async fn populate(pool: &PgPool, registry: &SchemaRegistry) -> Result<(), StoreError> {
    stream::iter(registry.schemas().chunks(200).map(Ok))
        .try_for_each(|batch| kinds::create_batch(pool, batch))
        .await?;
    publish(pool).await
}

async fn publish(pool: &PgPool) -> Result<(), StoreError> {
    let mut tx = pool.begin().await?;
    kinds::lock(&mut tx).await?;
    sqlx::query("INSERT INTO inventory.metadata(key,value) VALUES('schema_version',$1) ON CONFLICT(key) DO NOTHING")
        .bind(b"15".as_slice())
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
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

pub(super) async fn open_writer_registry(
    pool: &PgPool,
    expected_catalog: Option<&[u8]>,
) -> Result<SchemaRegistry, StoreError> {
    match state(pool).await? {
        SchemaState::Ready => {}
        SchemaState::Incomplete => return Err(StoreError::InitializationIncomplete),
        SchemaState::Absent => return Err(StoreError::Invalid("inventory schema is missing")),
    }
    if expected_catalog.is_some_and(|catalog| catalog.len() > super::super::MAX_CATALOG_BYTES) {
        return Err(StoreError::Invalid("catalog size exceeds bounds"));
    }
    let mut tx = pool.begin().await?;
    sqlx::query("LOCK TABLE inventory.metadata, inventory.kind_schemas IN SHARE ROW EXCLUSIVE MODE")
        .execute(&mut *tx)
        .await?;
    let version = bounded_metadata(&mut *tx, "schema_version").await?;
    let found = std::str::from_utf8(&version)
        .ok()
        .and_then(|text| text.parse::<i64>().ok())
        .ok_or(StoreError::Invalid("invalid PostgreSQL schema version"))?;
    if !matches!(found, 14 | 15) {
        return Err(StoreError::SchemaVersion { found });
    }
    let registry = load_registry_tx(&mut tx).await?;
    if expected_catalog.is_some_and(|catalog| {
        Sha256::digest(catalog).as_slice() != registry.digest()
    }) {
        return Err(StoreError::CatalogMismatch);
    }
    if found == 14 {
        sqlx::query("UPDATE inventory.metadata SET value='15' WHERE key='schema_version'")
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(registry)
}

async fn load_registry_tx(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<SchemaRegistry, StoreError> {
    let catalog = bounded_metadata(&mut **tx, "reviewed_catalog").await?;
    let rows = sqlx::query(
        "SELECT kind,table_name,definition FROM inventory.kind_schemas ORDER BY kind LIMIT 65537",
    )
    .fetch_all(&mut **tx)
    .await?;
    if rows.len() > 65_536 {
        return Err(StoreError::Invalid("stored schema count exceeds bounds"));
    }
    let schemas = rows.into_iter().map(decode_row).collect::<Result<Vec<_>, _>>()?;
    let registry = SchemaRegistry::new(&catalog, schemas)?;
    let stored_digest = bounded_metadata(&mut **tx, "catalog_digest").await?;
    if registry.digest().as_slice() != stored_digest.as_slice() {
        return Err(StoreError::CatalogMismatch);
    }
    Ok(registry)
}

pub(super) async fn metadata(pool: &PgPool, key: &str) -> Result<Vec<u8>, StoreError> {
    bounded_metadata(pool, key).await
}

async fn bounded_metadata<'e>(
    executor: impl sqlx::Executor<'e, Database = Postgres>,
    key: &str,
) -> Result<Vec<u8>, StoreError> {
    Ok(
        sqlx::query(
            "SELECT value FROM inventory.metadata WHERE key=$1 AND octet_length(value)<=$2",
        )
        .bind(key)
        .bind(
            i64::try_from(super::super::MAX_CATALOG_BYTES)
                .map_err(|_| StoreError::Invalid("catalog size bound overflow"))?,
        )
        .fetch_one(executor)
        .await?
        .try_get("value")?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST: [u8; 32] = [7; 32];

    #[test]
    fn given_absent_schema_when_planned_then_fresh() {
        assert_eq!(
            plan(SchemaState::Absent, None, &DIGEST).ok(),
            Some(Plan::Fresh)
        );
    }

    #[test]
    fn given_incomplete_schema_with_same_digest_when_planned_then_resume() {
        assert_eq!(
            plan(SchemaState::Incomplete, Some(&DIGEST), &DIGEST).ok(),
            Some(Plan::Resume)
        );
    }

    #[test]
    fn given_incomplete_schema_with_other_digest_when_planned_then_catalog_mismatch() {
        assert!(matches!(
            plan(SchemaState::Incomplete, Some(&[9; 32]), &DIGEST),
            Err(StoreError::CatalogMismatch)
        ));
    }

    #[test]
    fn given_incomplete_schema_without_digest_when_planned_then_refused() {
        assert!(matches!(
            plan(SchemaState::Incomplete, None, &DIGEST),
            Err(StoreError::Invalid(_))
        ));
    }

    #[test]
    fn given_ready_schema_when_planned_then_create_is_refused() {
        assert!(matches!(
            plan(SchemaState::Ready, Some(&DIGEST), &DIGEST),
            Err(StoreError::Invalid("inventory schema already exists"))
        ));
    }
}
