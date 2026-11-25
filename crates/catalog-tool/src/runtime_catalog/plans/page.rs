#[cfg(test)]
mod tests;
mod tokens;
use super::super::{RuntimeCatalogCompileError, encode, model_path};
use super::{OperationOverlay, item_value};
use crate::overlay::Paginator;
use inventory_core::catalog::{HttpBinding, Operation, Service};
use inventory_core::record::Value;

type Descriptor<'a> = inventory_core::catalog::Paginator<'a>;

struct PageInputs<'p, 'a> {
    overlay: &'p OperationOverlay,
    service: Service<'a>,
    operation: Operation<'a>,
    protocol: i64,
    services: &'p super::super::ServiceIndex<'a>,
}

pub(super) fn compile(
    overlay: &OperationOverlay,
    service: Service<'_>,
    operation: Operation<'_>,
    protocol: i64,
    services: &super::super::ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let inputs = PageInputs {
        overlay,
        service,
        operation,
        protocol,
        services,
    };
    let descriptor = operation.paginator()?;
    validate_items(overlay, descriptor.as_ref())?;
    let mut fields = vec![
        (1, item_value(overlay.items.as_ref(), services)?),
        (
            2,
            Value::List(tokens::compile(&inputs, descriptor.as_ref())?),
        ),
    ];
    fields.extend(page_size_value(&inputs, descriptor.as_ref())?.map(|value| (3, value)));
    fields.extend(maximum_value(descriptor.as_ref())?.map(|value| (4, value)));
    fields.extend(truncated_value(overlay, service, operation, services)?.map(|value| (5, value)));
    encode::map(fields)
}

fn maximum_value(
    descriptor: Option<&Descriptor<'_>>,
) -> Result<Option<Value>, RuntimeCatalogCompileError> {
    descriptor
        .and_then(|value| value.max_page_size)
        .map(|value| u32::try_from(value).map(|value| Value::Int(i64::from(value))))
        .transpose()
        .map_err(|_| RuntimeCatalogCompileError::Path {
            path: "page max_page_size".to_owned(),
        })
}

fn validate_items(
    overlay: &OperationOverlay,
    descriptor: Option<&inventory_core::catalog::Paginator<'_>>,
) -> Result<(), RuntimeCatalogCompileError> {
    match descriptor.and_then(|page| page.items) {
        Some(expected)
            if selected_items(overlay).is_some_and(|actual| actual.as_str() == expected) =>
        {
            Ok(())
        }
        Some(_) => Err(RuntimeCatalogCompileError::Path {
            path: "page items".to_owned(),
        }),
        None => Ok(()),
    }
}

fn selected_items(overlay: &OperationOverlay) -> Option<&crate::overlay::MemberPath> {
    use crate::overlay::{ItemSelector, TagEndpointSelector};
    match overlay.items.as_ref()? {
        ItemSelector::Member(path) | ItemSelector::Explicit { path, .. } => Some(path),
        ItemSelector::Root => {
            overlay
                .tag_endpoint
                .as_ref()
                .map(|endpoint| match &endpoint.selector {
                    TagEndpointSelector::Map { path } | TagEndpointSelector::List { path, .. } => {
                        path
                    }
                })
        }
    }
}

fn placement(
    member: inventory_core::catalog::Member<'_>,
    path: Vec<inventory_core::record::FieldId>,
    protocol: i64,
) -> Result<Value, RuntimeCatalogCompileError> {
    if matches!(protocol, 0..=2) {
        return encode::map([(1, Value::Int(3)), (2, path_value(path))]);
    }
    let selected = match (protocol, member.wire()?.binding()?) {
        (3 | 4, HttpBinding::Document) if path.len() == 1 => {
            Some((1, Value::Text(query_name(member, protocol)?)))
        }
        (5 | 6, HttpBinding::Query(value)) if path.len() == 1 => {
            Some((1, Value::Text(value.to_owned())))
        }
        (5 | 6, HttpBinding::Header(value)) if path.len() == 1 => {
            Some((2, Value::Text(value.to_owned())))
        }
        (5 | 6, HttpBinding::Label) if path.len() == 1 => {
            Some((0, Value::Text(member.name()?.to_owned())))
        }
        (_, HttpBinding::Document) => Some((3, path_value(path))),
        (5 | 6, HttpBinding::Payload) if path.len() == 1 => Some((3, path_value(path))),
        _ => None,
    };
    let (kind, value) = selected.ok_or(RuntimeCatalogCompileError::Path {
        path: "page placement".to_owned(),
    })?;
    encode::map([(1, Value::Int(kind)), (2, value)])
}

fn query_name(
    member: inventory_core::catalog::Member<'_>,
    protocol: i64,
) -> Result<String, RuntimeCatalogCompileError> {
    let wire = member.wire()?;
    let modeled = match protocol {
        3 => wire.xml_name()?,
        4 => wire.ec2_query_name()?,
        _ => None,
    };
    Ok(modeled.map_or_else(
        || member.name().map(str::to_owned),
        |value| Ok(value.to_owned()),
    )?)
}

fn path_value(path: Vec<inventory_core::record::FieldId>) -> Value {
    Value::List(
        path.into_iter()
            .map(|id| Value::Int(i64::from(id.value())))
            .collect(),
    )
}
fn response_placement(
    member: inventory_core::catalog::Member<'_>,
    path: Vec<inventory_core::record::FieldId>,
    protocol: i64,
) -> Result<Value, RuntimeCatalogCompileError> {
    if matches!(protocol, 0..=2) {
        return encode::map([(1, Value::Int(3)), (2, path_value(path))]);
    }
    match (protocol, member.wire()?.binding()?) {
        (
            _,
            HttpBinding::Document
            | HttpBinding::Label
            | HttpBinding::Query(_)
            | HttpBinding::QueryParams,
        ) => encode::map([(1, Value::Int(3)), (2, path_value(path))]),
        (5 | 6, HttpBinding::Payload) if path.len() == 1 => {
            encode::map([(1, Value::Int(3)), (2, path_value(path))])
        }
        (5 | 6, HttpBinding::Header(name)) if path.len() == 1 => {
            encode::map([(1, Value::Int(2)), (2, Value::Text(name.to_owned()))])
        }
        _ => Err(RuntimeCatalogCompileError::Path {
            path: "page response placement".to_owned(),
        }),
    }
}

fn page_size_value(
    inputs: &PageInputs<'_, '_>,
    descriptor: Option<&Descriptor<'_>>,
) -> Result<Option<Value>, RuntimeCatalogCompileError> {
    let Some(value) = inputs.overlay.page_size else {
        return Ok(None);
    };
    let descriptor = descriptor.ok_or(RuntimeCatalogCompileError::Path {
        path: "page_size descriptor".to_owned(),
    })?;
    let path = descriptor
        .page_size
        .ok_or(RuntimeCatalogCompileError::Path {
            path: "page_size path".to_owned(),
        })?;
    let member = model_path::terminal_member(inputs.service, inputs.operation.input()?, path)?;
    let kind = model_path::kind(inputs.service, member.1.target)?;
    let max = validated_max(value, kind, descriptor)?;
    let literal = if matches!(kind, "string" | "enum") {
        crate::overlay::FixedValue::String(value.to_string())
    } else {
        crate::overlay::FixedValue::Integer(value)
    };
    super::fixed_input::compile(inputs.service, member.1.target, &literal, path)?;
    let placement = placement(member.1, member.0, inputs.protocol)?;
    let fields = [
        (1, encode::path(inputs.services, path)?),
        (2, Value::Int(i64::from(max))),
        (3, placement),
        (
            4,
            Value::Int(if matches!(kind, "string" | "enum") {
                0
            } else {
                1
            }),
        ),
    ];
    Ok(Some(encode::map(fields)?))
}

fn validated_max(
    value: i64,
    kind: &str,
    descriptor: &Descriptor<'_>,
) -> Result<u32, RuntimeCatalogCompileError> {
    let supported = matches!(
        kind,
        "integer" | "long" | "short" | "byte" | "string" | "enum"
    );
    let max = u32::try_from(value).map_err(|_| RuntimeCatalogCompileError::Path {
        path: "page validated_max".to_owned(),
    })?;
    if !supported || max == 0 || exceeds_modeled(max, descriptor)? {
        return Err(RuntimeCatalogCompileError::Path {
            path: "page validated_max".to_owned(),
        });
    }
    Ok(max)
}

fn exceeds_modeled(
    max: u32,
    descriptor: &Descriptor<'_>,
) -> Result<bool, RuntimeCatalogCompileError> {
    let modeled = descriptor.max_page_size.map(u32::try_from).transpose();
    let modeled = modeled.map_err(|_| RuntimeCatalogCompileError::Path {
        path: "page exceeds_modeled".to_owned(),
    })?;
    Ok(modeled.is_some_and(|limit| max > limit))
}

fn truncated_value(
    overlay: &OperationOverlay,
    service: Service<'_>,
    operation: Operation<'_>,
    services: &super::super::ServiceIndex<'_>,
) -> Result<Option<Value>, RuntimeCatalogCompileError> {
    let path = match overlay.paginator.as_ref() {
        Some(Paginator::Tokens { truncated, .. }) => truncated.as_ref(),
        Some(Paginator::LastListItem { truncated, .. }) => Some(truncated),
        _ => None,
    };
    path.map(|path| {
        let (_, member) =
            model_path::terminal_member(service, operation.output()?, &path.to_string())?;
        if model_path::kind(service, member.target)? != "boolean"
            || !matches!(member.wire()?.binding()?, HttpBinding::Document)
        {
            return Err(RuntimeCatalogCompileError::Path {
                path: path.to_string(),
            });
        }
        encode::path(services, &path.to_string())
    })
    .transpose()
}
