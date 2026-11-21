use super::{TestResult, compile_model, fixture_service};

fn malformed_model(shape_kind: &str, trait_id: &str, payload: &str) -> String {
    r#"{"smithy":"2.0","shapes":{"example#Service":{"type":"service","version":"1"},"example#Bad":{"type":"__SHAPE_KIND__","traits":{"__TRAIT__":__PAYLOAD__}},"example#Input":{"type":"structure","members":{"Name":{"target":"smithy.api#String"}}}}}"#
        .replace("__SHAPE_KIND__", shape_kind)
        .replace("__TRAIT__", trait_id)
        .replace("__PAYLOAD__", payload)
}
#[test]
fn known_service_operation_member_and_shape_traits_reject_wrong_json_types_with_context()
-> TestResult {
    let cases = [
        (
            "service",
            "aws.api#service",
            "\"not-an-object\"",
            "example#Service",
        ),
        (
            "service",
            "aws.auth#sigv4",
            "\"not-an-object\"",
            "example#Service",
        ),
        (
            "service",
            "aws.protocols#restJson1",
            "\"not-an-object\"",
            "example#Service",
        ),
        (
            "service",
            "aws.protocols#awsQueryCompatible",
            "17",
            "example#Service",
        ),
        (
            "service",
            "smithy.api#xmlNamespace",
            "{\"uri\":17}",
            "example#Service",
        ),
        (
            "service",
            "smithy.api#paginated",
            "{\"inputToken\":17}",
            "example#Service",
        ),
        (
            "service",
            "aws.protocols#restJson1",
            "{\"noErrorWrapping\":\"yes\"}",
            "example#Service",
        ),
        ("operation", "smithy.api#http", "17", "example#Bad"),
        (
            "operation",
            "smithy.api#endpoint",
            "{\"hostPrefix\":17}",
            "example#Bad",
        ),
        ("operation", "smithy.api#paginated", "[]", "example#Bad"),
        (
            "operation",
            "smithy.api#httpChecksumRequired",
            "\"true\"",
            "example#Bad",
        ),
        ("member", "smithy.api#jsonName", "17", "example#Input"),
        ("member", "smithy.api#httpHeader", "false", "example#Input"),
        ("member", "smithy.api#sensitive", "false", "example#Input"),
        ("member", "smithy.api#mediaType", "false", "example#Input"),
        ("member", "smithy.api#xmlNamespace", "17", "example#Input"),
        (
            "member",
            "smithy.api#timestampFormat",
            "17",
            "example#Input",
        ),
        ("member", "smithy.api#streaming", "\"yes\"", "example#Input"),
        (
            "member",
            "smithy.api#idempotencyToken",
            "17",
            "example#Input",
        ),
        ("member", "smithy.api#hostLabel", "17", "example#Input"),
        ("member", "smithy.api#xmlFlattened", "17", "example#Input"),
        ("member", "smithy.api#xmlAttribute", "17", "example#Input"),
        ("structure", "smithy.api#error", "17", "example#Bad"),
        (
            "structure",
            "smithy.api#httpError",
            "\"409\"",
            "example#Bad",
        ),
        ("structure", "smithy.api#xmlNamespace", "17", "example#Bad"),
        ("structure", "smithy.api#mediaType", "false", "example#Bad"),
        ("structure", "smithy.api#sensitive", "17", "example#Bad"),
        (
            "structure",
            "aws.protocols#awsQueryError",
            "[]",
            "example#Bad",
        ),
        (
            "structure",
            "smithy.api#retryable",
            "\"yes\"",
            "example#Bad",
        ),
    ];
    for (kind, trait_id, payload, shape) in cases {
        let owner = "service example";
        let model = if kind == "service" {
            r#"{"smithy":"2.0","shapes":{"example#Service":{"type":"service","version":"1","traits":{"__TRAIT__":__PAYLOAD__}},"example#Bad":{"type":"structure"}}}"#
                .replace("__TRAIT__", trait_id)
                .replace("__PAYLOAD__", payload)
        } else if kind == "member" {
            r#"{"smithy":"2.0","shapes":{"example#Service":{"type":"service","version":"1"},"example#Input":{"type":"structure","members":{"Name":{"target":"smithy.api#String","traits":{"__TRAIT__":__PAYLOAD__}}}}}}"#
                .replace("__TRAIT__", trait_id)
                .replace("__PAYLOAD__", payload)
        } else {
            malformed_model(kind, trait_id, payload)
        };
        let source = tempfile::tempdir()?;
        let model_path = source.path().join("models/example/model.json");
        std::fs::create_dir_all(model_path.parent().ok_or("fixture path has no parent")?)?;
        std::fs::write(&model_path, model)?;
        let fields = "Name\n";
        let registry = catalog_tool::smithy::Registry::parse(fields)?;
        let Err(error) =
            catalog_tool::smithy::build_catalog(source.path(), &registry, "fixture-commit")
        else {
            return Err("wrong JSON type of a known trait must fail compilation".into());
        };
        let rendered = error.to_string();
        assert!(rendered.contains(owner), "{rendered}");
        assert!(rendered.contains(shape), "{rendered}");
        assert!(rendered.contains(trait_id), "{rendered}");
    }
    Ok(())
}

#[test]
fn unknown_traits_are_ignored_and_known_absent_metadata_stays_absent() -> TestResult {
    let model = r#"{"smithy":"2.0","shapes":{"example#Service":{"type":"service","version":"1","traits":{"fixture#future":17}},"example#Read":{"type":"operation","traits":{"fixture#future":17}},"example#Input":{"type":"structure","members":{"Name":{"target":"smithy.api#String","traits":{"fixture#future":17}}}}}}"#;
    let (_source, bytes) = compile_model(model, "Name\n")?;
    let service = fixture_service(&bytes)?;
    assert_eq!(service.sdk_id()?, None);
    assert_eq!(service.signing_name()?, None);
    let mut operations = service.operations()?;
    let operation = operations.next().ok_or("operation missing")??;
    assert_eq!(operation.http()?, None);
    let input = service
        .shapes()?
        .find_map(|shape| match shape {
            Ok(shape) if shape.name().ok() == Some("Input") => Some(Ok(shape)),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .transpose()?
        .ok_or("input missing")?;
    assert_eq!(
        input
            .members()?
            .next()
            .ok_or("member missing")??
            .wire()?
            .json_name()?,
        None
    );
    Ok(())
}
