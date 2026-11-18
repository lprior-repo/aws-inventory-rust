mod metadata;
mod owners;
mod plans;
mod provenance;
mod schemas;
mod services;
pub(super) use provenance::compiled_inputs_sha256;

use super::{CallablePlan, RuntimeCatalogError};
use crate::{
    catalog::Catalog,
    inventory::KindSchema,
    record::{FieldId, ListRef, MapRef, ValueRef},
};
use std::sync::Arc;

pub(crate) struct Decoded<'a> {
    pub(crate) plans: Vec<CallablePlan<'a>>,
    pub(crate) schemas: Vec<Arc<KindSchema>>,
    pub(crate) name_paths: Vec<super::KindNamePath>,
    pub(crate) relations: Vec<super::RelationshipPlan>,
    pub(crate) coverage: Vec<super::KindCoverage>,
    pub(crate) decisions: Vec<super::ReviewDecision>,
    pub(crate) provenance: super::CatalogSourceProvenance,
}

pub(crate) fn read<'a>(
    bytes: &'a [u8],
    catalog: Catalog<'a>,
) -> Result<Decoded<'a>, RuntimeCatalogError> {
    let root = MapRef::from_container_bytes(bytes, crate::catalog::MAX_CATALOG_BYTES)?;
    let extension_id = FieldId::new(60_000)?;
    let extension = match root.get_field(extension_id)? {
        Some(ValueRef::Map(value)) => value,
        Some(_) => return Err(RuntimeCatalogError::Capability),
        None => return Err(RuntimeCatalogError::MissingExtension),
    };
    let version = metadata::integer(extension, 1)?;
    if version != 2 {
        return Err(RuntimeCatalogError::ExtensionVersion(version));
    }
    let services = services::Services::open(catalog)?;
    let plans = plans::read(&services, metadata::list(extension, 2)?)?;
    let schema_data = schemas::read(metadata::list(extension, 3)?)?;
    let relations = metadata::relations(metadata::list(extension, 4)?)?;
    let coverage = metadata::coverage(metadata::list(extension, 5)?)?;
    let decisions = metadata::decisions(metadata::list(extension, 6)?, &plans, &services)?;
    let provenance = provenance::read(metadata::map_field(extension, 7)?)?;
    provenance::validate_inputs(extension, &provenance)?;
    provenance::validate_models(bytes, catalog, &provenance)?;
    metadata::only_fields(extension, &[1, 2, 3, 4, 5, 6, 7])?;
    let schemas = schema_data.schemas;
    if plans.iter().any(|plan| {
        !schemas.iter().any(|schema| schema.kind() == plan.kind())
            || !coverage
                .iter()
                .any(|entry| entry.mapped && entry.kind.as_ref() == Some(plan.kind()))
    }) {
        return Err(RuntimeCatalogError::Capability);
    }
    validate_links(&schemas, &schema_data.name_paths, &relations, &coverage)?;
    plans::validate_model_links(
        &plans,
        &schemas,
        &schema_data.name_paths,
        &relations,
        &coverage,
    )?;
    Ok(Decoded {
        plans,
        schemas,
        provenance,
        name_paths: schema_data.name_paths,
        relations,
        coverage,
        decisions,
    })
}

fn validate_links(
    schemas: &[Arc<KindSchema>],
    names: &[super::KindNamePath],
    relations: &[super::RelationshipPlan],
    coverage: &[super::KindCoverage],
) -> Result<(), RuntimeCatalogError> {
    let schema_kinds = schemas
        .iter()
        .map(|schema| schema.kind().as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let mapped_kinds = coverage
        .iter()
        .filter(|entry| entry.mapped)
        .filter_map(|entry| entry.kind.as_ref().map(crate::inventory::Kind::as_str))
        .collect::<std::collections::BTreeSet<_>>();
    if !schema_kinds.is_subset(&mapped_kinds) || !mapped_kinds.is_subset(&schema_kinds) {
        return Err(RuntimeCatalogError::Capability);
    }
    if names
        .iter()
        .any(|entry| !schema_kinds.contains(entry.kind.as_str()))
        || relations.iter().any(|entry| {
            !schema_kinds.contains(entry.source_kind.as_str())
                || entry
                    .target_kinds
                    .iter()
                    .any(|kind| !schema_kinds.contains(kind.as_str()))
        })
    {
        return Err(RuntimeCatalogError::Capability);
    }
    Ok(())
}

pub(super) fn map(value: ValueRef<'_>) -> Result<MapRef<'_>, RuntimeCatalogError> {
    match value {
        ValueRef::Map(map) => Ok(map),
        _ => Err(RuntimeCatalogError::Capability),
    }
}

pub(super) fn list_value(value: ValueRef<'_>) -> Result<ListRef<'_>, RuntimeCatalogError> {
    match value {
        ValueRef::List(list) => Ok(list),
        _ => Err(RuntimeCatalogError::Capability),
    }
}

pub(super) fn integer_value(value: ValueRef<'_>) -> Result<i64, RuntimeCatalogError> {
    match value {
        ValueRef::Int(value) => Ok(value),
        _ => Err(RuntimeCatalogError::Capability),
    }
}

pub(super) fn text_value(value: ValueRef<'_>) -> Result<String, RuntimeCatalogError> {
    match value {
        ValueRef::Text(value) if !value.is_empty() && value.len() <= 16_384 => Ok(value.to_owned()),
        _ => Err(RuntimeCatalogError::Capability),
    }
}

pub(super) fn lookup(map: MapRef<'_>, key: u32) -> Result<ValueRef<'_>, RuntimeCatalogError> {
    let field = FieldId::new(key)?;
    map.get_field(field)?.ok_or(RuntimeCatalogError::Capability)
}
