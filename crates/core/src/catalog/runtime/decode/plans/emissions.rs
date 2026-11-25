mod namespace;

use super::{
    RuntimeCatalogError, capabilities, exclusions, map_exclusions, metadata, selection, tags,
};
use crate::{
    catalog::{Operation, Service, runtime::ResourceEmissionPlan},
    inventory::Kind,
    record::{FieldId, MapRef, ValueRef},
};

pub(super) fn additional(
    fields: MapRef<'_>,
    service_name: &str,
    service: Service<'_>,
    operation: Operation<'_>,
) -> Result<Vec<ResourceEmissionPlan>, RuntimeCatalogError> {
    let Some(value) = fields.get_field(FieldId::new(23)?)? else {
        return Ok(Vec::new());
    };
    metadata::bounded(
        super::super::list_value(value)?,
        |value| descriptor(value, service_name, service, operation),
        7,
    )
}

fn descriptor(
    value: ValueRef<'_>,
    service_name: &str,
    service: Service<'_>,
    operation: Operation<'_>,
) -> Result<ResourceEmissionPlan, RuntimeCatalogError> {
    let fields = super::super::map(value)?;
    metadata::only_fields(fields, &[1, 3, 5, 6, 9, 14, 17, 18, 21])?;
    let key = metadata::text(fields, 1)?;
    if key == "primary" {
        return Err(RuntimeCatalogError::Capability);
    }
    let kind = Kind::parse(&metadata::text(fields, 3)?)?;
    let items = selection::item(metadata::map_field(fields, 5)?)?;
    selection::validate_item(service, operation, &items)?;
    let identity = selection::identity(metadata::map_field(fields, 6)?)?;
    namespace::validate(&identity, service_name, operation.name()?)?;
    let tags = tags::read(fields, service, operation, &items)?;
    super::validation::exclusions(service, operation, &items, metadata::list(fields, 9)?)?;
    let exclusions = map_exclusions::read(
        fields,
        service,
        operation,
        &items,
        exclusions(metadata::list(fields, 9)?)?,
    )?;
    let (projection, classification) = capabilities::read(fields, service, operation, &items)?;
    Ok(ResourceEmissionPlan {
        key: key.into_boxed_str(),
        kind,
        items,
        identity,
        tags,
        exclusions,
        projection,
        classification,
    })
}
