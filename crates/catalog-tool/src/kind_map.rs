mod builtin;
mod construct;
mod fingerprint;
mod merge;
mod relation_targets;
mod source;
pub use source::{ApiOnlyMapping, BootstrapMapping, CfnMapping, MappingSource, OperationMapping};
mod emissions;
mod relation_fingerprint;
mod relation_modes;
mod relation_selectors;
mod target_kinds;
pub use builtin::embedded;
pub use emissions::{AdditionalEmissionMapping, ReviewedEmission};
pub use relation_modes::{
    CompoundRelationMapping, ParentRelationMapping, RelationComponentMapping,
    ReviewedCompoundRelation, ReviewedParentRelation, ReviewedRelationComponent,
};
pub use relation_selectors::{
    RelationGuardMapping, RelationMapping, RelationPath, RelationSource, RelationTarget,
    RelationTargetsMapping, ReviewedRelation, ReviewedRelationGuard, ReviewedRelationSource,
    ReviewedRelationTarget, ReviewedValuePath,
};
pub use target_kinds::ReviewedTargetKinds;
#[path = "kind_map/rows.rs"]
mod rows;
use crate::names::{Exceptions, NameError, SqlName};
use std::collections::BTreeSet;
use thiserror::Error;
mod bootstrap;
mod operations;
mod validation;
use operations::validate as validate_operations;
use validation::*;
const MAX_BYTES: usize = 2_097_152;
const MAX_ROWS: usize = 10_000;
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum KindMapError {
    #[error("kind map exceeds the {MAX_BYTES}-byte bound")]
    TooLarge,
    #[error("kind map exceeds the {MAX_ROWS}-row bound")]
    TooManyRows,
    #[error("invalid typed kind mapping entry {line}: {reason}")]
    InvalidRow { line: usize, reason: &'static str },
    #[error("duplicate kind map entry: {0}")]
    Duplicate(String),
    #[error("CFN source type is not accounted for: {0}")]
    MissingType(String),
    #[error("kind map contains a CFN type absent from its source: {0}")]
    ExtraType(String),
    #[error("operation {service}.{operation} references unknown kind {kind}")]
    UnknownKind {
        service: String,
        operation: String,
        kind: String,
    },
    #[error("bootstrap role references missing operation {service}.{operation}")]
    UnknownBootstrapOperation { service: String, operation: String },
    #[error("bootstrap operation {service}.{operation} is not API-only: {kind}")]
    BootstrapOperationNotApiOnly {
        service: String,
        operation: String,
        kind: String,
    },
    #[error("duplicate canonical kind target: {0}")]
    DuplicateCanonicalKind(String),
    #[error("invalid CFN source: {0}")]
    CfnSource(#[from] serde_json::Error),
    #[error("relation {relation} references unknown kind {kind}")]
    UnknownRelationKind { relation: String, kind: String },
    #[error("invalid SQL name for {kind}: {source}")]
    Name {
        kind: String,
        #[source]
        source: NameError,
    },
}

#[derive(serde::Deserialize)]
struct CfnSource {
    resource_types: Vec<CfnType>,
}
#[derive(serde::Deserialize)]
struct CfnType {
    type_name: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KindTarget {
    Mapped { kind: String, table_name: SqlName },
    Unmapped { reason: String },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KindEntry {
    pub cfn_type: String,
    pub target: KindTarget,
    pub display_name_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiOnlyKind {
    pub kind: String,
    pub table_name: SqlName,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum BootstrapPurpose {
    #[default]
    Enumeration,
    AccountIdentity {
        account_path: String,
    },
    RegionDiscovery {
        regions_path: String,
        region_name_path: String,
        opt_in_status_path: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOperation {
    pub service: String,
    pub operation: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KindMap {
    source_version: u16,
    source_sha256: [u8; 32],
    reviewer: String,
    reviewed: String,
    default_unmapped_reason: Option<String>,
    entries: Vec<KindEntry>,
    api_only: Vec<ApiOnlyKind>,
    relations: Vec<ReviewedRelation>,
    parent_relations: Vec<ReviewedParentRelation>,
    compound_relations: Vec<ReviewedCompoundRelation>,
    additional_emissions: Vec<ReviewedEmission>,
    bootstraps: Vec<bootstrap::BootstrapEntry>,
    operations: Vec<RuntimeOperation>,
}

impl KindMap {
    pub fn entries(&self) -> &[KindEntry] {
        &self.entries
    }
    pub fn api_only(&self) -> &[ApiOnlyKind] {
        &self.api_only
    }
    pub fn relations(&self) -> &[ReviewedRelation] {
        &self.relations
    }
    pub fn parent_relations(&self) -> &[ReviewedParentRelation] {
        &self.parent_relations
    }
    pub fn compound_relations(&self) -> &[ReviewedCompoundRelation] {
        &self.compound_relations
    }
    pub fn additional_emissions(&self) -> &[ReviewedEmission] {
        &self.additional_emissions
    }
    pub fn additional_emission_kind(
        &self,
        service: &str,
        operation: &str,
        emitter_key: &str,
    ) -> Option<&str> {
        emissions::lookup(&self.additional_emissions, service, operation, emitter_key)
    }
    pub fn bootstrap_for(&self, service: &str, operation: &str) -> Option<&BootstrapPurpose> {
        self.bootstraps
            .iter()
            .find(|entry| entry.service == service && entry.operation == operation)
            .map(|entry| &entry.purpose)
    }
    pub fn operations(&self) -> &[RuntimeOperation] {
        &self.operations
    }
    pub fn source_version(&self) -> u16 {
        self.source_version
    }
    pub fn source_sha256(&self) -> &[u8; 32] {
        &self.source_sha256
    }
    pub fn reviewer(&self) -> &str {
        &self.reviewer
    }
    pub fn reviewed(&self) -> &str {
        &self.reviewed
    }
    pub fn review_provenance(&self) -> &str {
        &self.reviewer
    }
    pub fn lookup(&self, cfn_type: &str) -> Option<&KindEntry> {
        self.entries.iter().find(|entry| entry.cfn_type == cfn_type)
    }
}

fn complete_cfn(map: &mut KindMap, cfn_source: &[u8]) -> Result<(), KindMapError> {
    if cfn_source.len() > MAX_BYTES {
        return Err(KindMapError::TooLarge);
    }
    let expected = cfn_types(cfn_source)?;
    let actual: BTreeSet<String> = map
        .entries
        .iter()
        .map(|entry| entry.cfn_type.clone())
        .collect();
    if let Some(extra) = actual.difference(&expected).next() {
        return Err(KindMapError::ExtraType(extra.clone()));
    }
    let missing: Vec<String> = expected.difference(&actual).cloned().collect();
    fill_unmapped(map, missing)?;
    map.entries
        .sort_by(|left, right| left.cfn_type.cmp(&right.cfn_type));
    Ok(())
}

fn fill_unmapped(map: &mut KindMap, missing: Vec<String>) -> Result<(), KindMapError> {
    let Some(reason) = map.default_unmapped_reason.clone() else {
        return match missing.into_iter().next() {
            Some(first) => Err(KindMapError::MissingType(first)),
            None => Ok(()),
        };
    };
    let unmapped = missing.into_iter().map(|cfn_type| KindEntry {
        cfn_type,
        target: KindTarget::Unmapped {
            reason: reason.clone(),
        },
        display_name_path: None,
    });
    map.entries.extend(unmapped);
    Ok(())
}

fn cfn_types(cfn_source: &[u8]) -> Result<BTreeSet<String>, KindMapError> {
    let source: CfnSource = serde_json::from_slice(cfn_source)?;
    let names: Vec<String> = source
        .resource_types
        .into_iter()
        .map(|entry| entry.type_name)
        .collect();
    if names.iter().any(|name| !valid_cfn_type(name)) {
        return Err(row_error(0, "invalid CFN source type"));
    }
    let expected: BTreeSet<String> = names.iter().cloned().collect();
    if expected.len() != names.len() {
        return Err(row_error(0, "duplicate CFN source type"));
    }
    Ok(expected)
}
