use super::super::{PlanParts, RuntimeCatalogError, tags};
use crate::catalog::runtime::{
    IdentityChoicePlan, IdentityFieldPlan, IdentityLeafPlan, IdentityPart, IdentityPlan,
    IdentityPredicatePlan, ItemPlan, SelectionKind,
};
use crate::catalog::{Operation, Service, Target};
use crate::record::FieldId;

pub(in crate::catalog::runtime::decode::plans) fn validate_item(
    service: Service<'_>,
    operation: Operation<'_>,
    items: &ItemPlan,
) -> Result<Target, RuntimeCatalogError> {
    let (path, kind) = match items {
        ItemPlan::Root => return Ok(operation.output()?),
        ItemPlan::Path(path) => {
            let target = tags::resolve(service, operation.output()?, path)?;
            return match tags::target_kind(service, target)? {
                "structure" => Ok(target),
                "list" => tags::shape(service, target)?
                    .element()?
                    .ok_or(RuntimeCatalogError::Capability),
                _ => Err(RuntimeCatalogError::Capability),
            };
        }
        ItemPlan::Select { path, kind } => (path, kind),
    };
    let target = tags::resolve(service, operation.output()?, path)?;
    let shape_kind = tags::target_kind(service, target)?;
    match kind {
        SelectionKind::Structure if shape_kind == "structure" => Ok(target),
        SelectionKind::Scalar => {
            scalar(service, target)?;
            Ok(target)
        }
        SelectionKind::List if shape_kind == "list" => tags::shape(service, target)?
            .element()?
            .ok_or(RuntimeCatalogError::Capability),
        SelectionKind::MapKeys if shape_kind == "map" => tags::shape(service, target)?
            .map_key()?
            .ok_or(RuntimeCatalogError::Capability),
        SelectionKind::MapEntries | SelectionKind::MapValues if shape_kind == "map" => {
            tags::shape(service, target)?
                .map_value()?
                .ok_or(RuntimeCatalogError::Capability)
        }
        _ => Err(RuntimeCatalogError::Capability),
    }
}

pub(in crate::catalog::runtime::decode::plans) fn validate_identity(
    parts: &PlanParts<'_>,
) -> Result<(), RuntimeCatalogError> {
    let item = validate_item(parts.service, parts.operation, &parts.items)?;
    match &parts.identity {
        IdentityPlan::Choice(IdentityChoicePlan::PreferArn { path, fallback, .. }) => {
            text(parts.service, selected(parts.service, item, path)?)?;
            leaf(parts, item, fallback)
        }
        IdentityPlan::Choice(IdentityChoicePlan::Discriminated(branches)) => {
            branches.iter().try_for_each(|branch| {
                match &branch.predicate {
                    IdentityPredicatePlan::Present(path) => {
                        scalar(parts.service, selected(parts.service, item, path)?)?
                    }
                    IdentityPredicatePlan::TextEquals { path, .. } => {
                        text(parts.service, selected(parts.service, item, path)?)?
                    }
                    IdentityPredicatePlan::RequestTextEquals { path, .. } => text(
                        parts.service,
                        selected(parts.service, parts.operation.input()?, path)?,
                    )?,
                }
                leaf(parts, item, &branch.identity)
            })
        }
        IdentityPlan::CorrelatedKey(fields) => fields
            .iter()
            .try_for_each(|field| identity_field(parts, item, field)),
        IdentityPlan::MapArn {
            path,
            resource_kind,
            request_url,
            ..
        } => {
            map_text(parts.service, item, path)?;
            if matches!(
                resource_kind,
                crate::catalog::runtime::ArnResourceKind::SqsQueue
            ) != request_url.is_some()
            {
                return Err(RuntimeCatalogError::Capability);
            }
            request_url.as_ref().map_or(Ok(()), |path| {
                text(
                    parts.service,
                    selected(parts.service, parts.operation.input()?, path)?,
                )
            })
        }
        _ => Err(RuntimeCatalogError::Capability),
    }
}

fn leaf(
    parts: &PlanParts<'_>,
    item: Target,
    leaf: &IdentityLeafPlan,
) -> Result<(), RuntimeCatalogError> {
    match leaf {
        IdentityLeafPlan::Arn(path) => text(parts.service, selected(parts.service, item, path)?),
        IdentityLeafPlan::Key(paths) => paths
            .iter()
            .try_for_each(|path| scalar(parts.service, selected(parts.service, item, path)?)),
        IdentityLeafPlan::Template(segments) => segments.iter().try_for_each(|part| match part {
            IdentityPart::Field(path) => {
                scalar(parts.service, selected(parts.service, item, path)?)
            }
            _ => Ok(()),
        }),
        IdentityLeafPlan::Fields(fields) => fields
            .iter()
            .try_for_each(|field| identity_field(parts, item, field)),
    }
}

fn identity_field(
    parts: &PlanParts<'_>,
    item: Target,
    field: &IdentityFieldPlan,
) -> Result<(), RuntimeCatalogError> {
    match field {
        IdentityFieldPlan::Selected(path) => {
            scalar(parts.service, selected(parts.service, item, path)?)
        }
        IdentityFieldPlan::SelectedDigest(path) => {
            digest_target(parts.service, selected(parts.service, item, path)?)
        }
        IdentityFieldPlan::Request(path) => scalar(
            parts.service,
            selected(parts.service, parts.operation.input()?, path)?,
        ),
        IdentityFieldPlan::SelectedArn(path) => {
            text(parts.service, selected(parts.service, item, path)?)
        }
        IdentityFieldPlan::RequestArn(path) => text(
            parts.service,
            selected(parts.service, parts.operation.input()?, path)?,
        ),
        IdentityFieldPlan::SelectedInteger {
            path,
            minimum,
            maximum,
        } => integral(
            parts.service,
            selected(parts.service, item, path)?,
            *minimum,
            *maximum,
        ),
        IdentityFieldPlan::RequestInteger {
            path,
            minimum,
            maximum,
        } => integral(
            parts.service,
            selected(parts.service, parts.operation.input()?, path)?,
            *minimum,
            *maximum,
        ),
        IdentityFieldPlan::SelectedStringList {
            path,
            max_items,
            max_item_bytes,
        } => {
            if !(1..=64).contains(max_items) || !(1..=4096).contains(max_item_bytes) {
                return Err(RuntimeCatalogError::Bounds);
            }
            let shape = tags::shape(parts.service, selected(parts.service, item, path)?)?;
            if shape.kind()? != "list" {
                return Err(RuntimeCatalogError::Capability);
            }
            text(
                parts.service,
                shape.element()?.ok_or(RuntimeCatalogError::Capability)?,
            )
        }
        IdentityFieldPlan::MapValue { path, .. } => map_text(parts.service, item, path),
        IdentityFieldPlan::Context { .. } => Ok(()),
    }
}

fn map_text(
    service: Service<'_>,
    item: Target,
    path: &[FieldId],
) -> Result<(), RuntimeCatalogError> {
    let shape = tags::shape(service, selected(service, item, path)?)?;
    if shape.kind()? != "map" {
        return Err(RuntimeCatalogError::Capability);
    }
    text(
        service,
        shape.map_key()?.ok_or(RuntimeCatalogError::Capability)?,
    )?;
    text(
        service,
        shape.map_value()?.ok_or(RuntimeCatalogError::Capability)?,
    )
}
fn selected(
    service: Service<'_>,
    item: Target,
    path: &[FieldId],
) -> Result<Target, RuntimeCatalogError> {
    tags::resolve(service, item, path)
}
fn text(service: Service<'_>, target: Target) -> Result<(), RuntimeCatalogError> {
    if matches!(tags::target_kind(service, target)?, "string" | "enum") {
        Ok(())
    } else {
        Err(RuntimeCatalogError::Capability)
    }
}
fn scalar(service: Service<'_>, target: Target) -> Result<(), RuntimeCatalogError> {
    if matches!(
        tags::target_kind(service, target)?,
        "string" | "enum" | "byte" | "short" | "integer" | "long" | "boolean"
    ) {
        Ok(())
    } else {
        Err(RuntimeCatalogError::Capability)
    }
}

fn integral(
    service: Service<'_>,
    target: Target,
    minimum: i64,
    maximum: i64,
) -> Result<(), RuntimeCatalogError> {
    let (low, high) = match tags::target_kind(service, target)? {
        "byte" => (i64::from(i8::MIN), i64::from(i8::MAX)),
        "short" => (i64::from(i16::MIN), i64::from(i16::MAX)),
        "integer" => (i64::from(i32::MIN), i64::from(i32::MAX)),
        "long" => (i64::MIN, i64::MAX),
        _ => return Err(RuntimeCatalogError::Capability),
    };
    if minimum <= maximum && minimum >= low && maximum <= high {
        Ok(())
    } else {
        Err(RuntimeCatalogError::Bounds)
    }
}

fn digest_target(service: Service<'_>, target: Target) -> Result<(), RuntimeCatalogError> {
    if matches!(
        tags::target_kind(service, target)?,
        "string"
            | "enum"
            | "intEnum"
            | "boolean"
            | "byte"
            | "short"
            | "integer"
            | "long"
            | "float"
            | "double"
            | "bigInteger"
            | "bigDecimal"
            | "blob"
            | "timestamp"
            | "structure"
            | "union"
            | "list"
            | "map"
    ) {
        Ok(())
    } else {
        Err(RuntimeCatalogError::Capability)
    }
}

pub(in crate::catalog::runtime::decode::plans) fn validate_projection(
    parts: &PlanParts<'_>,
) -> Result<(), RuntimeCatalogError> {
    let item = validate_item(parts.service, parts.operation, &parts.items)?;
    let groups = [
        (&parts.context_projection.selected, item, true),
        (
            &parts.context_projection.response,
            parts.operation.output()?,
            false,
        ),
        (
            &parts.context_projection.request,
            parts.operation.input()?,
            false,
        ),
    ];
    groups
        .into_iter()
        .try_for_each(|(paths, root, selected_namespace)| {
            paths.iter().enumerate().try_for_each(|(index, path)| {
                if paths.iter().take(index).any(|prior| prior == path) {
                    return Err(RuntimeCatalogError::Capability);
                }
                let target = tags::resolve(parts.service, root, path)?;
                scalar(parts.service, target)?;
                if let Some(projection) = &parts.projection {
                    let protected = projection.protected_path();
                    let normalized = if selected_namespace {
                        Some(path.as_ref())
                    } else {
                        match &parts.items {
                            ItemPlan::Root => Some(path.as_ref()),
                            ItemPlan::Path(prefix) | ItemPlan::Select { path: prefix, .. } => {
                                path.strip_prefix(prefix.as_ref())
                            }
                        }
                    };
                    if normalized.is_some_and(|path| {
                        path.starts_with(protected) || protected.starts_with(path)
                    }) {
                        return Err(RuntimeCatalogError::Capability);
                    }
                }
                Ok(())
            })
        })
}
