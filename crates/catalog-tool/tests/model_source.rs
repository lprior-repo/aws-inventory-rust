include!("smithy_helpers.rs");

#[test]
fn duplicate_shape_and_member_keys_are_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let dir = fixture()?;
    let duplicate_shape = r#"{"shapes":{"n#Svc":{"type":"service"},"n#Svc":{"type":"service"}}}"#;
    write_fixture(
        dir.path(),
        "models/service/deep/model.json",
        duplicate_shape,
    )?;
    let shapes = report(dir.path(), &[])?;
    assert_eq!(shapes.status.code(), Some(1));
    assert_eq!(shapes.stdout, b"");
    assert!(String::from_utf8(shapes.stderr)?.contains("duplicate key \"n#Svc\""));

    let duplicate_member = r#"{"shapes":{"n#Svc":{"type":"service"},"n#Input":{"type":"structure","members":{"Value":{"target":"smithy.api#String"},"Value":{"target":"smithy.api#String"}}}}}"#;
    write_fixture(
        dir.path(),
        "models/service/deep/model.json",
        duplicate_member,
    )?;
    let members = report(dir.path(), &[])?;
    assert_eq!(members.status.code(), Some(1));
    assert_eq!(members.stdout, b"");
    assert!(String::from_utf8(members.stderr)?.contains("duplicate key \"Value\""));
    Ok(())
}

#[test]
fn orphan_model_path_is_rejected_by_compiler_with_context() -> Result<(), Box<dyn std::error::Error>>
{
    let dir = fixture()?;
    std::fs::remove_file(dir.path().join("models/service/deep/model.json"))?;
    write_fixture(dir.path(), "models/model.json", MODEL)?;
    let registry = catalog_tool::smithy::Registry::parse("Shared\n")?;
    let model = dir.path().join("models/model.json");
    assert!(matches!(
        catalog_tool::smithy::build_catalog(dir.path(), &registry, "fixture"),
        Err(catalog_tool::smithy::Error::Compile(
            catalog_tool::smithy::CompileError::ModelPath(path)
        )) if path == model
    ));
    Ok(())
}

#[test]
fn malformed_model_is_rejected_before_publication() -> Result<(), Box<dyn std::error::Error>> {
    let dir = fixture()?;
    write_fixture(dir.path(), "models/service/deep/model.json", "{")?;
    let output = report(dir.path(), &[])?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert!(String::from_utf8(output.stderr)?.contains("invalid JSON"));
    Ok(())
}

#[test]
fn resolved_mixin_required_members_are_used_by_classification()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let model = r#"{"shapes":{"n#Service":{"type":"service"},"n#ListThings":{"type":"operation","input":{"target":"n#Input"}},"n#Input":{"type":"structure","mixins":[{"target":"n#Base"}]},"n#Base":{"type":"structure","members":{"Required":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}}}}}}"#;
    write_fixture(dir.path(), "models/service/deep/model.json", model)?;
    write_fixture(dir.path(), "catalog/overlay/must-deny.txt", "")?;
    let output = tool(&[
        "classes",
        "--root",
        dir.path().to_str().ok_or("invalid root")?,
        "--source",
        dir.path().to_str().ok_or("invalid source")?,
        "--write",
    ])?;
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"");
    let report_text = std::fs::read_to_string(dir.path().join("catalog/aws-classes.report"))?;
    assert!(report_text.lines().any(|line| line == "expand 1"));
    assert!(report_text.lines().any(|line| line == "enumerate 0"));
    Ok(())
}
