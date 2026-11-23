use super::*;
type TestResult = Result<(), Box<dyn std::error::Error>>;

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
    assert!(compare(&expected, &wrong).is_err());
    assert!(compare(&expected, &Keys::new()).is_err());
    compare(&expected, &expected)?;
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
    compare(&expected, &snapshot::added(actual, &baseline))?;
    let unexpected = oracle_keys(
        "resource\tkind\ta\tr\timplicit\nresource\tkind\ta\tr\tseeded\nresource\tkind\ta\tr\tunexpected\n",
    )?;
    let error = compare(&expected, &snapshot::added(unexpected, &baseline))
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
    compare(&expected, &snapshot::added(actual, &baseline))?;
    Ok(())
}

#[test]
fn given_oracle_identity_already_present_before_seed_when_delta_is_verified_then_it_cannot_false_pass()
-> TestResult {
    let expected = oracle_keys("resource\tkind\ta\tr\tseeded\n")?;
    let baseline = Baseline(oracle_keys("resource\tkind\ta\tr\tseeded\n")?);
    let actual = oracle_keys("resource\tkind\ta\tr\tseeded\n")?;
    assert!(compare(&expected, &snapshot::added(actual, &baseline)).is_err());
    let wrong = oracle_keys("resource\tkind\ta\tr\twrong\n")?;
    assert!(compare(&expected, &snapshot::added(wrong, &Baseline(Keys::new()))).is_err());
    Ok(())
}
