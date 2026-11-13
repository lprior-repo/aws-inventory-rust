use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use aws_sdk_sqs::config::http::HttpResponse;
use aws_sdk_sqs::error::SdkError;
use aws_sdk_sqs::operation::get_queue_attributes::GetQueueAttributesError;
use aws_sdk_sqs::operation::list_queues::ListQueuesOutput;
use aws_sdk_sqs::types::QueueAttributeName;
use aws_sdk_sqs::types::error::{QueueDoesNotExist, RequestThrottled};
use aws_smithy_types::body::SdkBody;
use aws_smithy_types::error::ErrorMetadata;
use inventory_core::CollectionEvent;
use inventory_engine::limiter::{ApiError, ApiExecutor, ExecutorConfig};
use serde_json::json;
use tokio::sync::mpsc;

use super::*;
use crate::common::error_category;

mod redrive;

type TestResult = Result<(), Box<dyn std::error::Error>>;

const URL: &str = "https://sqs.us-east-1.amazonaws.com/111111111111/orders";
const ARN: &str = "arn:aws:sqs:us-east-1:111111111111:orders";
const DLQ: &str = "arn:aws:sqs:us-east-1:111111111111:orders-dead";

fn context() -> Result<(Context, mpsc::Receiver<CollectionEvent>), ApiError> {
    let (sender, receiver) = mpsc::channel(8);
    Ok((
        Context {
            config: aws_types::SdkConfig::builder().build(),
            executor: Arc::new(ApiExecutor::new(ExecutorConfig::default())?),
            sender,
            account: "111111111111".into(),
            partition: "aws".into(),
            region: "us-east-1".into(),
            max_pages: 3,
            page_size: 100,
        },
        receiver,
    ))
}

fn attributes(pairs: &[(QueueAttributeName, &str)]) -> HashMap<QueueAttributeName, String> {
    pairs
        .iter()
        .map(|(name, value)| (name.clone(), (*value).to_owned()))
        .collect()
}

fn queue_url() -> Result<QueueUrl<'static>, Box<dyn std::error::Error>> {
    parse_queue_url(URL).map_err(|error| -> Box<dyn std::error::Error> { error.into() })
}

#[test]
fn maps_stable_arn_identity_and_bounded_attributes() -> TestResult {
    let (ctx, _receiver) = context()?;
    let url = queue_url()?;
    let attrs = attributes(&[
        (QueueAttributeName::QueueArn, ARN),
        (QueueAttributeName::ApproximateNumberOfMessages, "12"),
        (
            QueueAttributeName::ApproximateNumberOfMessagesNotVisible,
            "3",
        ),
        (QueueAttributeName::VisibilityTimeout, "30"),
        (QueueAttributeName::CreatedTimestamp, "1700000000"),
        (QueueAttributeName::FifoQueue, "false"),
        (QueueAttributeName::SqsManagedSseEnabled, "true"),
    ]);
    let tags = BTreeMap::from([("team".to_owned(), "orders".to_owned())]);
    let observation = map_queue(&ctx, URL, &url, &attrs, Some(tags))?;
    let resource = observation
        .resources
        .first()
        .ok_or("missing mapped queue")?;
    assert_eq!(resource.id, ARN);
    assert_eq!(resource.kind, "aws_sqs_queue");
    assert_eq!(resource.name.as_deref(), Some("orders"));
    assert_eq!(resource.account_id, "111111111111");
    assert_eq!(resource.region, "us-east-1");
    assert_eq!(resource.properties.get("queue_url"), Some(&json!(URL)));
    assert_eq!(
        resource.properties.get("approximate_number_of_messages"),
        Some(&json!(12))
    );
    assert_eq!(
        resource.properties.get("visibility_timeout_seconds"),
        Some(&json!(30))
    );
    assert_eq!(resource.properties.get("fifo_queue"), Some(&json!(false)));
    assert_eq!(
        resource.properties.get("sqs_managed_sse_enabled"),
        Some(&json!(true))
    );
    assert_eq!(
        resource.properties.get("tags_collected"),
        Some(&json!(true))
    );
    assert_eq!(resource.tags.get("team"), Some(&"orders".to_owned()));
    assert!(observation.edges.is_empty());
    assert_eq!(attribute_names().len(), 9);
    assert!(!attribute_names().contains(&QueueAttributeName::All));
    Ok(())
}

#[test]
fn tag_failure_is_visible_and_does_not_discard_the_queue() -> TestResult {
    let (ctx, _receiver) = context()?;
    let url = queue_url()?;
    let attrs = attributes(&[(QueueAttributeName::QueueArn, ARN)]);
    let observation = map_queue(&ctx, URL, &url, &attrs, None)?;
    let resource = observation
        .resources
        .first()
        .ok_or("missing mapped queue")?;
    assert_eq!(
        resource.properties.get("tags_collected"),
        Some(&json!(false))
    );
    assert!(resource.tags.is_empty());
    Ok(())
}

#[test]
fn accepts_aws_legacy_and_loopback_queue_urls() -> TestResult {
    for url in [
        URL,
        "https://sqs.us-east-1.amazonaws.com/111111111111/orders.fifo",
        "http://127.0.0.1:4566/000000000000/orders",
        "https://queue.amazonaws.com/111111111111/orders",
    ] {
        let parsed =
            parse_queue_url(url).map_err(|error| -> Box<dyn std::error::Error> { error.into() })?;
        assert!(!parsed.name.is_empty());
        assert_eq!(parsed.account.len(), 12);
    }
    Ok(())
}

#[test]
fn rejects_malformed_queue_urls() {
    for url in [
        "",
        "orders",
        "ftp://sqs.us-east-1.amazonaws.com/111111111111/orders",
        "https://user@sqs.us-east-1.amazonaws.com/111111111111/orders",
        "https://sqs.us-east-1.amazonaws.com/111/orders",
        "https://sqs.us-east-1.amazonaws.com/111111111111/",
        "https://sqs.us-east-1.amazonaws.com/111111111111/orders/extra",
        "https://sqs.us-east-1.amazonaws.com/111111111111/bad name",
        "https://sqs.us-east-1.amazonaws.com/111111111111/orders?x=1",
        "https://sqs.us-east-1.amazonaws.com/111111111111/orders#fragment",
    ] {
        assert!(parse_queue_url(url).is_err(), "accepted: {url}");
    }
}

#[test]
fn queue_url_and_arn_must_agree_with_the_scan_scope() -> TestResult {
    let (ctx, _receiver) = context()?;
    let url = queue_url()?;
    for arn in [
        "arn:aws:sqs:us-east-1:111111111111:other",
        "arn:aws:sqs:us-east-1:222222222222:orders",
        "arn:aws:sqs:us-west-2:111111111111:orders",
        "arn:aws-cn:sqs:us-east-1:111111111111:orders",
        "arn:aws:sns:us-east-1:111111111111:orders",
        "arn:aws:sqs:us-east-1:111111111111",
    ] {
        let attrs = attributes(&[(QueueAttributeName::QueueArn, arn)]);
        assert!(
            map_queue(&ctx, URL, &url, &attrs, None).is_err(),
            "accepted ARN: {arn}"
        );
    }
    let attrs = attributes(&[]);
    assert!(map_queue(&ctx, URL, &url, &attrs, None).is_err());
    Ok(())
}

#[test]
fn page_keeps_the_terminal_token_and_counts_failed_queues() -> TestResult {
    let (ctx, _receiver) = context()?;
    let url = queue_url()?;
    let attrs = attributes(&[(QueueAttributeName::QueueArn, ARN)]);
    let observation = map_queue(&ctx, URL, &url, &attrs, None)?;
    let page = page_body(
        &ListQueuesOutput::builder()
            .queue_urls(URL)
            .queue_urls("https://sqs.us-east-1.amazonaws.com/111111111111/orders-dead")
            .next_token("cursor-2")
            .build(),
        vec![observation],
    );
    assert_eq!(page.next_token.as_deref(), Some("cursor-2"));
    assert_eq!(page.listed_count, 2);
    assert_eq!(page.resources.len(), 1);
    let terminal = page_body(
        &ListQueuesOutput::builder().queue_urls(URL).build(),
        Vec::new(),
    );
    assert!(terminal.next_token.is_none());
    assert_eq!(terminal.listed_count, 1);
    assert_eq!(list_page_size(100), 100);
    assert_eq!(list_page_size(1), 1);
    assert_eq!(list_page_size(0), 1);
    assert_eq!(list_page_size(5_000), 1_000);
    Ok(())
}

#[test]
fn missing_queue_is_a_terminal_permanent_failure() -> TestResult {
    let modeled = QueueDoesNotExist::builder()
        .message("queue gone")
        .meta(
            ErrorMetadata::builder()
                .code("AWS.SimpleQueueService.NonExistentQueue")
                .message("queue gone")
                .build(),
        )
        .build();
    let response = HttpResponse::new(400_u16.try_into()?, SdkBody::empty());
    let error = SdkError::service_error(
        GetQueueAttributesError::QueueDoesNotExist(modeled),
        response,
    );
    let error = AwsScanError::Api(sdk_error(error));
    assert_eq!(error_category(&error), "api_error");
    assert!(matches!(error, AwsScanError::Api(ApiError::Permanent(_))));
    Ok(())
}

#[test]
fn throttled_queue_read_is_retryable() -> TestResult {
    let modeled = RequestThrottled::builder()
        .message("slow down")
        .meta(
            ErrorMetadata::builder()
                .code("RequestThrottled")
                .message("slow down")
                .build(),
        )
        .build();
    let response = HttpResponse::new(503_u16.try_into()?, SdkBody::empty());
    let error =
        SdkError::service_error(GetQueueAttributesError::RequestThrottled(modeled), response);
    let error = AwsScanError::Api(sdk_error(error));
    assert_eq!(error_category(&error), "throttled");
    assert!(matches!(error, AwsScanError::Api(ApiError::Throttled(_))));
    Ok(())
}
