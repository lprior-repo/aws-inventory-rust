use std::collections::BTreeMap;

use aws_sdk_sns::config::http::HttpResponse;
use aws_sdk_sns::error::SdkError;
use aws_sdk_sns::operation::get_topic_attributes::GetTopicAttributesError;
use aws_sdk_sns::types::error::{InvalidParameterException, NotFoundException};
use aws_smithy_types::body::SdkBody;
use aws_smithy_types::error::ErrorMetadata;
use serde_json::json;

use super::*;

#[test]
fn maps_topic_identity_and_bounded_attributes() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let attrs = attributes(&[
        ("TopicArn", TOPIC_ARN),
        ("DisplayName", "Orders"),
        ("Owner", "111111111111"),
        ("SubscriptionsConfirmed", "4"),
        ("SubscriptionsPending", "1"),
        ("SubscriptionsDeleted", "2"),
        ("Policy", r#"{"Version":"2012-10-17","Statement":[]}"#),
        (
            "DeliveryPolicy",
            r#"{"http":{"defaultHealthyRetryPolicy":{}}}"#,
        ),
        ("KmsMasterKeyId", "alias/aws/sns"),
        ("FifoTopic", "true"),
        ("ContentBasedDeduplication", "true"),
    ]);
    let tags = BTreeMap::from([("team".to_owned(), "orders".to_owned())]);
    let resource = map_topic(&ctx, TOPIC_ARN, &attrs, Some(tags))?;
    assert_eq!(resource.id, TOPIC_ARN);
    assert_eq!(resource.kind, "aws_sns_topic");
    assert_eq!(resource.name.as_deref(), Some(TOPIC_NAME));
    assert_eq!(resource.account_id, "111111111111");
    assert_eq!(resource.region, "us-east-1");
    assert_eq!(
        resource.properties.get("display_name"),
        Some(&json!("Orders"))
    );
    assert_eq!(
        resource.properties.get("owner"),
        Some(&json!("111111111111"))
    );
    assert_eq!(
        resource.properties.get("subscriptions_confirmed"),
        Some(&json!(4))
    );
    assert_eq!(
        resource.properties.get("subscriptions_pending"),
        Some(&json!(1))
    );
    assert_eq!(
        resource.properties.get("subscriptions_deleted"),
        Some(&json!(2))
    );
    assert_eq!(
        resource.properties.get("kms_master_key_id"),
        Some(&json!("alias/aws/sns"))
    );
    assert_eq!(resource.properties.get("fifo_topic"), Some(&json!(true)));
    assert_eq!(
        resource.properties.get("content_based_deduplication"),
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
fn policy_bodies_are_presence_only() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let attrs = attributes(&[
        ("TopicArn", TOPIC_ARN),
        (
            "Policy",
            r#"{"Statement":[{"Effect":"Allow","Principal":"*"}]}"#,
        ),
        (
            "DeliveryPolicy",
            r#"{"application":{"failureFeedbackRoleArn":"x"}}"#,
        ),
    ]);
    let resource = map_topic(&ctx, TOPIC_ARN, &attrs, None)?;
    assert_eq!(
        resource.properties.get("policy_configured"),
        Some(&json!(true))
    );
    assert_eq!(
        resource.properties.get("delivery_policy_configured"),
        Some(&json!(true))
    );
    let serialized = serde_json::to_string(&resource.properties)?;
    assert!(!serialized.contains("Statement"));
    assert!(!serialized.contains("failureFeedbackRoleArn"));
    assert_eq!(
        resource.properties.get("tags_collected"),
        Some(&json!(false))
    );
    Ok(())
}

#[test]
fn tag_failure_is_visible_and_does_not_discard_the_topic() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let attrs = attributes(&[("TopicArn", TOPIC_ARN)]);
    let resource = map_topic(&ctx, TOPIC_ARN, &attrs, None)?;
    assert!(resource.tags.is_empty());
    assert_eq!(
        resource.properties.get("tags_collected"),
        Some(&json!(false))
    );
    Ok(())
}

#[test]
fn topic_arn_must_match_the_scan_scope_and_service() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    for arn in [
        "not-an-arn",
        "arn:aws:sqs:us-east-1:111111111111:orders",
        "arn:aws:sns:us-west-2:111111111111:orders",
        "arn:aws:sns:us-east-1:222222222222:orders",
        "arn:aws-cn:sns:us-east-1:111111111111:orders",
        "arn:aws:sns:us-east-1:111111111111:",
        "arn:aws:sns:us-east-1:111:orders",
    ] {
        let attrs = attributes(&[("TopicArn", arn)]);
        assert!(
            map_topic(&ctx, arn, &attrs, None).is_err(),
            "accepted topic ARN: {arn}"
        );
    }
    let attrs = attributes(&[("TopicArn", "arn:aws:sns:us-east-1:111111111111:other")]);
    assert!(map_topic(&ctx, TOPIC_ARN, &attrs, None).is_err());
    Ok(())
}

#[test]
fn empty_tag_values_are_preserved() -> TestResult {
    let tags = tags_from(&[
        Tag::builder().key("cost-center").value("").build()?,
        Tag::builder().key("owner").value("platform").build()?,
    ]);
    assert_eq!(tags.len(), 2);
    assert_eq!(tags.get("cost-center").map(String::as_str), Some(""));
    assert_eq!(tags.get("owner").map(String::as_str), Some("platform"));
    Ok(())
}

#[test]
fn missing_topic_is_a_terminal_permanent_failure() -> TestResult {
    let modeled = NotFoundException::builder()
        .message("topic gone")
        .meta(
            ErrorMetadata::builder()
                .code("NotFound")
                .message("topic gone")
                .build(),
        )
        .build();
    let response = HttpResponse::new(404_u16.try_into()?, SdkBody::empty());
    let error = SdkError::service_error(
        GetTopicAttributesError::NotFoundException(modeled),
        response,
    );
    let error = AwsScanError::Api(sdk_error(error));
    assert_eq!(error_category(&error), "api_error");
    assert!(matches!(error, AwsScanError::Api(ApiError::Permanent(_))));
    Ok(())
}

#[test]
fn unmodeled_throttle_codes_stay_retryable() -> TestResult {
    let modeled = InvalidParameterException::builder()
        .message("slow down")
        .meta(
            ErrorMetadata::builder()
                .code("Throttled")
                .message("slow down")
                .build(),
        )
        .build();
    let response = HttpResponse::new(400_u16.try_into()?, SdkBody::empty());
    let error = SdkError::service_error(
        GetTopicAttributesError::InvalidParameterException(modeled),
        response,
    );
    let error = AwsScanError::Api(sdk_error(error));
    assert_eq!(error_category(&error), "throttled");
    assert!(matches!(error, AwsScanError::Api(ApiError::Throttled(_))));
    Ok(())
}
