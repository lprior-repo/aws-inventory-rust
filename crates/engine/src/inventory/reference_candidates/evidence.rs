use super::{BinaryStore, Kind, ResourceId, ResourceKey, StoreError, UnresolvedReference, commit};
use sqlx::{Row, Sqlite, Transaction};

pub(in crate::inventory) async fn supports(
    store: &BinaryStore, tx: &mut Transaction<'_, Sqlite>, reference: &UnresolvedReference, target: &ResourceKey,
) -> Result<bool, StoreError> {
    let rows = sqlx::query("SELECT target_kind,target_kinds,target_path,target_value FROM unresolved_refs WHERE from_account=?1 AND from_region=?2 AND from_kind=?3 AND from_id=?4 AND relation=?5 AND source=?6 AND target_scope=?7 AND (target_kind=?8 OR target_kinds<>X'')")
        .bind(reference.from.account.as_str()).bind(reference.from.region.as_str()).bind(reference.from.kind.as_str()).bind(reference.from.id.as_str())
        .bind(reference.relation.as_str()).bind(reference.source.as_str())
        .bind(commit::reference_scope(target.account.as_str(), target.region.as_str())?).bind(target.kind.as_str()).fetch_all(&mut **tx).await?;
    for row in rows {
        let kind = Kind::parse(&row.try_get::<String, _>("target_kind")?)?;
        let kinds = super::decode(row.try_get("target_kinds")?, &kind)?;
        if !kinds.contains(&target.kind) { continue; }
        let value = ResourceId::parse(&row.try_get::<String, _>("target_value")?)?;
        let reference_target = super::decode_target(target.account.clone(), target.region.clone(), kind, row.try_get("target_path")?, value)?.with_candidate_kinds(kinds)?;
        let candidate = UnresolvedReference { from: reference.from.clone(), relation: reference.relation.clone(), source: reference.source, target: reference_target };
        if let Some(resolution) = super::current_targets(store, tx, &candidate).await?
            && resolution.targets.contains(target) { return Ok(true); }
    }
    Ok(false)
}

