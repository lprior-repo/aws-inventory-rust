use std::sync::Arc;

use aws_sdk_kms::operation::{list_aliases::ListAliasesOutput, list_keys::ListKeysOutput};
use aws_sdk_kms::types::{AliasListEntry, KeyListEntry, KeyManagerType, KeyMetadata};
use aws_smithy_types::DateTime;
use inventory_core::CollectionEvent;
use inventory_engine::limiter::{ApiError, ApiExecutor, ExecutorConfig};
use tokio::sync::mpsc;

use super::*;
use crate::common::error_category;

mod alias_tests;
mod arn_tests;
mod key_tests;
mod pagination_tests;

pub(super) type TestResult = Result<(), Box<dyn std::error::Error>>;

pub(super) const KEY_ID: &str = "1234abcd-12ab-34cd-56ef-1234567890ab";
pub(super) const KEY_ARN: &str =
    "arn:aws:kms:us-east-1:111111111111:key/1234abcd-12ab-34cd-56ef-1234567890ab";
pub(super) const OTHER_KEY_ID: &str = "2234abcd-12ab-34cd-56ef-1234567890ab";
pub(super) const ALIAS_NAME: &str = "alias/orders";
pub(super) const ALIAS_ARN: &str = "arn:aws:kms:us-east-1:111111111111:alias/orders";

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

pub(super) fn metadata(
    arn: Option<&str>,
    key_id: &str,
    manager: KeyManagerType,
) -> Result<KeyMetadata, Box<dyn std::error::Error>> {
    Ok(KeyMetadata::builder()
        .key_id(key_id)
        .set_arn(arn.map(str::to_owned))
        .aws_account_id("111111111111")
        .description("orders encryption")
        .key_manager(manager)
        .key_usage(aws_sdk_kms::types::KeyUsageType::EncryptDecrypt)
        .key_state(aws_sdk_kms::types::KeyState::Enabled)
        .key_spec(aws_sdk_kms::types::KeySpec::SymmetricDefault)
        .origin(aws_sdk_kms::types::OriginType::AwsKms)
        .enabled(true)
        .multi_region(false)
        .creation_date(DateTime::from_secs(1))
        .build()?)
}

pub(super) fn key_entry(arn: Option<&str>, key_id: Option<&str>) -> KeyListEntry {
    KeyListEntry::builder()
        .set_key_arn(arn.map(str::to_owned))
        .set_key_id(key_id.map(str::to_owned))
        .build()
}

pub(super) fn alias_entry(
    arn: Option<&str>,
    name: Option<&str>,
    target: Option<&str>,
) -> AliasListEntry {
    AliasListEntry::builder()
        .set_alias_arn(arn.map(str::to_owned))
        .set_alias_name(name.map(str::to_owned))
        .set_target_key_id(target.map(str::to_owned))
        .creation_date(DateTime::from_secs(2))
        .last_updated_date(DateTime::from_secs(3))
        .build()
}
