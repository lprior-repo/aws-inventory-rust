use catalog_tool::classes::{Error, ReportMode, build_report, run};

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
    std::fs::create_dir_all(dir.path().join("catalog/overlay"))?;
    std::fs::write(dir.path().join("catalog/overlay/must-deny.txt"), "")?;
    std::fs::write(
        dir.path().join("models/sample/service/v1/sample.json"),
        model,
    )?;
    Ok(dir)
}

#[test]
fn missing_input_target_names_the_service_operation_and_target() -> TestResult {
    // Given a present dangling input, when reporting, then it cannot become input-free.
    let dir = fixture(MISSING)?;
    assert!(matches!(build_report(dir.path(), dir.path()),
        Err(Error::InvalidInput { service, operation, target })
        if service == "sample" && operation == "ListBad" && target == "n#Missing"));
    Ok(())
}

#[test]
fn non_structure_input_target_names_the_service_operation_and_target() -> TestResult {
    // Given a present string-shaped input, when reporting, then it is rejected explicitly.
    let dir = fixture(NON_STRUCTURE)?;
    assert!(matches!(build_report(dir.path(), dir.path()),
        Err(Error::InvalidInput { service, operation, target })
        if service == "sample" && operation == "ListBad" && target == "n#String"));
    Ok(())
}

#[test]
fn invalid_input_is_rejected_even_for_a_mutating_operation() -> TestResult {
    // Given a denied name with a malformed input, when loading the model,
    // then denial does not bypass the input boundary's validation.
    let dir = fixture(&MISSING.replace("ListBad", "PutBad"))?;
    assert!(matches!(build_report(dir.path(), dir.path()),
        Err(Error::InvalidInput { service, operation, target })
        if service == "sample" && operation == "PutBad" && target == "n#Missing"));
    Ok(())
}

#[test]
fn input_mixins_return_a_named_error_without_overwriting_the_report() -> TestResult {
    // Given an inherited required member and good committed bytes, when writing,
    // then unresolved mixins fail closed instead of silently permitting enumeration.
    let dir = fixture(MIXINS)?;
    let path = dir.path().join("catalog/aws-classes.report");
    std::fs::write(&path, "preserve\n")?;
    let error = run(dir.path(), dir.path(), ReportMode::Write).err();
    assert_eq!(
        error.as_ref().map(ToString::to_string),
        Some("sample/ListBad: mixins are not resolved".to_owned())
    );
    assert!(
        matches!(error, Some(Error::MixinsNotResolved { service, operation })
        if service == "sample" && operation == "ListBad")
    );
    assert_eq!(std::fs::read_to_string(path)?, "preserve\n");
    Ok(())
}

#[test]
fn a_dangling_mixin_reference_is_rejected_like_a_resolvable_one() -> TestResult {
    // Given an input whose mixin target is absent and good committed bytes,
    // when writing, then the reference is not ignored and nothing is replaced.
    let model = r#"{"shapes":{
 "n#Svc":{"type":"service"},
 "n#ListBad":{"type":"operation","input":{"target":"n#Input"}},
 "n#Input":{"type":"structure","mixins":[{"target":"n#Base"}],"members":{
   "AFilter":{"target":"smithy.api#String"}}}
}}"#;
    let dir = fixture(model)?;
    let path = dir.path().join("catalog/aws-classes.report");
    std::fs::write(&path, "preserve\n")?;
    let error = run(dir.path(), dir.path(), ReportMode::Write).err();
    assert!(
        matches!(error, Some(Error::MixinsNotResolved { service, operation })
        if service == "sample" && operation == "ListBad")
    );
    assert_eq!(std::fs::read_to_string(path)?, "preserve\n");
    Ok(())
}
