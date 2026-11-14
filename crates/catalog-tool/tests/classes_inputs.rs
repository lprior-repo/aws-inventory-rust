use catalog_tool::classes::{Error, build_report};

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
    std::fs::create_dir_all(dir.path().join("catalog/overlay"))?;
    std::fs::write(dir.path().join("catalog/overlay/must-deny.txt"), "")?;
    std::fs::write(
        dir.path().join("models/sample/service/v1/sample.json"),
        MODEL,
    )?;
    Ok(dir)
}

#[test]
fn absent_unit_empty_optional_and_required_inputs_are_classified() -> TestResult {
    // Given supported input forms and one required member, when reporting,
    // then only the required structure produces an expansion.
    let dir = fixture()?;
    assert_eq!(build_report(dir.path(), dir.path())?, EXPECTED);
    Ok(())
}

#[test]
fn mixed_optional_and_required_members_are_one_expansion() -> TestResult {
    // Given optional members before and after a required member in lexical order,
    // when resolving the input, then inspecting only the first or last member cannot suffice.
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
    assert_eq!(build_report(dir.path(), dir.path())?, EXPECTED);
    Ok(())
}

#[test]
fn model_without_a_service_directory_is_rejected() -> TestResult {
    // Given a JSON model directly under models, when reporting, then no scope is fabricated.
    let dir = fixture()?;
    std::fs::write(dir.path().join("models/direct.json"), MODEL)?;
    assert!(matches!(build_report(dir.path(), dir.path()),
        Err(Error::InvalidModelPath(path)) if path == dir.path().join("models/direct.json")));
    Ok(())
}

#[test]
fn missing_deny_file_is_not_treated_as_an_empty_policy() -> TestResult {
    // Given models but no policy file, when building, then the named I/O failure is returned.
    let dir = fixture()?;
    let path = dir.path().join("catalog/overlay/must-deny.txt");
    std::fs::remove_file(&path)?;
    assert!(matches!(build_report(dir.path(), dir.path()),
        Err(Error::Io { file, source })
        if file == path && source.kind() == std::io::ErrorKind::NotFound));
    Ok(())
}
