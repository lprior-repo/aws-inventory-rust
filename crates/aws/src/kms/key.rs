use std::collections::BTreeMap;

use aws_sdk_kms::types::{KeyManagerType, KeyMetadata, Tag};
use aws_smithy_types::DateTime;
use inventory_core::Resource;
use serde_json::json;

use super::arn::{is_in_scope, split_key_arn};
use crate::{AwsScanError, common::Context};

const MAX_TAG_BYTES: usize = 4_096;

pub(super) struct Rotation {
    pub enabled: bool,
    pub period_in_days: Option<i32>,
}

pub(super) fn tags_from(tags: &[Tag]) -> BTreeMap<String, String> {
    tags.iter()
        .filter(|tag| {
            let key = tag.tag_key();
            let value = tag.tag_value();
            !key.is_empty()
                && key.len() <= MAX_TAG_BYTES
                && !key.contains('\0')
                && value.len() <= MAX_TAG_BYTES
                && !value.contains('\0')
        })
        .map(|tag| (tag.tag_key().to_owned(), tag.tag_value().to_owned()))
        .collect()
}

pub(super) fn map_key(
    ctx: &Context,
    listed_arn: &str,
    metadata: &KeyMetadata,
    rotation: Option<Rotation>,
    tags: Option<BTreeMap<String, String>>,
) -> Result<Resource, AwsScanError> {
    let (parts, key_id) = split_key_arn(listed_arn).map_err(invalid)?;
    if !is_in_scope(&parts, ctx) {
        return Err(invalid("KMS key ARN is outside the scan scope"));
    }
    let returned = metadata
        .arn()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| invalid("DescribeKey lacks KeyMetadata.Arn"))?;
    if returned != listed_arn {
        return Err(invalid(
            "DescribeKey KeyMetadata.Arn does not match the listed key",
        ));
    }
    if metadata.key_id() != key_id {
        return Err(invalid(
            "DescribeKey KeyMetadata.KeyId does not match the listed key ARN",
        ));
    }
    if let Some(account) = metadata.aws_account_id()
        && account != ctx.account
    {
        return Err(invalid(
            "DescribeKey KeyMetadata.AWSAccountId is outside the scan scope",
        ));
    }
    let customer_managed = matches!(metadata.key_manager(), Some(KeyManagerType::Customer));
    let rotation = rotation.filter(|_| customer_managed);
    let rotation_collected = customer_managed && rotation.is_some();
    let tags_collected = tags.is_some();
    ctx.resource(
        "aws_kms_key",
        listed_arn.to_owned(),
        Some(key_id.to_owned()),
        json!({
            "key_arn": listed_arn,
            "key_id": key_id,
            "aws_account_id": metadata.aws_account_id(),
            "description": metadata.description(),
            "key_manager": metadata.key_manager().map(|value| value.as_str()),
            "key_usage": metadata.key_usage().map(|value| value.as_str()),
            "key_state": metadata.key_state().map(|value| value.as_str()),
            "key_spec": metadata.key_spec().map(|value| value.as_str()),
            "origin": metadata.origin().map(|value| value.as_str()),
            "enabled": metadata.enabled(),
            "multi_region": metadata.multi_region(),
            "creation_date": millis(metadata.creation_date())?,
            "rotation_enabled": rotation.as_ref().map(|value| value.enabled),
            "rotation_period_in_days": rotation.as_ref().and_then(|value| value.period_in_days),
            "rotation_collected": rotation_collected,
            "tags_collected": tags_collected,
        }),
        tags.unwrap_or_default(),
    )
}

pub(super) fn millis(value: Option<&DateTime>) -> Result<Option<i64>, AwsScanError> {
    value
        .map(|value| {
            value
                .to_millis()
                .map_err(|_| invalid("KMS key timestamp is out of range"))
        })
        .transpose()
}

pub(super) fn invalid(message: impl Into<String>) -> AwsScanError {
    AwsScanError::InvalidData(message.into())
}
