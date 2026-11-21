use super::*;
#[test]
fn all_published_client_error_responses_decode_through_compiled_models() {
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
        let root: Value = serde_json::from_str(fixture.source).expect("valid protocol fixture");
        let shapes = root["shapes"].as_object().expect("fixture shapes");
        let error_shape_ids = shapes
            .iter()
            .filter(|(_, shape)| shape["traits"]["smithy.api#error"].is_string())
            .filter(|(_, shape)| shape["traits"]["smithy.test#httpResponseTests"].is_array())
            .map(|(shape_id, _)| shape_id.clone())
            .collect::<Vec<_>>();
        let server_cases = error_shape_ids
            .iter()
            .flat_map(|shape_id| {
                shapes[shape_id]["traits"]["smithy.test#httpResponseTests"]
                    .as_array()
                    .expect("error response cases")
            })
            .filter(|case| case["appliesTo"].as_str() == Some("server"))
            .count();
        assert_eq!(server_cases, 0, "{} server-only cases", fixture.name);
        let client_cases = error_shape_ids
            .iter()
            .flat_map(|shape_id| {
                shapes[shape_id]["traits"]["smithy.test#httpResponseTests"]
                    .as_array()
                    .expect("error response cases")
            })
            .filter(|case| case["appliesTo"].as_str() != Some("server"))
            .count();
        assert_eq!(
            client_cases, fixture.expected_cases,
            "{} client cases",
            fixture.name
        );

        let service_ids = shapes
            .iter()
            .filter(|(_, shape)| shape["type"].as_str() == Some("service"))
            .filter(|(_, shape)| {
                error_shape_ids
                    .iter()
                    .any(|error| fixture_operation_for_error(shapes, shape, error).is_some())
            })
            .map(|(shape_id, _)| shape_id.clone())
            .collect::<Vec<_>>();
        let mut fixture_shapes = BTreeSet::new();
        for service_id in service_ids {
            let bytes = compile_service_model(&root, &service_id);
            let catalog = Catalog::borrow(&bytes).expect("compiled protocol catalog");
            let service_name = service_id.rsplit('#').next().expect("service name");
            let service = catalog
                .service(&service_name.replace('.', "-"))
                .expect("catalog service lookup")
                .expect("fixture service exists");
            let operations = service
                .operations()
                .expect("catalog operations")
                .collect::<Result<Vec<_>, _>>()
                .expect("valid catalog operation references")
                .into_iter()
                .map(|operation| {
                    (
                        operation.name().expect("catalog operation name").to_owned(),
                        operation,
                    )
                })
                .collect::<HashMap<_, _>>();
            let service_shape = &shapes[&service_id];

            for error_shape_id in &error_shape_ids {
                if fixture_shapes.contains(error_shape_id) {
                    continue;
                }
                let Some(operation_id) =
                    fixture_operation_for_error(shapes, service_shape, error_shape_id)
                else {
                    continue;
                };
                let operation_name = operation_id.rsplit('#').next().expect("operation name");
                let operation = operations
                    .get(operation_name)
                    .copied()
                    .expect("compiled operation for fixture error");
                let raw_errors = shapes[operation_id]["errors"]
                    .as_array()
                    .expect("operation error targets");
                let error_index = raw_errors
                    .iter()
                    .position(|target| target["target"].as_str() == Some(error_shape_id))
                    .expect("operation binds fixture error");
                let compiled_errors = operation
                    .errors()
                    .expect("compiled operation errors")
                    .collect::<Result<Vec<_>, _>>()
                    .expect("valid compiled error references");
                let error_target = match compiled_errors[error_index] {
                    Target::Shape(index) => service
                        .shape(index)
                        .expect("error shape reference")
                        .expect("catalog error target shape"),
                    _ => panic!("fixture error resolved to a non-shape target"),
                };
                assert_eq!(
                    error_target.name().expect("error shape name"),
                    error_shape_id.rsplit('#').next().expect("error shape name")
                );
                let modeled_error = error_target
                    .error()
                    .expect("modeled error metadata")
                    .expect("error shape carries modeled error");
                let shape = &shapes[error_shape_id];
                let cases = shape["traits"]["smithy.test#httpResponseTests"]
                    .as_array()
                    .expect("error response cases");
                for case in cases {
                    if case["appliesTo"].as_str() == Some("server") {
                        continue;
                    }
                    let body = fixture_body(case, fixture.protocol);
                    let status = fixture_status(case);
                    let headers = fixture_headers(case);
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
                            panic!("published service failure must reach root dispatch: {result:?}")
                        }
                    };
                    assert_eq!(
                        decoded.code.as_deref(),
                        Some(
                            expected_fixture_code(error_shape_id, modeled_error, &headers).as_str()
                        ),
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
                    decoded_cases += 1;
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
        assert_eq!(
            decoded_cases,
            fixtures[..=fixture.index()]
                .iter()
                .map(|item| item.expected_cases)
                .sum::<usize>()
        );
    }
    assert_eq!(decoded_cases, 57);
}
