use super::*;
use std::collections::BTreeSet;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_unlistable_type_when_reported_then_no_difference_is_counted() -> TestResult {
    assert_eq!(
        not_listable("AWS::Example::Thing"),
        Report {
            text: "AWS::Example::Thing\tnot listable\n".into(),
            unexplained_seeded: 0,
            not_comparable_seeded: 0,
        }
    );
    assert!(not_listable("AWS::Example::Thing").passed());
    Ok(())
}

#[test]
fn given_explained_and_unexplained_identities_when_rendered_then_exact_counts_and_reasons_remain()
-> TestResult {
    let listed = BTreeSet::from(["system-1".to_owned(), "user-1".to_owned()]);
    let stored = BTreeSet::from(["extra-1".to_owned()]);
    let rules = identities::rules("AWS::SNS::Topic\tmissing_from_sweep:system-*\temulator topic")?;
    let report = render(
        "AWS::SNS::Topic",
        &identities::compare(&listed, &stored)?,
        &rules,
    )?;
    assert_eq!(
        report.text,
        "AWS::SNS::Topic\tmatched\t0\tmissing_from_sweep\t2\textra_in_sweep\t1\nAWS::SNS::Topic\tmissing_from_sweep\tsystem-1\texplained\temulator topic\nAWS::SNS::Topic\tmissing_from_sweep\tuser-1\tunexplained\t\nAWS::SNS::Topic\textra_in_sweep\textra-1\tunexplained\t\n"
    );
    assert_eq!(report.unexplained_seeded, 2);
    assert!(!report.passed());
    Ok(())
}

#[test]
fn given_only_explained_differences_when_rendered_then_report_passes() -> TestResult {
    let listed = BTreeSet::from(["system-1".to_owned()]);
    let stored = BTreeSet::new();
    let rules = identities::rules("AWS::SNS::Topic\tmissing_from_sweep:system-*\temulator topic")?;
    let report = render(
        "AWS::SNS::Topic",
        &identities::compare(&listed, &stored)?,
        &rules,
    )?;
    assert_eq!(report.unexplained_seeded, 0);
    assert!(report.passed());
    Ok(())
}

#[test]
fn given_control_characters_when_rendered_then_each_identity_stays_on_one_tsv_row() -> TestResult {
    let listed = BTreeSet::from(["one\ttwo\r\n\\".to_owned()]);
    let stored = BTreeSet::new();
    let report = render(
        "AWS::SNS::Topic",
        &identities::compare(&listed, &stored)?,
        &[],
    )?;
    assert_eq!(
        report.text,
        "AWS::SNS::Topic\tmatched\t0\tmissing_from_sweep\t1\textra_in_sweep\t0\nAWS::SNS::Topic\tmissing_from_sweep\tone\\ttwo\\r\\n\\\\\tunexplained\t\n"
    );
    assert_eq!(report.unexplained_seeded, 1);
    Ok(())
}

#[test]
fn given_sdk_failures_when_classified_then_only_explicit_unsupported_codes_are_skipped()
-> TestResult {
    assert!(client::unsupported(Some("UnsupportedActionException")));
    assert!(client::unsupported(Some("UnsupportedAction")));
    assert!(client::unsupported(Some("TypeNotFoundException")));
    assert!(!client::unsupported(Some("NotSupported")));
    assert!(!client::unsupported(Some("NotSupportedException")));
    assert!(!client::unsupported(Some("NotImplemented")));
    assert!(!client::unsupported(Some("NotImplementedException")));
    assert!(!client::unsupported(Some("AccessDeniedException")));
    assert!(!client::unsupported(Some("ResourceNotFoundException")));
    assert!(!client::unsupported(None));
    Ok(())
}

#[test]
fn given_output_aliasing_rules_or_inputs_when_validated_then_rejected() -> TestResult {
    let work = tempfile::tempdir()?;
    let mut args = Args {
        database: "postgres://localhost/inventory".into(),
        endpoint: "https://localhost".into(),
        ca_bundle: work.path().join("ca.pem"),
        region: "us-east-1".into(),
        account: None,
        report: work.path().join("report.tsv"),
        catalog: work.path().join("catalog"),
        universal: work.path().join("universal"),
        differences: work.path().join("differences.tsv"),
        seeds: work.path().join("seeds"),
        oracle: Some(work.path().join("oracle.tsv")),
    };
    [
        &args.catalog,
        &args.universal,
        &args.ca_bundle,
        args.oracle.as_ref().ok_or("missing oracle")?,
    ]
    .into_iter()
    .try_for_each(|path| std::fs::write(path, "input"))?;
    [
        args.differences.clone(),
        args.ca_bundle.clone(),
        args.catalog.clone(),
        args.universal.clone(),
        args.oracle.clone().ok_or("missing oracle")?,
    ]
    .into_iter()
    .try_for_each(|path| -> TestResult {
        args.report = path;
        let error = output::validate(&args)
            .err()
            .ok_or("expected a collision error")?;
        assert_eq!(
            error.to_string(),
            "Cloud Control outputs must not replace inventory or input files"
        );
        Ok(())
    })
}

#[test]
fn given_reason_with_tab_when_rendered_then_the_reason_stays_in_one_tsv_column() -> TestResult {
    let mut rules = identities::rules("AWS::SNS::Topic\tmissing_from_sweep:system-*\treason")?;
    rules.first_mut().ok_or("missing rule")?.reason = "one\ttwo";
    let listed = BTreeSet::from(["system-1".to_owned()]);
    let stored = BTreeSet::new();
    let report = render(
        "AWS::SNS::Topic",
        &identities::compare(&listed, &stored)?,
        &rules,
    )?;
    assert_eq!(
        report.text,
        "AWS::SNS::Topic\tmatched\t0\tmissing_from_sweep\t1\textra_in_sweep\t0\nAWS::SNS::Topic\tmissing_from_sweep\tsystem-1\texplained\tone\\ttwo\n"
    );
    Ok(())
}

#[test]
fn given_unmapped_kinds_when_reported_then_section_is_counted_sorted_and_explicit() -> TestResult {
    let kinds = BTreeSet::from(["AWS::Example::Z".to_owned(), "AWS::ApiOnly::A".to_owned()]);
    assert_eq!(
        unmapped(&kinds)?,
        "# no CloudFormation type\tcount\t2\nAWS::ApiOnly::A\tno CloudFormation type\nAWS::Example::Z\tno CloudFormation type\n# Cloud Control comparisons\n"
    );
    Ok(())
}
