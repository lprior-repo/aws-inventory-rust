mod evidence;

pub(super) use evidence::supports;

use super::{BinaryStore, StoreError, commit};
use inventory_core::inventory::{Edge, Kind, ReferenceTarget, ResourceId, ResourceKey, ScalarType, UnresolvedReference};
use inventory_core::record::{FieldId, Key, Record, Value, ValueRef};
use sqlx::{QueryBuilder, Row, Sqlite, Transaction};
use std::collections::BTreeMap;

pub(super) fn encode(target: &ReferenceTarget) -> Result<Vec<u8>, StoreError> {
    if target.candidate_kinds().len() == 1 { return Ok(Vec::new()); }
    Ok(Record::encode(&Value::Map(BTreeMap::from([(Key::Field(FieldId::new(1)?),
        Value::List(target.candidate_kinds().iter().map(|kind| Value::Text(kind.as_str().to_owned())).collect()))])))?.into_bytes())
}

pub(super) fn decode(bytes: &[u8], primary: &Kind) -> Result<Vec<Kind>, StoreError> {
    if bytes.is_empty() { return Ok(vec![primary.clone()]); }
    let record = Record::from_bytes(bytes.to_vec())?;
    match record.root().get_field(FieldId::new(1)?)? {
        Some(ValueRef::List(list)) if list.count()? <= 64 => list.iter()?.map(|value| match value? {
            ValueRef::Text(text) => Ok(Kind::parse(text)?),
            _ => Err(StoreError::Invalid("reference candidate kind is not text")),
        }).collect(),
        _ => Err(StoreError::Invalid("reference candidate kinds are absent or excessive")),
    }
}

pub(super) async fn resolve(
    store: &BinaryStore, tx: &mut Transaction<'_, Sqlite>, reference: &UnresolvedReference,
) -> Result<u64, StoreError> {
    let Some(resolution) = current_targets(store, tx, reference).await? else { return Ok(0); };
    let status = if !resolution.targets.is_empty() { "resolved" } else if resolution.ambiguous { "ambiguous" } else { "unresolved" };
    update_status(tx, reference, status).await?;
    if resolution.ambiguous { retract_matches(store, tx, reference).await?; }
    let mut inserted = 0_u64;
    for to in resolution.targets {
        if commit::persist_edge(tx, &Edge { from: reference.from.clone(), to: to.clone(), relation: reference.relation.clone(), source: reference.source }).await? {
            if inserted == 0 { commit::enqueue(tx, &reference.from, false).await?; }
            commit::enqueue(tx, &to, false).await?;
            inserted = inserted.checked_add(1).ok_or(StoreError::Invalid("reference edge count overflow"))?;
        }
    }
    Ok(inserted)
}

struct Resolution {
    targets: Vec<ResourceKey>,
    ambiguous: bool,
}

async fn current_targets(
    store: &BinaryStore, tx: &mut Transaction<'_, Sqlite>, reference: &UnresolvedReference,
) -> Result<Option<Resolution>, StoreError> {
    if !super::relation_permissions::promotable(&store.registry, tx, reference).await? { return Ok(None); }
    store.registry.permissions.validate_reference(reference)?;
    let grants = super::relation_permissions::targets(&store.registry, tx, reference).await?;
    let mut resolution = Resolution { targets: Vec::new(), ambiguous: false };
    for expected in grants {
        match match_target(store, tx, &reference.target, expected.as_ref()).await? {
            TargetMatch::Unique(target) if !resolution.targets.contains(&target) => resolution.targets.push(target),
            TargetMatch::Ambiguous => resolution.ambiguous = true,
            TargetMatch::Unique(_) | TargetMatch::Absent => {}
        }
    }
    Ok(Some(resolution))
}

enum TargetMatch {
    Absent,
    Unique(ResourceKey),
    Ambiguous,
}

async fn match_target(
    store: &BinaryStore, tx: &mut Transaction<'_, Sqlite>, target: &ReferenceTarget, expected: Option<&ResourceId>,
) -> Result<TargetMatch, StoreError> {
    let mut matched = None;
    for kind in target.candidate_kinds() {
        let schema = store.registry.schemas().iter().find(|schema| schema.kind() == kind)
            .ok_or(StoreError::Invalid("referenced kind is absent from schema registry"))?;
        let Some(column) = schema.columns().iter().find(|column| column.path() == target.path() && column.scalar() == ScalarType::Text) else { continue; };
        let mut query = QueryBuilder::<Sqlite>::new("SELECT id FROM ");
        query.push(commit::quote(schema.table().as_str())).push(" WHERE account=").push_bind(target.account().as_str())
            .push(" AND region=").push_bind(target.region().as_str()).push(" AND ").push(commit::quote(column.name()))
            .push("=").push_bind(target.value().as_str());
        if let Some(expected) = expected { query.push(" AND id=").push_bind(expected.as_str()); }
        query.push(" LIMIT 2");
        for row in query.build().fetch_all(&mut **tx).await? {
            if matched.is_some() { return Ok(TargetMatch::Ambiguous); }
            matched = Some(ResourceKey { account: target.account().clone(), region: target.region().clone(), kind: kind.clone(), id: ResourceId::parse(&row.try_get::<String, _>("id")?)? });
        }
    }
    Ok(matched.map_or(TargetMatch::Absent, TargetMatch::Unique))
}

async fn update_status(tx: &mut Transaction<'_, Sqlite>, reference: &UnresolvedReference, status: &str) -> Result<(), StoreError> {
    let target = &reference.target;
    sqlx::query("UPDATE unresolved_refs SET resolution=?1 WHERE from_account=?2 AND from_region=?3 AND from_kind=?4 AND from_id=?5 AND target_scope=?6 AND target_kind=?7 AND target_path=?8 AND target_value=?9 AND relation=?10 AND source=?11")
        .bind(status).bind(reference.from.account.as_str()).bind(reference.from.region.as_str()).bind(reference.from.kind.as_str()).bind(reference.from.id.as_str())
        .bind(commit::reference_scope(target.account().as_str(), target.region().as_str())?).bind(target.kind().as_str()).bind(commit::reference_path(target.path()))
        .bind(target.value().as_str()).bind(reference.relation.as_str()).bind(reference.source.as_str()).execute(&mut **tx).await?;
    Ok(())
}

async fn retract_matches(store: &BinaryStore, tx: &mut Transaction<'_, Sqlite>, reference: &UnresolvedReference) -> Result<(), StoreError> {
    let target = &reference.target;
    for kind in target.candidate_kinds() {
        let schema = store.registry.schemas().iter().find(|schema| schema.kind() == kind)
            .ok_or(StoreError::Invalid("referenced kind is absent from schema registry"))?;
        let Some(column) = schema.columns().iter().find(|column| column.path() == target.path() && column.scalar() == ScalarType::Text) else { continue; };
        let mut query = QueryBuilder::<Sqlite>::new("SELECT to_id FROM edges WHERE from_account=");
        query.push_bind(reference.from.account.as_str()).push(" AND from_region=").push_bind(reference.from.region.as_str())
            .push(" AND from_kind=").push_bind(reference.from.kind.as_str()).push(" AND from_id=").push_bind(reference.from.id.as_str())
            .push(" AND relation=").push_bind(reference.relation.as_str()).push(" AND source=").push_bind(reference.source.as_str())
            .push(" AND to_account=").push_bind(target.account().as_str()).push(" AND to_region=").push_bind(target.region().as_str())
            .push(" AND to_kind=").push_bind(kind.as_str()).push(" AND to_id IN(SELECT id FROM ").push(commit::quote(schema.table().as_str()))
            .push(" WHERE account=").push_bind(target.account().as_str()).push(" AND region=").push_bind(target.region().as_str())
            .push(" AND ").push(commit::quote(column.name())).push("=").push_bind(target.value().as_str()).push(")");
        for row in query.build().fetch_all(&mut **tx).await? {
            let key = ResourceKey { account: target.account().clone(), region: target.region().clone(), kind: kind.clone(), id: ResourceId::parse(&row.try_get::<String, _>("to_id")?)? };
            super::references::retract(store, tx, &key, reference).await?;
        }
    }
    Ok(())
}
