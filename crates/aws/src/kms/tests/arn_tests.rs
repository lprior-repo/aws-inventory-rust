use super::*;

#[test]
fn key_arns_accept_uuid_and_multi_region_ids() -> TestResult {
    let (parts, key_id) = arn::split_key_arn(KEY_ARN)
        .map_err(|error| -> Box<dyn std::error::Error> { error.into() })?;
    assert_eq!(parts.partition, "aws");
    assert_eq!(parts.region, "us-east-1");
    assert_eq!(parts.account, "111111111111");
    assert_eq!(key_id, KEY_ID);
    let multi = "arn:aws:kms:us-east-1:111111111111:key/mrk-1234abcd12ab34cd56ef1234567890ab";
    let (_, key_id) = arn::split_key_arn(multi)
        .map_err(|error| -> Box<dyn std::error::Error> { error.into() })?;
    assert_eq!(key_id, "mrk-1234abcd12ab34cd56ef1234567890ab");
    Ok(())
}

#[test]
fn malformed_key_arns_are_rejected() {
    for arn in [
        "",
        "not-an-arn",
        "arn:aws:sns:us-east-1:111111111111:key/1234abcd-12ab-34cd-56ef-1234567890ab",
        "arn:aws:kms:us-east-1:111111111111:alias/orders",
        "arn:aws:kms:us-east-1:111111111111:key/not-a-key",
        "arn:aws:kms:us-east-1:111:key/1234abcd-12ab-34cd-56ef-1234567890ab",
        "arn:aws:kms::111111111111:key/1234abcd-12ab-34cd-56ef-1234567890ab",
        "arn:aws:kms:us-east-1:111111111111:key/1234abcd12ab34cd56ef1234567890ab",
    ] {
        assert!(arn::split_key_arn(arn).is_err(), "accepted: {arn}");
    }
}

#[test]
fn alias_names_keep_their_prefix_and_reject_path_traversal() -> TestResult {
    let (_, name) = arn::split_alias_arn(ALIAS_ARN)
        .map_err(|error| -> Box<dyn std::error::Error> { error.into() })?;
    assert_eq!(name, ALIAS_NAME);
    for arn in [
        "arn:aws:kms:us-east-1:111111111111:orders",
        "arn:aws:kms:us-east-1:111111111111:alias/",
        "arn:aws:kms:us-east-1:111111111111:alias//orders",
        "arn:aws:kms:us-east-1:111111111111:alias/orders/",
        "arn:aws:kms:us-east-1:111111111111:alias/orders name",
    ] {
        assert!(arn::split_alias_arn(arn).is_err(), "accepted: {arn}");
    }
    Ok(())
}

#[test]
fn scope_mismatches_are_rejected_for_keys_and_alias_targets() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    for arn in [
        "arn:aws:kms:us-west-2:111111111111:key/1234abcd-12ab-34cd-56ef-1234567890ab",
        "arn:aws:kms:us-east-1:222222222222:key/1234abcd-12ab-34cd-56ef-1234567890ab",
        "arn:aws-cn:kms:us-east-1:111111111111:key/1234abcd-12ab-34cd-56ef-1234567890ab",
    ] {
        let (parts, _) = arn::split_key_arn(arn)
            .map_err(|error| -> Box<dyn std::error::Error> { error.into() })?;
        assert!(!arn::is_in_scope(&parts, &ctx), "accepted scope: {arn}");
    }
    let foreign = "arn:aws:kms:us-west-2:111111111111:key/1234abcd-12ab-34cd-56ef-1234567890ab";
    assert!(arn::validate_target_key_id(foreign, &ctx).is_err());
    assert!(arn::validate_target_key_id(KEY_ID, &ctx).is_ok());
    assert!(arn::validate_target_key_id(KEY_ARN, &ctx).is_ok());
    Ok(())
}
