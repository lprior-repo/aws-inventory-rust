mod dependencies;
mod provenance;
mod persistence;
mod trust;
mod stages;
mod bindings;
mod source_authority;

mod typed;
use super::{PageCommit, StoreError};
use inventory_core::{
    catalog::runtime::{ReferenceOwnerRule, RelationScope, RelationSource, RelationshipPlan, ResourceEmissionPlan, RuntimeCatalog},
    inventory::{Edge, Kind, Observation, Region, ResourceKey, UnresolvedReference},
    record::{FieldId, MapRef, ValueRef},
};
use std::{collections::BTreeMap, sync::Arc};
pub(super) use persistence::{load_registry, open_with_catalog, persist};
pub(super) use dependencies::{edge_references, reference_dependencies};
pub(super) use provenance::{activate_reference, clear_active, mark_direct_reference, mark_reference, promotable, targets, ReferenceProvenance};
pub(super) use provenance::{initialize as initialize_provenance, invalidate_prior_authority};
pub(super) use source_authority::{SourceAuthority, page_authority};
use typed::{source_matches, rule_reference, reference_matches_observation};

type Operations = BTreeMap<String, BTreeMap<String, stages::Stages>>;

#[derive(Clone, Debug)]
struct Authority {
    relations: Vec<RelationshipPlan>,
    trust: Vec<trust::TrustRule>,
    emissions: Vec<ResourceEmissionPlan>,
}

#[derive(Clone, Debug)]
pub(super) enum Permissions {
    Unavailable,
    Reviewed(Reviewed),
}

#[derive(Clone, Debug)]
pub(super) struct Reviewed {
    catalog: Arc<[u8]>,
    operations: Operations,
    relations: BTreeMap<Kind, Vec<RelationshipPlan>>,
    trust: Vec<trust::TrustRule>,
}

impl Permissions {
    pub(super) fn new(bytes: &[u8], catalog: &RuntimeCatalog<'_>) -> Result<Self, StoreError> {
        let mut operations: Operations = BTreeMap::new();
        for plan in catalog.plans() {
            operations.entry(plan.service_name().to_owned()).or_default()
                .insert(plan.operation_name().to_owned(), stages::Stages::compile(catalog, plan)?);
        }
        let mut relations: BTreeMap<Kind, Vec<RelationshipPlan>> = BTreeMap::new();
        for relation in catalog.relationships() {
            relations.entry(relation.source_kind.clone()).or_default().push(relation.clone());
        }
        let trust = operations.values().flat_map(|operations| operations.values())
            .flat_map(|stages| stages.authority.trust.iter().cloned()).collect();
        Ok(Self::Reviewed(Reviewed { catalog: Arc::from(bytes), operations, relations, trust }))
    }

    pub(super) fn catalog(&self) -> Option<&[u8]> {
        match self { Self::Unavailable => None, Self::Reviewed(reviewed) => Some(&reviewed.catalog) }
    }

    fn checked_stage(&self, lease: &super::WorkLease) -> Result<(&Authority, stages::Output), StoreError> {
        let Self::Reviewed(reviewed) = self else { return Err(unreviewed()); };
        let scope = lease.scope();
        reviewed.operations.get(scope.service().as_str())
            .and_then(|operations| operations.get(scope.operation().as_str())).ok_or_else(unreviewed)?
            .checked(scope, lease.bindings())
    }

    fn authority(&self, lease: &super::WorkLease) -> Result<&Authority, StoreError> {
        let (authority, output) = self.checked_stage(lease)?;
        if output != stages::Output::Metadata { return Err(unreviewed()); }
        Ok(authority)
    }
    pub(super) fn validate_completion(&self, lease: &super::WorkLease) -> Result<(), StoreError> {
        if matches!(self, Self::Unavailable) { return Ok(()); }
        self.checked_stage(lease).map(|_| ())
    }


    pub(super) fn validate_page(&self, page: &PageCommit) -> Result<(), StoreError> {
        if matches!(self, Self::Unavailable) {
            return if page.unresolved.is_empty() && page.edges.is_empty() { Ok(()) } else { Err(unreviewed()) };
        }
        let (authority, output) = self.checked_stage(&page.lease)?;
        if output != stages::Output::Metadata {
            if !page.observations.is_empty() || !page.unresolved.is_empty() || !page.edges.is_empty()
                || (output != stages::Output::Tags && !page.tags.is_empty()) {
                return Err(unreviewed());
            }
            return Ok(());
        }
        if page.unresolved.is_empty() && page.edges.is_empty() { return Ok(()); }
        let sources: BTreeMap<_, _> = page.observations.iter().map(|observation| (observation.key(), observation)).collect();
        if sources.len() != page.observations.len() { return Err(StoreError::Invalid("duplicate relationship source observations")); }
        for reference in &page.unresolved {
            let source = sources.get(&reference.from).ok_or_else(unreviewed)?;
            authority.reference(&page.lease, source, reference)?;
        }
        for edge in &page.edges {
            let source = sources.get(&edge.from).ok_or_else(unreviewed)?;
            let target = sources.get(&edge.to).ok_or_else(unreviewed)?;
            authority.edge(&page.lease, source, target, edge)?;
        }
        Ok(())
    }

    pub(super) fn validate_reference(&self, reference: &UnresolvedReference) -> Result<(), StoreError> {
        let Self::Reviewed(reviewed) = self else { return Err(unreviewed()); };
        if reviewed.relations.get(&reference.from.kind).is_some_and(|rules| rules.iter().any(|rule| matches_reference(rule, reference)))
            || reviewed.trust.iter().any(|rule| rule.reference(reference)) { Ok(()) } else { Err(unreviewed()) }
    }

}

impl Authority {
    fn reference(&self, lease: &super::WorkLease, source: &Observation, reference: &UnresolvedReference) -> Result<(), StoreError> {
        for rule in &self.relations {
            if matches_reference(rule, reference) && source_matches(rule, lease, source, reference)? { return Ok(()); }
        }
        for rule in &self.trust {
            if rule.reference(reference) && rule.declares(source, reference.target.value().as_str())? { return Ok(()); }
        }
        Err(unreviewed())
    }

    fn edge(&self, lease: &super::WorkLease, source: &Observation, target: &Observation, edge: &Edge) -> Result<(), StoreError> {
        for rule in &self.relations {
            let Some(reference) = rule_reference(rule, lease, source, target, edge)? else { continue; };
            if reference_matches_observation(&reference, target, lease.bindings())? { return Ok(()); }
        }
        for rule in &self.trust {
            if rule.edge(edge) && rule.declares(source, edge.to.id.as_str())?
                && rule.target_matches(target, edge.to.id.as_str())? { return Ok(()); }
        }
        Err(unreviewed())
    }
}

fn matches_reference(rule: &RelationshipPlan, reference: &UnresolvedReference) -> bool {
    let target = &reference.target;
    rule.source_kind == reference.from.kind && rule.relation == reference.relation
        && (if rule.source == RelationSource::Parent { rule.target_kinds.contains(target.kind()) } else { rule.target_kind == *target.kind() })
        && rule.target_path.as_ref() == target.path()
        && rule.target_mode == target.mode()
        && rule.components.len() == target.components().len()
        && rule.components.iter().zip(target.components()).all(|(rule, term)| rule.target_mode == term.mode && rule.target_path == term.path)
        && rule.target_kinds.len() == target.candidate_kinds().len()
        && rule.target_kinds.iter().all(|kind| target.candidate_kinds().contains(kind))
        && owner_matches(rule, &reference.from, target.account(), target.region(), target.value().as_str())
}

fn owner_matches(rule: &RelationshipPlan, from: &ResourceKey, account: &inventory_core::inventory::AccountId, region: &Region, value: &str) -> bool {
    match &rule.owner {
        ReferenceOwnerRule::ExecutionAccount => account == &from.account && execution_arn_matches(from, region, value) && match rule.scope {
            RelationScope::AccountRegion => region == &from.region,
            RelationScope::Account => *region == Region::Global,
            RelationScope::HomeRegion => match &from.region {
                Region::Regional(home) => matches!(region, Region::Regional(target) if target == home),
                Region::Global => false,
            },
        },
        ReferenceOwnerRule::Arn(owner) => source_partition_matches(from, owner.partition.as_str())
            && owner.owner(value, &from.region)
                .is_ok_and(|(expected_account, expected_region)| expected_account == *account && expected_region == *region),
    }
}

fn source_partition_matches(from: &ResourceKey, partition: &str) -> bool {
    if !from.id.as_str().starts_with("arn:") { return true; }
    execution_arn_matches(from, &from.region, from.id.as_str())
        && from.id.as_str().split(':').nth(1) == Some(partition)
}

fn execution_arn_matches(from: &ResourceKey, region: &Region, value: &str) -> bool {
    if !value.starts_with("arn:") { return true; }
    if value.len() > 4096 || !value.is_ascii() { return false; }
    let mut parts = value.splitn(6, ':');
    if parts.next() != Some("arn") { return false; }
    let Some(partition @ ("aws" | "aws-cn" | "aws-us-gov")) = parts.next() else { return false; };
    if from.id.as_str().starts_with("arn:") && from.id.as_str().split(':').nth(1) != Some(partition) { return false; }
    let Some(service) = parts.next() else { return false; };
    if service.is_empty() || !service.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-') { return false; }
    let expected_region = match region { Region::Global => "", Region::Regional(region) => region.as_str() };
    if parts.next() != Some(expected_region) { return false; }
    let Some(account) = parts.next() else { return false; };
    if !account.is_empty() && account != from.account.as_str() { return false; }
    parts.next().is_some_and(|resource| !resource.is_empty()
        && !resource.bytes().any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace() || matches!(byte, b'*' | b'?')))
}


pub(super) fn scalar<'a>(root: MapRef<'a>, path: &[FieldId]) -> Result<Option<ValueRef<'a>>, StoreError> {
    let Some((last, parents)) = path.split_last() else { return Ok(root.get_name("$")?); };
    let parent = parents.iter().try_fold(Some(root), |current, field| -> Result<_, StoreError> {
        match current.map(|map| map.get_field(*field)).transpose()?.flatten() {
            Some(ValueRef::Map(map)) => Ok(Some(map)),
            _ => Ok(None),
        }
    })?;
    Ok(parent.map(|map| map.get_field(*last)).transpose()?.flatten())
}

fn unreviewed() -> StoreError { StoreError::Invalid("relationship evidence is not permitted by the checked catalog") }

#[cfg(test)]
mod tests;
