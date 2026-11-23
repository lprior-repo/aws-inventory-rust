use super::*;
type TestResult = Result<(), Box<dyn std::error::Error>>;
#[test]
fn official_queue_endpoint_families_retain_case_sensitive_names() -> TestResult {
    let account = AccountId::parse("123456789012")?;
    for (region, host) in [
        ("us-east-1", "sqs.us-east-1.amazonaws.com"),
        ("us-east-1", "sqs-fips.us-east-1.api.aws"),
        ("us-east-1", "queue.amazonaws.com"),
        ("eu-west-1", "eu-west-1.queue.amazonaws.com"),
        ("cn-north-1", "sqs.cn-north-1.amazonaws.com.cn"),
        ("cn-north-1", "sqs.cn-north-1.api.amazonwebservices.com.cn"),
        ("us-iso-east-1", "sqs.us-iso-east-1.c2s.ic.gov"),
    ] {
        let region = RegionId::parse(region)?;
        let trusted = IdentityContext {
            account: &account,
            region: &region,
        };
        assert_eq!(
            queue_url(
                &format!("https://{host}/123456789012/MyQueue.fifo"),
                &trusted
            )?,
            "MyQueue.fifo"
        );
    }
    Ok(())
}
#[test]
fn queue_url_rejects_spoofing_cross_scope_and_ambiguous_separators() -> TestResult {
    let account = AccountId::parse("123456789012")?;
    let region = RegionId::parse("us-east-1")?;
    let trusted = IdentityContext {
        account: &account,
        region: &region,
    };
    for url in [
        "http://sqs.us-east-1.amazonaws.com/123456789012/Queue",
        "https://sqs.us-east-1.amazonaws.com.evil.test/123456789012/Queue",
        "https://user@sqs.us-east-1.amazonaws.com/123456789012/Queue",
        "https://sqs.us-east-1.amazonaws.com:443/123456789012/Queue",
        "https://sqs.us-east-2.amazonaws.com/123456789012/Queue",
        "https://sqs.us-east-1.amazonaws.com/999999999999/Queue",
        "https://sqs.us-east-1.amazonaws.com/123456789012/Queue%2fOther",
        "https://sqs.us-east-1.amazonaws.com/123456789012/Queue/Other",
        "https://sqs.us-east-1.amazonaws.com/123456789012/Queue?x=1",
        "https://sqs.us-east-1.amazonaws.com/123456789012/Queue#fragment",
        "https://sqs.us-east-1.amazonaws.com/123456789012/",
    ] {
        assert!(queue_url(url, &trusted).is_err(), "{url}");
    }
    Ok(())
}
#[test]
fn selected_queue_arn_must_match_requested_queue_and_resource_type() -> TestResult {
    let account = AccountId::parse("123456789012")?;
    let region = RegionId::parse("us-east-1")?;
    let trusted = IdentityContext {
        account: &account,
        region: &region,
    };
    let url = "https://sqs.us-east-1.amazonaws.com/123456789012/Queue";
    assert!(
        validate_resource_arn(
            "arn:aws:sqs:us-east-1:123456789012:Queue",
            &ArnResourceKind::SqsQueue,
            &trusted,
            Some(url)
        )
        .is_ok()
    );
    for arn in [
        "arn:aws:sqs:us-east-1:123456789012:Other",
        "arn:aws:sns:us-east-1:123456789012:Queue",
        "arn:aws:sqs:us-west-2:123456789012:Queue",
        "arn:aws-cn:sqs:us-east-1:123456789012:Queue",
        "arn:aws:sqs:us-east-1:999999999999:Queue",
        "arn:aws:sqs:us-east-1:123456789012:Queue:extra",
    ] {
        assert!(
            validate_resource_arn(arn, &ArnResourceKind::SqsQueue, &trusted, Some(url)).is_err(),
            "{arn}"
        );
    }
    assert!(
        validate_resource_arn(
            "arn:aws:sqs:us-east-1:123456789012:Queue",
            &ArnResourceKind::SqsQueue,
            &trusted,
            None
        )
        .is_err()
    );
    Ok(())
}
