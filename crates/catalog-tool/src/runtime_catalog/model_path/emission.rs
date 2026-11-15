use super::{RuntimeCatalogCompileError, ValidationContext, operation_for, optional_resolve, string_target};
use crate::{kind_map::RelationSource, overlay::ResourceProjection, runtime_catalog::emission_sources};

pub(super) fn active_kind(context: &ValidationContext<'_, '_>, kind: &str) -> bool {
    emission_sources::sources(context.kinds, context.checked).any(|source| source.emits_kind(kind))
}

pub(super) fn validate_kind_origin_path(context: &ValidationContext<'_, '_>, kind: &str,
    path: &str, require_string: bool, origin: RelationSource,
) -> Result<(), RuntimeCatalogCompileError> {
    let found = emission_sources::sources(context.kinds, context.checked)
        .filter(|source| source.emits_kind(kind))
        .try_fold(false, |found, source| {
            let (service, operation) = operation_for(context, source.service, source.operation)?;
            let target = match origin {
                RelationSource::SelectedOutput => {
                    if matches!(source.projection, Some(ResourceProjection::IamTrustSummary(protected)) if protected.as_str() == path) {
                        return Err(RuntimeCatalogCompileError::Capability);
                    }
                    super::item_target_for_selector(service, operation, source.items)?
                }
                RelationSource::Request => operation.input()?,
            };
            let Some(terminal) = optional_resolve(service, target, path)? else { return Ok(found); };
            if require_string { string_target(service, terminal)?; }
            Ok(true)
        })?;
    if found { Ok(()) } else { Err(RuntimeCatalogCompileError::Capability) }
}
