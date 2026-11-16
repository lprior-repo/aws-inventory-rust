use super::{RuntimeCatalogError, integer, list, map, only_fields, selected_path, text};
use crate::{catalog::runtime::{RelationGuard, RelationPathMode, RelationSource, RelationTargetMode, RelationshipPlan}, record::{FieldId, MapRef, ValueRef}};

pub(super) fn source_mode(fields: MapRef<'_>) -> Result<RelationPathMode, RuntimeCatalogError> {
    match fields.get_field(FieldId::new(11)?)? {
        None | Some(ValueRef::Int(0)) => Ok(RelationPathMode::Fields),
        Some(ValueRef::Int(1)) => Ok(RelationPathMode::RootText),
        _ => Err(RuntimeCatalogError::Capability),
    }
}

pub(super) fn target_mode(fields: MapRef<'_>) -> Result<RelationTargetMode, RuntimeCatalogError> {
    match fields.get_field(FieldId::new(12)?)? {
        None => Ok(if selected_path(list(fields, 4)?)?.is_empty() { RelationTargetMode::SelectedRootText } else { RelationTargetMode::SelectedFields }),
        Some(ValueRef::Int(0)) => Ok(RelationTargetMode::SelectedFields),
        Some(ValueRef::Int(1)) => Ok(RelationTargetMode::SelectedRootText),
        Some(ValueRef::Int(2)) => Ok(RelationTargetMode::BoundContextFields),
        Some(ValueRef::Int(3)) => Ok(RelationTargetMode::BoundContextRootText),
        Some(ValueRef::Int(4)) => Ok(RelationTargetMode::CanonicalId),
        _ => Err(RuntimeCatalogError::Capability),
    }
}

pub(super) fn guard(fields: MapRef<'_>) -> Result<Option<RelationGuard>, RuntimeCatalogError> {
    let Some(value) = fields.get_field(FieldId::new(13)?)? else { return Ok(None); };
    let fields = map(value)?;
    only_fields(fields, &[1, 2, 3, 4])?;
    let source = match integer(fields, 1)? { 0 => RelationSource::SelectedOutput, 1 => RelationSource::Request, _ => return Err(RuntimeCatalogError::Capability) };
    let path_mode = match integer(fields, 2)? { 0 => RelationPathMode::Fields, 1 => RelationPathMode::RootText, _ => return Err(RuntimeCatalogError::Capability) };
    let path = selected_path(list(fields, 3)?)?;
    if path.is_empty() != (path_mode == RelationPathMode::RootText) { return Err(RuntimeCatalogError::Capability); }
    let equals = text(fields, 4)?;
    if equals.is_empty() { return Err(RuntimeCatalogError::Capability); }
    Ok(Some(RelationGuard { source, path_mode, path, equals }))
}

pub(super) fn validate(relation: &RelationshipPlan) -> Result<(), RuntimeCatalogError> {
    if relation.source_path.is_empty() != (relation.source_path_mode == RelationPathMode::RootText)
        || relation.target_path.is_empty() != matches!(relation.target_mode, RelationTargetMode::SelectedRootText | RelationTargetMode::BoundContextRootText | RelationTargetMode::CanonicalId) {
        return Err(RuntimeCatalogError::Capability);
    }
    if relation.source == RelationSource::Parent && (relation.target_mode != RelationTargetMode::CanonicalId
        || relation.source_path_mode != RelationPathMode::RootText || !relation.source_path.is_empty()
        || relation.guard.is_some() || !relation.components.is_empty()) {
        return Err(RuntimeCatalogError::Capability);
    }
    Ok(())
}

pub(super) fn components(fields: MapRef<'_>) -> Result<Box<[crate::catalog::runtime::RelationshipComponentPlan]>, RuntimeCatalogError> {
    let Some(value) = fields.get_field(FieldId::new(14)?)? else { return Ok(Box::new([])); };
    let ValueRef::List(values) = value else { return Err(RuntimeCatalogError::Capability); };
    super::bounded(values, component, 7).map(Vec::into_boxed_slice)
}

fn component(value: ValueRef<'_>) -> Result<crate::catalog::runtime::RelationshipComponentPlan, RuntimeCatalogError> {
    use crate::catalog::runtime::RelationshipComponentPlan;
    let fields = map(value)?;
    only_fields(fields, &[1, 2, 3, 4, 5])?;
    let source = match integer(fields, 1)? { 0 => RelationSource::SelectedOutput, 1 => RelationSource::Request, _ => return Err(RuntimeCatalogError::Capability) };
    let source_path = selected_path(list(fields, 2)?)?;
    let source_path_mode = match integer(fields, 3)? { 0 => RelationPathMode::Fields, 1 => RelationPathMode::RootText, _ => return Err(RuntimeCatalogError::Capability) };
    let target_path = selected_path(list(fields, 4)?)?;
    let target_mode = match integer(fields, 5)? {
        0 => RelationTargetMode::SelectedFields, 1 => RelationTargetMode::SelectedRootText,
        2 => RelationTargetMode::BoundContextFields, 3 => RelationTargetMode::BoundContextRootText,
        4 => RelationTargetMode::CanonicalId, _ => return Err(RuntimeCatalogError::Capability),
    };
    if source_path.is_empty() != (source_path_mode == RelationPathMode::RootText)
        || target_path.is_empty() != matches!(target_mode, RelationTargetMode::SelectedRootText | RelationTargetMode::BoundContextRootText | RelationTargetMode::CanonicalId) {
        return Err(RuntimeCatalogError::Capability);
    }
    Ok(RelationshipComponentPlan { source, source_path, source_path_mode, target_mode, target_path })
}
