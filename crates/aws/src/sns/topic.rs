//! Topic attribute mapping. Policy and delivery-policy bodies are presence-only.

use std::collections::{BTreeMap, HashMap};

use aws_sdk_sns::types::Tag;
use inventory_core::Resource;
use serde_json::json;

use super::arn::{is_in_scope, split_sns_arn, validate_topic_name};
use crate::{AwsScanError, common::Context};

const MAX_TAG_BYTES: usize = 4_096;

/// Topic tags preserve empty values, which SNS permits, while dropping entries
/// the inventory model cannot validate.
pub(super) fn tags_from(tags: &[Tag]) -> BTreeMap<String, String> {
    tags.iter()
        .filter(|tag| {
            let key = tag.key();
            let value = tag.value();
            !key.is_empty()
                && key.len() <= MAX_TAG_BYTES
                && !key.contains('\0')
                && value.len() <= MAX_TAG_BYTES
                && !value.contains('\0')
        })
        .map(|tag| (tag.key().to_owned(), tag.value().to_owned()))
        .collect()
}

pub(super) fn map_topic(
    ctx: &Context,
    arn: &str,
    attributes: &HashMap<String, String>,
    tags: Option<BTreeMap<String, String>>,
) -> Result<Resource, AwsScanError> {
    let parts = split_sns_arn(arn).map_err(invalid)?;
    if !is_in_scope(&parts, ctx) {
        return Err(invalid("SNS topic ARN is outside the scan scope"));
    }
    validate_topic_name(parts.resource).map_err(invalid)?;
    if let Some(returned) = attribute(attributes, "TopicArn")
        && returned != arn
    {
        return Err(invalid(
            "GetTopicAttributes TopicArn does not match the listed topic",
        ));
    }
    let tags_collected = tags.is_some();
    ctx.resource(
        "aws_sns_topic",
        arn.to_owned(),
        Some(parts.resource.to_owned()),
        json!({
            "topic_arn": arn,
            "display_name": attribute(attributes, "DisplayName"),
            "owner": attribute(attributes, "Owner"),
            "subscriptions_confirmed": number(attributes, "SubscriptionsConfirmed"),
            "subscriptions_pending": number(attributes, "SubscriptionsPending"),
            "subscriptions_deleted": number(attributes, "SubscriptionsDeleted"),
            "policy_configured": attribute(attributes, "Policy").is_some(),
            "delivery_policy_configured": attribute(attributes, "DeliveryPolicy").is_some(),
            "kms_master_key_id": attribute(attributes, "KmsMasterKeyId"),
            "fifo_topic": flag(attributes, "FifoTopic"),
            "content_based_deduplication": flag(attributes, "ContentBasedDeduplication"),
            "tags_collected": tags_collected,
        }),
        tags.unwrap_or_default(),
    )
}

fn attribute<'a>(attributes: &'a HashMap<String, String>, key: &str) -> Option<&'a str> {
    attributes
        .get(key)
        .map(String::as_str)
        .filter(|value| !value.is_empty())
}

fn number(attributes: &HashMap<String, String>, key: &str) -> Option<u64> {
    attribute(attributes, key).and_then(|value| value.parse().ok())
}

fn flag(attributes: &HashMap<String, String>, key: &str) -> Option<bool> {
    attribute(attributes, key).map(|value| value == "true")
}

pub(super) fn invalid(message: impl Into<String>) -> AwsScanError {
    AwsScanError::InvalidData(message.into())
}
