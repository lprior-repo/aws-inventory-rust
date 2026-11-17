use super::{commit, BinaryStore, Kind, ReferenceTarget, ResourceId, ResourceKey, ScalarType, StoreError, TargetMatch};
use inventory_core::{catalog::runtime::RelationTargetMode, record::{MapRef, ValueRef}};
use sqlx::{QueryBuilder, Row, Sqlite, Transaction};
use futures_util::TryStreamExt;

pub(super) async fn match_target(store: &BinaryStore, tx: &mut Transaction<'_, Sqlite>, target: &ReferenceTarget, only: Option<&Kind>, expected: Option<&ResourceId>) -> Result<TargetMatch, StoreError> {
    let mut matched = None;
    let bound = terms(target).any(|(mode, _, _)| is_bound(mode));
    for kind in target.candidate_kinds().iter().filter(|kind| only.is_none_or(|only| only == *kind)) {
        let schema = store.registry.schemas().iter().find(|schema| schema.kind() == kind)
            .ok_or(StoreError::Invalid("referenced kind is absent from schema registry"))?;
        let mut query = QueryBuilder::<Sqlite>::new(if bound { "SELECT p.id, f.bound_request FROM " } else { "SELECT p.id FROM " });
        query.push(commit::quote(schema.table().as_str())).push(" p");
        if bound {
            query.push(" JOIN resource_facets f ON f.account=p.account AND f.region=p.region AND f.kind=").push_bind(kind.as_str())
                .push(" AND f.id=p.id AND f.source_authority IS NOT NULL AND f.bound_request IS NOT NULL");
        }
        query.push(" WHERE p.account=").push_bind(target.account().as_str()).push(" AND p.region=").push_bind(target.region().as_str());
        let mut available = true;
        for (mode, path, value) in terms(target) {
            match mode {
                RelationTargetMode::CanonicalId => { query.push(" AND p.id=").push_bind(value.as_str()); }
                RelationTargetMode::SelectedFields | RelationTargetMode::SelectedRootText => {
                    let Some(column) = schema.columns().iter().find(|column| column.path() == path && column.scalar() == ScalarType::Text) else { available = false; break; };
                    query.push(" AND p.").push(commit::quote(column.name())).push("=").push_bind(value.as_str());
                }
                RelationTargetMode::BoundContextFields | RelationTargetMode::BoundContextRootText => {}
            }
        }
        if !available { continue; }
        if let Some(expected) = expected { query.push(" AND p.id=").push_bind(expected.as_str()); }
        if !bound { query.push(" LIMIT 2"); }
        let statement = query.build();
        let mut rows = statement.fetch(&mut **tx);
        let mut ids = Vec::<ResourceId>::with_capacity(2);
        while let Some(row) = rows.try_next().await? {
            if !bound || matches_bound(target, &row.try_get::<Vec<u8>, _>("bound_request")?)? {
                let id = ResourceId::parse(&row.try_get::<String, _>("id")?)?;
                if !ids.contains(&id) { ids.push(id); }
                if ids.len() == 2 { break; }
            }
        }
        for id in ids {
            if matched.is_some() { return Ok(TargetMatch::Ambiguous); }
            matched = Some(ResourceKey { account: target.account().clone(), region: target.region().clone(), kind: kind.clone(), id });
        }
    }
    Ok(matched.map_or(TargetMatch::Absent, TargetMatch::Unique))
}

fn terms(target: &ReferenceTarget) -> impl Iterator<Item = (RelationTargetMode, &[inventory_core::record::FieldId], &ResourceId)> {
    std::iter::once((target.mode(), target.path(), target.value()))
        .chain(target.components().iter().map(|term| (term.mode, term.path.as_ref(), &term.value)))
}
fn is_bound(mode: RelationTargetMode) -> bool { matches!(mode, RelationTargetMode::BoundContextFields | RelationTargetMode::BoundContextRootText) }
fn matches_bound(target: &ReferenceTarget, bytes: &[u8]) -> Result<bool, StoreError> {
    let root = MapRef::from_bytes_bounded(bytes)?;
    terms(target).filter(|(mode, _, _)| is_bound(*mode)).try_fold(true, |matches, (_, path, value)| {
        Ok(matches && matches!(crate::inventory::relation_permissions::scalar(root, path)?, Some(ValueRef::Text(text)) if text == value.as_str()))
    })
}
