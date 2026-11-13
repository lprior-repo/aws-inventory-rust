use std::collections::HashMap;
use std::sync::Arc;

use aws_sdk_sns::types::{Subscription, Tag, Topic};
use inventory_core::CollectionEvent;
use inventory_engine::limiter::{ApiError, ApiExecutor, ExecutorConfig};
use tokio::sync::mpsc;

use super::*;
use crate::common::error_category;

mod pagination_tests;
mod subscription_tests;
mod topic_tests;

pub(super) type TestResult = Result<(), Box<dyn std::error::Error>>;

pub(super) const TOPIC_ARN: &str = "arn:aws:sns:us-east-1:111111111111:orders";
pub(super) const TOPIC_NAME: &str = "orders";
pub(super) const SUBSCRIPTION_ARN: &str =
    "arn:aws:sns:us-east-1:111111111111:orders:2bcfbf39-05c3-41de-beaa-fcfcc21c8f55";

pub(super) fn context(
    region: &str,
) -> Result<(Context, mpsc::Receiver<CollectionEvent>), ApiError> {
    let (sender, receiver) = mpsc::channel(8);
    Ok((
        Context {
            config: aws_types::SdkConfig::builder().build(),
            executor: Arc::new(ApiExecutor::new(ExecutorConfig::default())?),
            sender,
            account: "111111111111".into(),
            partition: "aws".into(),
            region: region.into(),
            max_pages: 3,
            page_size: 100,
        },
        receiver,
    ))
}

pub(super) fn attributes(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect()
}

pub(super) fn subscription(
    arn: &str,
    protocol: &str,
    endpoint: Option<&str>,
    topic: Option<&str>,
) -> Subscription {
    Subscription::builder()
        .subscription_arn(arn)
        .owner("111111111111")
        .protocol(protocol)
        .set_endpoint(endpoint.map(str::to_owned))
        .set_topic_arn(topic.map(str::to_owned))
        .build()
}

pub(super) fn topic(arn: &str) -> Topic {
    Topic::builder().topic_arn(arn).build()
}
