use super::{RuntimeCatalogError, metadata, tags};
use crate::{
    catalog::{Operation, Service, Target},
    record::{FieldId, MapRef, MemberExclusions, ValueRef},
};

pub(super) fn read(
    fields: MapRef<'_>,
    service: Service<'_>,
    operation: Operation<'_>,
) -> Result<MemberExclusions, RuntimeCatalogError> {
    let Some(value) = fields.get_field(FieldId::new(24)?)? else {
        return Ok(MemberExclusions::default());
    };
    let ValueRef::List(list) = value else {
        return Err(RuntimeCatalogError::Capability);
    };
    let members = metadata::bounded(
        list,
        |value| {
            let path = metadata::id_path(super::super::list_value(value)?)?;
            validate(service, operation.output()?, &path)?;
            path.last().copied().ok_or(RuntimeCatalogError::Capability)
        },
        8,
    )?;
    Ok(MemberExclusions::new(members)?)
}

fn validate(
    service: Service<'_>,
    output: Target,
    path: &[FieldId],
) -> Result<(), RuntimeCatalogError> {
    path.iter()
        .try_fold(output, |target, field| {
            let target = aggregate_element(service, target)?;
            Ok(tags::member(tags::shape(service, target)?, *field)?.target)
        })
        .map(|_| ())
}

fn aggregate_element(service: Service<'_>, target: Target) -> Result<Target, RuntimeCatalogError> {
    let target = (0..8).try_fold(target, |target, _| {
        match tags::target_kind(service, target)? {
            "list" => tags::shape(service, target)?
                .element()?
                .ok_or(RuntimeCatalogError::Capability),
            "map" => Err(RuntimeCatalogError::Capability),
            _ => Ok(target),
        }
    })?;
    if tags::target_kind(service, target)? == "list" {
        return Err(RuntimeCatalogError::Bounds);
    }
    Ok(target)
}
