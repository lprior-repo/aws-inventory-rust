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
            (crate::kind_map::KindTarget::Mapped { kind, .. }, Some(path)) if active_kind(context, kind) => {
                validate_kind_path(context, kind, path, true)
            }
            _ => Ok(()),
        }
    })
}

fn validate_relations(
    context: &ValidationContext<'_, '_>,
) -> Result<(), RuntimeCatalogCompileError> {
    use crate::kind_map::ReviewedRelationTarget;
    context.kinds.relations().iter()
        .filter(|relation| active_kind(context, &relation.source_kind)
            && relation.target_kinds.iter().all(|kind| active_kind(context, kind)))
        .try_for_each(|relation| {
            validate_relation_source(context, &relation.source_kind, &relation.source, true)?;
            if let Some(guard) = &relation.guard {
                validate_relation_source(context, &relation.source_kind, &guard.source, false)?;
            }
            relation.target_kinds.iter().try_for_each(|kind| match &relation.target {
                ReviewedRelationTarget::SelectedOutput(path) => validate_relation_value(context, kind, path, false),
                ReviewedRelationTarget::BoundContext(path) => validate_relation_value(context, kind, path, true),
                ReviewedRelationTarget::CanonicalId => Ok(()),
            })
        })?;
    for relation in context.kinds.compound_relations().iter().filter(|relation| active_kind(context, &relation.source_kind)
        && relation.target_kinds.iter().all(|kind| active_kind(context, kind))) {
        for component in &relation.components {
            validate_relation_source(context, &relation.source_kind, &component.source, false)?;
            for kind in relation.target_kinds.iter() {
                match &component.target {
                    ReviewedRelationTarget::SelectedOutput(path) => validate_relation_value(context, kind, path, false)?,
                    ReviewedRelationTarget::BoundContext(path) => validate_relation_value(context, kind, path, true)?,
                    ReviewedRelationTarget::CanonicalId => {}
                }
            }
        }
    }
    Ok(())
}

fn validate_bootstrap_paths(
    context: &ValidationContext<'_, '_>,
) -> Result<(), RuntimeCatalogCompileError> {
    context
        .kinds
        .operations()
        .iter()
        .filter(|row| active_operation(context, &row.service, &row.operation))
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
    validate_kind_origin_path(context, kind, path, require_string, false)
}

fn validate_kind_origin_path(
    context: &ValidationContext<'_, '_>,
    kind: &str,
    path: &str,
    require_string: bool,
    request: bool,
) -> Result<(), RuntimeCatalogCompileError> {
    let found = context.kinds.operations().iter()
        .filter(|row| operation_has_kind(context, row, kind) && active_operation(context, &row.service, &row.operation))
        .try_fold(false, |found, row| {
            validate_operation_path(context, row, path, require_string, request).map(|present| found || present)
        })?;
    if found { Ok(()) } else { Err(RuntimeCatalogCompileError::Capability) }
}

fn validate_operation_path(
    context: &ValidationContext<'_, '_>,
    row: &crate::kind_map::RuntimeOperation,
    path: &str,
    require_string: bool,
    request: bool,
) -> Result<bool, RuntimeCatalogCompileError> {
    let (service, operation) = operation_for(context, &row.service, &row.operation)?;
    let target = match request {
        false => {
            let selector = operation_for_checked(context, &row.service, &row.operation)?
                .plan()
                .items
                .as_ref();
            item_target_for_selector(service, operation, selector)?
        }
        true => operation.input()?,
    };
    let terminal = if path.is_empty() { target } else {
        let Some(terminal) = optional_resolve(service, target, path)? else { return Ok(false) };
        terminal
    };
    if require_string {
        string_target(service, terminal)?;
    }
    Ok(true)
}

fn validate_relation_source(context: &ValidationContext<'_, '_>, kind: &str,
    source: &crate::kind_map::ReviewedRelationSource, allow_list: bool,
) -> Result<(), RuntimeCatalogCompileError> {
    use crate::kind_map::{ReviewedRelationSource, ReviewedValuePath};
    let (path, request) = match source {
        ReviewedRelationSource::SelectedOutput(path) => (path, false),
        ReviewedRelationSource::Request(path) => (path, true),
    };
    let path = match path { ReviewedValuePath::Fields(path) => path.as_str(), ReviewedValuePath::RootText => "" };
    let found = context.kinds.operations().iter()
        .filter(|row| operation_has_kind(context, row, kind) && active_operation(context, &row.service, &row.operation))
        .try_fold(false, |found, row| {
            let (service, operation) = operation_for(context, &row.service, &row.operation)?;
            let target = if request { operation.input()? } else {
                item_target_for_selector(service, operation, operation_for_checked(context, &row.service, &row.operation)?.plan().items.as_ref())?
            };
            let terminal = if path.is_empty() { target } else {
                let Some(terminal) = optional_resolve(service, target, path)? else { return Ok(found); };
                terminal
            };
            let terminal = if allow_list && self::kind(service, terminal)? == "list" { list_element(service, terminal)? } else { terminal };
            string_target(service, terminal)?;
            Ok::<_, RuntimeCatalogCompileError>(true)
        })?;
    if found { Ok(()) } else { Err(RuntimeCatalogCompileError::Capability) }
}

fn validate_relation_value(context: &ValidationContext<'_, '_>, kind: &str,
    path: &crate::kind_map::ReviewedValuePath, request: bool,
) -> Result<(), RuntimeCatalogCompileError> {
    use crate::kind_map::ReviewedValuePath;
    let path = match path { ReviewedValuePath::Fields(path) => path.as_str(), ReviewedValuePath::RootText => "" };
    validate_kind_origin_path(context, kind, path, true, request)
}

fn item_target_for_selector(
    service: Service<'_>,
    operation: Operation<'_>,
    selector: Option<&crate::overlay::ItemSelector>,
) -> Result<Target, RuntimeCatalogCompileError> {
    super::selection::target(service, operation.output()?, selector)
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

pub(super) fn active_operation(context: &ValidationContext<'_, '_>, service: &str, operation: &str) -> bool {
    operation_for_checked(context, service, operation).is_ok_and(|call| {
        let plan = call.plan();
        plan.emission == crate::overlay::Emission::EmitMetadata && plan.tag_endpoint.is_none() && !plan.endpoint_discovery
    })
}

pub(super) fn active_kind(context: &ValidationContext<'_, '_>, kind: &str) -> bool {
    context.kinds.operations().iter().any(|row| operation_has_kind(context, row, kind)
        && active_operation(context, &row.service, &row.operation))
}

fn operation_has_kind(context: &ValidationContext<'_, '_>, row: &crate::kind_map::RuntimeOperation, kind: &str) -> bool {
    row.kind == kind || operation_for_checked(context, &row.service, &row.operation).is_ok_and(|call| {
        call.plan().classification.as_ref().is_some_and(|classification| {
            classification.variants.values().any(|candidate| candidate == kind)
        })
    })
}

pub(super) fn optional_resolve(service: Service<'_>, target: Target, path: &str) -> Result<Option<Target>, RuntimeCatalogCompileError> {
    path.split('.').try_fold(Some(target), |target, segment| {
        let Some(target) = target else { return Ok(None) };
        match kind(service, target)? {
            "string" | "enum" | "intEnum" | "boolean" | "byte" | "short" | "integer"
            | "long" | "float" | "double" | "bigInteger" | "bigDecimal" | "timestamp"
            | "blob" => return Ok(None),
            "structure" | "union" => {}
            _ => return Err(RuntimeCatalogCompileError::Capability),
        }
        let shape = shape_for(service, target)?;
        shape.members()?.try_fold(None, |found, candidate| {
            let candidate = candidate?;
            if candidate.name()? == segment { Ok(Some(candidate.target)) } else { Ok(found) }
        }).map_err(RuntimeCatalogCompileError::Catalog)
    })
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
    if matches!(kind, "string" | "enum") {
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
