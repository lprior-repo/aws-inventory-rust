use super::*;

#[test]
fn state_machine_and_activity_arns_split_exact_identity() -> TestResult {
    let parts = arn::split_state_machine_arn(STATE_MACHINE_ARN)
        .map_err(|error| -> Box<dyn std::error::Error> { error.into() })?;
    assert_eq!(parts.partition, "aws");
    assert_eq!(parts.region, "us-east-1");
    assert_eq!(parts.account, "111111111111");
    assert_eq!(parts.name, STATE_MACHINE_NAME);
    let parts = arn::split_activity_arn(ACTIVITY_ARN)
        .map_err(|error| -> Box<dyn std::error::Error> { error.into() })?;
    assert_eq!(parts.name, ACTIVITY_NAME);
    Ok(())
}

#[test]
fn malformed_arns_are_rejected() {
    for value in [
        "",
        "not-an-arn",
        "arn:aws:sns:us-east-1:111111111111:stateMachine:orders",
        "arn:aws:states:us-east-1:111111111111:execution:orders:run",
        "arn:aws:states:us-east-1:111111111111:stateMachine:",
        "arn:aws:states:us-east-1:111:stateMachine:orders",
        "arn:aws:states::111111111111:stateMachine:orders",
        "arn:aws:states:us-east-1:111111111111:activity:",
    ] {
        assert!(
            arn::split_state_machine_arn(value).is_err(),
            "accepted state machine ARN: {value}"
        );
    }
    for value in [
        STATE_MACHINE_ARN,
        "arn:aws:states:us-east-1:111111111111:activity:bad:name",
    ] {
        assert!(
            arn::split_activity_arn(value).is_err(),
            "accepted activity ARN: {value}"
        );
    }
}

#[test]
fn scope_mismatches_are_rejected() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    for value in [
        "arn:aws:states:us-west-2:111111111111:stateMachine:orders",
        "arn:aws:states:us-east-1:222222222222:stateMachine:orders",
        "arn:aws-cn:states:us-east-1:111111111111:stateMachine:orders",
    ] {
        let parts = arn::split_state_machine_arn(value)
            .map_err(|error| -> Box<dyn std::error::Error> { error.into() })?;
        assert!(!arn::is_in_scope(&parts, &ctx), "accepted scope: {value}");
    }
    Ok(())
}

#[test]
fn role_targets_require_exact_in_scope_iam_arns() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    assert_eq!(arn::role_target(ROLE_ARN, &ctx).as_deref(), Some(ROLE_ARN));
    for value in [
        "runner",
        "arn:aws:iam::111111111111:user/runner",
        "arn:aws:iam::222222222222:role/runner",
        "arn:aws-cn:iam::111111111111:role/runner",
        "arn:aws:iam:us-east-1:111111111111:role/runner",
        "arn:aws:iam::111111111111:role/",
    ] {
        assert!(arn::role_target(value, &ctx).is_none(), "accepted: {value}");
    }
    Ok(())
}

#[test]
fn kms_targets_require_exact_in_scope_key_arns() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    assert_eq!(arn::kms_target(KEY_ARN, &ctx).as_deref(), Some(KEY_ARN));
    for value in [
        "alias/aws/states",
        "1234abcd-12ab-34cd-56ef-1234567890ab",
        "arn:aws:kms:us-west-2:111111111111:key/1234abcd-12ab-34cd-56ef-1234567890ab",
        "arn:aws:kms:us-east-1:222222222222:key/1234abcd-12ab-34cd-56ef-1234567890ab",
        "arn:aws:kms:us-east-1:111111111111:alias/orders",
        "arn:aws:kms:us-east-1:111111111111:key/not-a-key",
    ] {
        assert!(arn::kms_target(value, &ctx).is_none(), "accepted: {value}");
    }
    Ok(())
}
