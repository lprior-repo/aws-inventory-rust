use std::sync::Arc;

use aws_sdk_sfn::operation::describe_activity::DescribeActivityOutput;
use aws_sdk_sfn::operation::describe_state_machine::DescribeStateMachineOutput;
use aws_sdk_sfn::types::{
    ActivityListItem, EncryptionConfiguration, EncryptionType, StateMachineListItem,
    StateMachineStatus, StateMachineType,
};
use aws_smithy_types::DateTime;
use inventory_core::CollectionEvent;
use inventory_engine::limiter::{ApiError, ApiExecutor, ExecutorConfig};
use tokio::sync::mpsc;

use super::activity::map_activity;
use super::failure::failure;
use super::page::LIST_MAX_RESULTS;
use super::state_machine::map_state_machine;
use super::*;
use crate::common::error_category;

mod activity_tests;
mod arn_tests;
mod pagination_tests;
mod state_machine_tests;

pub(super) type TestResult = Result<(), Box<dyn std::error::Error>>;

pub(super) const STATE_MACHINE_ARN: &str =
    "arn:aws:states:us-east-1:111111111111:stateMachine:orders";
pub(super) const STATE_MACHINE_NAME: &str = "orders";
pub(super) const ACTIVITY_ARN: &str = "arn:aws:states:us-east-1:111111111111:activity:ship";
pub(super) const ACTIVITY_NAME: &str = "ship";
pub(super) const ROLE_ARN: &str = "arn:aws:iam::111111111111:role/inventory/runner";
pub(super) const KEY_ARN: &str =
    "arn:aws:kms:us-east-1:111111111111:key/1234abcd-12ab-34cd-56ef-1234567890ab";

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

pub(super) fn state_machine_item(
    arn: &str,
    name: &str,
) -> Result<StateMachineListItem, Box<dyn std::error::Error>> {
    Ok(StateMachineListItem::builder()
        .state_machine_arn(arn)
        .name(name)
        .r#type(StateMachineType::Standard)
        .creation_date(DateTime::from_secs(2))
        .build()?)
}

pub(super) fn activity_item(
    arn: &str,
    name: &str,
) -> Result<ActivityListItem, Box<dyn std::error::Error>> {
    Ok(ActivityListItem::builder()
        .activity_arn(arn)
        .name(name)
        .creation_date(DateTime::from_secs(2))
        .build()?)
}

pub(super) fn encryption(key: &str) -> Result<EncryptionConfiguration, Box<dyn std::error::Error>> {
    Ok(EncryptionConfiguration::builder()
        .kms_key_id(key)
        .r#type(EncryptionType::CustomerManagedKmsKey)
        .build()?)
}

pub(super) fn state_machine_description(
    arn: &str,
    name: &str,
    key: Option<&str>,
) -> Result<DescribeStateMachineOutput, Box<dyn std::error::Error>> {
    let mut builder = DescribeStateMachineOutput::builder()
        .state_machine_arn(arn)
        .name(name)
        .definition("{\"StartAt\":\"SECRET-DEFINITION\",\"States\":{}}")
        .variable_references("orders", vec!["$.secret".to_owned()])
        .role_arn(ROLE_ARN)
        .r#type(StateMachineType::Standard)
        .creation_date(DateTime::from_secs(3))
        .status(StateMachineStatus::Active)
        .revision_id("rev-1")
        .description("orders workflow");
    if let Some(key) = key {
        builder = builder.encryption_configuration(encryption(key)?);
    }
    Ok(builder.build()?)
}

pub(super) fn activity_description(
    arn: &str,
    name: &str,
    key: Option<&str>,
) -> Result<DescribeActivityOutput, Box<dyn std::error::Error>> {
    let mut builder = DescribeActivityOutput::builder()
        .activity_arn(arn)
        .name(name)
        .creation_date(DateTime::from_secs(4));
    if let Some(key) = key {
        builder = builder.encryption_configuration(encryption(key)?);
    }
    Ok(builder.build()?)
}
