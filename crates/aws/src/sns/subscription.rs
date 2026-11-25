use std::collections::BTreeMap;

use aws_sdk_sns::types::Subscription;
use inventory_core::{Edge, Resource};
use serde_json::json;
use sha2::{Digest, Sha256};

use super::arn::{
    ArnRef, endpoint, is_in_request_region, split_sns_arn, split_subscription_arn,
    validate_topic_name,
};
use super::topic::invalid;
use crate::{AwsScanError, common::Context};

pub(super) const PENDING_CONFIRMATION: &str = "PendingConfirmation";

pub(super) struct SubscriptionObservation {
    pub resource: Resource,
    pub edge: Option<Edge>,
}

pub(super) fn map_subscription(
    ctx: &Context,
    subscription: &Subscription,
) -> Result<Option<SubscriptionObservation>, AwsScanError> {
    let raw_arn = subscription
        .subscription_arn()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| invalid("ListSubscriptions lacks SubscriptionArn"))?;
    let endpoint = endpoint(subscription.endpoint()).map_err(invalid)?;
    let owner = subscription
        .owner()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| invalid("ListSubscriptions lacks Owner"))?;
    let topic_arn = subscription.topic_arn().filter(|value| !value.is_empty());
    let pending = raw_arn == PENDING_CONFIRMATION;
    let identity = if pending {
        let topic_arn =
            topic_arn.ok_or_else(|| invalid("pending SNS subscription lacks TopicArn"))?;
        let parts = split_sns_arn(topic_arn).map_err(invalid)?;
        validate_topic_name(parts.resource).map_err(invalid)?;
        if !owns_in_request_region(&parts, owner, ctx) {
            return Ok(None);
        }
        pending_identity(
            topic_arn,
            subscription.protocol().unwrap_or_default(),
            endpoint.unwrap_or_default(),
        )
    } else {
        let (parts, _, _) = split_subscription_arn(raw_arn).map_err(invalid)?;
        if !owns_in_request_region(&parts, owner, ctx) {
            return Ok(None);
        }
        raw_arn.to_owned()
    };
    let target = topic_arn.and_then(|arn| {
        let parts = split_sns_arn(arn).ok()?;
        if !is_in_request_region(&parts, ctx) || validate_topic_name(parts.resource).is_err() {
            return None;
        }
        Some(arn)
    });
    let resource = ctx.resource(
        "aws_sns_subscription",
        identity,
        None,
        json!({
            "subscription_arn": raw_arn,
            "topic_arn": topic_arn,
            "protocol": subscription.protocol(),
            "endpoint": endpoint,
            "owner": subscription.owner(),
            "pending_confirmation": pending,
            "identity_source": if pending { "pending_confirmation" } else { "subscription_arn" },
        }),
        BTreeMap::new(),
    )?;
    let edge = target.map(|arn| {
        let mut edge = ctx.edge(&resource, arn.to_owned(), "SUBSCRIBES_TO");
        edge.properties = json!({"evidence": "topic_arn"});
        edge
    });
    Ok(Some(SubscriptionObservation { resource, edge }))
}

/// A subscription belongs to the scan when the request's partition and region
/// match its ARN and its `Owner` is the scanned account. The ARN's account
/// segment is the topic owner's and may legitimately differ.
fn owns_in_request_region(parts: &ArnRef<'_>, owner: &str, ctx: &Context) -> bool {
    is_in_request_region(parts, ctx) && owner == ctx.account
}

fn pending_identity(topic_arn: &str, protocol: &str, endpoint: &str) -> String {
    let seed = format!("{protocol}\n{endpoint}");
    format!(
        "{topic_arn}:pending:{:x}",
        base16ct::HexDisplay(&Sha256::digest(seed.as_bytes()))
    )
}
