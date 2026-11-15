use super::{BinaryStore, StoreError, commit, reference_candidates};
use inventory_core::inventory::{Kind, Observation, Relation, ResourceId, ResourceKey, ScalarType, UnresolvedReference};
use sqlx::{Row, Sqlite, Transaction};

pub(super) async fn refresh(
    store: &BinaryStore, tx: &mut Transaction<'_, Sqlite>, target: &ResourceKey,
    observation: &Observation,
) -> Result<u64, StoreError> {
    refresh_columns(store, tx, target, Some(observation)).await
}

pub(super) async fn refresh_missing(
    store: &BinaryStore, tx: &mut Transaction<'_, Sqlite>, target: &ResourceKey,
) -> Result<u64, StoreError> {
    refresh_columns(store, tx, target, None).await
}

async fn refresh_columns(
    store: &BinaryStore, tx: &mut Transaction<'_, Sqlite>, target: &ResourceKey,
    observation: Option<&Observation>,
) -> Result<u64, StoreError> {
    let schema = store.registry.schemas().iter().find(|schema| schema.kind() == &target.kind)
        .ok_or(StoreError::Invalid("target kind is absent from schema registry"))?;
    let mut inserted = 0_u64;
    for column in schema.columns().iter().filter(|column| column.scalar() == ScalarType::Text) {
        let rows = sqlx::query("SELECT from_account,from_region,from_kind,from_id,target_kind,target_kinds,target_value,relation,source FROM unresolved_refs WHERE target_scope=?1 AND target_path=?2 AND (target_kind=?3 OR target_kinds<>X'')")
            .bind(commit::reference_scope(target.account.as_str(), target.region.as_str())?)
            .bind(commit::reference_path(column.path())).bind(target.kind.as_str()).fetch_all(&mut **tx).await?;
        for row in rows {
            let kind = Kind::parse(&row.try_get::<String, _>("target_kind")?)?;
            let encoded: &[u8] = row.try_get("target_kinds")?;
            let kinds = if encoded.is_empty() {
                if kind != target.kind { continue; }
                None
            } else {
                let kinds = reference_candidates::decode(encoded, &kind)?;
                if !kinds.contains(&target.kind) { continue; }
                Some(kinds)
            };
            let reference_target = store.registry.reference_target(target.account.clone(), target.region.clone(), kind,
                column.path().to_vec(), ResourceId::parse(&row.try_get::<String, _>("target_value")?)?)?;
            let reference_target = if let Some(kinds) = kinds { reference_target.with_candidate_kinds(kinds)? } else { reference_target };
            let reference = UnresolvedReference {
                from: commit::source_key(&row)?,
                relation: Relation::parse(&row.try_get::<String, _>("relation")?)?,
                source: commit::edge_source(&row.try_get::<String, _>("source")?)?,
                target: reference_target,
            };
            if !super::relation_permissions::promotable(&store.registry, tx, &reference).await? { continue; }
            let value = observation.map(|observation| commit::text_field(observation, column.path())).transpose()?.flatten();
            if value != Some(reference.target.value().as_str()) {
                retract(store, tx, target, &reference).await?;
            }
            inserted = inserted.checked_add(reference_candidates::resolve(store, tx, &reference).await?)
                .ok_or(StoreError::Invalid("reference edge count overflow"))?;
        }
    }
    Ok(inserted)
}

pub(super) async fn retract(
    store: &BinaryStore, tx: &mut Transaction<'_, Sqlite>, target: &ResourceKey, reference: &UnresolvedReference,
) -> Result<(), StoreError> {
    if reference_candidates::supports(store, tx, reference, target).await? { return Ok(()); }
    let from = &reference.from;
    let result = sqlx::query("DELETE FROM edges WHERE from_account=?1 AND from_region=?2 AND from_kind=?3 AND from_id=?4 AND to_account=?5 AND to_region=?6 AND to_kind=?7 AND to_id=?8 AND relation=?9 AND source=?10")
        .bind(from.account.as_str()).bind(from.region.as_str()).bind(from.kind.as_str()).bind(from.id.as_str())
        .bind(target.account.as_str()).bind(target.region.as_str()).bind(target.kind.as_str()).bind(target.id.as_str())
        .bind(reference.relation.as_str()).bind(reference.source.as_str()).execute(&mut **tx).await?;
    if result.rows_affected() > 0 {
        commit::enqueue(tx, from, false).await?;
        commit::enqueue(tx, target, false).await?;
    }
    Ok(())
}
