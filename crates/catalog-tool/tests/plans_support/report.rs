use super::{
    TestResult, model,
    report_fixture::{EXPECTED, fixture, policy, rich_model},
    write_model,
};
use catalog_tool::{classes, plans};
use serde_json::json;

#[test]
fn report_has_exact_nonzero_fixture_bytes_for_every_counter_and_service_column() -> TestResult {
    super::report_fixture::assert_fixture_report()
}

#[test]
fn denied_list_stays_in_pagination_population_but_not_enumeration() -> TestResult {
    let dir = fixture()?;
    let report = plans::build_report_with_policy(dir.path(), &policy()?)?;
    assert_eq!(report, EXPECTED);
    let classes = classes::build_report_with_policy(dir.path(), &policy()?)?;
    assert!(classes.lines().any(|line| line == "enumerate 8"));
    assert!(classes.lines().any(|line| line == "expand 4"));
    assert!(report.lines().any(|line| line == "list_describe_denied 1"));
    Ok(())
}

#[test]
fn report_is_deterministic_under_file_creation_order() -> TestResult {
    let first = tempfile::tempdir()?;
    let second = tempfile::tempdir()?;
    let value = rich_model()?;
    ["alpha", "beta"]
        .into_iter()
        .try_for_each(|s| write_model(first.path(), s, &value))?;
    ["beta", "alpha"]
        .into_iter()
        .try_for_each(|s| write_model(second.path(), s, &value))?;
    let report = plans::build_report_with_policy(first.path(), &policy()?)?;
    assert_eq!(
        report,
        plans::build_report_with_policy(second.path(), &policy()?)?
    );
    assert!(report.ends_with("service alpha 8 4 2 2 3 6 4\nservice beta 8 4 2 2 3 6 4\n"));
    Ok(())
}

#[test]
fn vocabulary_is_taken_from_other_services_before_missing_paginators_are_split() -> TestResult {
    let dir = tempfile::tempdir()?;
    write_model(dir.path(), "zeta", &super::paginated())?;
    write_model(
        dir.path(),
        "alpha",
        &model(json!({
            "n#ListOther":{"type":"operation","output":{"target":"n#Output"}},
            "n#Output":{"type":"structure","members":{"Token":{"target":"smithy.api#String"}}}
        })),
    )?;
    let report = plans::build_report_with_policy(dir.path(), &classes::DenyList::parse("")?)?;
    assert!(
        report
            .lines()
            .any(|line| line == "pagination_needs_overlay_token_output_only 1")
    );
    assert!(
        report
            .lines()
            .any(|line| line == "service alpha 1 0 0 0 0 0 1")
    );
    Ok(())
}

#[test]
fn batch_counterparts_are_same_service_exact_then_remove_one_trailing_s() -> TestResult {
    let dir = tempfile::tempdir()?;
    write_model(
        dir.path(),
        "alpha",
        &model(json!({
            "n#BatchGetExact":{"type":"operation"},"n#GetExact":{"type":"operation"},
            "n#BatchDescribeThings":{"type":"operation"},"n#DescribeThing":{"type":"operation"},
            "n#BatchGetMissing":{"type":"operation"},"n#BatchGetAcross":{"type":"operation"}
        })),
    )?;
    write_model(
        dir.path(),
        "beta",
        &model(json!({"n#GetAcross":{"type":"operation"}})),
    )?;
    let report = plans::build_report_with_policy(dir.path(), &classes::DenyList::parse("")?)?;
    assert!(report.lines().any(|line| line == "batch_reads 4"));
    assert!(
        report
            .lines()
            .any(|line| line == "batch_reads_with_single_counterpart 2")
    );
    Ok(())
}

#[test]
fn invalid_classifier_inputs_and_unmatched_denials_remain_fail_closed() -> TestResult {
    let dir = tempfile::tempdir()?;
    let absent = classes::DenyList::parse("ListAbsent\tcontent\n")?;
    write_model(
        dir.path(),
        "alpha",
        &model(json!({"n#ListRoots":{"type":"operation"}})),
    )?;
    assert!(matches!(
        plans::build_report_with_policy(dir.path(), &absent),
        Err(plans::Error::Classes(classes::Error::UnmatchedEntry {
            line: 1,
            ..
        }))
    ));
    write_model(
        dir.path(),
        "alpha",
        &model(json!({"n#ListRoots":{"type":"operation","input":{"target":"n#Missing"}}})),
    )?;
    assert!(matches!(
        plans::build_report_with_policy(dir.path(), &classes::DenyList::parse("")?),
        Err(plans::Error::Classes(classes::Error::InvalidInput { .. }))
    ));
    Ok(())
}

#[test]
fn existing_classifier_fixture_and_helpers_remain_compatible() -> TestResult {
    let dir = crate::classes_support::fixture()?;
    assert_eq!(
        classes::build_report_with_policy(dir.path(), &crate::classes_support::policy()?)?,
        crate::classes_support::REPORT
    );
    let report = plans::build_report_with_policy(dir.path(), &crate::classes_support::policy()?)?;
    assert!(report.lines().any(|line| line == "enumerate 2"));
    assert!(report.lines().any(|line| line == "expand 1"));
    assert!(
        report
            .lines()
            .any(|line| line == "expand_needs_overlay_not_in_resource 1")
    );
    Ok(())
}
