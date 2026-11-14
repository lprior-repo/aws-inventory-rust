use super::*;

fn cause(text: &str, owner: &str) -> Result<CompileError, Box<dyn std::error::Error>> {
    let source = tempfile::tempdir()?;
    identities::model(source.path(), "ec2", "model.json", text)?;
    let CompileError::Shape { shape, source } = service_failure(source.path(), "ec2")? else {
        return Err("expected owning shape context".into());
    };
    assert_eq!(shape, owner);
    Ok(*source)
}

#[test]
fn unsupported_kind_keeps_service_shape_and_typed_cause() -> TestResult {
    let bad = MODEL.replace("\"type\":\"string\"", "\"type\":\"future\"");
    assert!(matches!(cause(&bad, "n#Text")?, CompileError::Kind));
    Ok(())
}

#[test]
fn missing_member_target_keeps_service_shape_and_typed_cause() -> TestResult {
    let bad = MODEL.replace("\"Zulu\":{\"target\":\"smithy.api#String\"}", "\"Zulu\":{}");
    assert!(matches!(cause(&bad, "n#Input")?, CompileError::Target(name) if name == "Zulu"));
    Ok(())
}

#[test]
fn non_string_arn_payload_keeps_service_shape_and_trait_cause() -> TestResult {
    let bad = MODEL.replace("\"template\":\"thing/{id}\"", "\"template\":42");
    assert!(matches!(cause(&bad, "n#Thing")?, CompileError::Trait(name) if name == "aws.api#arn"));
    Ok(())
}

#[test]
fn operation_target_keeps_service_operation_shape_and_typed_cause() -> TestResult {
    let bad = MODEL.replace(
        "\"output\":{\"target\":\"n#Things\"}",
        "\"output\":{\"target\":\"n#Gone\"}",
    );
    assert!(matches!(cause(&bad, "n#ListThings")?, CompileError::Target(name) if name == "n#Gone"));
    Ok(())
}

#[test]
fn missing_registry_member_keeps_service_shape_and_typed_cause() -> TestResult {
    let bad = MODEL.replace("\"Zulu\":", "\"NewName\":");
    assert!(
        matches!(cause(&bad, "n#Input")?, CompileError::MissingMember(name) if name == "NewName")
    );
    Ok(())
}
