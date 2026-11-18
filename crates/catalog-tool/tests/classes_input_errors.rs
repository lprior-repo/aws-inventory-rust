use catalog_tool::{
    ReportMode,
    classes::{DenyList, Error, build_report_with_policy, run, run_with_policy},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const MISSING: &str = r#"{"shapes":{
 "n#Svc":{"type":"service"},
 "n#ListBad":{"type":"operation","input":{"target":"n#Missing"}}
}}"#;
const NON_STRUCTURE: &str = r#"{"shapes":{
 "n#Svc":{"type":"service"},
 "n#ListBad":{"type":"operation","input":{"target":"n#String"}},
 "n#String":{"type":"string"}
}}"#;
const MIXINS: &str = r#"{"shapes":{
 "n#Svc":{"type":"service"},
 "n#ListBad":{"type":"operation","input":{"target":"n#Input"}},
 "n#Input":{"type":"structure","mixins":[{"target":"n#Base"}],"members":{
   "AFilter":{"target":"smithy.api#String"}}},
 "n#Base":{"type":"structure","traits":{"smithy.api#mixin":{}},"members":{
   "ZParent":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}}}}
}}"#;

fn fixture(model: &str) -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    std::fs::create_dir_all(dir.path().join("models/sample/service/v1"))?;
    std::fs::write(
        dir.path().join("models/sample/service/v1/sample.json"),
        model,
    )?;
    Ok(dir)
}

#[test]
fn missing_input_target_names_the_service_operation_and_target() -> TestResult {
    let dir = fixture(MISSING)?;
    assert!(matches!(
        build_report_with_policy(dir.path(), &DenyList::parse("")?),
        Err(Error::InvalidInput { file, source })
            if file.ends_with("sample.json")
                && source.to_string().contains("MissingShape")
                && source.to_string().contains("n#Missing")
    ));
    Ok(())
}

#[test]
fn non_structure_input_target_names_the_service_operation_and_target() -> TestResult {
    let dir = fixture(NON_STRUCTURE)?;
    assert!(matches!(
        build_report_with_policy(dir.path(), &DenyList::parse("")?),
        Err(Error::InvalidInput { file, source })
            if file.ends_with("sample.json")
                && source.to_string().contains("NonStructure")
                && source.to_string().contains("n#String")
    ));
    Ok(())
}

#[test]
fn invalid_input_is_rejected_even_for_a_mutating_operation() -> TestResult {
    let dir = fixture(&MISSING.replace("ListBad", "PutBad"))?;
    assert!(matches!(
        build_report_with_policy(dir.path(), &DenyList::parse("")?),
        Err(Error::InvalidInput { file, source: _ })
            if file.ends_with("sample.json")
    ));
    Ok(())
}

#[test]
fn resolved_mixins_contribute_required_input_members() -> TestResult {
    let dir = fixture(MIXINS)?;
    run_with_policy(
        dir.path(),
        dir.path(),
        ReportMode::Write,
        &DenyList::parse("")?,
    )?;
    let report = std::fs::read_to_string(dir.path().join("catalog/aws-classes.report"))?;
    assert!(report.lines().any(|line| line == "expand 1"));
    assert!(report.lines().any(|line| line == "enumerate 0"));
    Ok(())
}

#[test]
fn a_dangling_mixin_reference_is_rejected_like_a_resolvable_one() -> TestResult {
    let model = r#"{"shapes":{
 "n#Svc":{"type":"service"},
 "n#ListBad":{"type":"operation","input":{"target":"n#Input"}},
 "n#Input":{"type":"structure","mixins":[{"target":"n#Base"}],"members":{
   "AFilter":{"target":"smithy.api#String"}}}
}}"#;
    let dir = fixture(model)?;
    let path = dir.path().join("catalog/aws-classes.report");
    std::fs::create_dir_all(dir.path().join("catalog"))?;
    std::fs::write(&path, "preserve\n")?;
    let error = run(dir.path(), dir.path(), ReportMode::Write).err();
    assert!(matches!(
        error,
        Some(Error::InvalidInput { file, source })
            if file.ends_with("sample.json")
                && source.to_string().contains("MissingShape")
                && source.to_string().contains("n#Base")
    ));
    assert_eq!(std::fs::read_to_string(path)?, "preserve\n");
    Ok(())
}
