use serde_json::json;

use super::*;

const SQS_ENDPOINT: &str = "arn:aws:sqs:us-east-1:111111111111:orders";

#[test]
fn maps_confirmed_subscription_identity_and_edge() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let item = subscription(SUBSCRIPTION_ARN, "sqs", Some(SQS_ENDPOINT), Some(TOPIC_ARN));
    let observation = map_subscription(&ctx, &item)?.ok_or("in-scope subscription was filtered")?;
    let resource = &observation.resource;
    assert_eq!(resource.id, SUBSCRIPTION_ARN);
    assert_eq!(resource.kind, "aws_sns_subscription");
    assert_eq!(resource.account_id, "111111111111");
    assert_eq!(resource.region, "us-east-1");
    assert_eq!(
        resource.properties.get("subscription_arn"),
        Some(&json!(SUBSCRIPTION_ARN))
    );
    assert_eq!(
        resource.properties.get("topic_arn"),
        Some(&json!(TOPIC_ARN))
    );
    assert_eq!(resource.properties.get("protocol"), Some(&json!("sqs")));
    assert_eq!(
        resource.properties.get("endpoint"),
        Some(&json!(SQS_ENDPOINT))
    );
    assert_eq!(
        resource.properties.get("owner"),
        Some(&json!("111111111111"))
    );
    assert_eq!(
        resource.properties.get("pending_confirmation"),
        Some(&json!(false))
    );
    assert_eq!(
        resource.properties.get("identity_source"),
        Some(&json!("subscription_arn"))
    );
    let edge = observation.edge.ok_or("missing topic edge")?;
    assert_eq!(edge.from_id, SUBSCRIPTION_ARN);
    assert_eq!(edge.to_id, TOPIC_ARN);
    assert_eq!(edge.relation, "SUBSCRIBES_TO");
    assert_eq!(edge.properties.get("evidence"), Some(&json!("topic_arn")));
    Ok(())
}

#[test]
fn pending_subscriptions_use_a_deterministic_synthetic_identity() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let item = subscription(
        "PendingConfirmation",
        "email",
        Some("ops@example.com"),
        Some(TOPIC_ARN),
    );
    let first = map_subscription(&ctx, &item)?.ok_or("pending subscription was filtered")?;
    let second = map_subscription(&ctx, &item)?.ok_or("pending subscription was filtered")?;
    let identity = &first.resource.id;
    assert!(identity.starts_with(TOPIC_ARN));
    assert!(!identity.contains("ops@example.com"));
    assert_eq!(identity, &second.resource.id);
    assert_eq!(
        first.resource.properties.get("pending_confirmation"),
        Some(&json!(true))
    );
    assert_eq!(
        first.resource.properties.get("identity_source"),
        Some(&json!("pending_confirmation"))
    );
    assert_eq!(
        first.resource.properties.get("endpoint"),
        Some(&json!("ops@example.com"))
    );
    assert!(first.edge.is_some());
    let other = subscription(
        "PendingConfirmation",
        "email",
        Some("other@example.com"),
        Some(TOPIC_ARN),
    );
    let other = map_subscription(&ctx, &other)?.ok_or("pending subscription was filtered")?;
    assert_ne!(identity, &other.resource.id);
    Ok(())
}

#[test]
fn edge_requires_an_identifiable_in_scope_topic() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    for (topic, expected_edge) in [
        (Some(TOPIC_ARN), true),
        (Some("arn:aws:sns:us-west-2:111111111111:orders"), false),
        (Some("arn:aws:sns:us-east-1:222222222222:orders"), true),
        (Some("arn:aws-cn:sns:us-east-1:111111111111:orders"), false),
        (
            Some("arn:aws:sns:us-east-1:111111111111:orders:extra"),
            false,
        ),
        (Some("arn:aws:sqs:us-east-1:111111111111:orders"), false),
        (Some("not-an-arn"), false),
        (None, false),
    ] {
        let item = subscription(SUBSCRIPTION_ARN, "sqs", Some(SQS_ENDPOINT), topic);
        let observation = map_subscription(&ctx, &item)?.ok_or("in-scope subscription filtered")?;
        assert_eq!(
            observation.edge.is_some(),
            expected_edge,
            "topic: {topic:?}"
        );

        assert_eq!(
            observation.resource.properties.get("topic_arn"),
            Some(&json!(topic))
        );
    }
    Ok(())
}

#[test]
fn account_wide_listing_is_partitioned_by_region_without_duplicates() -> TestResult {
    let (east, _east_receiver) = context("us-east-1")?;
    let (west, _west_receiver) = context("us-west-2")?;
    let west_arn = "arn:aws:sns:us-west-2:111111111111:orders:7d9e2bb4-75e1-4d24-bd41-0a9f4e77b2b0";
    let west_topic = "arn:aws:sns:us-west-2:111111111111:orders";
    let listing = [
        subscription(SUBSCRIPTION_ARN, "sqs", Some(SQS_ENDPOINT), Some(TOPIC_ARN)),
        subscription(
            west_arn,
            "sqs",
            Some("arn:aws:sqs:us-west-2:111111111111:q"),
            Some(west_topic),
        ),
        subscription(SUBSCRIPTION_ARN, "sqs", Some(SQS_ENDPOINT), Some(TOPIC_ARN)),
    ];
    let collect = |ctx: &Context| -> TestResult2 {
        let observations: Vec<_> = listing
            .iter()
            .filter_map(|item| map_subscription(ctx, item).ok().flatten())
            .collect();
        Ok(dedupe_subscriptions(observations))
    };
    let east_items = collect(&east)?;
    let west_items = collect(&west)?;
    assert_eq!(east_items.len(), 1);
    assert_eq!(
        east_items.first().map(|item| item.resource.id.as_str()),
        Some(SUBSCRIPTION_ARN)
    );
    assert_eq!(west_items.len(), 1);
    assert_eq!(
        west_items.first().map(|item| item.resource.id.as_str()),
        Some(west_arn)
    );
    assert_ne!(
        east_items.first().map(|item| &item.resource.id),
        west_items.first().map(|item| &item.resource.id)
    );
    Ok(())
}

type TestResult2 = Result<Vec<SubscriptionObservation>, Box<dyn std::error::Error>>;

#[test]
fn malformed_subscription_arns_and_endpoints_are_explicit_errors() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    for arn in [
        "not-an-arn",
        "arn:aws:sns:us-east-1:111111111111:orders",
        "arn:aws:sns:us-east-1:111111111111:orders:bad id",
        "arn:aws:sns:us-east-1:111:orders:2bcfbf39-05c3-41de-beaa-fcfcc21c8f55",
        "arn:aws:sqs:us-east-1:111111111111:orders:2bcfbf39-05c3-41de-beaa-fcfcc21c8f55",
    ] {
        let item = subscription(arn, "sqs", Some(SQS_ENDPOINT), Some(TOPIC_ARN));
        assert!(
            map_subscription(&ctx, &item).is_err(),
            "accepted subscription ARN: {arn}"
        );
    }
    let oversized = "a".repeat(5_000);
    for endpoint in [Some("nul\u{0}byte"), Some(oversized.as_str())] {
        let item = subscription(SUBSCRIPTION_ARN, "https", endpoint, Some(TOPIC_ARN));
        assert!(
            map_subscription(&ctx, &item).is_err(),
            "accepted malformed endpoint"
        );
    }
    let missing = Subscription::builder()
        .owner("111111111111")
        .protocol("sqs")
        .topic_arn(TOPIC_ARN)
        .build();
    assert!(map_subscription(&ctx, &missing).is_err());
    let no_owner = Subscription::builder()
        .subscription_arn(SUBSCRIPTION_ARN)
        .protocol("sqs")
        .topic_arn(TOPIC_ARN)
        .build();
    assert!(map_subscription(&ctx, &no_owner).is_err());
    let pending_without_topic = subscription(
        "PendingConfirmation",
        "email",
        Some("ops@example.com"),
        None,
    );
    assert!(map_subscription(&ctx, &pending_without_topic).is_err());
    let foreign_pending = subscription(
        "PendingConfirmation",
        "email",
        Some("ops@example.com"),
        Some("arn:aws:sns:us-west-2:111111111111:orders"),
    );
    assert!(map_subscription(&ctx, &foreign_pending)?.is_none());
    Ok(())
}

const CROSS_ACCOUNT_ARN: &str =
    "arn:aws:sns:us-east-1:222222222222:orders:2bcfbf39-05c3-41de-beaa-fcfcc21c8f55";
const CROSS_ACCOUNT_TOPIC: &str = "arn:aws:sns:us-east-1:222222222222:orders";

fn owned_by(owner: &str, arn: &str, topic: &str) -> Subscription {
    Subscription::builder()
        .subscription_arn(arn)
        .owner(owner)
        .protocol("sqs")
        .endpoint(SQS_ENDPOINT)
        .topic_arn(topic)
        .build()
}

#[test]
fn subscription_to_another_accounts_topic_is_owned_by_the_scanned_account() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let item = owned_by("111111111111", CROSS_ACCOUNT_ARN, CROSS_ACCOUNT_TOPIC);
    let observation = map_subscription(&ctx, &item)?.ok_or("cross-account subscription omitted")?;
    assert_eq!(observation.resource.id, CROSS_ACCOUNT_ARN);
    assert_eq!(observation.resource.account_id, "111111111111");
    let edge = observation.edge.ok_or("missing cross-account topic edge")?;
    assert_eq!(edge.from_id, CROSS_ACCOUNT_ARN);
    assert_eq!(edge.to_id, CROSS_ACCOUNT_TOPIC);
    Ok(())
}

#[test]
fn pending_subscription_to_another_accounts_topic_is_owned_by_the_scanned_account() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let item = owned_by("111111111111", "PendingConfirmation", CROSS_ACCOUNT_TOPIC);
    let observation = map_subscription(&ctx, &item)?.ok_or("cross-account pending omitted")?;
    assert!(observation.resource.id.starts_with(CROSS_ACCOUNT_TOPIC));
    assert!(observation.edge.is_some());
    Ok(())
}

#[test]
fn subscription_owned_by_another_account_or_region_is_not_mapped() -> TestResult {
    let (ctx, _receiver) = context("us-east-1")?;
    let foreign_owner = owned_by("222222222222", SUBSCRIPTION_ARN, TOPIC_ARN);
    assert!(map_subscription(&ctx, &foreign_owner)?.is_none());
    let foreign_region = owned_by(
        "111111111111",
        "arn:aws:sns:us-west-2:222222222222:orders:2bcfbf39-05c3-41de-beaa-fcfcc21c8f55",
        "arn:aws:sns:us-west-2:222222222222:orders",
    );
    assert!(map_subscription(&ctx, &foreign_region)?.is_none());
    let foreign_partition = owned_by(
        "111111111111",
        "arn:aws-cn:sns:us-east-1:222222222222:orders:2bcfbf39-05c3-41de-beaa-fcfcc21c8f55",
        "arn:aws-cn:sns:us-east-1:222222222222:orders",
    );
    assert!(map_subscription(&ctx, &foreign_partition)?.is_none());
    Ok(())
}
