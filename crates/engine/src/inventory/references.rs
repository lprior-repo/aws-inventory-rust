use super::commit;
use super::{BinaryStore, StoreError};
use inventory_core::inventory::{Observation, ResourceKey, ScalarType};
use inventory_core::record::FieldId;
use sqlx::{Row, Sqlite, Transaction};

pub(super) async fn reconcile_inbound(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    target: &ResourceKey,
    observation: &Observation,
) -> Result<(), StoreError> {
    let schema = store
        .registry
        .schemas()
        .iter()
        .find(|schema| schema.kind() == &target.kind)
        .ok_or(StoreError::Invalid(
            "target kind is absent from schema registry",
        ))?;
    for column in schema
        .columns()
        .iter()
        .filter(|column| column.scalar() == ScalarType::Text)
    {
        reconcile_column(tx, target, observation, column.path()).await?;
    }
    Ok(())
}

async fn reconcile_column(
    tx: &mut Transaction<'_, Sqlite>,
    target: &ResourceKey,
    observation: &Observation,
    path: &[FieldId],
) -> Result<(), StoreError> {
    let scope = commit::reference_scope(target.account.as_str(), target.region.as_str())?;
    let encoded_path = commit::reference_path(path);
    let rows = stale_rows(
        tx,
        &scope,
        target,
        &encoded_path,
        commit::text_field(observation, path)?,
    )
    .await?;
    for row in rows {
        retract_edge(tx, target, &row).await?;
    }
    Ok(())
}

async fn stale_rows(
    tx: &mut Transaction<'_, Sqlite>,
    scope: &[u8],
    target: &ResourceKey,
    path: &[u8],
    value: Option<&str>,
) -> Result<Vec<sqlx::sqlite::SqliteRow>, StoreError> {
    match value {
        Some(value) => Ok(sqlx::query("SELECT from_account,from_region,from_kind,from_id,relation,source FROM unresolved_refs WHERE target_scope=?1 AND target_kind=?2 AND target_path=?3 AND target_value<>?4")
            .bind(scope).bind(target.kind.as_str()).bind(path).bind(value).fetch_all(&mut **tx).await?),
        None => Ok(sqlx::query("SELECT from_account,from_region,from_kind,from_id,relation,source FROM unresolved_refs WHERE target_scope=?1 AND target_kind=?2 AND target_path=?3")
            .bind(scope).bind(target.kind.as_str()).bind(path).fetch_all(&mut **tx).await?),
    }
}

async fn retract_edge(
    tx: &mut Transaction<'_, Sqlite>,
    target: &ResourceKey,
    row: &sqlx::sqlite::SqliteRow,
) -> Result<(), StoreError> {
    let from = commit::source_key(row)?;
    let relation: String = row.try_get("relation")?;
    let source: String = row.try_get("source")?;
    let result = sqlx::query("DELETE FROM edges WHERE from_account=?1 AND from_region=?2 AND from_kind=?3 AND from_id=?4 AND to_account=?5 AND to_region=?6 AND to_kind=?7 AND to_id=?8 AND relation=?9 AND source=?10")
        .bind(from.account.as_str()).bind(from.region.as_str()).bind(from.kind.as_str()).bind(from.id.as_str())
        .bind(target.account.as_str()).bind(target.region.as_str()).bind(target.kind.as_str()).bind(target.id.as_str()).bind(relation).bind(source).execute(&mut **tx).await?;
    if result.rows_affected() > 0 {
        commit::enqueue(tx, &from, false).await?;
        commit::enqueue(tx, target, false).await?;
    }
    Ok(())
}
