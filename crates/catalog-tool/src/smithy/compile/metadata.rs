use super::{CompileError, encode::map};
use crate::smithy::model::Member;
use inventory_core::{catalog::CatalogKey, record::Value};
use serde_json::Value as Json;
use std::collections::BTreeMap;

pub(super) fn shape_fields(traits: &BTreeMap<String, Json>) -> Result<Vec<(CatalogKey, Value)>, CompileError> {
    let mut fields = wire_fields(traits, false)?;
    text_field(traits, "smithy.api#xmlName", CatalogKey::ShapeXmlName, &mut fields)?;
    if let Some(namespace) = find(traits, "smithy.api#xmlNamespace") {
        push_text(&mut fields, CatalogKey::ShapeXmlNamespace, text_member(namespace, "uri", "smithy.api#xmlNamespace")?);
        optional_text_member(namespace, "prefix", "smithy.api#xmlNamespace", CatalogKey::ShapeXmlPrefix, &mut fields)?;
    }
    text_field(traits, "smithy.api#timestampFormat", CatalogKey::ShapeTimestampFormat, &mut fields)?;
    if let Some(kind) = find(traits, "smithy.api#error").and_then(Json::as_str) { push_text(&mut fields, CatalogKey::ErrorKind, kind); }
    if let Some(code) = find(traits, "smithy.api#httpError").and_then(Json::as_i64) { fields.push((CatalogKey::HttpError, Value::Int(code))); }
    if let Some(error) = find(traits, "aws.protocols#awsQueryError") { query_error(error, &mut fields)?; }
    if let Some(value) = find(traits, "smithy.api#retryable") { retryable(value, &mut fields); }
    Ok(fields)
}

pub(super) fn member_fields(member: &Member) -> Result<Vec<(CatalogKey, Value)>, CompileError> {
    wire_fields(&member.traits, true)
}

pub(super) fn pseudo_field(traits: &BTreeMap<String, Json>, key: CatalogKey) -> Result<Option<(CatalogKey, Value)>, CompileError> {
    let fields = wire_fields(traits, true)?;
    if fields.is_empty() { return Ok(None); }
    Ok(Some((key, map(fields)?)))
}

fn wire_fields(traits: &BTreeMap<String, Json>, member: bool) -> Result<Vec<(CatalogKey, Value)>, CompileError> {
    let mut fields = Vec::new();
    text_field(traits, "smithy.api#mediaType", CatalogKey::MediaType, &mut fields)?;
    if member {
        for (name, key) in [("smithy.api#jsonName", CatalogKey::JsonName), ("smithy.api#xmlName", CatalogKey::MemberXmlName), ("aws.api#ec2QueryName", CatalogKey::Ec2QueryName), ("smithy.api#timestampFormat", CatalogKey::TimestampFormat), ("smithy.api#httpQuery", CatalogKey::HttpQuery), ("smithy.api#httpHeader", CatalogKey::HttpHeader), ("smithy.api#httpPrefixHeaders", CatalogKey::HttpPrefixHeaders)] {
            text_field(traits, name, key, &mut fields)?;
        }
        member_markers(traits, &mut fields);
        namespace_fields(traits, &mut fields)?;
    } else {
        marker_field(traits, "smithy.api#sparse", CatalogKey::Sparse, &mut fields);
        marker_field(traits, "smithy.api#streaming", CatalogKey::Streaming, &mut fields);
        marker_field(traits, "smithy.api#sensitive", CatalogKey::Sensitive, &mut fields);
    }
    Ok(fields)
}

fn member_markers(traits: &BTreeMap<String, Json>, fields: &mut Vec<(CatalogKey, Value)>) {
    for (name, key) in [("smithy.api#httpLabel", CatalogKey::HttpLabel), ("smithy.api#httpQueryParams", CatalogKey::HttpQueryParams), ("smithy.api#httpPayload", CatalogKey::HttpPayload), ("smithy.api#httpResponseCode", CatalogKey::HttpResponseCode), ("smithy.api#hostLabel", CatalogKey::MemberHostLabel), ("smithy.api#xmlFlattened", CatalogKey::XmlFlattened), ("smithy.api#xmlAttribute", CatalogKey::XmlAttribute), ("smithy.api#idempotencyToken", CatalogKey::IdempotencyToken), ("smithy.api#sensitive", CatalogKey::Sensitive), ("smithy.api#sparse", CatalogKey::Sparse), ("smithy.api#streaming", CatalogKey::Streaming)] {
        marker_field(traits, name, key, fields);
    }
}

fn namespace_fields(traits: &BTreeMap<String, Json>, fields: &mut Vec<(CatalogKey, Value)>) -> Result<(), CompileError> {
    if let Some(namespace) = find(traits, "smithy.api#xmlNamespace") {
        push_text(fields, CatalogKey::MemberXmlUri, text_member(namespace, "uri", "smithy.api#xmlNamespace")?);
        optional_text_member(namespace, "prefix", "smithy.api#xmlNamespace", CatalogKey::MemberXmlPrefix, fields)?;
    }
    Ok(())
}

fn query_error(error: &Json, fields: &mut Vec<(CatalogKey, Value)>) -> Result<(), CompileError> {
    push_text(fields, CatalogKey::AwsQueryError, text_member(error, "code", "aws.protocols#awsQueryError")?);
    if let Some(status) = error.get("httpResponseCode").and_then(Json::as_i64) { fields.push((CatalogKey::AwsQueryStatus, Value::Int(status))); }
    Ok(())
}

fn retryable(value: &Json, fields: &mut Vec<(CatalogKey, Value)>) {
    fields.push((CatalogKey::ErrorRetryable, Value::Bool(true)));
    if value.get("throttling").and_then(Json::as_bool).unwrap_or(false) { fields.push((CatalogKey::ErrorThrottling, Value::Bool(true))); }
}

fn text_field(traits: &BTreeMap<String, Json>, name: &str, key: CatalogKey, fields: &mut Vec<(CatalogKey, Value)>) -> Result<(), CompileError> {
    if let Some(value) = find(traits, name) { push_text(fields, key, as_text(value, name)?); }
    Ok(())
}

fn marker_field(traits: &BTreeMap<String, Json>, name: &str, key: CatalogKey, fields: &mut Vec<(CatalogKey, Value)>) {
    if find(traits, name).is_some() { fields.push((key, Value::Bool(true))); }
}

fn optional_text_member(value: &Json, field: &str, name: &str, key: CatalogKey, out: &mut Vec<(CatalogKey, Value)>) -> Result<(), CompileError> {
    if let Some(value) = value.get(field) { push_text(out, key, as_text(value, name)?); }
    Ok(())
}

fn find<'a>(traits: &'a BTreeMap<String, Json>, id: &str) -> Option<&'a Json> {
    traits.get(id)
}

fn as_text<'a>(value: &'a Json, name: &str) -> Result<&'a str, CompileError> { value.as_str().ok_or_else(|| CompileError::Trait(name.to_owned())) }
fn text_member<'a>(value: &'a Json, key: &str, name: &str) -> Result<&'a str, CompileError> { value.get(key).and_then(Json::as_str).ok_or_else(|| CompileError::Trait(name.to_owned())) }
fn push_text(fields: &mut Vec<(CatalogKey, Value)>, key: CatalogKey, value: &str) { fields.push((key, Value::Text(value.to_owned()))); }
