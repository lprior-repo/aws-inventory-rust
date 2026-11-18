use super::{
    RuntimeCatalogCompileError, ValidationContext, list_element, operation_for, optional_resolve,
    string_target,
};
use crate::{overlay::ResourceProjection, runtime_catalog::emission_sources};

/// Which side of the call a relation path is read from.
#[derive(Clone, Copy)]
pub(super) enum Origin {
    SelectedOutput,
    Request,
}

/// What the resolved terminal must be.
#[derive(Clone, Copy)]
pub(super) enum Terminal {
    Text,
    TextOrList,
}

/// A kind is active when any callable emission, primary or additional, produces it.
pub(in crate::runtime_catalog) fn active_kind(
    context: &ValidationContext<'_, '_>,
    kind: &str,
) -> bool {
    emission_sources::sources(context.kinds, context.checked).any(|source| source.emits_kind(kind))
}

/// Requires `path` to resolve to `terminal` on at least one emission of `kind`.
pub(super) fn validate_kind_origin_path(
    context: &ValidationContext<'_, '_>,
    kind: &str,
    path: &str,
    origin: Origin,
    terminal: Terminal,
) -> Result<(), RuntimeCatalogCompileError> {
    let found = emission_sources::sources(context.kinds, context.checked)
        .filter(|source| source.emits_kind(kind))
        .try_fold(false, |found, source| {
            let (service, operation) = operation_for(context, source.service, source.operation)?;
            let target = match origin {
                Origin::SelectedOutput => {
                    if matches!(source.projection, Some(ResourceProjection::IamTrustSummary(protected)) if protected.as_str() == path) {
                        return Err(RuntimeCatalogCompileError::Capability);
                    }
                    super::item_target_for_selector(service, operation, source.items)?
                }
                Origin::Request => operation.input()?,
            };
            let resolved = if path.is_empty() { target } else {
                let Some(resolved) = optional_resolve(service, target, path)? else { return Ok(found); };
                resolved
            };
            let resolved = match terminal {
                Terminal::TextOrList if super::kind(service, resolved)? == "list" => list_element(service, resolved)?,
                Terminal::Text | Terminal::TextOrList => resolved,
            };
            string_target(service, resolved)?;
            Ok(true)
        })?;
    if found {
        Ok(())
    } else {
        Err(RuntimeCatalogCompileError::Capability)
    }
}
