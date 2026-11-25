use super::*;
pub(super) fn page_size(service: &str, operation: &str) -> TestResult<usize> {
    Ok(match (service, operation) {
        (
            "evs",
            "ListEnvironments"
            | "ListEnvironmentHosts"
            | "ListEnvironmentConnectors"
            | "ListEnvironmentVlans",
        ) => 100,
        (
            "deadline",
            "ListFarms"
            | "ListFleets"
            | "ListQueues"
            | "ListLimits"
            | "ListBudgets"
            | "ListStorageProfiles"
            | "ListQueueEnvironments"
            | "ListQueueFleetAssociations"
            | "ListQueueLimitAssociations"
            | "ListVolumes",
        ) => 100,
        (
            "entityresolution",
            "ListIdMappingWorkflows"
            | "ListMatchingWorkflows"
            | "ListIdMappingJobs"
            | "ListMatchingJobs",
        ) => 25,
        (
            "fis",
            "ListExperimentTemplates" | "ListExperiments" | "ListTargetAccountConfigurations",
        ) => 100,
        ("frauddetector", "GetDetectors") => 10,
        ("datazone", "ListDomains") => 25,
        ("datazone", "ListProjectProfiles") => 50,
        ("secrets-manager", "ListSecrets") => 100,
        _ => return Err(format!("unknown page size for {service}.{operation}").into()),
    })
}
pub(super) fn json_target(service: &str) -> TestResult<&'static str> {
    match service {
        "evs" => Ok("AmazonElasticVMwareService"),
        "frauddetector" => Ok("AWSHawksNestServiceFacade"),
        "secrets-manager" => Ok("secretsmanager"),
        _ => Err(format!("service {service} is not AWS JSON POST protocol").into()),
    }
}
pub(super) fn is_rpc(service: &str) -> bool {
    service == "fms"
}
pub(super) fn build_rpc_expected(
    service: &str,
    operation: &str,
    second: bool,
    parent_id: Option<&str>,
) -> TestResult<Json> {
    let mut fields = serde_json::Map::new();
    if let Some(field) = json_binding(service, operation) {
        let value = parent_id.ok_or(format!("missing parent for {service}.{operation}"))?;
        fields.insert(field.to_owned(), json!(value));
    }
    if second {
        fields.insert("NextToken".to_owned(), json!("same-next-token"));
    }
    Ok(json!(fields))
}
fn rpc_path(service: &str, operation: &str) -> TestResult<String> {
    match service {
        "fms" => Ok(format!("/service/AWSFMS_20180101/operation/{operation}")),
        _ => Err(format!("unknown RPC service {service}").into()),
    }
}
pub(super) fn assert_rpc(
    wire: &http::Request<bytes::Bytes>,
    service: &str,
    operation: &str,
    expected: &Json,
) -> TestResult {
    assert_eq!(
        wire.method(),
        http::Method::POST,
        "expected POST for {service}.{operation}"
    );
    let expected_path = rpc_path(service, operation)?;
    assert_eq!(
        wire.uri().path(),
        &expected_path,
        "RPC path for {service}.{operation}"
    );
    assert!(
        wire.uri().query().is_none(),
        "RPC query must be None for {service}.{operation}"
    );
    assert!(
        wire.headers().get("x-amz-target").is_none(),
        "RPC should not have x-amz-target for {service}.{operation}"
    );
    let content_type = wire
        .headers()
        .get("content-type")
        .ok_or("RPC content-type missing")?
        .to_str()?;
    assert_eq!(
        content_type, "application/cbor",
        "RPC content-type for {service}.{operation}"
    );
    let protocol = wire
        .headers()
        .get("smithy-protocol")
        .ok_or("RPC smithy-protocol missing")?
        .to_str()?;
    assert_eq!(
        protocol, "rpc-v2-cbor",
        "RPC smithy-protocol for {service}.{operation}"
    );
    let body = wire.body();
    let expected_map = expected
        .as_object()
        .ok_or("expected RPC body is not an object")?;
    let mut decoder = minicbor::Decoder::new(body);
    let mut actual = serde_json::Map::new();
    if let Some(n) = decoder.map()? {
        let n = usize::try_from(n).map_err(|_| "RPC body map size exceeds usize")?;
        for _ in 0..n {
            let key = decoder.str()?.to_owned();
            if actual.contains_key(&key) {
                return Err(
                    format!("duplicate field {key} in RPC body for {service}.{operation}").into(),
                );
            }
            let value = decode_cbor_value(&mut decoder)?;
            actual.insert(key, value);
        }
    } else {
        loop {
            if decoder.datatype()? == minicbor::data::Type::Break {
                decoder.skip()?;
                break;
            }
            let key = decoder.str()?.to_owned();
            if actual.contains_key(&key) {
                return Err(
                    format!("duplicate field {key} in RPC body for {service}.{operation}").into(),
                );
            }
            let value = decode_cbor_value(&mut decoder)?;
            actual.insert(key, value);
        }
    }
    assert_eq!(
        actual.len(),
        expected_map.len(),
        "RPC body field count mismatch for {service}.{operation}"
    );
    assert_eq!(
        decoder.position(),
        body.len(),
        "trailing CBOR data in RPC body for {service}.{operation}"
    );
    assert_eq!(expected_map, &actual, "RPC body for {service}.{operation}");
    Ok(())
}
fn decode_cbor_value(decoder: &mut minicbor::Decoder<'_>) -> TestResult<Json> {
    match decoder.datatype()? {
        minicbor::data::Type::U8
        | minicbor::data::Type::U16
        | minicbor::data::Type::U32
        | minicbor::data::Type::U64 => Ok(json!(decoder.u64()?)),
        minicbor::data::Type::String => Ok(json!(decoder.str()?)),
        _ => Err("unsupported CBOR type in RPC body".into()),
    }
}
pub(super) fn json_binding(service: &str, operation: &str) -> Option<&'static str> {
    match (service, operation) {
        ("fms", "ListResourceSetResources") => Some("Identifier"),
        ("frauddetector", "DescribeDetector") => Some("detectorId"),
        _ => None,
    }
}
pub fn json_to_cbor(value: &Json) -> TestResult<Vec<u8>> {
    let mut encoder = minicbor::Encoder::new(Vec::new());
    encode_json(&mut encoder, value)?;
    Ok(encoder.into_writer())
}
fn encode_json(encoder: &mut minicbor::Encoder<Vec<u8>>, value: &Json) -> TestResult {
    match value {
        Json::Null => {
            encoder
                .null()
                .map_err(|e| format!("CBOR null error: {e}"))?;
        }
        Json::Bool(b) => {
            encoder
                .bool(*b)
                .map_err(|e| format!("CBOR bool error: {e}"))?;
        }
        Json::Number(n) => {
            if let Some(i) = n.as_i64() {
                encoder.i64(i).map_err(|e| format!("CBOR i64 error: {e}"))?;
            } else if let Some(u) = n.as_u64() {
                encoder.u64(u).map_err(|e| format!("CBOR u64 error: {e}"))?;
            } else {
                return Err("unsupported JSON number".into());
            }
        }
        Json::String(s) => {
            encoder.str(s).map_err(|e| format!("CBOR str error: {e}"))?;
        }
        Json::Array(arr) => {
            encoder
                .array(u64::try_from(arr.len()).map_err(|_| "array length exceeds u64")?)
                .map_err(|e| format!("CBOR array error: {e}"))?;
            for item in arr {
                encode_json(encoder, item)?;
            }
        }
        Json::Object(map) => {
            encoder
                .map(u64::try_from(map.len()).map_err(|_| "map length exceeds u64")?)
                .map_err(|e| format!("CBOR map error: {e}"))?;
            for (key, val) in map {
                encoder
                    .str(key)
                    .map_err(|e| format!("CBOR key error: {e}"))?;
                encode_json(encoder, val)?;
            }
        }
    }
    Ok(())
}
