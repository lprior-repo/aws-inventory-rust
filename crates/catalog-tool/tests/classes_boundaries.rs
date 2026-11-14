use catalog_tool::classes::{
    DenyList, Error, ParseError, ParseErrorKind, ReportMode, build_report, run,
};
use std::{fs, path::Path};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const COMMITTED: &str = "catalog/aws-classes.report";
const MODEL: &str = r#"{"shapes":{"n#Svc":{"type":"service"}}}"#;
const EMPTY_REPORT: &str = concat!(
    "# enumerate and expand are candidates; ",
    "callable only with a reviewed service overlay (VISION 5.1).\n",
    "operations 0\nenumerate 0\nexpand 0\ndenied 0\nunclassified 0\n",
    "denied content 0\ndenied secret 0\ndenied paid 0\ndenied mutating 0\n",
    "denied reference-data 0\nno_read_verb 0\nreadonly_denied 0\n",
    "readonly_unclassified 0\nmust_deny_entries 0\nservice sample 0 0 0 0\n"
);

fn model_fixture(count: usize) -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let models = root.path().join("models/sample/service/v1");
    let overlay = root.path().join("catalog/overlay");
    fs::create_dir_all(&models)?;
    fs::create_dir_all(&overlay)?;
    fs::write(overlay.join("must-deny.txt"), "")?;
    (0..count)
        .try_for_each(|index| fs::write(models.join(format!("model-{index:04}.json")), MODEL))?;
    Ok(root)
}

fn write_stale_report(root: &Path, bytes: usize) -> std::io::Result<()> {
    fs::write(root.join(COMMITTED), vec![b'x'; bytes])
}

#[test]
fn nonempty_deny_list_is_not_empty() -> TestResult {
    let deny = DenyList::parse("GetA\tsecret\n")?;
    assert_eq!(deny.len(), 1);
    assert!(!deny.is_empty());
    Ok(())
}

#[test]
fn digit_initial_deny_key_is_rejected() {
    assert_eq!(
        DenyList::parse("1Get\tsecret\n").err(),
        Some(ParseError {
            line: 1,
            kind: ParseErrorKind::InvalidKey
        })
    );
}

#[test]
fn later_invalid_deny_key_byte_is_rejected() {
    assert_eq!(
        DenyList::parse("Get-A\tsecret\n").err(),
        Some(ParseError {
            line: 1,
            kind: ParseErrorKind::InvalidKey
        })
    );
}

#[test]
fn classifier_accepts_exactly_4096_model_files() -> TestResult {
    let root = model_fixture(4096)?;
    let actual = build_report(root.path(), root.path())?;
    assert_eq!(actual, EMPTY_REPORT);
    Ok(())
}

#[test]
fn classifier_rejects_4097_model_files() -> TestResult {
    let root = model_fixture(4097)?;
    let actual = build_report(root.path(), root.path());
    assert!(matches!(actual, Err(Error::Limit)));
    Ok(())
}

#[test]
fn report_text_accepts_exactly_4_mib() -> TestResult {
    let root = model_fixture(1)?;
    write_stale_report(root.path(), 4_194_304)?;
    let actual = run(root.path(), root.path(), ReportMode::Check);
    assert!(matches!(actual, Err(Error::ReportMismatch(_))));
    assert_eq!(fs::metadata(root.path().join(COMMITTED))?.len(), 4_194_304);
    Ok(())
}

#[test]
fn report_text_rejects_one_byte_over_4_mib() -> TestResult {
    let root = model_fixture(1)?;
    write_stale_report(root.path(), 4_194_305)?;
    let actual = run(root.path(), root.path(), ReportMode::Check);
    assert!(matches!(actual, Err(Error::Limit)));
    assert_eq!(fs::metadata(root.path().join(COMMITTED))?.len(), 4_194_305);
    Ok(())
}
