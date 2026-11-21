use super::*;
#[test]
fn all_published_client_error_responses_decode_through_compiled_models() -> TestResult {
    let fixtures = [
        Fixture::new(
            include_str!("../../../../tests/fixtures/protocol-tests/awsJson1_0.json"),
            Protocol::AwsJson10,
            "awsJson1_0",
            16,
        ),
        Fixture::new(
            include_str!("../../../../tests/fixtures/protocol-tests/awsJson1_1.json"),
            Protocol::AwsJson11,
            "awsJson1_1",
            14,
        ),
        Fixture::new(
            include_str!("../../../../tests/fixtures/protocol-tests/awsQuery.json"),
            Protocol::AwsQuery,
            "awsQuery",
            3,
        ),
        Fixture::new(
            include_str!("../../../../tests/fixtures/protocol-tests/ec2Query.json"),
            Protocol::Ec2Query,
            "ec2Query",
            2,
        ),
        Fixture::new(
            include_str!("../../../../tests/fixtures/protocol-tests/restJson1.json"),
            Protocol::RestJson1,
            "restJson1",
            14,
        ),
        Fixture::new(
            include_str!("../../../../tests/fixtures/protocol-tests/restXml.json"),
            Protocol::RestXml,
            "restXml",
            3,
        ),
        Fixture::new(
            include_str!("../../../../tests/fixtures/protocol-tests/rpcv2Cbor.json"),
            Protocol::RpcV2Cbor,
            "rpcv2Cbor",
            5,
        ),
    ];
    let mut decoded_cases = 0usize;

    for fixture in fixtures {
        let root: Value = serde_json::from_str(fixture.source)?;
        let shapes = root
            .get("shapes")
            .and_then(Value::as_object)
            .ok_or("fixture shapes")?;
        let error_shape_ids = shapes
            .iter()
            .filter(|(_, shape)| {
                shape
                    .pointer("/traits/smithy.api#error")
                    .is_some_and(Value::is_string)
            })
            .filter(|(_, shape)| response_cases(shape).is_some())
            .map(|(shape_id, _)| shape_id.clone())
            .collect::<Vec<_>>();
        let mut server_cases = 0usize;
        let mut client_cases = 0usize;
        for shape_id in &error_shape_ids {
            let cases = shapes
                .get(shape_id)
                .and_then(response_cases)
                .ok_or("error response cases")?;
            for case in cases {
                if case["appliesTo"].as_str() == Some("server") {
                    server_cases = server_cases.checked_add(1).ok_or("server case count")?;
                } else {
                    client_cases = client_cases.checked_add(1).ok_or("client case count")?;
                }
            }
        }
        assert_eq!(server_cases, 0, "{} server-only cases", fixture.name);
        assert_eq!(
            client_cases, fixture.expected_cases,
            "{} client cases",
            fixture.name
        );

        let mut service_ids = Vec::new();
        for (shape_id, shape) in shapes {
            if shape["type"].as_str() != Some("service") {
                continue;
            }
            let mut reachable = false;
            for error in &error_shape_ids {
                if matches!(
                    fixture_operation_for_error(shapes, shape, error),
                    Ok(Some(_))
                ) {
                    reachable = true;
                    break;
                }
            }
            if reachable {
                service_ids.push(shape_id.clone());
            }
        }
        let mut fixture_shapes = BTreeSet::new();
        for service_id in service_ids {
            let bytes = compile_service_model(&root, &service_id)?;
            let catalog = Catalog::borrow(&bytes)?;
            let service_name = service_id.rsplit('#').next().ok_or("service name")?;
            let service = catalog
                .service(&service_name.replace('.', "-"))?
                .ok_or("fixture service exists")?;
            let mut operations = HashMap::new();
            for operation in service.operations()?.collect::<Result<Vec<_>, _>>()? {
                operations.insert(operation.name()?.to_owned(), operation);
            }
            let service_shape = shapes.get(&service_id).ok_or("service shape")?;

            for error_shape_id in &error_shape_ids {
                if fixture_shapes.contains(error_shape_id) {
                    continue;
                }
                let Some(operation_id) =
                    fixture_operation_for_error(shapes, service_shape, error_shape_id)?
                else {
                    continue;
                };
                let operation_name = operation_id.rsplit('#').next().ok_or("operation name")?;
                let operation = *operations
                    .get(operation_name)
                    .ok_or("compiled operation for fixture error")?;
                let raw_errors = shapes
                    .get(operation_id)
                    .and_then(|operation| operation.get("errors"))
                    .and_then(Value::as_array)
                    .ok_or("operation error targets")?;
                let error_index = raw_errors
                    .iter()
                    .position(|target| target["target"].as_str() == Some(error_shape_id))
                    .ok_or("operation binds fixture error")?;
                let compiled_errors = operation.errors()?.collect::<Result<Vec<_>, _>>()?;
                let error_target = match compiled_errors.get(error_index).ok_or("compiled error")? {
                    Target::Shape(index) => {
                        service.shape(*index)?.ok_or("catalog error target shape")?
                    }
                    _ => {
                        return Err("fixture error resolved to a non-shape target".into());
                    }
                };
                assert_eq!(
                    error_target.name()?,
                    error_shape_id
                        .rsplit('#')
                        .next()
                        .ok_or("error shape name")?
                );
                let modeled_error = error_target.error()?.ok_or("modeled error metadata")?;
                let shape = shapes.get(error_shape_id).ok_or("error shape")?;
                let cases = response_cases(shape).ok_or("error response cases")?;
                for case in cases {
                    if case["appliesTo"].as_str() == Some("server") {
                        continue;
                    }
                    let body = fixture_body(case, fixture.protocol)?;
                    let status = fixture_status(case)?;
                    let headers = fixture_headers(case)?;
                    let expected_code =
                        expected_fixture_code(error_shape_id, modeled_error, &headers)?;
                    let response = ResponseContext {
                        service,
                        operation,
                        protocol: fixture.protocol,
                        body: &body,
                        headers: &headers,
                        status,
                        limits: DecodeLimits::default(),
                    };
                    let decoded = match crate::response::decode(response) {
                        Err(DecodeError::Service(error)) => error,
                        result => {
                            return Err(format!(
                                "published service failure must reach root dispatch: {result:?}"
                            )
                            .into());
                        }
                    };
                    assert_eq!(
                        decoded.code.as_deref(),
                        Some(expected_code.as_str()),
                        "{} {}",
                        fixture.name,
                        error_shape_id
                    );
                    assert_eq!(
                        decoded.class,
                        expected_fixture_class(error_shape_id),
                        "{} {}",
                        fixture.name,
                        error_shape_id
                    );
                    assert!(!decoded.to_string().contains("private"));
                    if let Some(code) = &decoded.code {
                        assert!(
                            decoded.to_string().contains(code),
                            "service error code should appear in display for diagnosis"
                        );
                    }
                    decoded_cases = decoded_cases.checked_add(1).ok_or("decoded case count")?;
                    fixture_shapes.insert(error_shape_id.clone());
                }
            }
        }
        assert_eq!(
            fixture_shapes.len(),
            error_shape_ids.len(),
            "{} every error shape is operation-reachable",
            fixture.name
        );
        let expected_so_far = fixtures
            .iter()
            .filter(|item| matches!((item.index(), fixture.index()), (Some(i), Some(j)) if i <= j))
            .map(|item| item.expected_cases)
            .sum::<usize>();
        assert_eq!(decoded_cases, expected_so_far);
    }
    assert_eq!(decoded_cases, 57);
    Ok(())
}

fn response_cases(shape: &Value) -> Option<&Vec<Value>> {
    shape
        .pointer("/traits/smithy.test#httpResponseTests")?
        .as_array()
}
