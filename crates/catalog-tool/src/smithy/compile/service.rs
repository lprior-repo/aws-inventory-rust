use super::{CompileError, encode::{map, name}};
use crate::smithy::model::{ModelFile, Shape, ShapeKind};
use inventory_core::{catalog::CatalogKey, record::Value};
use serde_json::Value as Json;
use std::collections::BTreeMap;

pub(super) fn fields(model: &ModelFile, service_name: &str, _indexes: &BTreeMap<&str, usize>) -> Result<Vec<(CatalogKey, Value)>, CompileError> {
    let (id, shape) = service_shape(model)?;
    let protocols = ["aws.protocols#awsJson1_0", "aws.protocols#awsJson1_1", "aws.protocols#restJson1", "aws.protocols#awsQuery", "aws.protocols#ec2Query", "aws.protocols#restXml", "smithy.protocols#rpcv2Cbor"]
        .iter().filter(|id| shape.traits.contains_key(**id)).map(|id| Value::Text((*id).to_owned())).collect::<Vec<_>>();
    let resources = model.shapes.iter().filter(|(_, s)| matches!(s.kind, ShapeKind::Resource)).map(|(id, _)| name(id)).collect::<Vec<_>>();
    let mut out = vec![(CatalogKey::Name, Value::Text(service_name.to_owned())), (CatalogKey::ServiceShape, Value::Text(id.to_owned()))];
    if !protocols.is_empty() { out.push((CatalogKey::Protocols, Value::List(protocols))); }
    if !resources.is_empty() { out.push((CatalogKey::Resources, Value::List(resources))); }
    out.push((CatalogKey::ApiVersion, Value::Text(service_version(shape).map_err(|error| error.in_shape(id))?.to_owned())));
    add_service_strings(shape, &mut out)?;
    add_protocol_flags(shape, &mut out);
    add_namespace(shape, &mut out)?;
    Ok(out)
}

pub(super) fn service_shape(model: &ModelFile) -> Result<(&str, &Shape), CompileError> {
    model.shapes.iter().find(|(_, shape)| matches!(shape.kind, ShapeKind::Service))
        .map(|(id, shape)| (id.as_str(), shape)).ok_or(CompileError::Kind)
}

fn add_service_strings(shape: &Shape, out: &mut Vec<(CatalogKey, Value)>) -> Result<(), CompileError> {
    if let Some(service) = shape.traits.get("aws.api#service") {
        for (field, key) in [("sdkId", CatalogKey::SdkId), ("arnNamespace", CatalogKey::ArnNamespace)] {
            if let Some(value) = service.get(field) { out.push((key, Value::Text(as_text(value, field)?.to_owned()))); }
        }
        if let Some(value) = service.get("endpointPrefix") { out.push((CatalogKey::EndpointPrefix, Value::Text(as_text(value, "aws.api#service.endpointPrefix")?.to_owned()))); }
        else { add_trait_string(shape, "aws.api#endpointPrefix", CatalogKey::EndpointPrefix, out)?; }
    } else {
        add_trait_string(shape, "aws.api#endpointPrefix", CatalogKey::EndpointPrefix, out)?;
    }
    if let Some(sigv4) = shape.traits.get("aws.auth#sigv4") {
        if let Some(name) = sigv4.get("name") { out.push((CatalogKey::Sigv4Name, Value::Text(as_text(name, "aws.auth#sigv4.name")?.to_owned()))); }
    }
    Ok(())
}

fn add_trait_string(shape: &Shape, id: &str, key: CatalogKey, out: &mut Vec<(CatalogKey, Value)>) -> Result<(), CompileError> {
    if let Some(value) = shape.traits.get(id) { out.push((key, Value::Text(as_text(value, id)?.to_owned()))); }
    Ok(())
}

fn service_version(shape: &Shape) -> Result<&str, CompileError> {
    shape.version.as_deref().ok_or_else(|| CompileError::Trait("service.version".to_owned()))
}

fn add_protocol_flags(shape: &Shape, out: &mut Vec<(CatalogKey, Value)>) {
    if shape.traits.contains_key("aws.protocols#awsQueryCompatible") { out.push((CatalogKey::AwsQueryCompatible, Value::Bool(true))); }
    let protocols = ["aws.protocols#awsJson1_0", "aws.protocols#awsJson1_1", "aws.protocols#restJson1", "aws.protocols#awsQuery", "aws.protocols#ec2Query", "aws.protocols#restXml", "smithy.protocols#rpcv2Cbor"];
    let no_wrap = protocols.iter().filter(|id| shape.traits.get(**id).and_then(|v| v.get("noErrorWrapping")).and_then(Json::as_bool) == Some(true))
        .map(|id| Value::Text((*id).to_owned())).collect::<Vec<_>>();
    if !no_wrap.is_empty() { out.push((CatalogKey::NoErrorWrapping, Value::List(no_wrap))); }
}

fn add_namespace(shape: &Shape, out: &mut Vec<(CatalogKey, Value)>) -> Result<(), CompileError> {
    let Some(namespace) = shape.traits.get("smithy.api#xmlNamespace") else { return Ok(()) };
    let uri = namespace.get("uri").and_then(Json::as_str).ok_or_else(|| CompileError::Trait("smithy.api#xmlNamespace".to_owned()))?;
    let mut fields = vec![(CatalogKey::XmlUri, Value::Text(uri.to_owned()))];
    if let Some(prefix) = namespace.get("prefix") {
        let prefix = prefix.as_str().ok_or_else(|| CompileError::Trait("smithy.api#xmlNamespace".to_owned()))?;
        fields.push((CatalogKey::XmlPrefix, Value::Text(prefix.to_owned())));
    }
    out.push((CatalogKey::XmlNamespace, map(fields)?));
    Ok(())
}
