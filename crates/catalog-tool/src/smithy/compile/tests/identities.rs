use super::*;
pub(super) fn model(source: &Path, directory: &str, file: &str, text: &str) -> std::io::Result<()> {
    let dir = source.join("models").join(directory);
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join(file), text)
}
fn two_services() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let source = tempfile::tempdir()?;
    model(source.path(), "storage-gateway", "model.json", MODEL)?;
    model(
        source.path(),
        "ec2",
        "model.json",
        &MODEL.replace("n#Svc", "other#Svc"),
    )?;
    Ok(build_catalog(
        source.path(),
        &Registry::parse("Zulu\nAlpha\n")?,
        "fixture",
    )?)
}
#[test]
fn directory_identity_disambiguates_equal_unqualified_service_shape_names() -> TestResult {
    // Given equal short names in different namespaces, directories remain the service identity.
    let bytes = two_services()?;
    let catalog = Catalog::borrow(&bytes)?;
    let ec2 = catalog.service("ec2")?.ok_or("ec2")?;
    assert_eq!(ec2.shape_id()?, "other#Svc");
    let gateway = catalog.service("storage-gateway")?.ok_or("gateway")?;
    assert_eq!(gateway.shape_id()?, "n#Svc");
    let names = catalog
        .services()?
        .map(|s| s?.name())
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(names, vec!["ec2", "storage-gateway"]);
    assert_eq!(catalog.service("Svc")?.map(|s| s.shape_count()), None);
    Ok(())
}
#[test]
fn same_full_service_shape_id_in_two_directories_is_a_typed_error() -> TestResult {
    // Given two directories sharing a full Smithy identity, compilation refuses ambiguity.
    let source = tempfile::tempdir()?;
    model(source.path(), "a", "model.json", MODEL)?;
    model(source.path(), "b", "model.json", MODEL)?;
    let cause = service_failure(source.path(), "b")?;
    assert!(matches!(cause, CompileError::DuplicateServiceShape(id) if id == "n#Svc"));
    Ok(())
}
#[test]
fn multiple_models_in_one_directory_cannot_replace_each_others_service_identity() -> TestResult {
    // Given distinct service shapes under one directory, no duplicate lookup key is written.
    let source = tempfile::tempdir()?;
    model(source.path(), "a", "first.json", MODEL)?;
    model(
        source.path(),
        "a",
        "second.json",
        &MODEL.replace("n#Svc", "other#Svc"),
    )?;
    let cause = service_failure(source.path(), "a")?;
    assert!(matches!(cause, CompileError::DuplicateService(name) if name == "a"));
    Ok(())
}
#[test]
fn required_input_changes_enumeration_flags_and_report_counts() -> TestResult {
    // Given a required input member, when compiled, ListThings is no longer input-free.
    let required = MODEL.replace(
        "\"Zulu\":{\"target\":\"smithy.api#String\"}",
        "\"Zulu\":{\"target\":\"smithy.api#String\",\"traits\":{\"smithy.api#required\":{}}}",
    );
    let bytes = root(record(&required, "Zulu\nAlpha\n")?)?;
    let catalog = Catalog::borrow(&bytes)?;
    let counts = catalog.counts()?;
    assert_eq!(counts.get("list_describe_root"), Some(0));
    assert_eq!(counts.get("services_with_root"), Some(0));
    let service = catalog.service("example")?.ok_or("service")?;
    assert_eq!(
        service.operations()?.next().ok_or("operation")??.flags()?,
        23
    );
    Ok(())
}
#[test]
fn missing_member_targets_and_malformed_arn_templates_fail_compilation() -> TestResult {
    // Given malformed modeled content, no empty target or template is synthesized.
    let missing = MODEL.replace("\"Zulu\":{\"target\":\"smithy.api#String\"}", "\"Zulu\":{}");
    let (shape, cause) = shape_failure(&missing, "Zulu\nAlpha\n")?;
    assert_eq!(shape, "n#Input");
    assert!(matches!(cause, CompileError::Target(name) if name == "Zulu"));
    let malformed = MODEL.replace("\"template\":\"thing/{id}\"", "\"template\":42");
    let (shape, cause) = shape_failure(&malformed, "Zulu\nAlpha\n")?;
    assert_eq!(shape, "n#Thing");
    assert!(matches!(cause, CompileError::Arn(name) if name == "n#Thing"));
    Ok(())
}
