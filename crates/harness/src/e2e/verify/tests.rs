use super::*;
type TestResult = Result<(), Box<dyn std::error::Error>>;

pub(super) fn compare_keys(expected: &Keys, actual: &Keys) -> Result<(), HarnessError> {
    compare(expected, actual, &counts_for(actual))
}

fn counts_for(actual: &Keys) -> Counts {
    actual
        .iter()
        .map(|(kind, identities)| (kind.clone(), identities.len()))
        .collect()
}

fn compare_against_baseline(
    required: Keys,
    mut actual: Keys,
    baseline: &Baseline,
    output: &mut Vec<u8>,
) -> Result<(), HarnessError> {
    let mut counts = counts_for(&actual);
    let expected = Expected {
        required,
        deferred: Keys::new(),
    };
    snapshot::relevant(&mut actual, &mut counts, baseline, &expected, output)?;
    compare(&expected.required, &actual, &counts)
}

#[test]
fn given_misleading_log_fields_when_verified_then_only_terminal_summary_counts() -> TestResult {
    assert!(
        zero(
            "debug failed=0\nresource_writes_scope=invocation failed=2",
            "failed"
        )
        .is_err()
    );
    assert!(
        zero(
            "resource_writes_scope=invocation resource_writes=1 failed=0",
            "resource_writes"
        )
        .is_err()
    );
    assert!(
        zero(
            "resource_writes_scope=invocation resource_writes=00junk",
            "resource_writes"
        )
        .is_err()
    );
    assert!(zero("failed=0", "failed").is_err());
    zero(
        "resource_writes_scope=invocation failed=2\nresource_writes_scope=invocation failed=0",
        "failed",
    )?;
    Ok(())
}

#[test]
fn given_equal_counts_but_wrong_identity_when_compared_then_verification_fails() -> TestResult {
    let expected = oracle_keys("resource\tkind\taccount\tregion\tseeded\n")?;
    let wrong = oracle_keys("resource\tkind\taccount\tregion\tother\n")?;
    assert!(compare_keys(&expected, &wrong).is_err());
    assert!(compare_keys(&expected, &Keys::new()).is_err());
    compare_keys(&expected, &expected)?;
    Ok(())
}

#[test]
fn given_count_mismatch_when_verified_then_error_names_kind_count_and_first_five_ids() -> TestResult
{
    let expected = oracle_keys(concat!(
        "resource\tAWS::EC2::VPC\ta\tr\tvpc-1\n",
        "resource\tAWS::EC2::VPC\ta\tr\tvpc-2\n",
        "resource\tAWS::EC2::VPC\ta\tr\tvpc-3\n",
        "resource\tAWS::EC2::VPC\ta\tr\tvpc-4\n",
        "resource\tAWS::EC2::VPC\ta\tr\tvpc-5\n",
        "resource\tAWS::EC2::VPC\ta\tr\tvpc-6\n",
    ))?;
    let error = compare_keys(&expected, &Keys::new())
        .err()
        .ok_or("a count mismatch passed")?;
    assert_eq!(
        error.to_string(),
        concat!(
            "seeded identities/counts differ for 1 of 1 kinds: AWS::EC2::VPC ",
            "expected=6 actual=0 missing=[a/r/vpc-1, a/r/vpc-2, a/r/vpc-3, a/r/vpc-4, ",
            "a/r/vpc-5, +1 more] unexpected=[]",
        )
    );
    Ok(())
}

#[test]
fn given_malformed_empty_or_duplicate_oracle_when_parsed_then_verification_refuses_vacuous_success()
{
    [
        "",
        "resource\tkind\taccount\tregion\t",
        "resource\tkind\taccount\tregion\tid\nresource\tkind\taccount\tregion\tid",
        "unexpected\tx",
    ]
    .iter()
    .for_each(|text| assert!(oracle_keys(text).is_err()));
}

#[test]
fn given_edge_with_empty_endpoint_when_parsed_then_oracle_is_malformed() -> TestResult {
    let base = "resource\tkind\taccount\tregion\tid\n";
    let good = format!("{base}edge\tkind\tid\tcontains\tkind\tchild\n");
    assert_eq!(oracle_keys(&good)?, oracle_keys(base)?);
    let invalid = format!("{base}edge\tkind\tid\tcontains\tkind\t\n");
    let error = oracle_keys(&invalid)
        .err()
        .ok_or("accepted an empty edge endpoint")?;
    assert_eq!(error.to_string(), "malformed seed oracle row");
    Ok(())
}

#[test]
fn given_unlisted_kind_in_delta_when_compared_then_first_five_unexpected_ids_are_named()
-> TestResult {
    let expected = oracle_keys("resource\tlisted\ta\tr\tid\n")?;
    let actual = oracle_keys(concat!(
        "resource\tlisted\ta\tr\tid\n",
        "resource\tunlisted\ta\tr\tone\nresource\tunlisted\ta\tr\ttwo\n",
        "resource\tunlisted\ta\tr\tthree\nresource\tunlisted\ta\tr\tfour\n",
        "resource\tunlisted\ta\tr\tfive\nresource\tunlisted\ta\tr\tsix\n",
    ))?;
    let error = compare_keys(&expected, &actual)
        .err()
        .ok_or("unlisted kind passed")?;
    assert_eq!(
        error.to_string(),
        concat!(
            "seeded identities/counts differ for 1 of 2 kinds: unlisted expected=0 actual=6 ",
            "missing=[] unexpected=[a/r/five, a/r/four, a/r/one, a/r/six, a/r/three, +1 more]"
        )
    );
    Ok(())
}

#[test]
fn given_secret_in_child_log_when_reporting_then_only_numeric_metrics_escape() {
    let log = "password=hunter2\nresource_writes_scope=invocation failed=3 unsupported=9 database=postgres://secret";
    assert_eq!(outcome(log), "unsupported=9 failed=3");
}

#[test]
fn given_preseed_defaults_when_comparing_full_identity_delta_then_only_new_seeded_resources_match()
-> TestResult {
    let baseline = Baseline(oracle_keys("resource\tkind\ta\tr\timplicit\n")?);
    let actual = oracle_keys("resource\tkind\ta\tr\timplicit\nresource\tkind\ta\tr\tseeded\n")?;
    let expected = oracle_keys("resource\tkind\ta\tr\tseeded\n")?;
    compare_against_baseline(expected.clone(), actual, &baseline, &mut Vec::new())?;
    let unexpected = oracle_keys(
        "resource\tkind\ta\tr\timplicit\nresource\tkind\ta\tr\tseeded\nresource\tkind\ta\tr\tunexpected\n",
    )?;
    let error = compare_against_baseline(expected, unexpected, &baseline, &mut Vec::new())
        .err()
        .ok_or("unexpected post-seed identity passed")?;
    assert!(error.to_string().contains("a/r/unexpected"));
    Ok(())
}

#[test]
fn given_same_id_in_distinct_scopes_when_subtracting_then_account_region_and_kind_are_preserved()
-> TestResult {
    let baseline = Baseline(oracle_keys("resource\tkind\ta\tr\tid\n")?);
    let actual = oracle_keys(
        "resource\tkind\ta\tr\tid\nresource\tkind\tb\tr\tid\nresource\tkind\ta\ts\tid\nresource\tother\ta\tr\tid\n",
    )?;
    let expected = oracle_keys(
        "resource\tkind\tb\tr\tid\nresource\tkind\ta\ts\tid\nresource\tother\ta\tr\tid\n",
    )?;
    compare_against_baseline(expected, actual, &baseline, &mut Vec::new())?;
    Ok(())
}

#[test]
fn given_oracle_identity_already_present_before_seed_when_delta_is_verified_then_it_counts()
-> TestResult {
    let expected = oracle_keys("resource\tkind\ta\tr\tseeded\n")?;
    let baseline = Baseline(oracle_keys("resource\tkind\ta\tr\tseeded\n")?);
    let actual = oracle_keys("resource\tkind\ta\tr\tseeded\n")?;
    let mut output = Vec::new();
    compare_against_baseline(expected.clone(), actual, &baseline, &mut output)?;
    assert_eq!(String::from_utf8(output)?, "preexisting kind a/r/seeded\n");
    let mut output = Vec::new();
    let error = compare_against_baseline(expected, Keys::new(), &baseline, &mut output)
        .err()
        .ok_or("deleted baseline identity passed")?;
    assert!(error.to_string().contains("missing=[a/r/seeded]"));
    assert!(output.is_empty());
    Ok(())
}

#[test]
fn given_sql_count_disagrees_with_distinct_ids_when_verified_then_equal_identity_sets_do_not_pass()
-> TestResult {
    let expected = Expected {
        required: oracle_keys("resource\tkind\ta\tr\tseeded\n")?,
        deferred: Keys::new(),
    };
    let mut actual = oracle_keys("resource\tkind\ta\tr\tseeded\nresource\tkind\ta\tr\tdefault\n")?;
    let baseline = Baseline(oracle_keys(
        "resource\tkind\ta\tr\tdefault\nresource\tkind\ta\tr\tgone\n",
    )?);
    let mut counts = Counts::from([("kind".into(), 3)]);
    snapshot::relevant(
        &mut actual,
        &mut counts,
        &baseline,
        &expected,
        &mut Vec::new(),
    )?;
    let error = compare(&expected.required, &actual, &counts)
        .err()
        .ok_or("count comparison discarded the SQL count")?;
    assert_eq!(
        error.to_string(),
        "seeded identities/counts differ for 1 of 1 kinds: kind expected=1 actual=2 missing=[] unexpected=[]"
    );
    Ok(())
}

#[test]
fn given_service_created_oracle_row_when_compared_then_its_identity_is_still_required() -> TestResult
{
    let expected =
        oracle_keys("resource\tAWS::Connect::ContactFlow\ta\tr\tflow\tservice-created\n")?;
    let missing = compare_keys(&expected, &Keys::new())
        .err()
        .ok_or("service-created resource was excluded from verification")?;
    assert!(missing.to_string().contains("missing=[a/r/flow]"));
    compare_keys(
        &expected,
        &oracle_keys("resource\tAWS::Connect::ContactFlow\ta\tr\tflow\n")?,
    )?;
    Ok(())
}

struct ReportSession<'a> {
    output: &'a Path,
    stages: &'a mut Vec<&'static str>,
}

impl crate::e2e::workflow::Session for ReportSession<'_> {
    async fn stage(&mut self, stage: crate::e2e::workflow::Stage) -> Result<(), HarnessError> {
        self.stages.push(stage.name());
        if stage != crate::e2e::workflow::Stage::Counts {
            return Ok(());
        }
        let expected = oracle_keys("resource\tkind\ta\tr\tseeded\n");
        let plans = coverage_status::Plans::from([(
            "kind".into(),
            BTreeSet::from(["test:ListResources".into()]),
        )]);
        let mut evidence = coverage_status::Evidence::default();
        evidence.probe(
            "test:ListResources".into(),
            coverage_status::Probe::Complete,
        );
        report_then_verify(
            expected,
            async |_| {
                coverage_status::publish(self.output, &plans, &evidence, Selection::default())
            },
            async |expected| {
                if !self.output.is_file() {
                    return Err(HarnessError::invalid("counts started without report"));
                }
                compare_keys(&expected, &Keys::new())
            },
        )
        .await
    }

    async fn finish(self, interrupted: bool) -> Result<(), HarnessError> {
        self.stages
            .push(if interrupted { "interrupted" } else { "stop" });
        Ok(())
    }
}

#[tokio::test]
async fn given_missing_resource_when_stages_run_then_report_precedes_failed_counts_and_resweep_stops()
-> TestResult {
    let root = tempfile::tempdir()?;
    let output = root.path().join("coverage-status.report");
    let mut stages = Vec::new();
    let error = crate::e2e::workflow::drive(ReportSession {
        output: &output,
        stages: &mut stages,
    })
    .await
    .err()
    .ok_or("missing seeded identity passed")?;
    assert_eq!(
        error.to_string(),
        "seeded identities/counts differ for 1 of 1 kinds: kind expected=1 actual=0 missing=[a/r/seeded] unexpected=[]"
    );
    assert_eq!(
        stages,
        ["baseline", "seed", "sweep", "verify-counts", "stop"]
    );
    assert_eq!(
        std::fs::read_to_string(output)?,
        concat!(
            "totals\tcallable=1\temulator_verified=0\timplemented=1\n",
            "kind\timplemented\tempty: nothing seeded\n",
        )
    );
    Ok(())
}
