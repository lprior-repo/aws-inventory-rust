use super::{CompileError, as_text, find, service};
use crate::smithy::model::{ModelFile, Shape, ShapeKind, Target};
use inventory_core::{catalog::CatalogKey, record::Value};
use serde_json::Value as Json;
use std::collections::BTreeMap;

pub(super) fn paginator(
    shape: &Shape,
    input: &Option<Target>,
    model: &ModelFile,
    out: &mut Vec<(CatalogKey, Value)>,
) -> Result<(), CompileError> {
    let Some(operation) = find(&shape.traits, "smithy.api#paginated") else {
        return Ok(());
    };
    let defaults = find(
        &service::service_shape(model)?.1.traits,
        "smithy.api#paginated",
    );
    paginator_fields(operation, defaults, input, model, out)
}

fn paginator_fields(
    operation: &Json,
    defaults: Option<&Json>,
    input: &Option<Target>,
    model: &ModelFile,
    out: &mut Vec<(CatalogKey, Value)>,
) -> Result<(), CompileError> {
    paginator_names(operation, defaults, out)?;
    if let Some(path) = paginator_value(operation, defaults, "pageSize").and_then(Json::as_str) {
        if let Some(max) = range_max(model, input, path)? {
            out.push((CatalogKey::PaginatedLimit, Value::Int(max)));
        }
    }
    Ok(())
}

fn paginator_names(
    operation: &Json,
    defaults: Option<&Json>,
    out: &mut Vec<(CatalogKey, Value)>,
) -> Result<(), CompileError> {
    for (name, key) in [
        ("inputToken", CatalogKey::PaginatedInput),
        ("outputToken", CatalogKey::PaginatedOutput),
        ("items", CatalogKey::PaginatedResult),
        ("pageSize", CatalogKey::PaginatedPageSize),
    ] {
        if let Some(value) = paginator_value(operation, defaults, name) {
            out.push((key, Value::Text(as_text(value, name)?.to_owned())));
        }
    }
    Ok(())
}

fn paginator_value<'a>(
    operation: &'a Json,
    defaults: Option<&'a Json>,
    name: &str,
) -> Option<&'a Json> {
    operation
        .get(name)
        .or_else(|| defaults.and_then(|value| value.get(name)))
}

fn range_max(
    model: &ModelFile,
    input: &Option<Target>,
    path: &str,
) -> Result<Option<i64>, CompileError> {
    let shape = input
        .as_ref()
        .and_then(|target| model.shapes.get(&target.target))
        .ok_or_else(page_size_error)?;
    range_path(model, shape, path)
}

fn range_path<'a>(
    model: &'a ModelFile,
    mut shape: &'a Shape,
    path: &str,
) -> Result<Option<i64>, CompileError> {
    let mut parts = path.split('.').peekable();
    while let Some(part) = parts.next() {
        let terminal = parts.peek().is_none();
        let (member_max, target) = range_member(model, shape, part, terminal)?;
        let Some(target_shape) = target else {
            return Ok(member_max);
        };
        if terminal {
            return Ok(member_max.or(range_max_trait(&target_shape.traits)?));
        }
        shape = target_shape;
    }
    Ok(None)
}

fn range_member<'a>(
    model: &'a ModelFile,
    shape: &Shape,
    part: &str,
    terminal: bool,
) -> Result<(Option<i64>, Option<&'a Shape>), CompileError> {
    let members = match &shape.kind {
        ShapeKind::Structure { members } => members,
        _ => return Err(page_size_error()),
    };
    let member = members.get(part).ok_or_else(page_size_error)?;
    let member_max = range_max_trait(&member.traits)?;
    let target = member
        .target
        .as_deref()
        .ok_or_else(|| CompileError::Target(part.to_owned()))?;
    match model.shapes.get(target) {
        Some(target_shape) => Ok((member_max, Some(target_shape))),
        None if terminal && target.starts_with("smithy.api#") => Ok((member_max, None)),
        None => Err(CompileError::Target(target.to_owned())),
    }
}

fn range_max_trait(traits: &BTreeMap<String, Json>) -> Result<Option<i64>, CompileError> {
    let Some(value) = find(traits, "smithy.api#range").and_then(|range| range.get("max")) else {
        return Ok(None);
    };
    match value.as_i64() {
        Some(max) if max >= 0 => Ok(Some(max)),
        _ => Err(CompileError::Trait("smithy.api#range.max".to_owned())),
    }
}

fn page_size_error() -> CompileError {
    CompileError::Trait("smithy.api#paginated.pageSize".to_owned())
}
