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
    // Given an unsupported shape, the boundary preserves directory identity and its full owner ID.
    let bad = MODEL.replace("\"type\":\"string\"", "\"type\":\"future\"");
    assert!(matches!(cause(&bad, "n#Text")?, CompileError::Kind));
    Ok(())
}

#[test]
fn missing_member_target_keeps_service_shape_and_typed_cause() -> TestResult {
    // Given a member missing its target, the member name does not replace its owning shape context.
    let bad = MODEL.replace("\"Zulu\":{\"target\":\"smithy.api#String\"}", "\"Zulu\":{}");
    assert!(matches!(cause(&bad, "n#Input")?, CompileError::Target(name) if name == "Zulu"));
    Ok(())
}

#[test]
fn malformed_arn_keeps_service_shape_and_typed_cause() -> TestResult {
    // Given a non-string ARN, the underlying ARN error survives both context layers.
    let bad = MODEL.replace("\"template\":\"thing/{id}\"", "\"template\":42");
    assert!(matches!(cause(&bad, "n#Thing")?, CompileError::Arn(name) if name == "n#Thing"));
    Ok(())
}

#[test]
fn operation_target_keeps_service_operation_shape_and_typed_cause() -> TestResult {
    // Given an unresolved operation output, diagnostics retain that operation's full shape ID.
    let bad = MODEL.replace(
        "\"output\":{\"target\":\"n#Things\"}",
        "\"output\":{\"target\":\"n#Gone\"}",
    );
    assert!(matches!(cause(&bad, "n#ListThings")?, CompileError::Target(name) if name == "n#Gone"));
    Ok(())
}

#[test]
fn missing_registry_member_keeps_service_shape_and_typed_cause() -> TestResult {
    // Given a new member absent from the registry, its name and both owning contexts survive.
    let bad = MODEL.replace("\"Zulu\":", "\"NewName\":");
    assert!(
        matches!(cause(&bad, "n#Input")?, CompileError::MissingMember(name) if name == "NewName")
    );
    Ok(())
}
