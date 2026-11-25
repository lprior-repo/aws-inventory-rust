use super::*;
mod queue_boundaries;

#[tokio::test]
async fn given_queue_urls_when_attributes_are_collected_then_only_canonical_arn_rows_persist()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(bytes()?)?;
    let list = catalog
        .lookup("sqs", "ListQueues")
        .ok_or("queue discovery absent")?;
    let detail = catalog
        .lookup("sqs", "GetQueueAttributes")
        .ok_or("queue attributes absent")?;
    assert_eq!(
        list.emission(),
        inventory_core::catalog::runtime::Emission::DiscoveryOnly
    );
    let size = list
        .pagination()
        .page_size
        .as_ref()
        .ok_or("queue pagination size absent")?;
    let first = request(
        list,
        &[super::super::super::input::InputField {
            path: size.path.clone(),
            value: Value::Int(i64::from(size.max)),
        }],
    )?;
    let input: serde_json::Value = serde_json::from_slice(first.body())?;
    assert_eq!(
        input.get("MaxResults").and_then(serde_json::Value::as_u64),
        Some(1000)
    );
    let urls = decode(list, br#"{"QueueUrls":["https://sqs.us-east-1.amazonaws.com/123456789012/work"],"NextToken":"page-two"}"#)?;
    let cursor = list
        .pagination()
        .tokens
        .first()
        .ok_or("queue continuation absent")?;
    let value = path_value(urls.root(), &cursor.response_path)?
        .ok_or("queue next token absent")?
        .to_value()?;
    let next = request(
        list,
        &[
            super::super::super::input::InputField {
                path: size.path.clone(),
                value: Value::Int(i64::from(size.max)),
            },
            super::super::super::input::InputField {
                path: cursor.request_path.clone(),
                value,
            },
        ],
    )?;
    let input: serde_json::Value = serde_json::from_slice(next.body())?;
    assert_eq!(
        input.get("NextToken").and_then(serde_json::Value::as_str),
        Some("page-two")
    );
    let row = item_records(&urls, list.items())?
        .into_iter()
        .next()
        .ok_or("queue discovery row absent")?;
    let fields = child_fields(&catalog, list, detail, &row)?;
    let attributes = request(detail, &fields)?;
    let request_record = request_bindings(detail, &fields)?;
    let input: serde_json::Value = serde_json::from_slice(attributes.body())?;
    assert_eq!(
        input.get("QueueUrl").and_then(serde_json::Value::as_str),
        Some("https://sqs.us-east-1.amazonaws.com/123456789012/work")
    );
    let names = input
        .get("AttributeNames")
        .and_then(serde_json::Value::as_array)
        .ok_or("reviewed attribute whitelist absent")?;
    for expected in [
        "QueueArn",
        "Policy",
        "KmsMasterKeyId",
        "FifoQueue",
        "RedrivePolicy",
        "CreatedTimestamp",
    ] {
        assert!(names.iter().any(|name| name.as_str() == Some(expected)));
    }
    assert!(!names.iter().any(|name| {
        name.as_str()
            .is_some_and(|name| name == "All" || name.starts_with("Approximate"))
    }));
    let response = decode(detail, br#"{"Attributes":{"QueueArn":"arn:aws:sqs:us-east-1:123456789012:work","VisibilityTimeout":"30","ApproximateNumberOfMessages":"3","LastModifiedTimestamp":"42"}}"#)?;
    let observations = observations_with_request(
        detail,
        item_records(&response, detail.items())?,
        &request_record,
    )?;
    let hash = observations
        .first()
        .ok_or("queue observation absent")?
        .content_hash();
    let changed = decode(detail, br#"{"Attributes":{"QueueArn":"arn:aws:sqs:us-east-1:123456789012:work","VisibilityTimeout":"30","ApproximateNumberOfMessages":"99","LastModifiedTimestamp":"100"}}"#)?;
    assert_eq!(
        hash,
        observations_with_request(
            detail,
            item_records(&changed, detail.items())?,
            &request_record
        )?
        .first()
        .ok_or("queue repeat absent")?
        .content_hash()
    );
    let (persisted, _) = persist(
        detail,
        vec![(detail, None, request_record.clone(), observations)],
    )
    .await?;
    assert_eq!(
        persisted
            .iter()
            .map(|(kind, id, _)| (kind.as_str(), id.as_str()))
            .collect::<Vec<_>>(),
        [("AWS::SQS::Queue", "arn:aws:sqs:us-east-1:123456789012:work")]
    );
    let missing = decode(detail, br#"{"Attributes":{"VisibilityTimeout":"30"}}"#)?;
    assert!(
        observations_with_request(
            detail,
            item_records(&missing, detail.items())?,
            &request_record
        )
        .is_err()
    );
    Ok(())
}

fn subscriptions_xml(arn: &str, endpoint: &str) -> String {
    format!(
        "<ListSubscriptionsResponse><ListSubscriptionsResult><Subscriptions><member><SubscriptionArn>{arn}</SubscriptionArn><TopicArn>arn:aws:sns:us-east-1:123456789012:topic</TopicArn><Protocol>https</Protocol><Endpoint>{endpoint}</Endpoint><Owner>123456789012</Owner></member></Subscriptions></ListSubscriptionsResult></ListSubscriptionsResponse>"
    )
}

#[tokio::test]
async fn given_confirmed_pending_and_deleted_subscriptions_when_collected_then_no_sentinels_or_raw_endpoints_become_ids()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(bytes()?)?;
    let list = catalog
        .lookup("sns", "ListSubscriptions")
        .ok_or("subscription list absent")?;
    let detail = catalog
        .lookup("sns", "GetSubscriptionAttributes")
        .ok_or("subscription attributes absent")?;
    let confirmed = "arn:aws:sns:us-east-1:123456789012:topic:0fbbf013-2c09-4c13-b31a-62857ec41b47";
    let mut rows = Vec::new();
    for (arn, endpoint) in [
        (
            confirmed,
            "https://user:password@example.com/receiver?token=secret-confirmed",
        ),
        (
            "PendingConfirmation",
            "https://user:password@example.com/receiver?token=secret-pending-one",
        ),
        (
            "PendingConfirmation",
            "https://user:password@example.com/receiver?token=secret-pending-two",
        ),
        (
            "Deleted",
            "https://user:password@example.com/receiver?token=secret-deleted",
        ),
    ] {
        let response = decode(list, subscriptions_xml(arn, endpoint).as_bytes())?;
        let row = item_records(&response, list.items())?
            .into_iter()
            .next()
            .ok_or("subscription row absent")?;
        let fields = child_fields_optional(&catalog, list, detail, &row)?;
        if arn == confirmed {
            let encoded = request(detail, &fields.ok_or("confirmed detail was skipped")?)?;
            let body = std::str::from_utf8(encoded.body())?;
            assert!(body.contains("SubscriptionArn=arn%3Aaws%3Asns%3Aus-east-1%3A123456789012%3Atopic%3A0fbbf013-2c09-4c13-b31a-62857ec41b47"));
        } else {
            assert!(
                fields.is_none(),
                "sentinel was scheduled as a subscription ARN"
            );
        }
        rows.push(row);
    }
    let bindings = request_bindings(list, &[])?;
    let observations = observations_with_request(list, rows, &bindings)?;
    let (persisted, _) = persist(list, vec![(list, None, bindings, observations)]).await?;
    assert_eq!(persisted.len(), 4);
    assert_eq!(
        persisted
            .iter()
            .filter(|(_, id, _)| id == confirmed)
            .count(),
        1
    );
    for (kind, id, record) in persisted {
        assert_eq!(kind, "AWS::SNS::Subscription");
        assert!(!matches!(id.as_str(), "PendingConfirmation" | "Deleted"));
        assert!(!id.contains("password") && !id.contains("secret-") && !id.contains("https://"));
        assert!(
            !record
                .windows(b"password".len())
                .any(|window| window == b"password")
        );
        assert!(
            !record
                .windows(b"secret-".len())
                .any(|window| window == b"secret-")
        );
    }
    Ok(())
}

#[test]
fn given_malformed_subscription_arns_when_details_bind_then_errors_are_not_silent_skips()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(bytes()?)?;
    let list = catalog
        .lookup("sns", "ListSubscriptions")
        .ok_or("subscription list absent")?;
    let detail = catalog
        .lookup("sns", "GetSubscriptionAttributes")
        .ok_or("subscription attributes absent")?;
    for arn in [
        "",
        "pendingconfirmation",
        "arn:aws:sns:us-east-1:123456789012:topic",
        "arn:aws:sns:us-west-2:123456789012:topic:id",
        "arn:aws:sqs:us-east-1:123456789012:topic:id",
    ] {
        let response = decode(
            list,
            subscriptions_xml(arn, "https://example.com").as_bytes(),
        )?;
        let row = item_records(&response, list.items())?
            .into_iter()
            .next()
            .ok_or("hostile subscription row absent")?;
        assert!(
            child_fields_optional(&catalog, list, detail, &row).is_err(),
            "malformed ARN was skipped or accepted: {arn}"
        );
    }
    Ok(())
}

const TOPIC_OWNER_ACCOUNT: &str = "999999999999";
const SCANNED_ACCOUNT: &str = "123456789012";
const CROSS_ACCOUNT_SUBSCRIPTION: &str =
    "arn:aws:sns:us-east-1:999999999999:topic:0fbbf013-2c09-4c13-b31a-62857ec41b47";

fn owned_subscriptions_xml(arn: &str, owner: &str) -> String {
    let topic = arn.rsplit_once(':').map_or(arn, |(topic, _)| topic);
    format!(
        "<ListSubscriptionsResponse><ListSubscriptionsResult><Subscriptions><member><SubscriptionArn>{arn}</SubscriptionArn><TopicArn>{topic}</TopicArn><Protocol>https</Protocol><Endpoint>https://example.com</Endpoint><Owner>{owner}</Owner></member></Subscriptions></ListSubscriptionsResult></ListSubscriptionsResponse>"
    )
}

fn subscription_attributes_xml(arn: &str, owner: &str) -> String {
    format!(
        "<GetSubscriptionAttributesResponse><GetSubscriptionAttributesResult><Attributes><entry><key>SubscriptionArn</key><value>{arn}</value></entry><entry><key>Owner</key><value>{owner}</value></entry><entry><key>Endpoint</key><value>https://example.com</value></entry></Attributes></GetSubscriptionAttributesResult></GetSubscriptionAttributesResponse>"
    )
}

#[tokio::test]
async fn given_cross_account_topic_subscription_owned_by_scanned_account_when_collected_then_list_and_detail_persist()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(bytes()?)?;
    let list = catalog
        .lookup("sns", "ListSubscriptions")
        .ok_or("subscription list absent")?;
    let detail = catalog
        .lookup("sns", "GetSubscriptionAttributes")
        .ok_or("subscription attributes absent")?;
    let response = decode(
        list,
        owned_subscriptions_xml(CROSS_ACCOUNT_SUBSCRIPTION, SCANNED_ACCOUNT).as_bytes(),
    )?;
    let row = item_records(&response, list.items())?
        .into_iter()
        .next()
        .ok_or("subscription row absent")?;
    let fields = child_fields_optional(&catalog, list, detail, &row)?
        .ok_or("cross-account subscription detail was skipped")?;
    let encoded = request(detail, &fields)?;
    assert!(
        std::str::from_utf8(encoded.body())?
            .contains("SubscriptionArn=arn%3Aaws%3Asns%3Aus-east-1%3A999999999999%3Atopic%3A")
    );
    let bindings = request_bindings(list, &[])?;
    let observations = observations_with_request(list, vec![row], &bindings)?;
    let (persisted, _) = persist(list, vec![(list, None, bindings, observations)]).await?;
    assert_eq!(
        persisted
            .iter()
            .map(|(kind, id, _)| (kind.as_str(), id.as_str()))
            .collect::<Vec<_>>(),
        [("AWS::SNS::Subscription", CROSS_ACCOUNT_SUBSCRIPTION)]
    );
    let request_record = request_bindings(detail, &fields)?;
    let attributes = decode(
        detail,
        subscription_attributes_xml(CROSS_ACCOUNT_SUBSCRIPTION, SCANNED_ACCOUNT).as_bytes(),
    )?;
    let observations = observations_with_request(
        detail,
        item_records(&attributes, detail.items())?,
        &request_record,
    )?;
    let (persisted, _) =
        persist(detail, vec![(detail, None, request_record, observations)]).await?;
    assert_eq!(
        persisted
            .iter()
            .map(|(kind, id, _)| (kind.as_str(), id.as_str()))
            .collect::<Vec<_>>(),
        [("AWS::SNS::Subscription", CROSS_ACCOUNT_SUBSCRIPTION)]
    );
    Ok(())
}

#[test]
fn given_subscription_rows_when_owner_differs_then_only_foreign_owned_rows_on_scanned_topics_are_skipped()
-> TestResult {
    const OWN_TOPIC_SUBSCRIPTION: &str =
        "arn:aws:sns:us-east-1:123456789012:topic:0fbbf013-2c09-4c13-b31a-62857ec41b47";
    let catalog = RuntimeCatalog::from_bytes(bytes()?)?;
    let list = catalog
        .lookup("sns", "ListSubscriptions")
        .ok_or("subscription list absent")?;
    let account = AccountId::parse(SCANNED_ACCOUNT)?;
    let region = RegionId::parse("us-east-1")?;
    let trusted = IdentityContext {
        account: &account,
        region: &region,
    };
    let request = request_bindings(list, &[])?;
    let skipped = |arn: &str, owner: &str| -> TestResult<bool> {
        let response = decode(list, owned_subscriptions_xml(arn, owner).as_bytes())?;
        let row = item_records(&response, list.items())?
            .into_iter()
            .next()
            .ok_or("subscription row absent")?;
        Ok(foreign_owned(
            &row,
            list.identity(),
            &trusted,
            Some(&request),
        )?)
    };
    // Another account's subscription on the scanned account's topic: not ours.
    assert!(skipped(OWN_TOPIC_SUBSCRIPTION, TOPIC_OWNER_ACCOUNT)?);
    // The scanned account's own subscription, whatever topic account its ARN names.
    assert!(!skipped(OWN_TOPIC_SUBSCRIPTION, SCANNED_ACCOUNT)?);
    assert!(!skipped(CROSS_ACCOUNT_SUBSCRIPTION, SCANNED_ACCOUNT)?);
    // Neither owner nor ARN is the scanned account: left to identity validation to reject.
    assert!(!skipped(CROSS_ACCOUNT_SUBSCRIPTION, TOPIC_OWNER_ACCOUNT)?);
    Ok(())
}

#[test]
fn given_subscription_owned_by_another_account_when_collected_then_identity_is_rejected()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(bytes()?)?;
    let list = catalog
        .lookup("sns", "ListSubscriptions")
        .ok_or("subscription list absent")?;
    let detail = catalog
        .lookup("sns", "GetSubscriptionAttributes")
        .ok_or("subscription attributes absent")?;
    let response = decode(
        list,
        owned_subscriptions_xml(CROSS_ACCOUNT_SUBSCRIPTION, TOPIC_OWNER_ACCOUNT).as_bytes(),
    )?;
    let row = item_records(&response, list.items())?
        .into_iter()
        .next()
        .ok_or("subscription row absent")?;
    let bindings = request_bindings(list, &[])?;
    assert!(observations_with_request(list, vec![row], &bindings).is_err());
    let fields = request_bindings(detail, &[])?;
    let attributes = decode(
        detail,
        subscription_attributes_xml(CROSS_ACCOUNT_SUBSCRIPTION, TOPIC_OWNER_ACCOUNT).as_bytes(),
    )?;
    assert!(
        observations_with_request(detail, item_records(&attributes, detail.items())?, &fields)
            .is_err()
    );
    Ok(())
}
