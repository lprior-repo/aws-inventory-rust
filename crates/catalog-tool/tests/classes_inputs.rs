use catalog_tool::classes::{DenyList, Error, build_report, build_report_with_policy};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const MODEL: &str = r#"{"shapes":{
 "n#Svc":{"type":"service"},
 "n#ListAbsent":{"type":"operation"},
 "n#ListUnit":{"type":"operation","input":{"target":"smithy.api#Unit"}},
 "n#DescribeEmpty":{"type":"operation","input":{"target":"n#Empty"}},
 "n#DescribeOptional":{"type":"operation","input":{"target":"n#Optional"}},
 "n#DescribeRequired":{"type":"operation","input":{"target":"n#Required"}},
 "n#Empty":{"type":"structure"},
 "n#Optional":{"type":"structure","members":{"Filter":{"target":"n#String"}}},
 "n#Required":{"type":"structure","members":{"Parent":{"target":"n#String","traits":{"smithy.api#required":{}}}}},
 "n#String":{"type":"string"},
 "smithy.api#Unit":{"type":"structure","members":{"Ignored":{"traits":{"smithy.api#required":{}}}}}
}}"#;

const EXPECTED: &str = concat!(
    "# enumerate and expand are candidates; ",
    "callable only with a reviewed service overlay (VISION 5.1).\n",
    "operations 5\nenumerate 4\nexpand 1\ndenied 0\nunclassified 0\n",
    "denied content 0\ndenied secret 0\ndenied paid 0\ndenied mutating 0\n",
    "denied reference-data 0\nno_read_verb 0\nreadonly_denied 0\n",
    "readonly_unclassified 0\nmust_deny_entries 0\nservice sample 4 1 0 0\n"
);

fn fixture() -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    std::fs::create_dir_all(dir.path().join("models/sample/service/v1"))?;
    std::fs::write(
        dir.path().join("models/sample/service/v1/sample.json"),
        MODEL,
    )?;
    Ok(dir)
}

#[test]
fn absent_unit_empty_optional_and_required_inputs_are_classified() -> TestResult {
    let dir = fixture()?;
    assert_eq!(
        build_report_with_policy(dir.path(), &DenyList::parse("")?)?,
        EXPECTED
    );
    Ok(())
}

#[test]
fn mixed_optional_and_required_members_are_one_expansion() -> TestResult {
    let dir = fixture()?;
    let model = MODEL.replacen(
        r#""Parent":{"target":"n#String","traits":{"smithy.api#required":{}}}"#,
        concat!(
            r#""AFilter":{"target":"n#String"},"#,
            r#""ZParent":{"target":"n#String","traits":{"smithy.api#required":{}}},"#,
            r#""ZZTail":{"target":"n#String"}"#
        ),
        1,
    );
    std::fs::write(
        dir.path().join("models/sample/service/v1/sample.json"),
        model,
    )?;
    assert_eq!(
        build_report_with_policy(dir.path(), &DenyList::parse("")?)?,
        EXPECTED
    );
    Ok(())
}

#[test]
fn model_without_a_service_directory_is_rejected() -> TestResult {
    let dir = fixture()?;
    let path = dir.path().join("models/model.json");
    std::fs::write(&path, MODEL)?;
    assert!(
        matches!(build_report_with_policy(dir.path(), &DenyList::parse("")?),
        Err(Error::ModelSource(source)) if source.to_string() == format!("invalid model path: {}", path.display()))
    );
    Ok(())
}
#[test]
fn production_report_rejects_missing_builtin_denials_without_a_policy_file() -> TestResult {
    let dir = fixture()?;
    assert!(matches!(build_report(dir.path(), dir.path()),
        Err(Error::UnmatchedEntry { line: 21, key }) if key == "GetAuthorizationToken"));
    assert!(!dir.path().join("catalog").exists());
    Ok(())
}
