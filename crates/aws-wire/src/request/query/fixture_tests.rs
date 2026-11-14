use super::encode;
use crate::request::{CompressionPolicy, Protocol, RequestContext, RequestLimits};
use catalog_tool::smithy::{Registry, build_catalog};
use flate2::read::GzDecoder;
use inventory_core::{catalog::{Catalog, Operation, Service, Target}, record::{Float, Key, Record, Time, Value}};
use serde_json::{Map as JsonMap, Value as Json};
use std::{collections::{BTreeMap, BTreeSet}, error::Error, fs, io::Read};

const AWS_QUERY: &str = include_str!("../../../tests/fixtures/protocol-tests/awsQuery.json");
const EC2_QUERY: &str = include_str!("../../../tests/fixtures/protocol-tests/ec2Query.json");
const AUTO_TOKEN: &str = "00000000-0000-4000-8000-000000000000";
type FixtureResult<T> = Result<T, Box<dyn Error>>;

fn fixture_error(reason: impl Into<String>) -> Box<dyn Error> {
    Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, reason.into()))
}

#[test]
fn published_aws_query_request_cases_are_encoded_or_accounted_for_as_server_cases() -> FixtureResult<()> {
    check_fixture(AWS_QUERY, "awsQuery", Protocol::AwsQuery)
}

#[test]
fn published_ec2_query_request_cases_are_encoded_or_accounted_for_as_server_cases() -> FixtureResult<()> {
    check_fixture(EC2_QUERY, "ec2Query", Protocol::Ec2Query)
}

fn check_fixture(source: &str, model_dir: &str, protocol: Protocol) -> FixtureResult<()> {
    let fixture: Json = serde_json::from_str(source)?;
    let shape_map = fixture.get("shapes").and_then(Json::as_object)
        .ok_or_else(|| fixture_error("fixture shapes absent"))?;
    let registry = fixture_registry(&fixture)?;
    let directory = tempfile::tempdir()?;
    let file = directory.path().join("models").join(model_dir).join("fixture.json");
    fs::create_dir_all(file.parent().ok_or_else(|| fixture_error("fixture model path absent"))?)?;
    fs::write(file, source)?;
    let catalog_bytes = build_catalog(directory.path(), &registry, "protocol-fixture")?;
    let catalog = Catalog::borrow(&catalog_bytes)?;
    let service = catalog.services()?.find_map(|entry| {
        entry.ok().filter(|service| service.name().ok() == Some(model_dir))
    }).ok_or_else(|| fixture_error(format!("fixture service {model_dir} absent")))?;

    let mut seen = BTreeSet::new();
    let mut unsupported = BTreeMap::new();
    for (shape_id, shape) in shape_map {
        if shape.get("type").and_then(Json::as_str) != Some("operation") { continue; }
        let Some(cases) = shape.get("traits").and_then(|traits| traits.get("smithy.test#httpRequestTests")).and_then(Json::as_array) else { continue; };
        let operation_name = shape_id.rsplit('#').next().unwrap_or(shape_id);
        let operation = find_operation(service, operation_name)?;
        for case in cases {
            let id = case.get("id").and_then(Json::as_str)
                .ok_or_else(|| fixture_error("request fixture without id"))?;
            assert!(seen.insert(id.to_owned()), "duplicate request fixture {id}");
            match case.get("appliesTo").and_then(Json::as_str).unwrap_or("both") {
                "client" | "both" => {}
                "server" => {
                    unsupported.insert(id.to_owned(), "published test explicitly applies to server behavior");
                    continue;
                }
                _ => return Err(fixture_error(format!("{id}: invalid appliesTo value"))),
            }
            let params = case.get("params").cloned().unwrap_or_else(|| Json::Object(JsonMap::new()));
            let input = convert(service, &registry, operation.input()?, &params)?;
            let record = Record::encode(&input)?;
            let auto_token = id.contains("IdempotencyTokenAutoFill") && !id.ends_with("IsSet");
            let context = RequestContext {
                service, operation, protocol, input: record.root(), endpoint: "https://example.com",
                limits: RequestLimits::default(), idempotency_token: auto_token,
                compression: CompressionPolicy::default(),
            };
            let request = encode(context).map_err(|error| fixture_error(format!("{id}: {error}")))?;
            if operation.request_compression()?.is_none() {
                assert!(request.headers().get(http::header::CONTENT_ENCODING).is_none(), "{id} absent compression metadata");
            }
            assert_eq!(request.method().as_str(), case.get("method").and_then(Json::as_str).unwrap_or("POST"), "{id}");
            assert_eq!(request.uri().path(), case.get("uri").and_then(Json::as_str).unwrap_or("/"), "{id}");
            let actual_bytes = decode_request_body(&request)?;
            let actual_body = std::str::from_utf8(&actual_bytes)?;
            let expected_body = match case.get("body").and_then(Json::as_str) {
                Some(body) => body.to_owned(),
                None => {
                    if operation.request_compression()?.is_none() {
                        return Err(fixture_error(format!("{id}: absent body requires modeled compression")));
                    }
                    let mut uncompressed_context = context;
                    uncompressed_context.compression = CompressionPolicy::Disabled;
                    let uncompressed = encode(uncompressed_context)?;
                    assert!(request.headers().get(http::header::CONTENT_ENCODING).is_some(), "{id} expected compressed request");
                    assert_compression_boundaries(context, uncompressed_context, &uncompressed)?;
                    std::str::from_utf8(uncompressed.body())?.to_owned()
                }
            };
            let mut actual_pairs = actual_body.split('&').collect::<Vec<_>>();
            let mut expected_pairs = expected_body.split('&').collect::<Vec<_>>();
            actual_pairs.sort_unstable();
            expected_pairs.sort_unstable();
            assert_eq!(actual_pairs, expected_pairs, "{id}");
            let exact_order = matches!(id,
                "QuerySimpleQueryMaps" | "QuerySimpleQueryMapsWithXmlName" | "QueryComplexQueryMaps"
                    | "QueryQueryMapWithMemberXmlName" | "QueryFlattenedQueryMaps"
                    | "QueryFlattenedQueryMapsWithXmlName" | "QueryQueryMapOfLists" | "QueryNestedStructWithMap"
                    | "QueryLists" | "NestedStructures" | "FlattenedQueryLists" | "QueryListArgWithXmlNameMember"
                    | "QueryFlattenedListArgWithXmlName" | "QueryNestedStructWithList" | "QueryXmlLists"
                    | "Ec2NestedStructures" | "Ec2Lists" | "Ec2ListArgWithXmlNameMember"
                    | "Ec2ListMemberWithXmlName" | "Ec2ListNestedStructWithList" | "Ec2XmlLists");
            if exact_order { assert_eq!(actual_body, expected_body, "{id} wire ordering"); }
            let envelope = format!("Action={operation_name}&Version=2020-01-08");
            assert!(actual_body.starts_with(&envelope), "{id} action/version order");
            let expected_headers = case.get("headers").and_then(Json::as_object);
            for (name, value) in request.headers() {
                let actual = value.to_str()?;
                if name == http::header::CONTENT_ENCODING
                    && operation.request_compression()?.is_some()
                {
                    assert_eq!(actual, "gzip", "{id} modeled request compression");
                    continue;
                }
                if name == http::header::HOST || name == http::header::CONTENT_LENGTH {
                    continue;
                }
                let expected = expected_headers.and_then(|headers| headers.get(name.as_str()))
                    .and_then(Json::as_str)
                    .ok_or_else(|| fixture_error(format!("{id}: unexpected header {name}")))?;
                assert_eq!(actual, expected, "{id} header {name}");
            }
            if let Some(headers) = expected_headers {
                for (name, value) in headers {
                    let expected = value.as_str()
                        .ok_or_else(|| fixture_error(format!("{id}: non-string expected header")))?;
                    assert_eq!(request.headers().get(name.as_str()).and_then(|value| value.to_str().ok()), Some(expected), "{id} header {name}");
                }
            }
            let content_length = request.headers().get(http::header::CONTENT_LENGTH)
                .and_then(|value| value.to_str().ok());
            let expected_length = request.body().len().to_string();
            assert_eq!(content_length, Some(expected_length.as_str()), "{id} Content-Length");
        }
    }
    assert_query_limits(service, protocol, &registry)?;
    assert_eq!(seen.len(), shape_map.values().filter(|shape| shape.get("type").and_then(Json::as_str) == Some("operation"))
        .flat_map(|shape| shape.get("traits").and_then(|traits| traits.get("smithy.test#httpRequestTests")).and_then(Json::as_array).into_iter().flatten())
        .count(), "all published request cases must be accounted for");
    for (id, reason) in &unsupported {
        assert!(!reason.is_empty(), "{id} lacks a concrete unsupported reason");
    }
    Ok(())
}
fn decode_request_body(request: &http::Request<bytes::Bytes>) -> FixtureResult<Vec<u8>> {
    let encodings = request.headers().get_all(http::header::CONTENT_ENCODING);
    let is_gzip = encodings.iter().any(|value| value.to_str().ok() == Some("gzip"));
    if !is_gzip {
        return Ok(request.body().to_vec());
    }
    let mut decoded = Vec::new();
    GzDecoder::new(request.body().as_ref())
        .take((RequestLimits::default().body_bytes() as u64) + 1)
        .read_to_end(&mut decoded)?;
    if decoded.len() > RequestLimits::default().body_bytes() {
        return Err(fixture_error("decompressed request exceeds body limit"));
    }
    Ok(decoded)
}

fn assert_compression_boundaries(
    context: RequestContext<'_>,
    plain_context: RequestContext<'_>,
    plain_request: &http::Request<bytes::Bytes>,
) -> FixtureResult<()> {
    let body = plain_request.body().to_vec();
    let disabled_body = body.clone();
    let disabled_pointer = disabled_body.as_ptr();
    let mut disabled_headers = plain_request.headers().clone();
    let disabled_context = RequestContext { compression: CompressionPolicy::Disabled, ..context };
    let unchanged = crate::request::compression::apply(disabled_context, &mut disabled_headers, disabled_body)?;
    assert_eq!(unchanged.as_ptr(), disabled_pointer);
    assert_eq!(disabled_headers, *plain_request.headers());

    let above_threshold = body.len().checked_add(1).ok_or_else(|| fixture_error("fixture body length overflow"))?;
    let above_context = RequestContext {
        compression: CompressionPolicy::Enabled(crate::request::CompressionThreshold::new(above_threshold)?),
        ..plain_context
    };
    let mut above_headers = plain_request.headers().clone();
    assert_eq!(crate::request::compression::apply(above_context, &mut above_headers, body.clone())?, body);
    assert_eq!(above_headers, *plain_request.headers());

    let exact_context = RequestContext {
        compression: CompressionPolicy::Enabled(crate::request::CompressionThreshold::new(body.len())?),
        ..plain_context
    };
    let mut exact_headers = plain_request.headers().clone();
    let exact_body = crate::request::compression::apply(exact_context, &mut exact_headers, body.clone())?;
    assert_eq!(decode_header_body(&exact_headers, &exact_body)?, body);
    let expected_length = exact_body.len().to_string();
    assert_eq!(
        exact_headers.get(http::header::CONTENT_LENGTH).and_then(|value| value.to_str().ok()),
        Some(expected_length.as_str()),
    );
    let existing_header_bytes = crate::request::common::check_headers(
        plain_request.headers(),
        RequestLimits::default().header_bytes(),
    )?;
    let header_limit = existing_header_bytes.checked_add(1)
        .ok_or_else(|| fixture_error("fixture header size overflow"))?;
    let limits = RequestLimits::new(
        RequestLimits::default().body_bytes(),
        header_limit,
        RequestLimits::default().uri_bytes(),
    )?;
    let constrained_context = RequestContext { limits, ..exact_context };
    let mut constrained_headers = plain_request.headers().clone();
    assert!(matches!(
        crate::request::compression::apply(constrained_context, &mut constrained_headers, body.clone()),
        Err(crate::request::EncodeError::Limit("headers")),
    ));
    assert_eq!(constrained_headers, *plain_request.headers());

    let mut appended_headers = plain_request.headers().clone();
    appended_headers.insert(http::header::CONTENT_ENCODING, http::HeaderValue::from_static("br"));
    let appended_body = crate::request::compression::apply(exact_context, &mut appended_headers, body.clone())?;
    let encodings = appended_headers.get_all(http::header::CONTENT_ENCODING).iter()
        .map(|value| value.to_str()).collect::<Result<Vec<_>, _>>()?;
    assert_eq!(encodings, ["br", "gzip"]);
    assert_eq!(decode_header_body(&appended_headers, &appended_body)?, body);
    Ok(())
}

fn decode_header_body(
    headers: &http::HeaderMap,
    body: &[u8],
) -> FixtureResult<Vec<u8>> {
    let mut decoded = Vec::new();
    if headers.get_all(http::header::CONTENT_ENCODING).iter()
        .any(|value| value.to_str().ok() == Some("gzip"))
    {
        GzDecoder::new(body)
            .take((RequestLimits::default().body_bytes() as u64) + 1)
            .read_to_end(&mut decoded)?;
        if decoded.len() > RequestLimits::default().body_bytes() {
            return Err(fixture_error("decompressed boundary body exceeds body limit"));
        }
        return Ok(decoded);
    }
    Ok(body.to_vec())
}


fn assert_query_limits(service: Service<'_>, protocol: Protocol, registry: &Registry) -> FixtureResult<()> {
    let empty = Value::Map(BTreeMap::new());
    let record = Record::encode(&empty)?;
    let operation = find_operation(service, "EmptyInputAndEmptyOutput")?;
    let limits = RequestLimits::new(8, 1024, 1024)?;
    let context = RequestContext {
        service, operation, protocol, input: record.root(), endpoint: "https://example.com",
        limits, idempotency_token: None,
        compression: CompressionPolicy::default(),
    };
    assert!(matches!(encode(context), Err(crate::request::EncodeError::Limit("body"))));

    let operation = find_list_operation(service)?;
    let Target::Shape(index) = operation.input()? else {
        return Err(fixture_error("list operation input is not a structure"));
    };
    let input_shape = service.shape(index)?.ok_or_else(|| fixture_error("list operation input shape absent"))?;
    let mut params = JsonMap::new();
    for item in input_shape.members()? {
        let member = item?;
        if let Target::Shape(list_index) = member.target {
            if let Some(shape) = service.shape(list_index)? {
                if shape.kind()? == "list" {
                    let name = service.field_name(member.id)?;
                    params.insert(name.to_owned(), Json::Array(vec![Json::String("x".to_owned()); 4_097]));
                    break;
                }
            }
        }
    }
    assert!(!params.is_empty(), "QueryLists fixture has no list member");
    let input = convert(service, registry, operation.input()?, &Json::Object(params))?;
    let record = Record::encode(&input)?;
    let context = RequestContext {
        service, operation, protocol, input: record.root(), endpoint: "https://example.com",
        limits: RequestLimits::default(), idempotency_token: None,
        compression: CompressionPolicy::default(),
    };
    assert!(matches!(encode(context), Err(crate::request::EncodeError::Limit("query traversal"))));
    Ok(())
}

fn fixture_registry(root: &Json) -> FixtureResult<Registry> {
    let registry = Registry::parse(include_str!("../../../../../catalog/field-ids.txt"))?;
    let mut names = BTreeSet::new();
    let mut pending = vec![root];
    while let Some(value) = pending.pop() {
        match value {
            Json::Object(map) => {
                if let Some(members) = map.get("members").and_then(Json::as_object) {
                    names.extend(members.keys().cloned());
                }
                pending.extend(map.values());
            }
            Json::Array(values) => pending.extend(values),
            _ => {}
        }
    }
    Ok(registry.extend(names)?)
}

fn find_operation(service: Service<'_>, name: &str) -> FixtureResult<Operation<'_>> {
    for entry in service.operations()? {
        let operation = entry?;
        if operation.name()? == name { return Ok(operation); }
    }
    Err(fixture_error(format!("operation {name} absent")))
}

fn find_list_operation(service: Service<'_>) -> FixtureResult<Operation<'_>> {
    for item in service.operations()? {
        let operation = item?;
        let Target::Shape(index) = operation.input()? else { continue; };
        let Some(shape) = service.shape(index)? else { continue; };
        for member in shape.members()? {
            let member = member?;
            if let Target::Shape(index) = member.target {
                if let Some(shape) = service.shape(index)? {
                    if shape.kind()? == "list" { return Ok(operation); }
                }
            }
        }
    }
    Err(fixture_error("service has no list input operation"))
}

fn convert(service: Service<'_>, registry: &Registry, target: Target, json: &Json) -> FixtureResult<Value> {
    match target {
        Target::Prelude(kind) => scalar_value(kind, json),
        Target::Shape(index) => {
            let shape = service.shape(index)?.ok_or_else(|| fixture_error(format!("shape {index} absent")))?;
            match shape.kind()? {
                "structure" | "union" => {
                    let object = json.as_object().ok_or_else(|| fixture_error("expected structure value"))?;
                    let mut fields = BTreeMap::new();
                    for item in shape.members()? {
                        let member = item?;
                        let name = service.field_name(member.id)?;
                        if let Some(value) = object.get(name) {
                            let id = registry.id(name).ok_or_else(|| fixture_error(format!("registry missing member {name}")))?;
                            fields.insert(Key::Field(id), convert(service, registry, member.target, value)?);
                        }
                    }
                    Ok(Value::Map(fields))
                }
                "list" => {
                    let items = json.as_array().ok_or_else(|| fixture_error("expected list value"))?;
                    let target = shape.element()?.ok_or_else(|| fixture_error("list target absent"))?;
                    items.iter().map(|item| convert(service, registry, target, item)).collect::<FixtureResult<Vec<_>>>()
                        .map(Value::List)
                }
                "map" => {
                    let object = json.as_object().ok_or_else(|| fixture_error("expected map value"))?;
                    let target = shape.map_value()?.ok_or_else(|| fixture_error("map value target absent"))?;
                    let mut entries = BTreeMap::new();
                    for (key, value) in object {
                        entries.insert(Key::Name(key.clone()), convert(service, registry, target, value)?);
                    }
                    Ok(Value::Map(entries))
                }
                kind => scalar_value(kind, json),
            }
        }
    }
}

fn scalar_value(kind: &str, json: &Json) -> FixtureResult<Value> {
    match kind {
        "string" | "enum" | "bigDecimal" => json.as_str().map(|value| Value::Text(value.to_owned()))
            .ok_or_else(|| fixture_error("expected text scalar")),
        "boolean" => json.as_bool().map(Value::Bool).ok_or_else(|| fixture_error("expected boolean scalar")),
        "byte" | "short" | "integer" | "long" | "bigInteger" | "intEnum" => json.as_i64().map(Value::Int)
            .ok_or_else(|| fixture_error("expected integer scalar")),
        "float" | "double" => {
            let value = json.as_f64().or_else(|| match json.as_str()? { "NaN" => Some(f64::NAN), "Infinity" => Some(f64::INFINITY), "-Infinity" => Some(f64::NEG_INFINITY), _ => None })
                .ok_or_else(|| fixture_error("expected float scalar"))?;
            Ok(Value::Float(Float::new(value)))
        }
        "timestamp" => {
            let seconds = json.as_i64().ok_or_else(|| fixture_error("expected integer timestamp"))?;
            Ok(Value::Time(Time::new(seconds, 0)?))
        }
        "blob" => Ok(Value::Bytes(decode_base64(json.as_str().ok_or_else(|| fixture_error("expected base64 blob"))?)?)),
        _ => Err(fixture_error(format!("unsupported fixture target {kind}"))),
    }
}

fn decode_base64(value: &str) -> FixtureResult<Vec<u8>> {
    let mut output = Vec::new();
    let mut block = [0u8; 4];
    let mut count = 0usize;
    for byte in value.bytes().filter(|byte| !byte.is_ascii_whitespace()) {
        let slot = block.get_mut(count).ok_or_else(|| fixture_error("invalid base64 fixture length"))?;
        *slot = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => 64,
            _ => return Err(fixture_error("invalid base64 fixture")),
        };
        count += 1;
        if count == 4 {
            output.push((block[0] << 2) | (block[1] >> 4));
            if block[2] < 64 { output.push((block[1] << 4) | (block[2] >> 2)); }
            if block[3] < 64 { output.push((block[2] << 6) | block[3]); }
            block = [0; 4];
            count = 0;
        }
    }
    if count != 0 { return Err(fixture_error("invalid base64 fixture length")); }
    Ok(output)
}
