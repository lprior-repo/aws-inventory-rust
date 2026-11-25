use super::*;
use crate::e2e::verify::coverage_status::Plans;
use std::collections::{BTreeMap, BTreeSet};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_persisted_failure_details_when_read_then_only_valid_codes_are_reported() {
    let long = format!("AWS service failure: {}", "x".repeat(129));
    for (detail, code) in [
        (
            Some("AWS service failure: AccessDenied (SecretAccessDenied)"),
            "SecretAccessDenied",
        ),
        (Some("AWS service failure: AccessDenied"), "AccessDenied"),
        (Some("Prefix: x"), "Prefix"),
        (None, "unknown"),
        (Some(""), "unknown"),
        (Some("AWS service failure: dénié"), "unknown"),
        (Some(long.as_str()), "unknown"),
    ] {
        assert_eq!(failed_code(detail), code, "{detail:?}");
    }
}

#[test]
fn given_seed_matrix_file_when_reported_then_rejection_is_joined_and_unknown_steps_fail()
-> TestResult {
    let work = tempfile::tempdir()?;
    let matrix = work.path().join("seed-matrix.tsv");
    let report = work.path().join("coverage-status.report");
    let step = "test\tCreateThing\tthing";
    let kind = "AWS::Test::Thing";
    let coverage = Coverage {
        callable: Plans::from([(kind.into(), BTreeSet::from(["test:ListThings".into()]))]),
        deferred: BTreeMap::new(),
        steps: BTreeMap::from([(step.into(), kind.into())]),
    };
    let mut evidence = Evidence::default();
    std::fs::write(&matrix, format!("{step}\trejected\tBadRequest\tdetail\n"))?;
    coverage.rejected(&read_text(&matrix)?, &mut evidence)?;
    coverage_status::publish(&report, &coverage.callable, &evidence, Selection::default())?;
    assert_eq!(
        std::fs::read_to_string(&report)?,
        concat!(
            "totals\tcallable=1\temulator_verified=0\timplemented=1\n",
            "AWS::Test::Thing\timplemented\tseed rejected: BadRequest\n",
        )
    );
    std::fs::write(
        &matrix,
        "test\tCreateThing\tunknown\trejected\tBadRequest\tdetail\n",
    )?;
    let error = coverage
        .rejected(&read_text(&matrix)?, &mut evidence)
        .err()
        .ok_or("unknown rejection step passed")?;
    assert_eq!(
        error.to_string(),
        "rejected seed matrix step is not in recipes"
    );
    Ok(())
}
