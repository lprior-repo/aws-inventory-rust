use super::*;

pub(super) type TestResult = Result<(), Box<dyn std::error::Error>>;

#[derive(Clone, Copy)]
pub(super) struct Fixture<'a> {
    pub(super) source: &'a str,
    pub(super) protocol: Protocol,
    pub(super) name: &'static str,
    pub(super) expected_cases: usize,
}

impl<'a> Fixture<'a> {
    pub(super) fn new(
        source: &'a str,
        protocol: Protocol,
        name: &'static str,
        expected_cases: usize,
    ) -> Self {
        Self {
            source,
            protocol,
            name,
            expected_cases,
        }
    }

    pub(super) fn index(self) -> Option<usize> {
        match self.name {
            "awsJson1_0" => Some(0),
            "awsJson1_1" => Some(1),
            "awsQuery" => Some(2),
            "ec2Query" => Some(3),
            "restJson1" => Some(4),
            "restXml" => Some(5),
            "rpcv2Cbor" => Some(6),
            _ => None,
        }
    }
}

pub(super) fn compile_service_model(
    fixture: &Value,
    service_id: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let shapes = fixture["shapes"].as_object().ok_or("fixture shapes")?;
    let service_name = service_id.rsplit('#').next().ok_or("service name")?;
    let mut selected = serde_json::Map::new();
    let mut pending = vec![service_id.to_owned()];
    while let Some(id) = pending.pop() {
        if selected.contains_key(&id) {
            continue;
        }
        if let Some(shape) = shapes.get(&id) {
            collect_targets(shape, &mut pending);
            selected.insert(id, shape.clone());
        }
    }
    let member_names = selected
        .values()
        .filter_map(|shape| shape["members"].as_object())
        .flat_map(|members| members.keys().cloned())
        .collect::<BTreeSet<_>>();
    let mut registry_text = member_names.into_iter().collect::<Vec<_>>().join("\n");
    if !registry_text.is_empty() {
        registry_text.push('\n');
    }
    let registry = Registry::parse(&registry_text)?;
    let root = tempfile::tempdir()?;
    let model_dir = root
        .path()
        .join("models")
        .join(service_name.replace('.', "-"));
    std::fs::create_dir_all(&model_dir)?;
    let model = serde_json::to_vec(&json!({ "shapes": selected }))?;
    std::fs::write(model_dir.join("model.json"), model)?;
    Ok(build_catalog(root.path(), &registry, "error-fixture")?)
}

fn collect_targets(value: &Value, pending: &mut Vec<String>) {
    match value {
        Value::Object(values) => {
            for (name, value) in values {
                if name == "target" {
                    if let Some(target) = value.as_str() {
                        pending.push(target.to_owned());
                    }
                } else {
                    collect_targets(value, pending);
                }
            }
        }
        Value::Array(values) => values
            .iter()
            .for_each(|value| collect_targets(value, pending)),
        _ => {}
    }
}

pub(super) fn fixture_operation_for_error<'a>(
    shapes: &serde_json::Map<String, Value>,
    service_shape: &'a Value,
    error_shape_id: &str,
) -> Result<Option<&'a str>, Box<dyn std::error::Error>> {
    for target in service_shape
        .get("operations")
        .and_then(Value::as_array)
        .ok_or("service operations")?
    {
        let operation_id = target["target"].as_str().ok_or("operation target")?;
        let operation_errors =
            shapes.get(operation_id).ok_or("operation shape")?["errors"].as_array();
        if operation_errors.is_some_and(|errors| {
            errors
                .iter()
                .any(|error| error["target"].as_str() == Some(error_shape_id))
        }) {
            return Ok(Some(operation_id));
        }
    }
    Ok(None)
}

pub(super) fn fixture_body(
    case: &Value,
    protocol: Protocol,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let body = match case.get("body") {
        Some(body) => body.as_str().ok_or("fixture response body")?,
        None => "",
    };
    if protocol == Protocol::RpcV2Cbor {
        use base64::Engine;
        return Ok(base64::engine::general_purpose::STANDARD.decode(body)?);
    }
    Ok(body.as_bytes().to_vec())
}

pub(super) fn fixture_status(case: &Value) -> Result<http::StatusCode, Box<dyn std::error::Error>> {
    let code = u16::try_from(case["code"].as_u64().ok_or("fixture HTTP status")?)?;
    Ok(http::StatusCode::from_u16(code)?)
}

pub(super) fn fixture_headers(case: &Value) -> Result<http::HeaderMap, Box<dyn std::error::Error>> {
    let mut headers = http::HeaderMap::new();
    if let Some(values) = case["headers"].as_object() {
        for (name, value) in values {
            headers.insert(
                http::header::HeaderName::from_bytes(name.as_bytes())?,
                http::header::HeaderValue::from_bytes(
                    value.as_str().ok_or("fixture header value")?.as_bytes(),
                )?,
            );
        }
    }
    Ok(headers)
}

pub(super) fn expected_fixture_code(
    shape_id: &str,
    modeled_error: inventory_core::catalog::ModeledError<'_>,
    headers: &http::HeaderMap,
) -> Result<String, Box<dyn std::error::Error>> {
    for name in ["x-amzn-errortype", "x-amzn-query-error"] {
        if let Some(value) = headers.get(name) {
            let raw = value.to_str()?.trim();
            let namespace = raw.split_once('#').map_or(raw, |(_, suffix)| suffix);
            let name = namespace
                .split_once(';')
                .map_or(namespace, |(name, _)| name);
            return Ok(name
                .split_once(':')
                .map_or(name, |(name, _)| name)
                .to_owned());
        }
    }
    match modeled_error.code {
        Some(code) => Ok(code.to_owned()),
        None => Ok(shape_id
            .rsplit('#')
            .next()
            .ok_or("error shape name")?
            .to_owned()),
    }
}

pub(super) fn expected_fixture_class(shape_id: &str) -> ErrorClass {
    match shape_id {
        "aws.protocoltests.json10#FooError"
        | "aws.protocoltests.json#FooError"
        | "aws.protocoltests.restjson#FooError" => ErrorClass::Retryable,
        "aws.protocoltests.restjson#ComplexError" | "aws.protocoltests.restxml#ComplexError" => {
            ErrorClass::AccessDenied
        }
        _ => ErrorClass::Fatal,
    }
}

pub(super) fn modeled_error_fixture() -> Value {
    json!({
        "shapes": {
            "example#ErrorService": {
                "type": "service",
                "version": "1",
                "traits": {"aws.protocols#awsJson1_1": {}},
                "operations": [{"target": "example#Get"}]
            },
            "example#Get": {
                "type": "operation",
                "input": {"target": "example#Input"},
                "output": {"target": "example#Output"},
                "errors": [
                    {"target": "example#ThrottleError"},
                    {"target": "example#RetryableFailure"},
                    {"target": "example#ExpiredToken"},
                    {"target": "example#ExpiredSignature"},
                    {"target": "example#SignatureDoesNotMatch"},
                    {"target": "example#InvalidSignatureException"}
                ]
            },
            "example#Input": {
                "type": "structure",
                "members": {"value": {"target": "smithy.api#String"}}
            },
            "example#Output": {
                "type": "structure",
                "members": {"value": {"target": "smithy.api#String"}}
            },
            "example#ThrottleError": {
                "type": "structure",
                "members": {},
                "traits": {
                    "smithy.api#error": "client",
                    "smithy.api#httpError": 400,
                    "smithy.api#retryable": {"throttling": true}
                }
            },
            "example#RetryableFailure": {
                "type": "structure",
                "members": {},
                "traits": {
                    "smithy.api#error": "client",
                    "smithy.api#httpError": 400,
                    "smithy.api#retryable": {}
                }
            },
            "example#ExpiredToken": {
                "type": "structure",
                "members": {},
                "traits": {"smithy.api#error": "client", "smithy.api#httpError": 403}
            },
            "example#ExpiredSignature": {
                "type": "structure",
                "members": {},
                "traits": {"smithy.api#error": "client", "smithy.api#httpError": 403}
            },
            "example#SignatureDoesNotMatch": {
                "type": "structure",
                "members": {},
                "traits": {"smithy.api#error": "client", "smithy.api#httpError": 403}
            },
            "example#InvalidSignatureException": {
                "type": "structure",
                "members": {},
                "traits": {"smithy.api#error": "client", "smithy.api#httpError": 403}
            }
        }
    })
}
