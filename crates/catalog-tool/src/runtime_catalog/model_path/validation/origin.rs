use super::{ValidationContext, item_target_for_selector, operation_for};
use crate::overlay::ResourceProjection;
use crate::runtime_catalog::emission_sources::ResourceSource;
use crate::runtime_catalog::model_path::{
    self,
    emission::{Origin, Terminal},
};
use crate::runtime_catalog::{RuntimeCatalogCompileError, emission_sources};
use inventory_core::catalog::{Service, Target};

pub(super) fn validate(
    context: &ValidationContext<'_, '_>,
    kind: &str,
    path: &str,
    origin: Origin,
    terminal: Terminal,
) -> Result<(), RuntimeCatalogCompileError> {
    let found = emission_sources::sources(context.kinds, context.checked)
        .filter(|source| source.emits_kind(kind))
        .try_fold(false, |found, source| {
            validate_source(context, &source, path, origin, terminal)
                .map(|resolved| found || resolved)
                .map_err(|error| contextual_error(kind, path, error))
        })?;
    if found {
        Ok(())
    } else {
        missing_source(context, kind, path)
    }
}

fn validate_source(
    context: &ValidationContext<'_, '_>,
    source: &ResourceSource<'_>,
    path: &str,
    origin: Origin,
    terminal: Terminal,
) -> Result<bool, RuntimeCatalogCompileError> {
    let (service, operation) = operation_for(context, source.service, source.operation)?;
    let target = match origin {
        Origin::SelectedOutput => {
            check_projection(source, path)?;
            item_target_for_selector(service, operation, source.items)?
        }
        Origin::Request => operation.input()?,
    };
    let resolved = if path.is_empty() {
        Some(target)
    } else {
        model_path::optional_resolve(service, target, path)?
    };
    match resolved {
        Some(target) => validate_terminal(service, target, terminal).map(|()| true),
        None => Ok(false),
    }
}

fn check_projection(
    source: &ResourceSource<'_>,
    path: &str,
) -> Result<(), RuntimeCatalogCompileError> {
    if matches!(source.projection, Some(ResourceProjection::IamTrustSummary(protected)) if protected.as_str() == path)
    {
        return Err(RuntimeCatalogCompileError::KindContext {
            kind: source.kind.to_owned(),
            path: path.to_owned(),
            reason: "path is a protected projection",
        });
    }
    Ok(())
}

fn validate_terminal(
    service: Service<'_>,
    target: Target,
    terminal: Terminal,
) -> Result<(), RuntimeCatalogCompileError> {
    let target = match terminal {
        Terminal::TextOrList if model_path::kind(service, target)? == "list" => {
            model_path::list_element(service, target)?
        }
        Terminal::Text | Terminal::TextOrList => target,
    };
    model_path::string_target(service, target)
}

fn contextual_error(
    kind: &str,
    path: &str,
    error: RuntimeCatalogCompileError,
) -> RuntimeCatalogCompileError {
    match error {
        RuntimeCatalogCompileError::Capability | RuntimeCatalogCompileError::Path { .. } => {
            RuntimeCatalogCompileError::KindContext {
                kind: kind.to_owned(),
                path: path.to_owned(),
                reason: "path does not end in a string",
            }
        }
        error => error,
    }
}

fn missing_source(
    context: &ValidationContext<'_, '_>,
    kind: &str,
    path: &str,
) -> Result<(), RuntimeCatalogCompileError> {
    let row = context
        .kinds
        .operations()
        .iter()
        .find(|row| row.kind == kind);
    match row {
        Some(row) => Err(RuntimeCatalogCompileError::CapabilityContext {
            service: row.service.to_owned(),
            operation: row.operation.to_owned(),
            path: path.to_owned(),
        }),
        None => Err(RuntimeCatalogCompileError::KindContext {
            kind: kind.to_owned(),
            path: path.to_owned(),
            reason: "no operation row produces this kind",
        }),
    }
}
