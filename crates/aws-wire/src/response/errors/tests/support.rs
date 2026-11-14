use super::*;
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

    pub(super) fn index(self) -> usize {
        match self.name {
            "awsJson1_0" => 0,
            "awsJson1_1" => 1,
            "awsQuery" => 2,
            "ec2Query" => 3,
            "restJson1" => 4,
            "restXml" => 5,
            "rpcv2Cbor" => 6,
            _ => panic!("unknown fixture"),
        }
    }
}

pub(super) fn compile_service_model(fixture: &Value, service_id: &str) -> Vec<u8> {
    let shapes = fixture["shapes"].as_object().expect("fixture shapes");
    let service_name = service_id.rsplit('#').next().expect("service name");
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
    let registry = Registry::parse(&registry_text).expect("fixture registry");
    let root = tempfile::tempdir().expect("temporary model root");
    let model_dir = root
        .path()
        .join("models")
        .join(service_name.replace('.', "-"));
    std::fs::create_dir_all(&model_dir).expect("model directory");
    let model = serde_json::to_vec(&json!({ "shapes": selected })).expect("model JSON");
    std::fs::write(model_dir.join("model.json"), model).expect("model file");
    build_catalog(root.path(), &registry, "error-fixture").expect("compiled fixture catalog")
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
) -> Option<&'a str> {
    for target in service_shape.get("operations")?.as_array()? {
        let operation_id = target["target"].as_str().expect("operation target");
        let operation_errors = shapes[operation_id]["errors"].as_array();
        if operation_errors.is_some_and(|errors| {
            errors
                .iter()
                .any(|error| error["target"].as_str() == Some(error_shape_id))
        }) {
            return Some(operation_id);
        }
    }
    None
}

pub(super) fn fixture_body(case: &Value, protocol: Protocol) -> Vec<u8> {
    let body = case
        .get("body")
        .map_or("", |body| body.as_str().expect("fixture response body"));
    if protocol == Protocol::RpcV2Cbor {
        use base64::Engine;
        return base64::engine::general_purpose::STANDARD
            .decode(body)
            .expect("valid CBOR fixture body");
    }
    body.as_bytes().to_vec()
}

pub(super) fn fixture_status(case: &Value) -> http::StatusCode {
    http::StatusCode::from_u16(
        u16::try_from(case["code"].as_u64().expect("fixture HTTP status"))
            .expect("fixture status fits u16"),
    )
    .expect("valid fixture HTTP status")
}

pub(super) fn fixture_headers(case: &Value) -> http::HeaderMap {
    let mut headers = http::HeaderMap::new();
    if let Some(values) = case["headers"].as_object() {
        for (name, value) in values {
            headers.insert(
                http::header::HeaderName::from_bytes(name.as_bytes())
                    .expect("valid fixture header name"),
                http::header::HeaderValue::from_bytes(
                    value.as_str().expect("fixture header value").as_bytes(),
                )
                .expect("valid fixture header value"),
            );
        }
    }
    headers
}

pub(super) fn expected_fixture_code(
    shape_id: &str,
    modeled_error: inventory_core::catalog::ModeledError<'_>,
    headers: &http::HeaderMap,
) -> String {
    for name in ["x-amzn-errortype", "x-amzn-query-error"] {
        if let Some(value) = headers.get(name) {
            let raw = value.to_str().expect("fixture code header").trim();
            let namespace = raw.split_once('#').map_or(raw, |(_, suffix)| suffix);
            let name = namespace
                .split_once(';')
                .map_or(namespace, |(name, _)| name);
            return name
                .split_once(':')
                .map_or(name, |(name, _)| name)
                .to_owned();
        }
    }
    modeled_error.code.map(str::to_owned).unwrap_or_else(|| {
        shape_id
            .rsplit('#')
            .next()
            .expect("error shape name")
            .to_owned()
    })
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
