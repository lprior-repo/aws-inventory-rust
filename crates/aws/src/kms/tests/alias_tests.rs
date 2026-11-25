use serde_json::json;

use super::*;

#[test]
fn maps_alias_identity_and_target_without_an_edge() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let resource = map_alias(
        &ctx,
        &alias_entry(Some(ALIAS_ARN), Some(ALIAS_NAME), Some(KEY_ID)),
    )?;
    assert_eq!(resource.id, ALIAS_ARN);
    assert_eq!(resource.kind, "aws_kms_alias");
    assert_eq!(resource.name.as_deref(), Some(ALIAS_NAME));
    assert_eq!(
        resource.properties.get("alias_arn"),
        Some(&json!(ALIAS_ARN))
    );
    assert_eq!(
        resource.properties.get("alias_name"),
        Some(&json!(ALIAS_NAME))
    );
    assert_eq!(
        resource.properties.get("target_key_id"),
        Some(&json!(KEY_ID))
    );
    assert_eq!(
        resource.properties.get("creation_date"),
        Some(&json!(2_000))
    );
    assert_eq!(
        resource.properties.get("last_updated_date"),
        Some(&json!(3_000))
    );
    assert_eq!(
        resource.properties.get("tags_collected"),
        Some(&json!(false))
    );
    assert!(resource.tags.is_empty());
    Ok(())
}

#[test]
fn missing_alias_arn_or_empty_target_is_rejected() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    assert!(map_alias(&ctx, &alias_entry(None, Some(ALIAS_NAME), Some(KEY_ID))).is_err());
    assert!(
        map_alias(
            &ctx,
            &alias_entry(Some(ALIAS_ARN), Some(ALIAS_NAME), Some(""))
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn alias_without_a_target_key_is_preserved_with_a_null_reference() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let resource = map_alias(&ctx, &alias_entry(Some(ALIAS_ARN), None, None))?;
    assert_eq!(resource.id, ALIAS_ARN);
    assert_eq!(resource.kind, "aws_kms_alias");
    assert_eq!(resource.name.as_deref(), Some(ALIAS_NAME));
    assert_eq!(
        resource.properties.get("target_key_id"),
        Some(&serde_json::Value::Null)
    );
    Ok(())
}

#[test]
fn alias_arn_and_name_must_agree_and_stay_in_scope() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    assert!(
        map_alias(
            &ctx,
            &alias_entry(Some(ALIAS_ARN), Some("alias/other"), Some(KEY_ID))
        )
        .is_err()
    );
    for arn in [
        "arn:aws:kms:us-west-2:111111111111:alias/orders",
        "arn:aws:kms:us-east-1:222222222222:alias/orders",
        "arn:aws:kms:us-east-1:111111111111:key/1234abcd-12ab-34cd-56ef-1234567890ab",
    ] {
        assert!(
            map_alias(
                &ctx,
                &alias_entry(Some(arn), Some(ALIAS_NAME), Some(KEY_ID))
            )
            .is_err(),
            "accepted: {arn}"
        );
    }
    Ok(())
}

#[test]
fn alias_targets_accept_key_ids_and_reject_foreign_arns() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let resource = map_alias(
        &ctx,
        &alias_entry(Some(ALIAS_ARN), Some(ALIAS_NAME), Some(KEY_ARN)),
    )?;
    assert_eq!(
        resource.properties.get("target_key_id"),
        Some(&json!(KEY_ARN))
    );
    let foreign = "arn:aws:kms:us-west-2:111111111111:key/1234abcd-12ab-34cd-56ef-1234567890ab";
    assert!(
        map_alias(
            &ctx,
            &alias_entry(Some(ALIAS_ARN), Some(ALIAS_NAME), Some(foreign))
        )
        .is_err()
    );
    let malformed = "alias/orders";
    assert!(
        map_alias(
            &ctx,
            &alias_entry(Some(ALIAS_ARN), Some(ALIAS_NAME), Some(malformed))
        )
        .is_err()
    );
    Ok(())
}
