use super::*;
use crate::e2e::{
    verify::{Baseline, Keys, compare, oracle_keys, verified_keys},
    workspace_root,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn coverage() -> Result<Coverage, HarnessError> {
    Coverage::load(
        &workspace_root().join("catalog/aws-api.catalog"),
        &workspace_root().join("crates/harness/seeds"),
        &["cloudwatch".into(), "ec2".into()],
    )
}

#[test]
fn given_catalog_deferred_alarm_when_classified_then_reviewed_safety_reason_remains_visible()
-> TestResult {
    let coverage = coverage()?;
    let reason = coverage
        .reason("AWS::CloudWatch::Alarm")?
        .ok_or("missing deferral")?;
    assert!(reason.contains("cloudwatch:DescribeAlarms:"));
    assert!(reason.contains("StateReasonData"));
    assert!(reason.contains("exclude_from_hash does not remove content"));
    assert!(coverage.reason("AWS::EC2::VPC")?.is_none());
    assert!(coverage.reason("AWS::Unknown::Resource").is_err());
    Ok(())
}

#[test]
fn given_deferred_and_callable_oracle_kinds_when_verified_then_only_deferred_kind_is_not_compared()
-> TestResult {
    let expected = oracle_keys(
        "resource\tAWS::CloudWatch::Alarm\ta\tr\talarm\nresource\tAWS::EC2::VPC\ta\tr\tseeded\n",
    )?;
    let verified = verified_keys(expected, &coverage()?)?;
    assert_eq!(verified.len(), 1);
    assert!(verified.contains_key("AWS::EC2::VPC"));
    assert!(compare(&verified, &Keys::new()).is_err());
    let actual = oracle_keys("resource\tAWS::EC2::VPC\ta\tr\tseeded\n")?;
    compare(
        &verified,
        &super::super::snapshot::added(actual, &Baseline(Keys::new())),
    )?;
    Ok(())
}

#[tokio::test]
async fn given_only_deferred_seeded_kinds_when_counted_then_no_database_is_queried() -> TestResult {
    let coverage = Coverage {
        callable: BTreeSet::new(),
        deferred: BTreeMap::from([("deferred".into(), "reviewed deferral".into())]),
    };
    let root = tempfile::tempdir()?;
    let oracle = root.path().join("oracle.tsv");
    std::fs::write(&oracle, "resource\tdeferred\ta\tr\tid\n")?;
    let error = crate::e2e::verify::counts(
        "not-a-database-url",
        &oracle,
        &Baseline(Keys::new()),
        &coverage,
    )
    .await
    .err()
    .ok_or("all-deferred oracle passed verification")?;
    assert_eq!(error.to_string(), "no callable seeded kinds to verify");
    Ok(())
}
