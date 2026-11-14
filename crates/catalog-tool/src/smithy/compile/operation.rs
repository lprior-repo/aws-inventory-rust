use super::{CompileError, encode::{map, name, optional_target, reference}, service};
use crate::smithy::model::{ModelFile, Shape, ShapeKind, Target};
use inventory_core::{catalog::CatalogKey, record::Value};
use serde_json::Value as Json;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn encode(id: &str, shape: &Shape, model: &ModelFile, indexes: &BTreeMap<&str, usize>) -> Result<Value, CompileError> {
    let ShapeKind::Operation { input, output } = &shape.kind else { return Err(CompileError::Kind) };
    let mut fields = vec![
        (CatalogKey::Name, name(id)),
        (CatalogKey::Flags, Value::Int(flags(super::encode::short_name(id), shape, input, model))),
        (CatalogKey::Input, optional_target(input, indexes)?),
        (CatalogKey::Output, optional_target(output, indexes)?),
    ];
    http_fields(shape, &mut fields)?;
    endpoint(shape, &mut fields)?;
    paginator(shape, input, model, &mut fields)?;
    operation_errors(shape, model, indexes, &mut fields)?;
    checksum(shape, &mut fields);
    media_type(shape, &mut fields)?;
    map(fields)
}

fn http_fields(shape: &Shape, out: &mut Vec<(CatalogKey, Value)>) -> Result<(), CompileError> {
    let Some(http) = find(&shape.traits, "smithy.api#http") else { return Ok(()) };
    let method = text(http, "method", "http")?;
    let uri = text(http, "uri", "http")?;
    let code = http.get("code").and_then(Json::as_i64).unwrap_or(200);
    out.extend([(CatalogKey::HttpMethod, Value::Text(method.to_owned())), (CatalogKey::HttpUri, Value::Text(uri.to_owned())), (CatalogKey::HttpSuccess, Value::Int(code))]);
    Ok(())
}

fn endpoint(shape: &Shape, out: &mut Vec<(CatalogKey, Value)>) -> Result<(), CompileError> {
    if let Some(value) = find(&shape.traits, "smithy.api#endpoint").and_then(|endpoint| endpoint.get("hostPrefix")) {
        out.push((CatalogKey::HostPrefix, Value::Text(as_text(value, "smithy.api#endpoint.hostPrefix")?.to_owned())));
    }
    Ok(())
}

fn paginator(shape: &Shape, input: &Option<Target>, model: &ModelFile, out: &mut Vec<(CatalogKey, Value)>) -> Result<(), CompileError> {
    let Some(paginator) = find(&shape.traits, "smithy.api#paginated") else { return Ok(()) };
    let defaults = find(&service::service_shape(model)?.1.traits, "smithy.api#paginated");
    for (name, key) in [("inputToken", CatalogKey::PaginatedInput), ("outputToken", CatalogKey::PaginatedOutput), ("items", CatalogKey::PaginatedResult), ("pageSize", CatalogKey::PaginatedPageSize)] {
        if let Some(value) = paginator.get(name).or_else(|| defaults.and_then(|value| value.get(name))) {
            out.push((key, Value::Text(as_text(value, name)?.to_owned())));
        }
    }
    if let Some(path) = paginator.get("pageSize").or_else(|| defaults.and_then(|value| value.get("pageSize"))).and_then(Json::as_str) {
        if let Some(max) = range_max(model, input, path)? { out.push((CatalogKey::PaginatedLimit, Value::Int(max))); }
    }
    Ok(())
}

fn range_max(model: &ModelFile, input: &Option<Target>, path: &str) -> Result<Option<i64>, CompileError> {
    let mut shape = input.as_ref().and_then(|target| model.shapes.get(&target.target))
        .ok_or_else(|| CompileError::Trait("smithy.api#paginated.pageSize".to_owned()))?;
    let mut parts = path.split('.').peekable();
    while let Some(part) = parts.next() {
        let members = match &shape.kind { ShapeKind::Structure { members } => members, _ => return Err(CompileError::Trait("smithy.api#paginated.pageSize".to_owned())) };
        let member = members.get(part).ok_or_else(|| CompileError::Trait("smithy.api#paginated.pageSize".to_owned()))?;
        let member_max = range_max_trait(&member.traits)?;
        let target = member.target.as_deref().ok_or_else(|| CompileError::Target(part.to_owned()))?;
        let Some(target_shape) = model.shapes.get(target) else {
            if target.starts_with("smithy.api#") && parts.peek().is_none() { return Ok(member_max); }
            return Err(CompileError::Target(target.to_owned()));
        };
        if parts.peek().is_none() { return Ok(member_max.or(range_max_trait(&target_shape.traits)?)); }
        shape = target_shape;
    }
    Ok(None)
}

fn range_max_trait(traits: &BTreeMap<String, Json>) -> Result<Option<i64>, CompileError> {
    let Some(value) = find(traits, "smithy.api#range").and_then(|range| range.get("max")) else { return Ok(None) };
    match value.as_i64() {
        Some(max) if max >= 0 => Ok(Some(max)),
        _ => Err(CompileError::Trait("smithy.api#range.max".to_owned())),
    }
}

fn operation_errors(shape: &Shape, model: &ModelFile, indexes: &BTreeMap<&str, usize>, out: &mut Vec<(CatalogKey, Value)>) -> Result<(), CompileError> {
    let service_shape = service::service_shape(model)?.1;
    let mut seen = BTreeSet::new();
    let mut errors = Vec::new();
    for error in service_shape.errors.iter().chain(shape.errors.iter()) {
        if seen.insert(error.target.as_str()) { errors.push(reference(&error.target, indexes)?); }
    }
    if !errors.is_empty() { out.push((CatalogKey::Errors, Value::List(errors))); }
    Ok(())
}
fn checksum(shape: &Shape, out: &mut Vec<(CatalogKey, Value)>) {
    if find(&shape.traits, "smithy.api#httpChecksumRequired").is_some() { out.push((CatalogKey::HttpChecksum, Value::Bool(true))); }
}

fn media_type(shape: &Shape, out: &mut Vec<(CatalogKey, Value)>) -> Result<(), CompileError> {
    if let Some(value) = find(&shape.traits, "smithy.api#mediaType") { out.push((CatalogKey::MediaType, Value::Text(as_text(value, "mediaType")?.to_owned()))); }
    if find(&shape.traits, "smithy.api#sensitive").is_some() { out.push((CatalogKey::Sensitive, Value::Bool(true))); }
    Ok(())
}

fn flags(name: &str, shape: &Shape, input: &Option<Target>, model: &ModelFile) -> i64 {
    let read = ["List", "Describe", "Get", "BatchGet", "Search", "Scan", "Query", "Lookup", "Select"].iter().any(|prefix| name.starts_with(prefix));
    [(find(&shape.traits, "smithy.api#readonly").is_some(), 1), (find(&shape.traits, "smithy.api#paginated").is_some(), 2), (name.starts_with("List") || name.starts_with("Describe"), 4), (input_free(input, model), 8), (read, 16)]
        .into_iter().fold(0, |flags, (present, mask)| if present { flags | mask } else { flags })
}

fn input_free(input: &Option<Target>, model: &ModelFile) -> bool {
    match input.as_ref().and_then(|target| model.shapes.get(&target.target)).map(|shape| &shape.kind) {
        Some(ShapeKind::Structure { members }) => members.values().all(|member| find(&member.traits, "smithy.api#required").is_none()),
        _ => true,
    }
}

fn find<'a>(traits: &'a BTreeMap<String, Json>, id: &str) -> Option<&'a Json> { traits.get(id) }
fn text<'a>(value: &'a Json, key: &str, name: &str) -> Result<&'a str, CompileError> { value.get(key).and_then(Json::as_str).ok_or_else(|| CompileError::Trait(name.to_owned())) }
fn as_text<'a>(value: &'a Json, name: &str) -> Result<&'a str, CompileError> { value.as_str().ok_or_else(|| CompileError::Trait(name.to_owned())) }
