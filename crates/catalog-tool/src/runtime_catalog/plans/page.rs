use super::super::{RuntimeCatalogCompileError, encode, model_path};
use super::{OperationOverlay, item_value};
use crate::overlay::{Paginator, TokenPair};
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
        (2, Value::List(token_values(&inputs, descriptor.as_ref())?)),
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
        .map_err(|_| RuntimeCatalogCompileError::Capability)
}

fn validate_items(
    overlay: &OperationOverlay,
    descriptor: Option<&inventory_core::catalog::Paginator<'_>>,
) -> Result<(), RuntimeCatalogCompileError> {
    match descriptor.and_then(|page| page.items) {
        Some(expected) => match overlay.items.as_ref() {
            Some(crate::overlay::ItemSelector::Member(actual))
                if expected == actual.to_string() =>
            {
                Ok(())
            }
            _ => Err(RuntimeCatalogCompileError::Capability),
        },
        None => Ok(()),
    }
}

fn token_values(
    inputs: &PageInputs<'_, '_>,
    descriptor: Option<&Descriptor<'_>>,
) -> Result<Vec<Value>, RuntimeCatalogCompileError> {
    let pair = match inputs.overlay.paginator.as_ref() {
        Some(Paginator::Token(pair)) => pair,
        Some(Paginator::Tokens { pairs, .. }) if pairs.len() == 1 => pairs
            .first()
            .ok_or(RuntimeCatalogCompileError::Capability)?,
        Some(Paginator::Tokens { .. }) => return Err(RuntimeCatalogCompileError::Capability),
        Some(Paginator::None) | None => return untokenized(descriptor),
    };
    validate_descriptor_token(pair, descriptor)?;
    Ok(vec![token_value(pair, inputs)?])
}

fn untokenized(
    descriptor: Option<&Descriptor<'_>>,
) -> Result<Vec<Value>, RuntimeCatalogCompileError> {
    if descriptor.is_none_or(|page| page.input_token.is_none() && page.output_token.is_none()) {
        Ok(Vec::new())
    } else {
        Err(RuntimeCatalogCompileError::Capability)
    }
}

fn validate_descriptor_token(
    pair: &TokenPair,
    descriptor: Option<&inventory_core::catalog::Paginator<'_>>,
) -> Result<(), RuntimeCatalogCompileError> {
    let expected = descriptor.ok_or(RuntimeCatalogCompileError::Capability)?;
    let input = pair.input.to_string();
    let output = pair.output.to_string();
    if expected.input_token == Some(input.as_str())
        && expected.output_token == Some(output.as_str())
    {
        Ok(())
    } else {
        Err(RuntimeCatalogCompileError::Capability)
    }
}

fn token_value(
    pair: &TokenPair,
    inputs: &PageInputs<'_, '_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let (service, operation) = (inputs.service, inputs.operation);
    let (input_path, output_path) = (pair.input.to_string(), pair.output.to_string());
    let input = model_path::terminal_member(service, operation.input()?, &input_path)?;
    let output = model_path::terminal_member(service, operation.output()?, &output_path)?;
    let value_type = token_type(model_path::kind(service, input.1.target)?)?;
    let request = placement(input.1, input.0, inputs.protocol)?;
    let response = response_placement(output.1, output.0)?;
    encode::map([
        (1, encode::path(inputs.services, &input_path)?),
        (2, encode::path(inputs.services, &output_path)?),
        (3, Value::Int(value_type)),
        (4, request),
        (5, response),
    ])
}

fn token_type(kind: &str) -> Result<i64, RuntimeCatalogCompileError> {
    match kind {
        "string" => Ok(0),
        "integer" | "long" | "short" | "byte" | "bigInteger" => Ok(1),
        "boolean" => Ok(2),
        _ => Err(RuntimeCatalogCompileError::Capability),
    }
}

fn placement(
    member: inventory_core::catalog::Member<'_>,
    path: Vec<inventory_core::record::FieldId>,
    protocol: i64,
) -> Result<Value, RuntimeCatalogCompileError> {
    let name = member.name()?.to_owned();
    let selected = match (protocol, member.wire()?.binding()?) {
        (3 | 4, HttpBinding::Document) => Some((1, Value::Text(query_name(member, protocol)?))),
        (_, HttpBinding::Query(value)) => Some((1, Value::Text(value.to_owned()))),
        (_, HttpBinding::Header(value)) => Some((2, Value::Text(value.to_owned()))),
        (_, HttpBinding::Label) => Some((0, Value::Text(name))),
        (_, HttpBinding::Document | HttpBinding::Payload) => Some((3, path_value(path))),
        (
            _,
            HttpBinding::QueryParams | HttpBinding::PrefixHeaders(_) | HttpBinding::ResponseCode,
        ) => None,
    };
    let (kind, value) = selected.ok_or(RuntimeCatalogCompileError::Capability)?;
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
) -> Result<Value, RuntimeCatalogCompileError> {
    match member.wire()?.binding()? {
        HttpBinding::Header(_) | HttpBinding::Document | HttpBinding::Payload => {
            placement(member, path, 0)
        }
        HttpBinding::Label
        | HttpBinding::Query(_)
        | HttpBinding::QueryParams
        | HttpBinding::PrefixHeaders(_)
        | HttpBinding::ResponseCode => Err(RuntimeCatalogCompileError::Capability),
    }
}

fn page_size_value(
    inputs: &PageInputs<'_, '_>,
    descriptor: Option<&Descriptor<'_>>,
) -> Result<Option<Value>, RuntimeCatalogCompileError> {
    let Some(value) = inputs.overlay.page_size else {
        return Ok(None);
    };
    let descriptor = descriptor.ok_or(RuntimeCatalogCompileError::Capability)?;
    let path = descriptor
        .page_size
        .ok_or(RuntimeCatalogCompileError::Capability)?;
    let member = model_path::terminal_member(inputs.service, inputs.operation.input()?, path)?;
    let kind = model_path::kind(inputs.service, member.1.target)?;
    let max = validated_max(value, kind, descriptor)?;
    let placement = placement(member.1, member.0, inputs.protocol)?;
    let fields = [
        (1, encode::path(inputs.services, path)?),
        (2, Value::Int(i64::from(max))),
        (3, placement),
    ];
    Ok(Some(encode::map(fields)?))
}

fn validated_max(
    value: i64,
    kind: &str,
    descriptor: &Descriptor<'_>,
) -> Result<u32, RuntimeCatalogCompileError> {
    let integer = matches!(kind, "integer" | "long" | "short" | "byte");
    let max = u32::try_from(value).map_err(|_| RuntimeCatalogCompileError::Capability)?;
    if !integer || max == 0 || exceeds_modeled(max, descriptor)? {
        return Err(RuntimeCatalogCompileError::Capability);
    }
    Ok(max)
}

fn exceeds_modeled(
    max: u32,
    descriptor: &Descriptor<'_>,
) -> Result<bool, RuntimeCatalogCompileError> {
    let modeled = descriptor.max_page_size.map(u32::try_from).transpose();
    let modeled = modeled.map_err(|_| RuntimeCatalogCompileError::Capability)?;
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
        _ => None,
    };
    path.map(|path| {
        let target = model_path::resolve(service, operation.output()?, &path.to_string())?.1;
        if model_path::kind(service, target)? != "boolean" {
            return Err(RuntimeCatalogCompileError::Capability);
        }
        encode::path(services, &path.to_string())
    })
    .transpose()
}
