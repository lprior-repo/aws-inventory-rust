use super::*;
use crate::e2e::{
    verify::{Baseline, Keys, oracle_keys, tests::compare_keys, verified_keys},
    workspace_root,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn coverage() -> Result<Coverage, HarnessError> {
    Coverage::load(
        &workspace_root().join("catalog/aws-api.catalog"),
        &workspace_root().join("crates/harness/seeds"),
        &["cloudwatch".into(), "ec2".into(), "ecs".into()],
    )
}

#[test]
fn given_catalog_deferred_ecs_task_definition_when_classified_then_reviewed_safety_reason_remains_visible()
-> TestResult {
    let coverage = coverage()?;
    let reason = coverage
        .reason("AWS::ECS::TaskDefinition")?
        .ok_or("missing deferral")?;
    assert!(reason.contains("ecs:ListTaskDefinitions:"));
    assert!(reason.contains("complete enumeration needs ACTIVE/INACTIVE/DELETE_IN_PROGRESS"));
    assert!(reason.contains("DescribeTaskDefinition remains content-denied"));
    assert!(reason.contains("plaintext environment and container command content"));
    assert!(coverage.reason("AWS::EC2::VPC")?.is_none());
    assert!(coverage.reason("AWS::Unknown::Resource").is_err());
    Ok(())
}

#[test]
fn given_deferred_and_callable_oracle_kinds_when_verified_then_only_deferred_kind_is_not_compared()
-> TestResult {
    let expected = oracle_keys(
        "resource\tAWS::ECS::TaskDefinition\ta\tr\ttask\nresource\tAWS::EC2::VPC\ta\tr\tseeded\n",
    )?;
    let verified = verified_keys(expected, &coverage()?)?;
    assert_eq!(verified.required.len(), 1);
    assert!(verified.required.contains_key("AWS::EC2::VPC"));
    assert!(compare_keys(&verified.required, &Keys::new()).is_err());
    let mut actual = oracle_keys(concat!(
        "resource\tAWS::EC2::VPC\ta\tr\tseeded\n",
        "resource\tAWS::ECS::TaskDefinition\ta\tr\ttask\n",
        "resource\tAWS::ECS::TaskDefinition\ta\tr\tunexpected\n",
    ))?;
    let mut counts = super::super::Counts::from([
        ("AWS::EC2::VPC".into(), 1),
        ("AWS::ECS::TaskDefinition".into(), 2),
    ]);
    super::super::snapshot::relevant(
        &mut actual,
        &mut counts,
        &Baseline(Keys::new()),
        &verified,
        &mut Vec::new(),
    )?;
    let error = super::super::compare(&verified.required, &actual, &counts)
        .err()
        .ok_or("unexpected resource of deferred kind passed")?;
    assert!(error.to_string().contains(
        "AWS::ECS::TaskDefinition expected=0 actual=1 missing=[] unexpected=[a/r/unexpected]"
    ));
    Ok(())
}

#[test]
fn given_seeded_cloudwatch_alarm_when_verified_then_its_identity_is_compared() -> TestResult {
    let coverage = coverage()?;
    let alarm = "resource\tAWS::CloudWatch::Alarm\ta\tr\talarm\n";
    let verified = verified_keys(oracle_keys(alarm)?, &coverage)?;

    assert_eq!(coverage.reason("AWS::CloudWatch::Alarm")?, None);
    let error = compare_keys(&verified.required, &Keys::new())
        .err()
        .ok_or("missing alarm passed comparison")?;
    assert_eq!(
        error.to_string(),
        "seeded identities/counts differ for 1 of 1 kinds: AWS::CloudWatch::Alarm expected=1 actual=0 missing=[a/r/alarm] unexpected=[]"
    );
    compare_keys(&verified.required, &oracle_keys(alarm)?)?;
    Ok(())
}

#[tokio::test]
async fn given_only_deferred_seeded_kinds_when_counted_then_no_database_is_queried() -> TestResult {
    let coverage = Coverage {
        callable: Plans::new(),
        deferred: BTreeMap::from([("deferred".into(), "reviewed deferral".into())]),
        steps: BTreeMap::new(),
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
