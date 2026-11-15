use super::RuntimeCatalogCompileError;
use inventory_core::{
    catalog::{Operation, Service, Shape, Target},
    record::FieldId,
};

pub(super) fn resolve(
    service: Service<'_>,
    mut target: Target,
    path: &str,
) -> Result<(Vec<FieldId>, Target), RuntimeCatalogCompileError> {
    let segments = path.split('.').collect::<Vec<_>>();
    if segments.is_empty() || segments.len() > 8 || segments.iter().any(|part| part.is_empty()) {
        return Err(RuntimeCatalogCompileError::Capability);
    }
    let mut fields = Vec::with_capacity(segments.len());
    segments.iter().try_for_each(|segment| {
        let shape = shape_for(service, target)?;
        let member = member(shape, segment)?;
        fields.push(member.id);
        target = member.target;
        Ok::<_, RuntimeCatalogCompileError>(())
    })?;
    Ok((fields, target))
}
pub(super) fn terminal_member<'a>(
    service: Service<'a>,
    target: Target,
    path: &str,
) -> Result<(Vec<FieldId>, inventory_core::catalog::Member<'a>), RuntimeCatalogCompileError> {
    let segments = path.split('.').collect::<Vec<_>>();
    if segments.is_empty() || segments.len() > 8 {
        return Err(RuntimeCatalogCompileError::Capability);
    }
    terminal_member_segments(service, target, &segments)
}

fn terminal_member_segments<'a>(
    service: Service<'a>,
    target: Target,
    segments: &[&str],
) -> Result<(Vec<FieldId>, inventory_core::catalog::Member<'a>), RuntimeCatalogCompileError> {
    let (head, tail) = segments
        .split_first()
        .ok_or(RuntimeCatalogCompileError::Capability)?;
    let current = member(shape_for(service, target)?, head)?;
    if tail.is_empty() {
        return Ok((vec![current.id], current));
    }
    let (path, terminal) = terminal_member_segments(service, current.target, tail)?;
    Ok((std::iter::once(current.id).chain(path).collect(), terminal))
}

pub(super) fn item_target(
    service: Service<'_>,
    operation: Operation<'_>,
    selector_path: &str,
) -> Result<Target, RuntimeCatalogCompileError> {
    let (_, list) = resolve(service, operation.output()?, selector_path)?;
    list_element(service, list)
}
pub(super) struct ValidationContext<'p, 'a> {
    pub(super) kinds: &'p crate::kind_map::KindMap,
    pub(super) checked: &'p crate::overlay::CheckedOverlays,
    pub(super) services: &'p super::ServiceIndex<'a>,
}

pub(super) fn validate_reviewed_paths(
    context: &ValidationContext<'_, '_>,
) -> Result<(), RuntimeCatalogCompileError> {
    validate_display_paths(context)?;
    validate_relations(context)?;
    validate_bootstrap_paths(context)
}

fn validate_display_paths(
    context: &ValidationContext<'_, '_>,
) -> Result<(), RuntimeCatalogCompileError> {
    context.kinds.entries().iter().try_for_each(|entry| {
        match (&entry.target, entry.display_name_path.as_deref()) {
            (crate::kind_map::KindTarget::Mapped { kind, .. }, Some(path)) => {
                validate_kind_path(context, kind, path, true)
            }
            _ => Ok(()),
        }
    })
}

fn validate_relations(
    context: &ValidationContext<'_, '_>,
) -> Result<(), RuntimeCatalogCompileError> {
    context.kinds.relations().iter().try_for_each(|relation| {
        validate_kind_path(context, &relation.source_kind, &relation.source_path, true)?;
        validate_kind_path(context, &relation.target_kind, &relation.target_path, true)
    })
}

fn validate_bootstrap_paths(
    context: &ValidationContext<'_, '_>,
) -> Result<(), RuntimeCatalogCompileError> {
    context
        .kinds
        .operations()
        .iter()
        .try_for_each(|row| validate_bootstrap(context, &row.service, &row.operation))
}

fn validate_bootstrap(
    context: &ValidationContext<'_, '_>,
    service_name: &str,
    operation_name: &str,
) -> Result<(), RuntimeCatalogCompileError> {
    match context.kinds.bootstrap_for(service_name, operation_name) {
        Some(crate::kind_map::BootstrapPurpose::AccountIdentity { account_path }) => {
            let (service, operation) = operation_for(context, service_name, operation_name)?;
            string_target_for_path(service, operation.output()?, account_path)
        }
        Some(crate::kind_map::BootstrapPurpose::RegionDiscovery {
            regions_path,
            region_name_path,
            opt_in_status_path,
        }) => validate_region_paths(
            context,
            service_name,
            operation_name,
            regions_path,
            region_name_path,
            opt_in_status_path.as_deref(),
        ),
        Some(crate::kind_map::BootstrapPurpose::Enumeration) | None => Ok(()),
    }
}

fn validate_region_paths(
    context: &ValidationContext<'_, '_>,
    service_name: &str,
    operation_name: &str,
    regions_path: &str,
    name_path: &str,
    opt_in_path: Option<&str>,
) -> Result<(), RuntimeCatalogCompileError> {
    let (service, operation) = operation_for(context, service_name, operation_name)?;
    let item = list_element(
        service,
        resolve(service, operation.output()?, regions_path)?.1,
    )?;
    string_target_for_path(service, item, name_path)?;
    opt_in_path.map_or(Ok(()), |path| string_target_for_path(service, item, path))
}

fn validate_kind_path(
    context: &ValidationContext<'_, '_>,
    kind: &str,
    path: &str,
    require_string: bool,
) -> Result<(), RuntimeCatalogCompileError> {
    let row = context
        .kinds
        .operations()
        .iter()
        .find(|row| row.kind == kind)
        .ok_or(RuntimeCatalogCompileError::Capability)?;
    let (service, operation) = operation_for(context, &row.service, &row.operation)?;
    let selector = operation_for_checked(context, &row.service, &row.operation)?
        .plan()
        .items
        .as_ref();
    let target = item_target_for_selector(service, operation, selector)?;
    let terminal = resolve(service, target, path)?.1;
    if require_string {
        string_target(service, terminal)
    } else {
        Ok(())
    }
}

fn item_target_for_selector(
    service: Service<'_>,
    operation: Operation<'_>,
    selector: Option<&crate::overlay::ItemSelector>,
) -> Result<Target, RuntimeCatalogCompileError> {
    match selector {
        Some(crate::overlay::ItemSelector::Root) => Ok(operation.output()?),
        Some(crate::overlay::ItemSelector::Member(items)) => {
            item_target(service, operation, &items.to_string())
        }
        None => Err(RuntimeCatalogCompileError::Capability),
    }
}

fn string_target_for_path(
    service: Service<'_>,
    target: Target,
    path: &str,
) -> Result<(), RuntimeCatalogCompileError> {
    string_target(service, resolve(service, target, path)?.1)
}

fn operation_for<'a>(
    context: &ValidationContext<'_, 'a>,
    service: &str,
    operation: &str,
) -> Result<(Service<'a>, Operation<'a>), RuntimeCatalogCompileError> {
    let service_view = context.services.get(service)?;
    let checked_operation = operation_for_checked(context, service, operation)?;
    Ok((
        service_view,
        super::plans::catalog_operation(service_view, checked_operation.operation())?,
    ))
}

fn operation_for_checked<'p, 'a>(
    context: &ValidationContext<'p, 'a>,
    service: &str,
    operation: &str,
) -> Result<&'p crate::overlay::CallableOperation, RuntimeCatalogCompileError> {
    context
        .checked
        .services()
        .iter()
        .find(|review| review.service() == service)
        .and_then(|review| {
            review
                .callable()
                .iter()
                .find(|candidate| candidate.operation() == operation)
        })
        .ok_or(RuntimeCatalogCompileError::Capability)
}
pub(super) fn list_element(
    service: Service<'_>,
    target: Target,
) -> Result<Target, RuntimeCatalogCompileError> {
    let list = shape_for(service, target)?;
    if list.kind()? != "list" {
        return Err(RuntimeCatalogCompileError::Capability);
    }
    list.element()?
        .ok_or(RuntimeCatalogCompileError::Capability)
}

pub(super) fn kind(
    service: Service<'_>,
    target: Target,
) -> Result<&'static str, RuntimeCatalogCompileError> {
    match target {
        Target::Prelude(kind) => Ok(kind),
        Target::Shape(_) => Ok(shape_for(service, target)?.kind()?),
        _ => Err(RuntimeCatalogCompileError::Capability),
    }
}

pub(super) fn string_target(
    service: Service<'_>,
    target: Target,
) -> Result<(), RuntimeCatalogCompileError> {
    let kind = match target {
        Target::Prelude(kind) => kind,
        Target::Shape(_) => kind(service, target)?,
        _ => return Err(RuntimeCatalogCompileError::Capability),
    };
    if kind == "string" {
        Ok(())
    } else {
        Err(RuntimeCatalogCompileError::Capability)
    }
}

fn shape_for<'a>(
    service: Service<'a>,
    target: Target,
) -> Result<Shape<'a>, RuntimeCatalogCompileError> {
    match target {
        Target::Shape(index) => service
            .shape(index)?
            .ok_or(RuntimeCatalogCompileError::Capability),
        _ => Err(RuntimeCatalogCompileError::Capability),
    }
}

fn member<'a>(
    shape: Shape<'a>,
    name: &str,
) -> Result<inventory_core::catalog::Member<'a>, RuntimeCatalogCompileError> {
    shape
        .members()?
        .try_fold(None, |found, candidate| {
            let candidate = candidate?;
            if candidate.name()? != name {
                return Ok(found);
            }
            if found.is_some() {
                return Err(inventory_core::catalog::CatalogError::Reference);
            }
            Ok(Some(candidate))
        })?
        .ok_or(RuntimeCatalogCompileError::Capability)
}
