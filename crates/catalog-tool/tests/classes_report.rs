mod classes_support;

use catalog_tool::{
    ReportMode,
    classes::{Error, build_report_with_policy, run_with_policy},
};
use classes_support::{REPORT, fixture, policy, write};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const COMMITTED: &str = "catalog/aws-classes.report";

fn run(root: &std::path::Path, source: &std::path::Path, mode: ReportMode) -> Result<(), Error> {
    run_with_policy(root, source, mode, &policy()?)
}

#[test]
fn two_service_report_matches_hand_counted_text() -> TestResult {
    let dir = fixture()?;
    assert_eq!(build_report_with_policy(dir.path(), &policy()?)?, REPORT);
    assert_eq!(build_report_with_policy(dir.path(), &policy()?)?, REPORT);
    Ok(())
}

#[test]
fn print_never_changes_the_committed_report() -> TestResult {
    let dir = fixture()?;
    write(dir.path(), COMMITTED, "preserve\n")?;
    run(dir.path(), dir.path(), ReportMode::Print)?;
    assert_eq!(
        std::fs::read_to_string(dir.path().join(COMMITTED))?,
        "preserve\n"
    );
    Ok(())
}

#[test]
fn write_creates_and_replaces_the_report_and_check_accepts_exact_bytes() -> TestResult {
    let dir = fixture()?;
    run(dir.path(), dir.path(), ReportMode::Write)?;
    assert_eq!(std::fs::read_to_string(dir.path().join(COMMITTED))?, REPORT);

    write(dir.path(), COMMITTED, "stale\n")?;
    run(dir.path(), dir.path(), ReportMode::Write)?;
    run(dir.path(), dir.path(), ReportMode::Check)?;
    assert_eq!(std::fs::read_to_string(dir.path().join(COMMITTED))?, REPORT);
    Ok(())
}

#[test]
fn check_reports_all_byte_sensitive_differences_without_writing() -> TestResult {
    let dir = fixture()?;
    let changed = REPORT.replacen(
        "operations 10\nenumerate 2\n",
        "operations 10 \nenumerate 2\r\n",
        1,
    );
    write(dir.path(), COMMITTED, &changed)?;
    let error = run(dir.path(), dir.path(), ReportMode::Check).err();
    assert_eq!(error.map(|e| e.to_string()), Some(concat!(
        "line 2 differs: expected Some(\"operations 10\\n\"), got Some(\"operations 10 \\n\")\n",
        "line 3 differs: expected Some(\"enumerate 2\\n\"), got Some(\"enumerate 2\\r\\n\")\n"
    ).to_owned()));
    assert_eq!(
        std::fs::read_to_string(dir.path().join(COMMITTED))?,
        changed
    );
    Ok(())
}

#[test]
fn unmatched_deny_name_is_a_typed_error_with_its_line() -> TestResult {
    let dir = fixture()?;
    let deny = catalog_tool::classes::DenyList::parse("# reason\nGetTypo\tsecret\n")?;
    let error = build_report_with_policy(dir.path(), &deny).err();
    assert!(matches!(error, Some(Error::UnmatchedEntry { line: 2, key }) if key == "GetTypo"));
    Ok(())
}

#[test]
fn unmatched_scoped_entry_cannot_be_satisfied_by_another_service() -> TestResult {
    let dir = fixture()?;
    let deny = catalog_tool::classes::DenyList::parse("alpha/ListBlocked\tcontent\n")?;
    assert!(matches!(build_report_with_policy(dir.path(), &deny),
        Err(Error::UnmatchedEntry { line: 1, key }) if key == "alpha/ListBlocked"));
    Ok(())
}

#[test]
fn model_error_leaves_a_preexisting_report_intact() -> TestResult {
    let dir = fixture()?;
    write(dir.path(), COMMITTED, "preserve\n")?;
    write(dir.path(), "models/z-last/service/v1/bad.json", "{")?;
    let error = run(dir.path(), dir.path(), ReportMode::Write).err();
    assert!(matches!(
        error,
        Some(Error::ModelSource(source)) if source.to_string().contains("bad.json")
    ));
    assert_eq!(
        std::fs::read_to_string(dir.path().join(COMMITTED))?,
        "preserve\n"
    );
    Ok(())
}

#[test]
fn unmatched_entry_leaves_a_preexisting_report_intact() -> TestResult {
    let dir = fixture()?;
    write(dir.path(), COMMITTED, "preserve\n")?;
    let deny = catalog_tool::classes::DenyList::parse("GetTypo\tsecret\n")?;
    assert!(matches!(
        run_with_policy(dir.path(), dir.path(), ReportMode::Write, &deny),
        Err(Error::UnmatchedEntry { line: 1, .. })
    ));
    assert_eq!(
        std::fs::read_to_string(dir.path().join(COMMITTED))?,
        "preserve\n"
    );
    Ok(())
}

#[test]
fn missing_report_is_a_named_io_error() -> TestResult {
    let dir = fixture()?;
    let error = run(dir.path(), dir.path(), ReportMode::Check).err();
    assert!(matches!(
        error,
        Some(Error::Report(catalog_tool::ReportError::Io { file, source }))
            if file == dir.path().join(COMMITTED)
                && source.kind() == std::io::ErrorKind::NotFound
    ));
    Ok(())
}

#[test]
fn explicit_policy_is_independent_of_source_and_output_roots() -> TestResult {
    let source = fixture()?;
    let root = tempfile::tempdir()?;
    run_with_policy(root.path(), source.path(), ReportMode::Write, &policy()?)?;
    assert_eq!(
        std::fs::read_to_string(root.path().join(COMMITTED))?,
        REPORT
    );
    assert!(!source.path().join("catalog").exists());
    Ok(())
}

#[test]
fn no_models_is_not_a_successful_empty_inventory() -> TestResult {
    let dir = tempfile::tempdir()?;
    std::fs::create_dir_all(dir.path().join("models"))?;
    assert!(matches!(
        build_report_with_policy(dir.path(), &catalog_tool::classes::DenyList::parse("")?),
        Err(Error::ModelSource(_))
    ));
    Ok(())
}

#[test]
fn missing_service_shape_is_rejected() -> TestResult {
    let dir = fixture()?;
    write(dir.path(), "models/z/service/v1/z.json", r#"{"shapes":{}}"#)?;
    assert!(matches!(
        build_report_with_policy(dir.path(), &policy()?),
        Err(Error::Model(catalog_tool::smithy::Error::NoService(_)))
    ));
    Ok(())
}

#[test]
fn two_service_shapes_are_rejected() -> TestResult {
    let dir = fixture()?;
    write(
        dir.path(),
        "models/z/service/v1/z.json",
        r#"{"shapes":{"n#A":{"type":"service"},"n#B":{"type":"service"}}}"#,
    )?;
    assert!(matches!(
        build_report_with_policy(dir.path(), &policy()?),
        Err(Error::Model(catalog_tool::smithy::Error::SeveralServices(
            _
        )))
    ));
    Ok(())
}

#[test]
fn check_rejects_a_single_changed_byte_without_writing() -> TestResult {
    let dir = fixture()?;
    let changed = REPORT.replacen("operations 10", "operations 11", 1);
    write(dir.path(), COMMITTED, &changed)?;
    assert!(matches!(
        run(dir.path(), dir.path(), ReportMode::Check),
        Err(Error::Report(catalog_tool::ReportError::Mismatch(actual)))
            if actual == "line 2 differs: expected Some(\"operations 10\\n\"), got Some(\"operations 11\\n\")\n"
    ));
    assert_eq!(
        std::fs::read_to_string(dir.path().join(COMMITTED))?,
        changed
    );
    Ok(())
}

#[test]
fn check_rejects_a_missing_final_newline_without_writing() -> TestResult {
    let dir = fixture()?;
    let Some(changed) = REPORT.strip_suffix('\n') else {
        return Err("report fixture must end in LF".into());
    };
    write(dir.path(), COMMITTED, changed)?;
    assert!(matches!(
        run(dir.path(), dir.path(), ReportMode::Check),
        Err(Error::Report(catalog_tool::ReportError::Mismatch(actual)))
            if actual == "line 17 differs: expected Some(\"service beta 1 0 4 0\\n\"), got Some(\"service beta 1 0 4 0\")\n"
    ));
    assert_eq!(
        std::fs::read_to_string(dir.path().join(COMMITTED))?,
        changed
    );
    Ok(())
}
