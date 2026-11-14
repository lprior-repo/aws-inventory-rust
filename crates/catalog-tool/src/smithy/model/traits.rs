use serde::{
    Deserializer,
    de::{IgnoredAny, MapAccess, Visitor},
};
use serde_json::Value;
use std::{collections::BTreeMap, fmt};

mod retained;
use retained::RETAIN_PAYLOADS;

pub(super) fn read<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, Value>, D::Error> {
    deserializer.deserialize_map(Traits)
}
struct Traits;
impl<'de> Visitor<'de> for Traits {
    type Value = BTreeMap<String, Value>;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a Smithy trait map")
    }
    fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
        std::iter::from_fn(|| next_entry(&mut access).transpose()).collect()
    }
}
fn next_entry<'de, A: MapAccess<'de>>(access: &mut A) -> Result<Option<(String, Value)>, A::Error> {
    let Some(name) = access.next_key::<String>()? else {
        return Ok(None);
    };
    let value = if retains_payload(&name) {
        access.next_value::<Value>()?
    } else {
        access.next_value::<IgnoredAny>()?;
        Value::Null
    };
    Ok(Some((name, value)))
}
fn retains_payload(name: &str) -> bool {
    RETAIN_PAYLOADS.contains(&name)
}
pub(super) fn validate_shape(shape: &super::Shape) -> Result<(), String> {
    validate_traits(&shape.traits)?;
    match &shape.kind {
        super::ShapeKind::Structure { members } | super::ShapeKind::Union { members } => members
            .values()
            .try_for_each(|member| validate_traits(&member.traits)),
        super::ShapeKind::List { member } => validate_traits(&member.traits),
        super::ShapeKind::Map { key, value } => {
            validate_traits(&key.traits)?;
            validate_traits(&value.traits)
        }
        _ => Ok(()),
    }
}

fn validate_traits(traits: &BTreeMap<String, Value>) -> Result<(), String> {
    traits
        .iter()
        .try_for_each(|(name, value)| validate_trait(name, value))
}

fn validate_trait(name: &str, value: &Value) -> Result<(), String> {
    let valid = validate_aws_trait(name, value)
        .or_else(|| validate_string_trait(name, value))
        .or_else(|| validate_wire_trait(name, value))
        .or_else(|| validate_scalar_trait(name, value))
        .or_else(|| validate_marker_trait(name, value))
        .or_else(|| validate_protocol_trait(name, value))
        .unwrap_or(true);
    if valid { Ok(()) } else { Err(name.to_owned()) }
}

fn validate_aws_trait(name: &str, value: &Value) -> Option<bool> {
    match name {
        "aws.api#arn" => Some(object_string(value, "template")),
        "aws.api#service" => Some(service_payload(value)),
        "aws.api#endpointPrefix" | "aws.api#arnNamespace" | "aws.protocols#ec2QueryName" => {
            Some(value.is_string())
        }
        "aws.auth#sigv4" => Some(object_string(value, "name")),
        _ => None,
    }
}

fn validate_string_trait(name: &str, value: &Value) -> Option<bool> {
    match name {
        "smithy.api#jsonName"
        | "smithy.api#xmlName"
        | "smithy.api#httpQuery"
        | "smithy.api#httpHeader"
        | "smithy.api#httpPrefixHeaders"
        | "smithy.api#mediaType" => Some(value.is_string()),
        "smithy.api#timestampFormat" => {
            Some(value.as_str().is_some_and(|format| {
                matches!(format, "date-time" | "http-date" | "epoch-seconds")
            }))
        }
        _ => None,
    }
}

fn validate_wire_trait(name: &str, value: &Value) -> Option<bool> {
    match name {
        "smithy.api#xmlNamespace" => {
            Some(object_string(value, "uri") && value.get("prefix").is_none_or(Value::is_string))
        }
        "smithy.api#http" => Some(http_payload(value)),
        "smithy.api#paginated" => Some(paginated_payload(value)),
        "smithy.api#requestCompression" => Some(request_compression_payload(value)),
        "smithy.api#endpoint" => Some(endpoint_payload(value)),
        "aws.protocols#awsQueryError" => Some(
            object_string(value, "code") && value.get("httpResponseCode").is_none_or(Value::is_i64),
        ),
        _ => None,
    }
}

fn validate_scalar_trait(name: &str, value: &Value) -> Option<bool> {
    match name {
        "smithy.api#httpError" => Some(value.as_i64().is_some_and(|n| (100..=599).contains(&n))),
        "smithy.api#error" => Some(
            value
                .as_str()
                .is_some_and(|kind| matches!(kind, "client" | "server")),
        ),
        "smithy.api#range" => Some(range_payload(value)),
        "smithy.api#retryable" => Some(
            value
                .as_object()
                .is_some_and(|fields| fields.get("throttling").is_none_or(Value::is_boolean)),
        ),
        _ => None,
    }
}

fn validate_marker_trait(name: &str, value: &Value) -> Option<bool> {
    match name {
        "smithy.api#readonly"
        | "smithy.api#required"
        | "smithy.api#sparse"
        | "smithy.api#streaming"
        | "smithy.api#sensitive"
        | "smithy.api#clientOptional"
        | "smithy.api#httpLabel"
        | "smithy.api#httpPayload"
        | "smithy.api#httpQueryParams"
        | "smithy.api#httpResponseCode"
        | "smithy.api#idempotencyToken"
        | "smithy.api#hostLabel"
        | "smithy.api#xmlFlattened"
        | "smithy.api#xmlAttribute"
        | "aws.customizations#s3UnwrappedXmlOutput"
        | "smithy.api#httpChecksumRequired" => Some(
            empty_object(value)
                || (name == "smithy.api#httpChecksumRequired" && value.is_boolean()),
        ),
        _ => None,
    }
}

fn validate_protocol_trait(name: &str, value: &Value) -> Option<bool> {
    match name {
        "aws.protocols#awsQueryCompatible" => Some(value.is_object()),
        "aws.protocols#restJson1"
        | "aws.protocols#awsJson1_0"
        | "aws.protocols#awsJson1_1"
        | "aws.protocols#awsQuery"
        | "aws.protocols#ec2Query"
        | "aws.protocols#restXml"
        | "smithy.protocols#rpcv2Cbor" => Some(protocol_payload(value)),
        _ => None,
    }
}

fn http_payload(value: &Value) -> bool {
    object_string(value, "method")
        && object_string(value, "uri")
        && value
            .get("code")
            .is_none_or(|code| code.as_i64().is_some_and(|n| (100..=599).contains(&n)))
}

fn range_payload(value: &Value) -> bool {
    value.as_object().is_some_and(|fields| {
        ["min", "max"]
            .iter()
            .all(|key| fields.get(*key).is_none_or(Value::is_number))
    })
}

fn empty_object(value: &Value) -> bool {
    value.as_object().is_some_and(serde_json::Map::is_empty)
}
fn protocol_payload(value: &Value) -> bool {
    value
        .as_object()
        .is_some_and(|fields| fields.get("noErrorWrapping").is_none_or(Value::is_boolean))
}
fn object_string(value: &Value, key: &str) -> bool {
    value.get(key).is_some_and(Value::is_string)
}
fn paginated_payload(value: &Value) -> bool {
    value.as_object().is_some_and(|fields| {
        ["inputToken", "outputToken", "items", "pageSize"]
            .iter()
            .all(|key| fields.get(*key).is_none_or(Value::is_string))
            && fields.get("additionalInputTokens").is_none_or(|tokens| {
                tokens
                    .as_array()
                    .is_some_and(|items| items.iter().all(Value::is_string))
            })
            && fields
                .get("stopOnDuplicateToken")
                .is_none_or(Value::is_boolean)
    })
}
fn request_compression_payload(value: &Value) -> bool {
    value.as_object().is_some_and(|fields| {
        fields
            .get("encodings")
            .and_then(Value::as_array)
            .is_some_and(|encodings| {
                !encodings.is_empty()
                    && encodings.iter().all(|encoding| {
                        encoding
                            .as_str()
                            .is_some_and(|name| name.eq_ignore_ascii_case("gzip"))
                    })
            })
    })
}
fn endpoint_payload(value: &Value) -> bool {
    value
        .as_object()
        .is_some_and(|fields| fields.get("hostPrefix").is_none_or(Value::is_string))
}
fn service_payload(value: &Value) -> bool {
    value.as_object().is_some_and(|fields| {
        [
            "sdkId",
            "endpointPrefix",
            "arnNamespace",
            "cloudFormationName",
            "cloudTrailEventSource",
            "cloudWatchEventSource",
        ]
        .iter()
        .all(|key| fields.get(*key).is_none_or(Value::is_string))
    })
}
#[cfg(test)]
#[path = "traits/tests.rs"]
mod tests;
