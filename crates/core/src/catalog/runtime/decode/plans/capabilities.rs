use super::{RuntimeCatalogError, metadata, tags};
use crate::{catalog::{Operation, Service}, catalog::runtime::{ItemPlan, KindClassificationPlan, ResourceProjectionPlan}, inventory::Kind, record::{FieldId, MapRef, ValueRef}};

pub(super) fn read(fields: MapRef<'_>, service: Service<'_>, operation: Operation<'_>, items: &ItemPlan) -> Result<(Option<ResourceProjectionPlan>, Option<KindClassificationPlan>), RuntimeCatalogError> {
    let item = tags::selected_item(service, operation, items)?;
    let projection = fields.get_field(FieldId::new(17)?)?.map(|value| {
        let fields = super::super::map(value)?;
        let path = metadata::id_path(metadata::list(fields, 2)?)?;
        match metadata::integer(fields, 1)? {
            0 | 1 => {
                metadata::only_fields(fields, &[1, 2])?;
                string_path(service, item, &path)?;
                if metadata::integer(fields, 1)? == 0 {
                    Ok(ResourceProjectionPlan::IamTrustSummary { path })
                } else {
                    Ok(ResourceProjectionPlan::SensitiveDigest { path })
                }
            }
            2 => {
                metadata::only_fields(fields, &[1, 2, 3])?;
                let target = tags::resolve(service, item, &path)?;
                let shape = tags::shape(service, target)?;
                if shape.kind()? != "map" { return Err(RuntimeCatalogError::Capability); }
                let value = shape.map_value()?.ok_or(RuntimeCatalogError::Capability)?;
                if !matches!(tags::target_kind(service, value)?, "string" | "enum") {
                    return Err(RuntimeCatalogError::Capability);
                }
                let key = metadata::text(fields, 3)?;
                if key.is_empty() || key.len() > 256 { return Err(RuntimeCatalogError::Capability); }
                Ok(ResourceProjectionPlan::SensitiveMapDigest { path, key })
            }
            _ => Err(RuntimeCatalogError::Capability),
        }
    }).transpose()?;
    let classification = fields.get_field(FieldId::new(18)?)?.map(|value| {
        let fields = super::super::map(value)?;
        metadata::only_fields(fields, &[1, 2])?;
        let path = metadata::id_path(metadata::list(fields, 1)?)?;
        string_path(service, item, &path)?;
        let variants = metadata::bounded(metadata::list(fields, 2)?, variant, 64)?;
        if variants.is_empty() || variants.iter().enumerate().any(|(index, (value, _))| variants.iter().take(index).any(|(previous, _)| previous == value)) {
            return Err(RuntimeCatalogError::Capability);
        }
        Ok(KindClassificationPlan { path, variants: variants.into_boxed_slice() })
    }).transpose()?;
    Ok((projection, classification))
}

fn string_path(service: Service<'_>, item: crate::catalog::Target, path: &[FieldId]) -> Result<(), RuntimeCatalogError> {
    if !matches!(tags::target_kind(service, tags::resolve(service, item, path)?)?, "string" | "enum") {
        return Err(RuntimeCatalogError::Capability);
    }
    Ok(())
}
fn variant(value: ValueRef<'_>) -> Result<(String, Kind), RuntimeCatalogError> {
    let fields = super::super::map(value)?;
    metadata::only_fields(fields, &[1, 2])?;
    let value = metadata::text(fields, 1)?;
    if value.is_empty() || value.len() > 256 || !value.is_ascii() { return Err(RuntimeCatalogError::Capability); }
    Ok((value, Kind::parse(&metadata::text(fields, 2)?)?))
}
