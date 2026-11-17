use super::*;

#[tokio::test]
async fn given_queue_urls_when_attributes_are_collected_then_only_canonical_arn_rows_persist() -> TestResult {
    let catalog = RuntimeCatalog::from_bytes(bytes()?)?;
    let list = catalog.lookup("sqs", "ListQueues").ok_or("queue discovery absent")?;
    let detail = catalog.lookup("sqs", "GetQueueAttributes").ok_or("queue attributes absent")?;
    assert_eq!(list.emission(), inventory_core::catalog::runtime::Emission::DiscoveryOnly);
    let size = list.pagination().page_size.as_ref().ok_or("queue pagination size absent")?;
    let first = request(list, &[super::super::super::input::InputField { path: size.path.clone(), value: Value::Int(i64::from(size.max)) }])?;
    let input: serde_json::Value = serde_json::from_slice(first.body())?;
    assert_eq!(input.get("MaxResults").and_then(serde_json::Value::as_u64), Some(1000));
    let urls = decode(list, br#"{"QueueUrls":["https://sqs.us-east-1.amazonaws.com/123456789012/work"],"NextToken":"page-two"}"#)?;
    let cursor = list.pagination().tokens.first().ok_or("queue continuation absent")?;
    let value = path_value(urls.root(), &cursor.response_path)?.ok_or("queue next token absent")?.to_value()?;
    let next = request(list, &[
        super::super::super::input::InputField { path: size.path.clone(), value: Value::Int(i64::from(size.max)) },
        super::super::super::input::InputField { path: cursor.request_path.clone(), value },
    ])?;
    let input: serde_json::Value = serde_json::from_slice(next.body())?;
    assert_eq!(input.get("NextToken").and_then(serde_json::Value::as_str), Some("page-two"));
    let row = item_records(&urls, list.items())?.into_iter().next().ok_or("queue discovery row absent")?;
    let fields = child_fields(&catalog, list, detail, &row)?;
    let attributes = request(detail, &fields)?;
    let request_record = request_bindings(detail, &fields)?;
    let input: serde_json::Value = serde_json::from_slice(attributes.body())?;
    assert_eq!(input.get("QueueUrl").and_then(serde_json::Value::as_str), Some("https://sqs.us-east-1.amazonaws.com/123456789012/work"));
    let names = input.get("AttributeNames").and_then(serde_json::Value::as_array).ok_or("reviewed attribute whitelist absent")?;
    for expected in ["QueueArn", "Policy", "KmsMasterKeyId", "FifoQueue", "RedrivePolicy", "CreatedTimestamp"] {
        assert!(names.iter().any(|name| name.as_str() == Some(expected)));
    }
    assert!(!names.iter().any(|name| name.as_str().is_some_and(|name| name == "All" || name.starts_with("Approximate"))));
    let response = decode(detail, br#"{"Attributes":{"QueueArn":"arn:aws:sqs:us-east-1:123456789012:work","VisibilityTimeout":"30","ApproximateNumberOfMessages":"3","LastModifiedTimestamp":"42"}}"#)?;
    let observations = observations_with_request(detail, item_records(&response, detail.items())?, &request_record)?;
    let hash = observations.first().ok_or("queue observation absent")?.content_hash();
    let changed = decode(detail, br#"{"Attributes":{"QueueArn":"arn:aws:sqs:us-east-1:123456789012:work","VisibilityTimeout":"30","ApproximateNumberOfMessages":"99","LastModifiedTimestamp":"100"}}"#)?;
    assert_eq!(hash, observations_with_request(detail, item_records(&changed, detail.items())?, &request_record)?.first().ok_or("queue repeat absent")?.content_hash());
    let (persisted, _) = persist(detail, vec![(detail, None, request_record.clone(), observations)]).await?;
    assert_eq!(persisted.iter().map(|(kind,id,_)| (kind.as_str(),id.as_str())).collect::<Vec<_>>(), [("AWS::SQS::Queue", "arn:aws:sqs:us-east-1:123456789012:work")]);
    let missing = decode(detail, br#"{"Attributes":{"VisibilityTimeout":"30"}}"#)?;
    assert!(observations_with_request(detail, item_records(&missing, detail.items())?, &request_record).is_err());
    Ok(())
}

fn subscriptions_xml(arn: &str, endpoint: &str) -> String {
    format!("<ListSubscriptionsResponse><ListSubscriptionsResult><Subscriptions><member><SubscriptionArn>{arn}</SubscriptionArn><TopicArn>arn:aws:sns:us-east-1:123456789012:topic</TopicArn><Protocol>https</Protocol><Endpoint>{endpoint}</Endpoint><Owner>123456789012</Owner></member></Subscriptions></ListSubscriptionsResult></ListSubscriptionsResponse>")
}

#[tokio::test]
async fn given_confirmed_pending_and_deleted_subscriptions_when_collected_then_no_sentinels_or_raw_endpoints_become_ids() -> TestResult {
    let catalog = RuntimeCatalog::from_bytes(bytes()?)?;
    let list = catalog.lookup("sns", "ListSubscriptions").ok_or("subscription list absent")?;
    let detail = catalog.lookup("sns", "GetSubscriptionAttributes").ok_or("subscription attributes absent")?;
    let confirmed = "arn:aws:sns:us-east-1:123456789012:topic:0fbbf013-2c09-4c13-b31a-62857ec41b47";
    let mut rows = Vec::new();
    for (arn, endpoint) in [
        (confirmed, "https://user:password@example.com/receiver?token=secret-confirmed"),
        ("PendingConfirmation", "https://user:password@example.com/receiver?token=secret-pending-one"),
        ("PendingConfirmation", "https://user:password@example.com/receiver?token=secret-pending-two"),
        ("Deleted", "https://user:password@example.com/receiver?token=secret-deleted"),
    ] {
        let response = decode(list, subscriptions_xml(arn, endpoint).as_bytes())?;
        let row = item_records(&response, list.items())?.into_iter().next().ok_or("subscription row absent")?;
        let fields = child_fields_optional(&catalog, list, detail, &row)?;
        if arn == confirmed {
            let encoded = request(detail, &fields.ok_or("confirmed detail was skipped")?)?;
            let body = std::str::from_utf8(encoded.body())?;
            assert!(body.contains("SubscriptionArn=arn%3Aaws%3Asns%3Aus-east-1%3A123456789012%3Atopic%3A0fbbf013-2c09-4c13-b31a-62857ec41b47"));
        } else {
            assert!(fields.is_none(), "sentinel was scheduled as a subscription ARN");
        }
        rows.push(row);
    }
    let bindings = request_bindings(list, &[])?;
    let observations = observations_with_request(list, rows, &bindings)?;
    let (persisted, _) = persist(list, vec![(list, None, bindings, observations)]).await?;
    assert_eq!(persisted.len(), 4);
    assert_eq!(persisted.iter().filter(|(_,id,_)| id == confirmed).count(), 1);
    for (kind, id, record) in persisted {
        assert_eq!(kind, "AWS::SNS::Subscription");
        assert!(!matches!(id.as_str(), "PendingConfirmation" | "Deleted"));
        assert!(!id.contains("password") && !id.contains("secret-") && !id.contains("https://"));
        assert!(!record.windows(b"password".len()).any(|window| window == b"password"));
        assert!(!record.windows(b"secret-".len()).any(|window| window == b"secret-"));
    }
    Ok(())
}

#[test]
fn given_malformed_subscription_arns_when_details_bind_then_errors_are_not_silent_skips() -> TestResult {
    let catalog = RuntimeCatalog::from_bytes(bytes()?)?;
    let list = catalog.lookup("sns", "ListSubscriptions").ok_or("subscription list absent")?;
    let detail = catalog.lookup("sns", "GetSubscriptionAttributes").ok_or("subscription attributes absent")?;
    for arn in ["", "pendingconfirmation", "arn:aws:sns:us-east-1:123456789012:topic", "arn:aws:sns:us-east-1:999999999999:topic:id", "arn:aws:sns:us-west-2:123456789012:topic:id", "arn:aws:sqs:us-east-1:123456789012:topic:id"] {
        let response = decode(list, subscriptions_xml(arn, "https://example.com").as_bytes())?;
        let row = item_records(&response, list.items())?.into_iter().next().ok_or("hostile subscription row absent")?;
        assert!(child_fields_optional(&catalog, list, detail, &row).is_err(), "malformed ARN was skipped or accepted: {arn}");
    }
    Ok(())
}

#[test]
fn given_hostile_queue_urls_when_child_requests_bind_then_mismatches_fail_without_skipping() -> TestResult {
    let catalog = RuntimeCatalog::from_bytes(bytes()?)?;
    let list = catalog.lookup("sqs", "ListQueues").ok_or("queue discovery absent")?;
    let detail = catalog.lookup("sqs", "GetQueueAttributes").ok_or("queue attributes absent")?;
    for url in [
        "http://sqs.us-east-1.amazonaws.com/123456789012/work",
        "https://sqs.us-east-1.amazonaws.com/999999999999/work",
        "https://sqs.us-west-2.amazonaws.com/123456789012/work",
        "https://sqs.us-east-1.amazonaws.com.evil.example/123456789012/work",
        "https://user:password@sqs.us-east-1.amazonaws.com/123456789012/work",
        "https://sqs.us-east-1.amazonaws.com:443/123456789012/work",
        "https://sqs.us-east-1.amazonaws.com/123456789012/work?token=secret",
        "https://sqs.us-east-1.amazonaws.com/123456789012/work#fragment",
        "https://sqs.us-east-1.amazonaws.com/123456789012/work%2Fother",
        "https://sqs.us-east-1.amazonaws.com/123456789012/work/other",
        "https://sqs.cn-north-1.amazonaws.com.cn/123456789012/work",
    ] {
        let body = serde_json::to_vec(&serde_json::json!({ "QueueUrls": [url] }))?;
        let response = decode(list, &body)?;
        let row = item_records(&response, list.items())?.into_iter().next().ok_or("hostile queue URL row absent")?;
        assert!(child_fields_optional(&catalog, list, detail, &row).is_err(), "hostile URL accepted or skipped: {url}");
    }
    Ok(())
}

#[test]
fn given_queue_arn_mismatches_when_attributes_identify_then_scope_or_queue_name_failure_is_explicit() -> TestResult {
    let catalog = RuntimeCatalog::from_bytes(bytes()?)?;
    let list = catalog.lookup("sqs", "ListQueues").ok_or("queue discovery absent")?;
    let detail = catalog.lookup("sqs", "GetQueueAttributes").ok_or("queue attributes absent")?;
    let response = decode(list, br#"{"QueueUrls":["https://sqs.us-east-1.amazonaws.com/123456789012/work"]}"#)?;
    let row = item_records(&response, list.items())?.into_iter().next().ok_or("queue URL row absent")?;
    let fields = child_fields(&catalog, list, detail, &row)?;
    let request = super::super::super::input::merge_input(&Record::encode(&Value::Map(BTreeMap::new()))?, &fields)?;
    for arn in [
        "arn:aws:sqs:us-east-1:999999999999:work",
        "arn:aws:sqs:us-west-2:123456789012:work",
        "arn:aws-cn:sqs:us-east-1:123456789012:work",
        "arn:aws:sns:us-east-1:123456789012:work",
        "arn:aws:sqs:us-east-1:123456789012:other",
        "arn:aws:sqs:us-east-1:123456789012:work/other",
        "",
    ] {
        let body = serde_json::to_vec(&serde_json::json!({ "Attributes": { "QueueArn": arn } }))?;
        let response = decode(detail, &body)?;
        assert!(observations_with_request(detail, item_records(&response, detail.items())?, &request).is_err(), "hostile queue ARN accepted: {arn}");
    }
    let response = decode(detail, br#"{"Attributes":{"QueueArn":"arn:aws:sqs:us-east-1:123456789012:work"}}"#)?;
    let missing_request = Record::encode(&Value::Map(BTreeMap::new()))?;
    assert!(observations_with_request(detail, item_records(&response, detail.items())?, &missing_request).is_err(), "missing request URL bypassed ARN correlation");
    Ok(())
}
