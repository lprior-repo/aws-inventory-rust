use std::collections::BTreeMap;

use aws_sdk_kms::types::{KeyManagerType, Tag};
use serde_json::json;

use super::*;

fn customer(
    arn: Option<&str>,
) -> Result<aws_sdk_kms::types::KeyMetadata, Box<dyn std::error::Error>> {
    metadata(arn, KEY_ID, KeyManagerType::Customer)
}

#[test]
fn maps_key_identity_and_bounded_metadata() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let resource = map_key(&ctx, KEY_ARN, &customer(Some(KEY_ARN))?, None, None)?;
    assert_eq!(resource.id, KEY_ARN);
    assert_eq!(resource.kind, "aws_kms_key");
    assert_eq!(resource.name.as_deref(), Some(KEY_ID));
    assert_eq!(resource.account_id, "111111111111");
    assert_eq!(resource.region, "us-east-1");
    assert_eq!(resource.properties.get("key_arn"), Some(&json!(KEY_ARN)));
    assert_eq!(resource.properties.get("key_id"), Some(&json!(KEY_ID)));
    assert_eq!(
        resource.properties.get("key_manager"),
        Some(&json!("CUSTOMER"))
    );
    assert_eq!(
        resource.properties.get("key_usage"),
        Some(&json!("ENCRYPT_DECRYPT"))
    );
    assert_eq!(
        resource.properties.get("key_state"),
        Some(&json!("Enabled"))
    );
    assert_eq!(resource.properties.get("enabled"), Some(&json!(true)));
    assert_eq!(
        resource.properties.get("creation_date"),
        Some(&json!(1_000))
    );
    let serialized = serde_json::to_string(&resource.properties)?;
    assert!(!serialized.contains("policy"));
    assert!(!serialized.contains("plaintext"));
    Ok(())
}

#[test]
fn customer_managed_rotation_and_tags_are_collected() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let tags = BTreeMap::from([("team".to_owned(), "orders".to_owned())]);
    let rotation = Rotation {
        enabled: true,
        period_in_days: Some(365),
    };
    let resource = map_key(
        &ctx,
        KEY_ARN,
        &customer(Some(KEY_ARN))?,
        Some(rotation),
        Some(tags),
    )?;
    assert_eq!(
        resource.properties.get("rotation_enabled"),
        Some(&json!(true))
    );
    assert_eq!(
        resource.properties.get("rotation_period_in_days"),
        Some(&json!(365))
    );
    assert_eq!(
        resource.properties.get("rotation_collected"),
        Some(&json!(true))
    );
    assert_eq!(
        resource.properties.get("tags_collected"),
        Some(&json!(true))
    );
    assert_eq!(resource.tags.get("team"), Some(&"orders".to_owned()));
    Ok(())
}

#[test]
fn rotation_failure_retains_the_key_without_inventing_status() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let resource = map_key(&ctx, KEY_ARN, &customer(Some(KEY_ARN))?, None, None)?;
    assert_eq!(resource.id, KEY_ARN);
    assert_eq!(
        resource.properties.get("rotation_enabled"),
        Some(&json!(null))
    );
    assert_eq!(
        resource.properties.get("rotation_period_in_days"),
        Some(&json!(null))
    );
    assert_eq!(
        resource.properties.get("rotation_collected"),
        Some(&json!(false))
    );
    assert_eq!(
        resource.properties.get("tags_collected"),
        Some(&json!(false))
    );
    assert!(resource.tags.is_empty());
    Ok(())
}

#[test]
fn tag_failure_retains_the_key_without_inventing_tags() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let rotation = Rotation {
        enabled: true,
        period_in_days: Some(365),
    };
    let resource = map_key(
        &ctx,
        KEY_ARN,
        &customer(Some(KEY_ARN))?,
        Some(rotation),
        None,
    )?;
    assert_eq!(
        resource.properties.get("tags_collected"),
        Some(&json!(false))
    );
    assert_eq!(
        resource.properties.get("rotation_collected"),
        Some(&json!(true))
    );
    assert!(resource.tags.is_empty());
    Ok(())
}

#[test]
fn aws_managed_keys_never_claim_customer_rotation_state() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let metadata = metadata(Some(KEY_ARN), KEY_ID, KeyManagerType::Aws)?;
    let rotation = Rotation {
        enabled: true,
        period_in_days: Some(365),
    };
    let resource = map_key(&ctx, KEY_ARN, &metadata, Some(rotation), None)?;
    assert_eq!(resource.properties.get("key_manager"), Some(&json!("AWS")));
    assert_eq!(
        resource.properties.get("rotation_enabled"),
        Some(&json!(null))
    );
    assert_eq!(
        resource.properties.get("rotation_collected"),
        Some(&json!(false))
    );
    assert_eq!(
        resource.properties.get("tags_collected"),
        Some(&json!(false))
    );
    Ok(())
}

#[test]
fn aws_managed_keys_skip_rotation_and_tag_reads() -> TestResult {
    let customer = metadata(Some(KEY_ARN), KEY_ID, KeyManagerType::Customer)?;
    let managed = metadata(Some(KEY_ARN), KEY_ID, KeyManagerType::Aws)?;
    let plan = enrichment(&customer);
    assert!(plan.rotation && plan.tags);
    let plan = enrichment(&managed);
    assert!(!plan.rotation && !plan.tags);
    Ok(())
}

#[test]
fn missing_or_mismatched_metadata_identity_is_rejected() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    assert!(map_key(&ctx, KEY_ARN, &customer(None)?, None, None).is_err());
    let wrong_arn = "arn:aws:kms:us-east-1:111111111111:key/2234abcd-12ab-34cd-56ef-1234567890ab";
    assert!(map_key(&ctx, KEY_ARN, &customer(Some(wrong_arn))?, None, None).is_err());
    let wrong_id = metadata(Some(KEY_ARN), OTHER_KEY_ID, KeyManagerType::Customer)?;
    assert!(map_key(&ctx, KEY_ARN, &wrong_id, None, None).is_err());
    let mut account = metadata(Some(KEY_ARN), KEY_ID, KeyManagerType::Customer)?;
    account.aws_account_id = Some("222222222222".into());
    assert!(map_key(&ctx, KEY_ARN, &account, None, None).is_err());
    Ok(())
}

#[test]
fn key_arn_must_match_the_scan_scope() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    for arn in [
        "",
        "arn:aws:kms:us-west-2:111111111111:key/1234abcd-12ab-34cd-56ef-1234567890ab",
        "arn:aws:kms:us-east-1:222222222222:key/1234abcd-12ab-34cd-56ef-1234567890ab",
        "arn:aws:kms:us-east-1:111111111111:alias/orders",
    ] {
        let metadata = metadata(Some(arn), KEY_ID, KeyManagerType::Customer)?;
        assert!(
            map_key(&ctx, arn, &metadata, None, None).is_err(),
            "accepted: {arn}"
        );
    }
    Ok(())
}

#[test]
fn tag_values_keep_empty_entries_and_drop_unstorable_keys() -> TestResult {
    let tags = tags_from(&[
        Tag::builder()
            .tag_key("cost-center")
            .tag_value("")
            .build()?,
        Tag::builder().tag_key("").tag_value("ignored").build()?,
    ]);
    assert_eq!(tags.len(), 1);
    assert_eq!(tags.get("cost-center").map(String::as_str), Some(""));
    Ok(())
}

#[tokio::test]
async fn rotation_failure_is_recorded_in_a_child_scope() -> TestResult {
    let (ctx, mut receiver) = context("us-east-1")?;
    let error = AwsScanError::InvalidData("rotation read failed".into());
    failure(&ctx, "aws_kms_key_rotation", Some(KEY_ARN), &error).await?;
    let event = receiver
        .recv()
        .await
        .ok_or("missing rotation failure event")?;
    let CollectionEvent::Failure(recorded) = event else {
        return Err("expected a failure event".into());
    };
    assert_eq!(recorded.scope.collector, "aws_kms_key_rotation");
    assert_eq!(recorded.scope.parent_id.as_deref(), Some(KEY_ARN));
    assert_eq!(recorded.category, error_category(&error));
    Ok(())
}
