use super::{Context, RuntimeCatalogCompileError, Value, encode};
use crate::{
    overlay::{CallableOperation, CheckedOverlays},
    runtime_catalog::{model_path, plans},
};
use std::collections::BTreeSet;

pub(super) fn column(
    context: &Context<'_, '_>,
    kind: &str,
) -> Result<Option<Value>, RuntimeCatalogCompileError> {
    let Some(checked) = context.checked else {
        return Ok(None);
    };
    let types = item_types(context, checked, kind)?;
    if types.len() > 1 {
        return Err(RuntimeCatalogCompileError::KindContext {
            kind: kind.to_owned(),
            path: "column".to_owned(),
            reason: "operations of this kind disagree on the item type",
        });
    }
    types
        .first()
        .copied()
        .map(|scalar| encode::map([(1, Value::List(Vec::new())), (2, Value::Int(scalar))]))
        .transpose()
}

fn item_types(
    context: &Context<'_, '_>,
    checked: &CheckedOverlays,
    kind: &str,
) -> Result<BTreeSet<i64>, RuntimeCatalogCompileError> {
    context
        .kinds
        .operations()
        .iter()
        .filter(|row| row.kind == kind)
        .filter_map(|row| callable(checked, row).map(|call| (row.service.as_str(), call)))
        .map(|(service, call)| item_type(context, kind, service, call))
        .filter_map(Result::transpose)
        .collect()
}

fn callable<'a>(
    checked: &'a CheckedOverlays,
    row: &crate::kind_map::RuntimeOperation,
) -> Option<&'a CallableOperation> {
    checked
        .services()
        .iter()
        .find(|review| review.service() == row.service)
        .and_then(|review| {
            review.callable().iter().find(|call| {
                call.operation() == row.operation
                    && call.plan().emission == crate::overlay::Emission::EmitMetadata
                    && call.plan().tag_endpoint.is_none()
                    && !call.plan().endpoint_discovery
            })
        })
}

fn item_type(
    context: &Context<'_, '_>,
    kind: &str,
    service_name: &str,
    call: &CallableOperation,
) -> Result<Option<i64>, RuntimeCatalogCompileError> {
    let service = plans::catalog_service(context.services, service_name)?;
    let operation = plans::catalog_operation(service, call.operation())?;
    let target = crate::runtime_catalog::selection::target(
        service,
        operation.output()?,
        call.plan().items.as_ref(),
    )?;
    let model_kind = model_path::kind(service, target)?;
    scalar_code(model_kind).map_err(|_| {
        RuntimeCatalogCompileError::KindContext {
            kind: kind.to_owned(),
            path: model_kind.to_owned(),
            reason: "unsupported item type",
        }
    })
}

fn scalar_code(kind: &str) -> Result<Option<i64>, RuntimeCatalogCompileError> {
    match kind {
        "boolean" => Ok(Some(0)),
        "byte" | "short" | "integer" | "long" => Ok(Some(1)),
        "float" | "double" => Ok(Some(2)),
        "string" | "enum" => Ok(Some(3)),
        "blob" => Ok(Some(4)),
        "timestamp" => Ok(Some(5)),
        "structure" => Ok(None),
        _ => Err(RuntimeCatalogCompileError::Path {
            path: format!("selected schema kind {kind}"),
        }),
    }
}
