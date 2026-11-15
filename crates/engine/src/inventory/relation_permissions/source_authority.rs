use super::{Authority, Permissions, StoreError, matches_reference, source_matches, stages, unreviewed};
use crate::inventory::{SchemaRegistry, WorkLease};
use inventory_core::{
    catalog::runtime::{ArnRegionRule, ReferenceOwnerRule, RelationScope, RelationSource, RelationshipPlan},
    inventory::{Observation, UnresolvedReference},
    record::FieldId,
};
use sha2::{Digest, Sha256};

/// Minting capability borrowed from the exact checked execution, never from a saved tuple.
pub(in crate::inventory) struct PageAuthority<'a> {
    registry: &'a SchemaRegistry,
    lease: &'a WorkLease,
    rules: &'a Authority,
    source_facet: Vec<u8>,
    base: [u8; 32],
}

/// Exact source payload capability, hashed once and borrowed by grants and codec.
pub(in crate::inventory) struct SourceAuthority<'source, 'lease> {
    page: &'source PageAuthority<'lease>,
    source: &'source Observation,
    authority: [u8; 32],
}

pub(in crate::inventory) fn page_authority<'a>(registry: &'a SchemaRegistry, lease: &'a WorkLease) -> Result<Option<PageAuthority<'a>>, StoreError> {
    if matches!(registry.permissions, Permissions::Unavailable) { return Ok(None); }
    let (rules, output) = registry.permissions.checked_stage(lease)?;
    if output != stages::Output::Metadata { return Ok(None); }
    let source_facet = crate::inventory::work::ownership_scope_bytes(lease.scope(), lease.bindings())?;
    let mut hash = Sha256::new();
    hash.update(b"aws-inventory/page-authority/v3\0");
    hash.update(registry.digest());
    bytes(&mut hash, &source_facet)?;
    // Scope identity deliberately excludes a volatile discovery-context digest. The
    // complete immutable input still distinguishes root executions with other inputs.
    hash.update(Sha256::digest(lease.bindings()));
    Ok(Some(PageAuthority { registry, lease, rules, source_facet, base: hash.finalize().into() }))
}

impl<'lease> PageAuthority<'lease> {
    pub(in crate::inventory) fn source<'source>(&'source self, source: &'source Observation) -> Result<SourceAuthority<'source, 'lease>, StoreError> {
        self.validate_source(source)?;
        let mut hash = Sha256::new();
        hash.update(b"aws-inventory/source-payload-authority/v3\0");
        hash.update(self.base);
        let key = source.key();
        for value in [key.account.as_str(), key.region.as_str(), key.kind.as_str(), key.id.as_str()] {
            bytes(&mut hash, value.as_bytes())?;
        }
        bytes(&mut hash, source.record().as_bytes())?;
        Ok(SourceAuthority { page: self, source, authority: hash.finalize().into() })
    }

    pub(in crate::inventory) fn validate_source(&self, source: &Observation) -> Result<(), StoreError> {
        let kind = &source.key().kind;
        let emitted = self.rules.classification.as_ref().map_or_else(
            || kind == &self.rules.kind,
            |classification| classification.classify(source.record()).is_ok_and(|expected| expected == kind),
        );
        if emitted { Ok(()) } else { Err(unreviewed()) }
    }
}

impl SourceAuthority<'_, '_> {
    pub(in crate::inventory) fn source_facet(&self) -> &[u8] { &self.page.source_facet }
    pub(in crate::inventory) fn source_authority(&self) -> &[u8; 32] { &self.authority }
    pub(in crate::inventory) fn registry(&self) -> &SchemaRegistry { self.page.registry }

    pub(in crate::inventory) fn reference_authority(&self, reference: &UnresolvedReference) -> Result<[u8; 32], StoreError> {
        if self.source.key() != &reference.from { return Err(unreviewed()); }
        let source = self.source;
        let mut hash = Sha256::new();
        hash.update(b"aws-inventory/source-reference-authority/v3\0");
        hash.update(self.authority);
        hash.update(super::provenance::proof(self.page.registry, reference));
        let mut matched = false;
        for rule in &self.page.rules.relations {
            if matches_reference(rule, reference)
                && source_matches(rule, self.page.lease.bindings(), source, reference.target.value().as_str())? {
                hash.update([1]);
                relation(&mut hash, rule)?;
                matched = true;
            }
        }
        for rule in &self.page.rules.trust {
            if rule.reference(reference) && rule.declares(source, reference.target.value().as_str())? {
                hash.update([2]);
                // Reference identity already binds target kind/path and principal;
                // the checked projection path additionally binds its actual origin.
                path(&mut hash, rule.dependency_path())?;
                matched = true;
            }
        }
        if !matched { return Err(unreviewed()); }
        hash.update([0]);
        Ok(hash.finalize().into())
    }
}

fn relation(hash: &mut Sha256, rule: &RelationshipPlan) -> Result<(), StoreError> {
    bytes(hash, rule.source_kind.as_str().as_bytes())?;
    hash.update([match rule.source { RelationSource::SelectedOutput => 0, RelationSource::Request => 1 }]);
    path(hash, &rule.source_path)?;
    bytes(hash, rule.target_kind.as_str().as_bytes())?;
    length(hash, rule.target_kinds.len())?;
    for kind in rule.target_kinds.iter() { bytes(hash, kind.as_str().as_bytes())?; }
    path(hash, &rule.target_path)?;
    bytes(hash, rule.relation.as_str().as_bytes())?;
    hash.update([match rule.scope { RelationScope::AccountRegion => 0, RelationScope::Account => 1, RelationScope::HomeRegion => 2 }]);
    match &rule.owner {
        ReferenceOwnerRule::ExecutionAccount => hash.update([0]),
        ReferenceOwnerRule::Arn(owner) => {
            hash.update([1]);
            bytes(hash, owner.partition.as_bytes())?;
            bytes(hash, owner.service.as_bytes())?;
            bytes(hash, owner.resource_prefix.as_bytes())?;
            hash.update([match owner.region { ArnRegionRule::Source => 0, ArnRegionRule::Global => 1 }]);
        }
    }
    Ok(())
}

fn length(hash: &mut Sha256, value: usize) -> Result<(), StoreError> {
    let value = u32::try_from(value).map_err(|_| StoreError::Invalid("relationship authority exceeds encoding bounds"))?;
    hash.update(value.to_le_bytes());
    Ok(())
}

fn bytes(hash: &mut Sha256, value: &[u8]) -> Result<(), StoreError> {
    length(hash, value.len())?;
    hash.update(value);
    Ok(())
}

fn path(hash: &mut Sha256, value: &[FieldId]) -> Result<(), StoreError> {
    length(hash, value.len())?;
    for field in value { hash.update(field.value().to_le_bytes()); }
    Ok(())
}
