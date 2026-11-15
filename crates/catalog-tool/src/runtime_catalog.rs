mod encode;
mod model_path;
mod plan_validation;
mod plans;
mod schemas;
mod provenance;
mod selection;
pub use provenance::CatalogSources;

use inventory_core::record::{FieldId, Key, Record, Value};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RuntimeCatalogCompileError {
    #[error("compiled catalog is invalid")]
    Catalog(#[from] inventory_core::catalog::CatalogError),
    #[error(
        "runtime capability metadata is unsupported for service {service}, operation {operation}, path {path}"
    )]
    CapabilityContext {
        service: String,
        operation: String,
        path: String,
    },
    #[error("runtime capability metadata is invalid or unsupported")]
    Capability,
    #[error("binary record encoding failed")]
    Record(#[from] inventory_core::record::RecordError),
    #[error("inventory metadata is invalid")]
    Inventory(#[from] inventory_core::inventory::InventoryError),
}
pub(super) struct ServiceIndex<'a> {
    by_name: BTreeMap<&'a str, inventory_core::catalog::Service<'a>>,
    field_ids: BTreeMap<&'a str, FieldId>,
}

impl<'a> ServiceIndex<'a> {
    fn build(
        catalog: inventory_core::catalog::Catalog<'a>,
    ) -> Result<Self, RuntimeCatalogCompileError> {
        let (by_name, field_ids) = catalog.services()?.try_fold(
            (BTreeMap::new(), BTreeMap::new()),
            |(mut services, mut fields), result| {
                let service = result?;
                let shape_id = service.shape_id()?;
                let namespace_key = shape_id
                    .split_once('#')
                    .and_then(|(namespace, _)| namespace.rsplit('.').next())
                    .ok_or(inventory_core::catalog::CatalogError::Reference)?;
                insert_service(&mut services, service.name()?, service)?;
                insert_service(&mut services, namespace_key, service)?;
                index_service_fields(&mut fields, service)?;
                Ok::<_, RuntimeCatalogCompileError>((services, fields))
            },
        )?;
        Ok(Self { by_name, field_ids })
    }

    pub(super) fn get(
        &self,
        key: &str,
    ) -> Result<inventory_core::catalog::Service<'a>, RuntimeCatalogCompileError> {
        self.by_name
            .get(key)
            .copied()
            .ok_or(RuntimeCatalogCompileError::Capability)
    }

    pub(super) fn field_id(&self, name: &str) -> Result<FieldId, RuntimeCatalogCompileError> {
        self.field_ids
            .get(name)
            .copied()
            .ok_or(RuntimeCatalogCompileError::Capability)
    }
}

fn insert_service<'a>(
    index: &mut BTreeMap<&'a str, inventory_core::catalog::Service<'a>>,
    key: &'a str,
    service: inventory_core::catalog::Service<'a>,
) -> Result<(), RuntimeCatalogCompileError> {
    match index.get(key) {
        Some(existing) if existing.shape_id()? != service.shape_id()? => {
            Err(inventory_core::catalog::CatalogError::Reference.into())
        }
        Some(_) => Ok(()),
        None => {
            index.insert(key, service);
            Ok(())
        }
    }
}
fn index_service_fields<'a>(
    fields: &mut BTreeMap<&'a str, FieldId>,
    service: inventory_core::catalog::Service<'a>,
) -> Result<(), inventory_core::catalog::CatalogError> {
    service.shapes()?.try_for_each(|shape| {
        shape?.members()?.try_for_each(|member| {
            let member = member?;
            let name = member.name()?;
            if fields
                .get(name)
                .is_some_and(|previous| *previous != member.id)
            {
                return Err(inventory_core::catalog::CatalogError::Reference);
            }
            fields.insert(name, member.id);
            Ok(())
        })
    })
}

pub fn augment(
    base_bytes: &[u8],
    checked: &crate::overlay::CheckedOverlays,
    kind_map: &crate::kind_map::KindMap,
    verified_endpoints: &BTreeMap<String, crate::endpoints::VerifiedEndpoint>,
) -> Result<Vec<u8>, RuntimeCatalogCompileError> {
    augment_with_sources(base_bytes, checked, kind_map, verified_endpoints, &CatalogSources::compiled(base_bytes))
}

pub fn augment_with_sources(
    base_bytes: &[u8],
    checked: &crate::overlay::CheckedOverlays,
    kind_map: &crate::kind_map::KindMap,
    verified_endpoints: &BTreeMap<String, crate::endpoints::VerifiedEndpoint>,
    sources: &CatalogSources,
) -> Result<Vec<u8>, RuntimeCatalogCompileError> {
    let mut root = decode_root(base_bytes)?;
    if root.contains_key(&Key::Field(FieldId::new(60_000)?)) {
        return Err(RuntimeCatalogCompileError::Capability);
    }
    let catalog = inventory_core::catalog::Catalog::borrow(base_bytes)?;
    sources.validate(catalog)?;
    let services = ServiceIndex::build(catalog)?;
    model_path::validate_reviewed_paths(&model_path::ValidationContext {
        kinds: kind_map,
        checked,
        services: &services,
    })?;
    let plans = plans::compile(checked, kind_map, verified_endpoints, &services)?;
    let extension = compile_extension(checked, kind_map, &services, plans, sources)?;
    root.insert(Key::Field(FieldId::new(60_000)?), extension);
    let bytes = Record::encode(&Value::Map(root))?.into_bytes();
    inventory_core::catalog::runtime::RuntimeCatalog::from_bytes(&bytes)
        .map_err(|_| RuntimeCatalogCompileError::Capability)?;
    Ok(bytes)
}
fn decode_root(bytes: &[u8]) -> Result<BTreeMap<Key, Value>, RuntimeCatalogCompileError> {
    inventory_core::record::MapRef::from_bytes_bounded(bytes)?
        .iter()?
        .map(|entry| {
            let (key, value) = entry?;
            Ok((key.to_key(), value.to_value()?))
        })
        .collect::<Result<BTreeMap<_, _>, inventory_core::record::RecordError>>()
        .map_err(RuntimeCatalogCompileError::Record)
}

fn compile_extension(
    checked: &crate::overlay::CheckedOverlays,
    kinds: &crate::kind_map::KindMap,
    services: &ServiceIndex<'_>,
    plans: Vec<Value>,
    sources: &CatalogSources,
) -> Result<Value, RuntimeCatalogCompileError> {
    encode::extension(
        plans,
        schemas::compile(kinds, services, Some(checked))?,
        encode::relations(kinds, services, checked)?,
        encode::coverage(kinds)?,
        plans::decisions(checked, services)?,
        sources,
        kinds,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use inventory_core::catalog::runtime::RelationScope;
    use inventory_core::record::{MapRef, ValueRef};

    const BASE: &[u8] = include_bytes!("../../../catalog/aws-api.catalog");

    #[test]
    fn real_kind_map_relation_roundtrips_through_runtime_reader()
    -> Result<(), Box<dyn std::error::Error>> {
        let kinds = crate::kind_map::embedded(
            include_bytes!("../../../catalog/cloudcontrol-universal.json"),
        )?;
        let bytes = catalog_with_extension(&kinds)?;
        let runtime = inventory_core::catalog::runtime::RuntimeCatalog::from_bytes(&bytes)?;
        let relationship = runtime
            .relationships()
            .first()
            .ok_or("kind registry has no relationship")?;
        assert_eq!(relationship.scope, RelationScope::AccountRegion);
        Ok(())
    }

    fn catalog_with_extension(
        kinds: &crate::kind_map::KindMap,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let services = ServiceIndex::build(inventory_core::catalog::Catalog::borrow(BASE)?)?;
        let schemas = schemas::compile(kinds, &services, None)?;
        let relations = encode::relations(kinds, &services)?;
        let extension = encode::extension(
            Vec::new(),
            schemas,
            relations,
            encode::coverage(kinds)?,
            Vec::new(),
        )?;
        let mut root = ValueRef::Map(MapRef::from_bytes_bounded(BASE)?).to_value()?;
        let Value::Map(fields) = &mut root else {
            return Err("compiled catalog root is not a map".into());
        };
        fields.insert(Key::Field(FieldId::new(60_000)?), extension);
        Ok(Record::encode(&root)?.into_bytes())
    }
}
