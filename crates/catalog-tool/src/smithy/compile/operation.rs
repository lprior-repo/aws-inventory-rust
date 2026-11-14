mod paginator;
use super::{
    CompileError,
    encode::{map, name, optional_target, reference},
    service,
};
use crate::smithy::model::{ModelFile, Shape, ShapeKind, Target};
use inventory_core::{catalog::CatalogKey, record::Value};
use paginator::paginator;
use serde_json::Value as Json;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn encode(
    id: &str,
    shape: &Shape,
    model: &ModelFile,
    indexes: &BTreeMap<&str, usize>,
) -> Result<Value, CompileError> {
    let mut fields = operation_fields(id, shape, model, indexes)?;
    append_metadata(shape, model, indexes, &mut fields)?;
    map(fields)
}

fn operation_fields(
    id: &str,
    shape: &Shape,
    model: &ModelFile,
    indexes: &BTreeMap<&str, usize>,
) -> Result<Vec<(CatalogKey, Value)>, CompileError> {
    let ShapeKind::Operation { input, output } = &shape.kind else {
        return Err(CompileError::Kind);
    };
    Ok(vec![
        (CatalogKey::Name, name(id)),
        (
            CatalogKey::Flags,
            Value::Int(flags(super::encode::short_name(id), shape, input, model)),
        ),
        (CatalogKey::Input, optional_target(input, indexes)?),
        (CatalogKey::Output, optional_target(output, indexes)?),
    ])
}

fn append_metadata(
    shape: &Shape,
    model: &ModelFile,
    indexes: &BTreeMap<&str, usize>,
    fields: &mut Vec<(CatalogKey, Value)>,
) -> Result<(), CompileError> {
    let input = operation_input(shape)?;
    http_fields(shape, fields)?;
    endpoint(shape, fields)?;
    paginator(shape, input, model, fields)?;
    operation_errors(shape, model, indexes, fields)?;
    checksum(shape, fields);
    if find(&shape.traits, "aws.customizations#s3UnwrappedXmlOutput").is_some() {
        fields.push((CatalogKey::S3UnwrappedXmlOutput, Value::Bool(true)));
    }
    request_compression(shape, fields)?;
    media_type(shape, fields)
}

fn operation_input(shape: &Shape) -> Result<&Option<Target>, CompileError> {
    match &shape.kind {
        ShapeKind::Operation { input, .. } => Ok(input),
        _ => Err(CompileError::Kind),
    }
}

fn http_fields(shape: &Shape, out: &mut Vec<(CatalogKey, Value)>) -> Result<(), CompileError> {
    let Some(http) = find(&shape.traits, "smithy.api#http") else {
        return Ok(());
    };
    let method = text(http, "method", "http")?;
    let uri = text(http, "uri", "http")?;
    let code = http.get("code").and_then(Json::as_i64).unwrap_or(200);
    out.extend([
        (CatalogKey::HttpMethod, Value::Text(method.to_owned())),
        (CatalogKey::HttpUri, Value::Text(uri.to_owned())),
        (CatalogKey::HttpSuccess, Value::Int(code)),
    ]);
    Ok(())
}

fn endpoint(shape: &Shape, out: &mut Vec<(CatalogKey, Value)>) -> Result<(), CompileError> {
    if let Some(value) =
        find(&shape.traits, "smithy.api#endpoint").and_then(|endpoint| endpoint.get("hostPrefix"))
    {
        out.push((
            CatalogKey::HostPrefix,
            Value::Text(as_text(value, "smithy.api#endpoint.hostPrefix")?.to_owned()),
        ));
    }
    Ok(())
}
fn request_compression(
    shape: &Shape,
    out: &mut Vec<(CatalogKey, Value)>,
) -> Result<(), CompileError> {
    let id = "smithy.api#requestCompression";
    let Some(value) = find(&shape.traits, id) else {
        return Ok(());
    };
    let encodings = value
        .get("encodings")
        .and_then(Json::as_array)
        .ok_or_else(|| CompileError::Trait(id.to_owned()))?;
    let values = encodings
        .iter()
        .map(|encoding| {
            let name = as_text(encoding, id)?;
            if !name.eq_ignore_ascii_case("gzip") {
                return Err(CompileError::Trait(id.to_owned()));
            }
            Ok(Value::Text("gzip".to_owned()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    out.push((CatalogKey::RequestCompression, Value::List(values)));
    Ok(())
}

fn operation_errors(
    shape: &Shape,
    model: &ModelFile,
    indexes: &BTreeMap<&str, usize>,
    out: &mut Vec<(CatalogKey, Value)>,
) -> Result<(), CompileError> {
    let service_shape = service::service_shape(model)?.1;
    let mut seen = BTreeSet::new();
    let mut errors = Vec::new();
    for error in service_shape.errors.iter().chain(shape.errors.iter()) {
        if seen.insert(error.target.as_str()) {
            errors.push(reference(&error.target, indexes)?);
        }
    }
    if !errors.is_empty() {
        out.push((CatalogKey::Errors, Value::List(errors)));
    }
    Ok(())
}
fn checksum(shape: &Shape, out: &mut Vec<(CatalogKey, Value)>) {
    if find(&shape.traits, "smithy.api#httpChecksumRequired").is_some() {
        out.push((CatalogKey::HttpChecksum, Value::Bool(true)));
    }
}

fn media_type(shape: &Shape, out: &mut Vec<(CatalogKey, Value)>) -> Result<(), CompileError> {
    if let Some(value) = find(&shape.traits, "smithy.api#mediaType") {
        out.push((
            CatalogKey::MediaType,
            Value::Text(as_text(value, "mediaType")?.to_owned()),
        ));
    }
    if find(&shape.traits, "smithy.api#sensitive").is_some() {
        out.push((CatalogKey::Sensitive, Value::Bool(true)));
    }
    Ok(())
}

fn flags(name: &str, shape: &Shape, input: &Option<Target>, model: &ModelFile) -> i64 {
    let read = [
        "List", "Describe", "Get", "BatchGet", "Search", "Scan", "Query", "Lookup", "Select",
    ]
    .iter()
    .any(|prefix| name.starts_with(prefix));
    [
        (find(&shape.traits, "smithy.api#readonly").is_some(), 1),
        (find(&shape.traits, "smithy.api#paginated").is_some(), 2),
        (name.starts_with("List") || name.starts_with("Describe"), 4),
        (input_free(input, model), 8),
        (read, 16),
    ]
    .into_iter()
    .fold(
        0,
        |flags, (present, mask)| if present { flags | mask } else { flags },
    )
}

fn input_free(input: &Option<Target>, model: &ModelFile) -> bool {
    match input
        .as_ref()
        .and_then(|target| model.shapes.get(&target.target))
        .map(|shape| &shape.kind)
    {
        Some(ShapeKind::Structure { members }) => members
            .values()
            .all(|member| find(&member.traits, "smithy.api#required").is_none()),
        _ => true,
    }
}

fn find<'a>(traits: &'a BTreeMap<String, Json>, id: &str) -> Option<&'a Json> {
    traits.get(id)
}
fn text<'a>(value: &'a Json, key: &str, name: &str) -> Result<&'a str, CompileError> {
    value
        .get(key)
        .and_then(Json::as_str)
        .ok_or_else(|| CompileError::Trait(name.to_owned()))
}
fn as_text<'a>(value: &'a Json, name: &str) -> Result<&'a str, CompileError> {
    value
        .as_str()
        .ok_or_else(|| CompileError::Trait(name.to_owned()))
}
