use super::{
    CompileError,
    encode::{map, name},
};
use crate::smithy::model::{ModelFile, Shape, ShapeKind};
use inventory_core::{catalog::CatalogKey, record::Value};
use serde_json::Value as Json;
use std::collections::BTreeMap;

pub(super) fn fields(
    model: &ModelFile,
    service_name: &str,
    _indexes: &BTreeMap<&str, usize>,
) -> Result<Vec<(CatalogKey, Value)>, CompileError> {
    let (id, shape) = service_shape(model)?;
    let mut out = vec![
        (CatalogKey::Name, Value::Text(service_name.to_owned())),
        (CatalogKey::ServiceShape, Value::Text(id.to_owned())),
        (
            CatalogKey::ApiVersion,
            Value::Text(
                service_version(shape)
                    .map_err(|error| error.in_shape(id))?
                    .to_owned(),
            ),
        ),
    ];
    add_protocols(shape, &mut out);
    add_resources(model, &mut out);
    add_service_strings(shape, &mut out)?;
    add_protocol_flags(shape, &mut out);
    add_namespace(shape, &mut out)?;
    Ok(out)
}

fn add_protocols(shape: &Shape, out: &mut Vec<(CatalogKey, Value)>) {
    let values = known_protocols()
        .iter()
        .filter(|id| shape.traits.contains_key(**id))
        .map(|id| Value::Text((*id).to_owned()))
        .collect::<Vec<_>>();
    out.push((CatalogKey::Protocols, Value::List(values)));
}

fn known_protocols() -> &'static [&'static str] {
    &[
        "aws.protocols#awsJson1_0",
        "aws.protocols#awsJson1_1",
        "aws.protocols#restJson1",
        "aws.protocols#awsQuery",
        "aws.protocols#ec2Query",
        "aws.protocols#restXml",
        "smithy.protocols#rpcv2Cbor",
    ]
}

fn add_resources(model: &ModelFile, out: &mut Vec<(CatalogKey, Value)>) {
    let values = model
        .shapes
        .iter()
        .filter(|(_, shape)| matches!(&shape.kind, ShapeKind::Resource))
        .map(|(id, _)| name(id))
        .collect::<Vec<_>>();
    out.push((CatalogKey::Resources, Value::List(values)));
}

pub(super) fn service_shape(model: &ModelFile) -> Result<(&str, &Shape), CompileError> {
    model
        .shapes
        .iter()
        .find(|(_, shape)| matches!(&shape.kind, ShapeKind::Service))
        .map(|(id, shape)| (id.as_str(), shape))
        .ok_or(CompileError::Kind)
}

fn add_service_strings(
    shape: &Shape,
    out: &mut Vec<(CatalogKey, Value)>,
) -> Result<(), CompileError> {
    add_api_service_strings(shape, out)?;
    add_signing_name(shape, out)
}

fn add_api_service_strings(
    shape: &Shape,
    out: &mut Vec<(CatalogKey, Value)>,
) -> Result<(), CompileError> {
    if let Some(service) = shape.traits.get("aws.api#service") {
        add_identity_strings(service, out)?;
        add_service_endpoint(shape, service, out)?;
    } else {
        add_trait_string(
            shape,
            "aws.api#endpointPrefix",
            CatalogKey::EndpointPrefix,
            out,
        )?;
    }
    Ok(())
}

fn add_identity_strings(
    service: &Json,
    out: &mut Vec<(CatalogKey, Value)>,
) -> Result<(), CompileError> {
    for (field, key) in [
        ("sdkId", CatalogKey::SdkId),
        ("arnNamespace", CatalogKey::ArnNamespace),
    ] {
        if let Some(value) = service.get(field) {
            out.push((key, Value::Text(as_text(value, field)?.to_owned())));
        }
    }
    Ok(())
}

fn add_service_endpoint(
    shape: &Shape,
    service: &Json,
    out: &mut Vec<(CatalogKey, Value)>,
) -> Result<(), CompileError> {
    if let Some(value) = service.get("endpointPrefix") {
        out.push((
            CatalogKey::EndpointPrefix,
            Value::Text(as_text(value, "aws.api#service.endpointPrefix")?.to_owned()),
        ));
        return Ok(());
    }
    add_trait_string(
        shape,
        "aws.api#endpointPrefix",
        CatalogKey::EndpointPrefix,
        out,
    )
}

fn add_signing_name(shape: &Shape, out: &mut Vec<(CatalogKey, Value)>) -> Result<(), CompileError> {
    let Some(value) = shape
        .traits
        .get("aws.auth#sigv4")
        .and_then(|trait_value| trait_value.get("name"))
    else {
        return Ok(());
    };
    out.push((
        CatalogKey::Sigv4Name,
        Value::Text(as_text(value, "aws.auth#sigv4.name")?.to_owned()),
    ));
    Ok(())
}

fn add_trait_string(
    shape: &Shape,
    id: &str,
    key: CatalogKey,
    out: &mut Vec<(CatalogKey, Value)>,
) -> Result<(), CompileError> {
    if let Some(value) = shape.traits.get(id) {
        out.push((key, Value::Text(as_text(value, id)?.to_owned())));
    }
    Ok(())
}

fn service_version(shape: &Shape) -> Result<&str, CompileError> {
    shape
        .version
        .as_deref()
        .ok_or_else(|| CompileError::Trait("service.version".to_owned()))
}

fn add_protocol_flags(shape: &Shape, out: &mut Vec<(CatalogKey, Value)>) {
    add_aws_query_compatible(shape, out);
    add_no_error_wrapping(shape, out);
}

fn add_aws_query_compatible(shape: &Shape, out: &mut Vec<(CatalogKey, Value)>) {
    if shape
        .traits
        .contains_key("aws.protocols#awsQueryCompatible")
    {
        out.push((CatalogKey::AwsQueryCompatible, Value::Bool(true)));
    }
}

fn add_no_error_wrapping(shape: &Shape, out: &mut Vec<(CatalogKey, Value)>) {
    let values = known_protocols()
        .iter()
        .filter(|id| {
            shape
                .traits
                .get(**id)
                .and_then(|value| value.get("noErrorWrapping"))
                .and_then(Json::as_bool)
                == Some(true)
        })
        .map(|id| Value::Text((*id).to_owned()))
        .collect::<Vec<_>>();
    if !values.is_empty() {
        out.push((CatalogKey::NoErrorWrapping, Value::List(values)));
    }
}

fn add_namespace(shape: &Shape, out: &mut Vec<(CatalogKey, Value)>) -> Result<(), CompileError> {
    let Some(namespace) = shape.traits.get("smithy.api#xmlNamespace") else {
        return Ok(());
    };
    let uri = namespace
        .get("uri")
        .and_then(Json::as_str)
        .ok_or_else(|| CompileError::Trait("smithy.api#xmlNamespace".to_owned()))?;
    let mut fields = vec![(CatalogKey::XmlUri, Value::Text(uri.to_owned()))];
    if let Some(prefix) = namespace.get("prefix") {
        let prefix = prefix
            .as_str()
            .ok_or_else(|| CompileError::Trait("smithy.api#xmlNamespace".to_owned()))?;
        fields.push((CatalogKey::XmlPrefix, Value::Text(prefix.to_owned())));
    }
    out.push((CatalogKey::XmlNamespace, map(fields)?));
    Ok(())
}

fn as_text<'a>(value: &'a Json, name: &str) -> Result<&'a str, CompileError> {
    value
        .as_str()
        .ok_or_else(|| CompileError::Trait(name.to_owned()))
}
