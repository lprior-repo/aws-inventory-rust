mod metadata;
mod plans;
mod schemas;

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
}

pub(crate) fn read<'a>(
    bytes: &'a [u8],
    catalog: Catalog<'a>,
) -> Result<Decoded<'a>, RuntimeCatalogError> {
    let root = MapRef::from_bytes_bounded(bytes)?;
    let extension_id = FieldId::new(60_000)?;
    let extension = match root.get_field(extension_id)? {
        Some(ValueRef::Map(value)) => value,
        Some(_) => return Err(RuntimeCatalogError::Capability),
        None => return Err(RuntimeCatalogError::MissingExtension),
    };
    let version = metadata::integer(extension, 1)?;
    if version != 1 {
        return Err(RuntimeCatalogError::ExtensionVersion(version));
    }
    let plans = plans::read(catalog, metadata::list(extension, 2)?)?;
    let schema_data = schemas::read(metadata::list(extension, 3)?)?;
    let relations = metadata::relations(metadata::list(extension, 4)?)?;
    let coverage = metadata::coverage(metadata::list(extension, 5)?)?;
    let decisions = metadata::decisions(metadata::list(extension, 6)?, &plans, catalog)?;
    metadata::only_fields(extension, &[1, 2, 3, 4, 5, 6])?;
    let schemas = schema_data.schemas;
    validate_links(&schemas, &schema_data.name_paths, &relations, &coverage)?;
    Ok(Decoded {
        plans,
        schemas,
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
                || !schema_kinds.contains(entry.target_kind.as_str())
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

pub(super) fn catalog_operation<'a>(
    catalog: Catalog<'a>,
    service_name: &str,
    operation_name: &str,
) -> Result<(crate::catalog::Service<'a>, crate::catalog::Operation<'a>), RuntimeCatalogError> {
    let service = catalog
        .service(service_name)?
        .ok_or(RuntimeCatalogError::Capability)?;
    let mut found = None;
    service.operations()?.try_for_each(|candidate| {
        let candidate = candidate?;
        if candidate.name()? == operation_name && found.replace(candidate).is_some() {
            return Err(RuntimeCatalogError::Capability);
        }
        Ok::<_, RuntimeCatalogError>(())
    })?;
    let operation = found.ok_or(RuntimeCatalogError::Capability)?;
    Ok((service, operation))
}

pub(super) fn validate_operations<'k>(
    catalog: Catalog<'_>,
    keys: impl Iterator<Item = (&'k str, &'k str)>,
) -> Result<(), RuntimeCatalogError> {
    let mut grouped = std::collections::BTreeMap::<&str, Vec<&str>>::new();
    keys.for_each(|(service, operation)| grouped.entry(service).or_default().push(operation));
    grouped
        .into_iter()
        .try_for_each(|(service_name, operations)| {
            let service = catalog
                .service(service_name)?
                .ok_or(RuntimeCatalogError::Capability)?;
            let mut names = service
                .operations()?
                .map(|operation| operation?.name())
                .collect::<Result<Vec<_>, _>>()?;
            names.sort_unstable();
            let present_once = operations.iter().all(|operation| {
                let start = names.partition_point(|name| name < operation);
                names.get(start..).map_or(0, |rest| {
                    rest.iter().take_while(|name| *name == operation).count()
                }) == 1
            });
            if present_once {
                Ok(())
            } else {
                Err(RuntimeCatalogError::Capability)
            }
        })
}

pub(super) fn lookup(map: MapRef<'_>, key: u32) -> Result<ValueRef<'_>, RuntimeCatalogError> {
    let field = FieldId::new(key)?;
    map.get_field(field)?.ok_or(RuntimeCatalogError::Capability)
}
