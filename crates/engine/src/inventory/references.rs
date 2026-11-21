use super::{BinaryStore, StoreError, commit, reference_candidates};
use inventory_core::inventory::{
    Kind, Observation, Relation, ResourceId, ResourceKey, UnresolvedReference,
};
use sqlx::{Row, Sqlite, Transaction};

pub(super) async fn refresh(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    target: &ResourceKey,
    observation: &Observation,
) -> Result<u64, StoreError> {
    refresh_columns(store, tx, target, Some(observation)).await
}

pub(super) async fn refresh_missing(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    target: &ResourceKey,
) -> Result<u64, StoreError> {
    refresh_columns(store, tx, target, None).await
}

async fn refresh_columns(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    target: &ResourceKey,
    _observation: Option<&Observation>,
) -> Result<u64, StoreError> {
    let mut inserted = 0_u64;
    let scope = commit::reference_scope(target.account.as_str(), target.region.as_str())?;
    // ES-10: Single indexed lookup via the derived candidate index.
    // One query drives all resolution lookups; no blob scan needed.
    let rows = sqlx::query(
        "SELECT r.from_account,r.from_region,r.from_kind,r.from_id,r.target_kind,r.target_kinds,r.target_path,r.target_value,r.relation,r.source FROM unresolved_ref_candidates c JOIN unresolved_refs r ON (r.from_account=c.from_account AND r.from_region=c.from_region AND r.from_kind=c.from_kind AND r.from_id=c.from_id AND r.target_scope=c.target_scope AND r.target_kind=c.target_kind AND r.target_path=c.target_path AND r.target_value=c.target_value AND r.relation=c.relation AND r.source=c.source) WHERE c.target_scope=?1 AND c.candidate_kind=?2",
    )
    .bind(&scope)
    .bind(target.kind.as_str())
    .fetch_all(&mut **tx)
    .await?;
    for row in rows {
        let kind = Kind::parse(&row.try_get::<String, _>("target_kind")?)?;
        let encoded: &[u8] = row.try_get("target_kinds")?;
        let kinds = if encoded.is_empty() {
            if kind != target.kind {
                continue;
            }
            None
        } else {
            let kinds = reference_candidates::decode(encoded, &kind)?;
            if !kinds.contains(&target.kind) {
                continue;
            }
            Some(kinds)
        };
        let reference_target = reference_candidates::decode_target(
            target.account.clone(),
            target.region.clone(),
            kind,
            row.try_get("target_path")?,
            ResourceId::parse(&row.try_get::<String, _>("target_value")?)?,
        )?;
        let reference_target = if let Some(kinds) = kinds {
            reference_target.with_candidate_kinds(kinds)?
        } else {
            reference_target
        };
        let reference = UnresolvedReference {
            from: commit::source_key(&row)?,
            relation: Relation::parse(&row.try_get::<String, _>("relation")?)?,
            source: commit::edge_source(&row.try_get::<String, _>("source")?)?,
            target: reference_target,
        };
        if !super::relation_permissions::promotable(&store.registry, tx, &reference).await? {
            continue;
        }
        retract(store, tx, target, &reference).await?;
        inserted = inserted
            .checked_add(reference_candidates::resolve(store, tx, &reference).await?)
            .ok_or(StoreError::Invalid("reference edge count overflow"))?;
    }
    Ok(inserted)
}

pub(super) async fn retract(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    target: &ResourceKey,
    reference: &UnresolvedReference,
) -> Result<(), StoreError> {
    if reference_candidates::supports(store, tx, reference, target).await? {
        return Ok(());
    }
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
