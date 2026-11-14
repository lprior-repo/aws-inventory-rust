use super::*;
use super::{
    compression::{assert_compression_boundaries, decode_request_body},
    limits::assert_query_limits,
    model::{convert, find_operation, fixture_registry},
};

pub(super) fn check_fixture(
    source: &str,
    model_dir: &str,
    protocol: Protocol,
) -> FixtureResult<()> {
    let fixture: Json = serde_json::from_str(source)?;
    let shape_map = fixture
        .get("shapes")
        .and_then(Json::as_object)
        .ok_or_else(|| fixture_error("fixture shapes absent"))?;
    let registry = fixture_registry(&fixture)?;
    let directory = tempfile::tempdir()?;
    let file = directory
        .path()
        .join("models")
        .join(model_dir)
        .join("fixture.json");
    fs::create_dir_all(
        file.parent()
            .ok_or_else(|| fixture_error("fixture model path absent"))?,
    )?;
    fs::write(file, source)?;
    let catalog_bytes = build_catalog(directory.path(), &registry, "protocol-fixture")?;
    let catalog = Catalog::borrow(&catalog_bytes)?;
    let service = catalog
        .services()?
        .find_map(|entry| {
            entry
                .ok()
                .filter(|service| service.name().ok() == Some(model_dir))
        })
        .ok_or_else(|| fixture_error(format!("fixture service {model_dir} absent")))?;

    let mut seen = BTreeSet::new();
    let mut unsupported = BTreeMap::new();
    for (shape_id, shape) in shape_map {
        if shape.get("type").and_then(Json::as_str) != Some("operation") {
            continue;
        }
        let Some(cases) = shape
            .get("traits")
            .and_then(|traits| traits.get("smithy.test#httpRequestTests"))
            .and_then(Json::as_array)
        else {
            continue;
        };
        let operation_name = shape_id.rsplit('#').next().unwrap_or(shape_id);
        let operation = find_operation(service, operation_name)?;
        for case in cases {
            let id = case
                .get("id")
                .and_then(Json::as_str)
                .ok_or_else(|| fixture_error("request fixture without id"))?;
            assert!(seen.insert(id.to_owned()), "duplicate request fixture {id}");
            match case
                .get("appliesTo")
                .and_then(Json::as_str)
                .unwrap_or("both")
            {
                "client" | "both" => {}
                "server" => {
                    unsupported.insert(
                        id.to_owned(),
                        "published test explicitly applies to server behavior",
                    );
                    continue;
                }
                _ => return Err(fixture_error(format!("{id}: invalid appliesTo value"))),
            }
            let params = case
                .get("params")
                .cloned()
                .unwrap_or_else(|| Json::Object(JsonMap::new()));
            let input = convert(service, &registry, operation.input()?, &params)
                .map_err(|error| fixture_error(format!("{id}: {error}")))?;
            let record = Record::encode(&input)?;
            let auto_token = id.contains("IdempotencyTokenAutoFill") && !id.ends_with("IsSet");
            let endpoint = fixture_endpoint(case);
            let context = RequestContext {
                service,
                operation,
                protocol,
                input: record.root(),
                endpoint: &endpoint,
                limits: RequestLimits::default(),
                idempotency_token: auto_token.then_some(AUTO_TOKEN),
                compression: CompressionPolicy::default(),
                addressing: None,
            };
            let request =
                encode(context).map_err(|error| fixture_error(format!("{id}: {error}")))?;
            if operation.request_compression()?.is_none() {
                assert!(
                    request
                        .headers()
                        .get(http::header::CONTENT_ENCODING)
                        .is_none(),
                    "{id} absent compression metadata"
                );
            }
            assert_eq!(
                request.method().as_str(),
                case.get("method").and_then(Json::as_str).unwrap_or("POST"),
                "{id}"
            );
            assert_eq!(
                request.uri().path(),
                case.get("uri").and_then(Json::as_str).unwrap_or("/"),
                "{id}"
            );
            let actual_bytes = decode_request_body(&request)?;
            let actual_body = std::str::from_utf8(&actual_bytes)?;
            let expected_body = match case.get("body").and_then(Json::as_str) {
                Some(body) => body.to_owned(),
                None => {
                    if operation.request_compression()?.is_none() {
                        return Err(fixture_error(format!(
                            "{id}: absent body requires modeled compression"
                        )));
                    }
                    let mut uncompressed_context = context;
                    uncompressed_context.compression = CompressionPolicy::Disabled;
                    let uncompressed = encode(uncompressed_context)?;
                    assert!(
                        request
                            .headers()
                            .get(http::header::CONTENT_ENCODING)
                            .is_some(),
                        "{id} expected compressed request"
                    );
                    assert_compression_boundaries(context, uncompressed_context, &uncompressed)?;
                    std::str::from_utf8(uncompressed.body())?.to_owned()
                }
            };
            let mut actual_pairs = actual_body.split('&').collect::<Vec<_>>();
            let mut expected_pairs = expected_body.split('&').collect::<Vec<_>>();
            actual_pairs.sort_unstable();
            expected_pairs.sort_unstable();
            assert_eq!(actual_pairs, expected_pairs, "{id}");
            let expected_headers = case.get("headers").and_then(Json::as_object);
            let forbidden_headers = case
                .get("forbidHeaders")
                .and_then(Json::as_array)
                .into_iter()
                .flatten()
                .filter_map(Json::as_str)
                .collect::<BTreeSet<_>>();
            for (name, value) in request.headers() {
                assert!(
                    !forbidden_headers
                        .iter()
                        .any(|header| header.eq_ignore_ascii_case(name.as_str())),
                    "{id} forbidden header {name}"
                );
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
                if let Some((_, expected)) = expected_headers.and_then(|headers| {
                    headers
                        .iter()
                        .find(|(header, _)| header.eq_ignore_ascii_case(name.as_str()))
                }) {
                    assert_eq!(
                        actual,
                        expected.as_str().ok_or_else(|| {
                            fixture_error(format!("{id}: non-string expected header"))
                        })?,
                        "{id} header {name}"
                    );
                }
            }
            if let Some(headers) = expected_headers {
                for (name, value) in headers {
                    let expected = value.as_str().ok_or_else(|| {
                        fixture_error(format!("{id}: non-string expected header"))
                    })?;
                    assert_eq!(
                        request
                            .headers()
                            .get(name.as_str())
                            .and_then(|value| value.to_str().ok()),
                        Some(expected),
                        "{id} header {name}"
                    );
                }
            }
            if let Some(headers) = case.get("requireHeaders").and_then(Json::as_array) {
                for name in headers.iter().filter_map(Json::as_str) {
                    assert!(
                        request
                            .headers()
                            .keys()
                            .any(|actual| actual.as_str().eq_ignore_ascii_case(name)),
                        "{id} required header {name}"
                    );
                }
            }
            if let Some(headers) = expected_headers {
                for (name, value) in headers {
                    let expected = value.as_str().ok_or_else(|| {
                        fixture_error(format!("{id}: non-string expected header"))
                    })?;
                    assert_eq!(
                        request
                            .headers()
                            .get(name.as_str())
                            .and_then(|value| value.to_str().ok()),
                        Some(expected),
                        "{id} header {name}"
                    );
                }
            }
            let content_length = request
                .headers()
                .get(http::header::CONTENT_LENGTH)
                .and_then(|value| value.to_str().ok());
            let expected_length = request.body().len().to_string();
            assert_eq!(
                content_length,
                Some(expected_length.as_str()),
                "{id} Content-Length"
            );
        }
    }
    assert_query_limits(service, protocol, &registry)?;
    assert_eq!(
        seen.len(),
        shape_map
            .values()
            .filter(|shape| shape.get("type").and_then(Json::as_str) == Some("operation"))
            .flat_map(|shape| shape
                .get("traits")
                .and_then(|traits| traits.get("smithy.test#httpRequestTests"))
                .and_then(Json::as_array)
                .into_iter()
                .flatten())
            .count(),
        "all published request cases must be accounted for"
    );
    for (id, reason) in &unsupported {
        assert!(
            !reason.is_empty(),
            "{id} lacks a concrete unsupported reason"
        );
    }
    Ok(())
}
