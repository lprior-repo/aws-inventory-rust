use super::*;
#[test]
fn given_hostile_queue_urls_when_child_requests_bind_then_mismatches_fail_without_skipping()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(bytes()?)?;
    let list = catalog
        .lookup("sqs", "ListQueues")
        .ok_or("queue discovery absent")?;
    let detail = catalog
        .lookup("sqs", "GetQueueAttributes")
        .ok_or("queue attributes absent")?;
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
        let row = item_records(&response, list.items())?
            .into_iter()
            .next()
            .ok_or("hostile queue URL row absent")?;
        assert!(
            child_fields_optional(&catalog, list, detail, &row).is_err(),
            "hostile URL accepted or skipped: {url}"
        );
    }
    Ok(())
}

#[test]
fn given_queue_arn_mismatches_when_attributes_identify_then_scope_or_queue_name_failure_is_explicit()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(bytes()?)?;
    let list = catalog
        .lookup("sqs", "ListQueues")
        .ok_or("queue discovery absent")?;
    let detail = catalog
        .lookup("sqs", "GetQueueAttributes")
        .ok_or("queue attributes absent")?;
    let response = decode(
        list,
        br#"{"QueueUrls":["https://sqs.us-east-1.amazonaws.com/123456789012/work"]}"#,
    )?;
    let row = item_records(&response, list.items())?
        .into_iter()
        .next()
        .ok_or("queue URL row absent")?;
    let fields = child_fields(&catalog, list, detail, &row)?;
    let request = crate::input::merge_input(
        &Record::encode(&Value::Map(BTreeMap::new()))?,
        &fields,
    )?;
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
        assert!(
            observations_with_request(detail, item_records(&response, detail.items())?, &request)
                .is_err(),
            "hostile queue ARN accepted: {arn}"
        );
    }
    let response = decode(
        detail,
        br#"{"Attributes":{"QueueArn":"arn:aws:sqs:us-east-1:123456789012:work"}}"#,
    )?;
    let missing_request = Record::encode(&Value::Map(BTreeMap::new()))?;
    assert!(
        observations_with_request(
            detail,
            item_records(&response, detail.items())?,
            &missing_request
        )
        .is_err(),
        "missing request URL bypassed ARN correlation"
    );
    Ok(())
}
