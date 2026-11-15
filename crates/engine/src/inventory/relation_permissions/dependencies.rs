use super::{matches_reference, owner_matches, scalar, source_matches, unreviewed, RelationSource, StoreError, ValueRef};
use crate::inventory::{SchemaRegistry, WorkLease};
use inventory_core::{inventory::{Edge, Observation, UnresolvedReference}, record::FieldId};

type Dependencies = Option<Vec<Vec<FieldId>>>;

pub(in crate::inventory) fn reference_dependencies(registry: &SchemaRegistry, lease: &WorkLease, source: &Observation, reference: &UnresolvedReference) -> Result<Dependencies, StoreError> {
    if source.key() != &reference.from { return Err(unreviewed()); }
    let authority = registry.permissions.authority(lease)?;
    let mut paths = Vec::new();
    for rule in &authority.relations {
        if !matches_reference(rule, reference) || !source_matches(rule, lease.bindings(), source, reference.target.value().as_str())? { continue; }
        if rule.source == RelationSource::Request { return Ok(None); }
        paths.push(rule.source_path.to_vec());
    }
    for rule in &authority.trust {
        if rule.reference(reference) && rule.declares(source, reference.target.value().as_str())? { paths.push(rule.dependency_path().to_vec()); }
    }
    checked_paths(paths)
}

fn checked_paths(mut paths: Vec<Vec<FieldId>>) -> Result<Dependencies, StoreError> {
    if paths.is_empty() { return Err(unreviewed()); }
    paths.sort_unstable();
    paths.dedup();
    Ok(Some(paths))
}

pub(in crate::inventory) fn edge_references(registry: &SchemaRegistry, lease: &WorkLease, source: &Observation, target: &Observation, edge: &Edge) -> Result<Vec<UnresolvedReference>, StoreError> {
    if source.key() != &edge.from || target.key() != &edge.to { return Err(unreviewed()); }
    let authority = registry.permissions.authority(lease)?;
    let mut references = Vec::new();
    for rule in &authority.relations {
        if rule.source_kind != edge.from.kind || rule.relation != edge.relation
            || rule.target_kind != edge.to.kind || rule.target_kinds.len() != 1 { continue; }
        let Some(ValueRef::Text(value)) = scalar(target.record().root(), &rule.target_path)? else { continue; };
        if owner_matches(rule, &edge.from, &edge.to.account, &edge.to.region, value)
            && source_matches(rule, lease.bindings(), source, value)? {
            push_reference(registry, &mut references, edge, &rule.target_path, value)?;
        }
    }
    for rule in &authority.trust {
        if rule.edge(edge) && rule.declares(source, edge.to.id.as_str())?
            && rule.target_matches(target, edge.to.id.as_str())? {
            push_reference(registry, &mut references, edge, rule.target_path(), edge.to.id.as_str())?;
        }
    }
    if references.is_empty() { return Err(unreviewed()); }
    Ok(references)
}

fn push_reference(registry: &SchemaRegistry, references: &mut Vec<UnresolvedReference>, edge: &Edge, path: &[FieldId], value: &str) -> Result<(), StoreError> {
    if references.iter().any(|reference| reference.target.path() == path && reference.target.value().as_str() == value) { return Ok(()); }
    let target = registry.reference_target(edge.to.account.clone(), edge.to.region.clone(),
        edge.to.kind.clone(), path.to_vec(), inventory_core::inventory::ResourceId::parse(value)?)?;
    references.push(UnresolvedReference { from: edge.from.clone(), relation: edge.relation.clone(), source: edge.source, target });
    Ok(())
}
