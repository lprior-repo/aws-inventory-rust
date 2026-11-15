use super::{Descriptor, PageInputs, RuntimeCatalogCompileError, encode, model_path};
use crate::overlay::{Paginator, TokenPair};
use inventory_core::record::Value;

pub(super) fn compile(
    inputs: &PageInputs<'_, '_>,
    descriptor: Option<&Descriptor<'_>>,
) -> Result<Vec<Value>, RuntimeCatalogCompileError> {
    let pairs = match inputs.overlay.paginator.as_ref() {
        Some(Paginator::Token(pair)) => std::slice::from_ref(pair),
        Some(Paginator::Tokens { pairs, .. }) => pairs,
        Some(Paginator::None) | None => return untokenized(descriptor),
    };
    validate_pairs(pairs, descriptor)?;
    pairs.iter().map(|pair| token_value(pair, inputs)).collect()
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

fn validate_pairs(
    pairs: &[TokenPair],
    descriptor: Option<&Descriptor<'_>>,
) -> Result<(), RuntimeCatalogCompileError> {
    if pairs.is_empty() || pairs.len() > 8 || conflicting_paths(pairs)
        || pairs.iter().all(|pair| pair.optional)
    {
        return Err(RuntimeCatalogCompileError::Capability);
    }
    if let Some(expected) = descriptor {
        let matches = pairs.iter().any(|pair| {
            expected.input_token.is_none_or(|path| path == pair.input.as_str())
                && expected.output_token.is_none_or(|path| path == pair.output.as_str())
        });
        if !matches {
            return Err(RuntimeCatalogCompileError::Capability);
        }
    }
    Ok(())
}

fn conflicting_paths(pairs: &[TokenPair]) -> bool {
    pairs.iter().enumerate().any(|(index, pair)| {
        pairs.iter().take(index).any(|previous| {
            overlaps(pair.input.as_str(), previous.input.as_str())
                || overlaps(pair.output.as_str(), previous.output.as_str())
        })
    })
}

fn overlaps(left: &str, right: &str) -> bool {
    left == right
        || left.strip_prefix(right).is_some_and(|tail| tail.starts_with('.'))
        || right.strip_prefix(left).is_some_and(|tail| tail.starts_with('.'))
}

fn token_value(
    pair: &TokenPair,
    inputs: &PageInputs<'_, '_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let service = inputs.service;
    validate_ancestors(service, inputs.operation.input()?, pair.input.as_str())?;
    validate_ancestors(service, inputs.operation.output()?, pair.output.as_str())?;
    let input = model_path::terminal_member(service, inputs.operation.input()?, pair.input.as_str())?;
    let output = model_path::terminal_member(service, inputs.operation.output()?, pair.output.as_str())?;
    let value_type = token_type(model_path::kind(service, input.1.target)?)?;
    if (pair.optional && input.1.required)
        || value_type != token_type(model_path::kind(service, output.1.target)?)?
    {
        return Err(RuntimeCatalogCompileError::Capability);
    }
    encode::map([
        (1, encode::path(inputs.services, pair.input.as_str())?),
        (2, encode::path(inputs.services, pair.output.as_str())?),
        (3, Value::Int(value_type)),
        (4, super::placement(input.1, input.0, inputs.protocol)?),
        (5, super::response_placement(output.1, output.0, inputs.protocol)?),
        (6, Value::Bool(pair.optional)),
    ])
}

fn token_type(kind: &str) -> Result<i64, RuntimeCatalogCompileError> {
    match kind {
        "string" | "enum" => Ok(0),
        "integer" | "long" | "short" | "byte" | "intEnum" => Ok(1),
        "boolean" => Ok(2),
        _ => Err(RuntimeCatalogCompileError::Capability),
    }
}

fn validate_ancestors(
    service: inventory_core::catalog::Service<'_>,
    target: inventory_core::catalog::Target,
    path: &str,
) -> Result<(), RuntimeCatalogCompileError> {
    path.match_indices('.').try_for_each(|(index, _)| {
        let prefix = path.get(..index).ok_or(RuntimeCatalogCompileError::Capability)?;
        let (_, member) = model_path::terminal_member(service, target, prefix)?;
        let binding = member.wire()?.binding()?;
        let body = matches!(binding, inventory_core::catalog::HttpBinding::Document)
            || (matches!(binding, inventory_core::catalog::HttpBinding::Payload) && !prefix.contains('.'));
        if !body
            || model_path::kind(service, member.target)? != "structure"
        {
            return Err(RuntimeCatalogCompileError::Capability);
        }
        Ok(())
    })
}
