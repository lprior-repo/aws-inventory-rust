use super::bootstrap::BootstrapEntry;
use super::{ApiOnlyKind, KindEntry, KindMapError, KindTarget, ReviewedRelation, RuntimeOperation};
use std::collections::BTreeSet;

pub(super) fn validate(
    known: &BTreeSet<&str>,
    api_only: &[ApiOnlyKind],
    operations: &[RuntimeOperation],
    relations: &[ReviewedRelation],
    bootstraps: &[BootstrapEntry],
) -> Result<(), KindMapError> {
    validate_operations(known, operations)?;
    validate_relations(known, relations)?;
    validate_bootstraps(api_only, operations, bootstraps)
}

pub(super) fn known_kinds<'a>(
    entries: &'a [KindEntry],
    api_only: &'a [ApiOnlyKind],
) -> BTreeSet<&'a str> {
    let mapped = entries.iter().filter_map(|entry| match &entry.target {
        KindTarget::Mapped { kind, .. } => Some(kind.as_str()),
        KindTarget::Unmapped { .. } => None,
    });
    mapped
        .chain(api_only.iter().map(|entry| entry.kind.as_str()))
        .collect()
}

fn validate_operations(
    known: &BTreeSet<&str>,
    operations: &[RuntimeOperation],
) -> Result<(), KindMapError> {
    operations.iter().try_for_each(|operation| {
        if known.contains(operation.kind.as_str()) {
            return Ok(());
        }
        Err(KindMapError::UnknownKind {
            service: operation.service.clone(),
            operation: operation.operation.clone(),
            kind: operation.kind.clone(),
        })
    })
}

fn validate_relations(
    known: &BTreeSet<&str>,
    relations: &[ReviewedRelation],
) -> Result<(), KindMapError> {
    relations.iter().try_for_each(|relation| {
        std::iter::once(relation.source_kind.as_str())
            .chain(relation.target_kinds.iter())
            .try_for_each(|kind| {
                if known.contains(kind) {
                    return Ok(());
                }
                Err(KindMapError::UnknownRelationKind {
                    relation: relation.relation.clone(),
                    kind: kind.to_owned(),
                })
            })
    })
}
fn validate_bootstraps(
    api_only: &[ApiOnlyKind],
    operations: &[RuntimeOperation],
    bootstraps: &[BootstrapEntry],
) -> Result<(), KindMapError> {
    bootstraps.iter().try_for_each(|entry| {
        let matched = operations.iter().find(|operation| {
            operation.service == entry.service && operation.operation == entry.operation
        });
        let Some(operation) = matched else {
            return Err(KindMapError::UnknownBootstrapOperation {
                service: entry.service.clone(),
                operation: entry.operation.clone(),
            });
        };
        if api_only.iter().any(|kind| kind.kind == operation.kind) {
            return Ok(());
        }
        Err(KindMapError::BootstrapOperationNotApiOnly {
            service: entry.service.clone(),
            operation: entry.operation.clone(),
            kind: operation.kind.clone(),
        })
    })
}
