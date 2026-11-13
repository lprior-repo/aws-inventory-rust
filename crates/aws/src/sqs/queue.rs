//! Queue URL/ARN validation and attribute mapping.

use std::collections::{BTreeMap, HashMap};

use aws_sdk_sqs::types::QueueAttributeName;
use inventory_core::{Edge, Resource};
use serde_json::{Value, json};

use crate::{AwsScanError, common::Context};

const ACCOUNT_DIGITS: usize = 12;
const QUEUE_NAME_MAX_BYTES: usize = 80;
const QUEUE_HANDLE_MAX_BYTES: usize = 512;

pub(super) struct QueueUrl<'a> {
    pub account: &'a str,
    pub name: &'a str,
}

struct QueueArn<'a> {
    partition: &'a str,
    region: &'a str,
    account: &'a str,
    name: &'a str,
}

pub(super) struct QueueObservation {
    pub resources: Vec<Resource>,
    pub edges: Vec<Edge>,
}

/// Validates the account/name path and returns the canonical queue name. The
/// authority is checked structurally but not pinned to an AWS DNS suffix so a
/// configured SQS endpoint remains usable; the queue ARN is the identity.
pub(super) fn parse_queue_url(url: &str) -> Result<QueueUrl<'_>, String> {
    if url.is_empty() || url.len() > QUEUE_HANDLE_MAX_BYTES {
        return Err("queue URL is empty or oversized".into());
    }
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .ok_or_else(|| "queue URL must use http or https".to_owned())?;
    let (authority, path) = rest
        .split_once('/')
        .ok_or_else(|| "queue URL lacks an account/name path".to_owned())?;
    if authority.is_empty()
        || authority.contains('@')
        || authority.contains('?')
        || authority.contains('#')
        || authority
            .bytes()
            .any(|byte| byte.is_ascii_whitespace() || byte.is_ascii_control())
    {
        return Err("queue URL authority is invalid".into());
    }
    let mut segments = path.split('/');
    let account = segments.next().unwrap_or_default();
    let name = segments.next().unwrap_or_default();
    if segments.next().is_some() {
        return Err("queue URL path must be exactly account/name".into());
    }
    if account.len() != ACCOUNT_DIGITS || !account.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("queue URL account segment is not 12 digits".into());
    }
    validate_queue_name(name)?;
    Ok(QueueUrl { account, name })
}

fn validate_queue_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.len() > QUEUE_NAME_MAX_BYTES
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err("queue name is invalid".into());
    }
    Ok(())
}

fn parse_queue_arn(arn: &str) -> Result<QueueArn<'_>, String> {
    if arn.len() > QUEUE_HANDLE_MAX_BYTES {
        return Err("queue ARN is oversized".into());
    }
    let mut parts = arn.splitn(6, ':');
    let scheme = parts.next();
    let partition = parts.next();
    let service = parts.next();
    let region = parts.next();
    let account = parts.next();
    let name = parts.next();
    let (Some("arn"), Some(partition), Some("sqs"), Some(region), Some(account), Some(name)) =
        (scheme, partition, service, region, account, name)
    else {
        return Err("queue ARN does not identify an SQS queue".into());
    };
    if partition.is_empty() || region.is_empty() {
        return Err("queue ARN lacks a partition or region".into());
    }
    if account.len() != ACCOUNT_DIGITS || !account.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("queue ARN account segment is not 12 digits".into());
    }
    validate_queue_name(name)?;
    Ok(QueueArn {
        partition,
        region,
        account,
        name,
    })
}

pub(super) fn map_queue(
    ctx: &Context,
    url: &str,
    queue: &QueueUrl<'_>,
    attributes: &HashMap<QueueAttributeName, String>,
    tags: Option<BTreeMap<String, String>>,
) -> Result<QueueObservation, AwsScanError> {
    let arn = attribute(attributes, &QueueAttributeName::QueueArn)
        .ok_or_else(|| invalid("GetQueueAttributes lacks QueueArn"))?
        .to_owned();
    let parsed = parse_queue_arn(&arn).map_err(invalid)?;
    if parsed.name != queue.name {
        return Err(invalid("queue ARN name does not match the queue URL"));
    }
    if parsed.account != queue.account || parsed.account != ctx.account {
        return Err(invalid(
            "queue ARN account does not match the queue URL or scan account",
        ));
    }
    if parsed.region != ctx.region {
        return Err(invalid("queue ARN region does not match the scan region"));
    }
    if parsed.partition != ctx.partition {
        return Err(invalid(
            "queue ARN partition does not match the caller partition",
        ));
    }
    let (redrive_target, max_receive_count) = redrive(attributes);
    let tags_collected = tags.is_some();
    let resource = ctx.resource(
        "aws_sqs_queue",
        arn.clone(),
        Some(queue.name.to_owned()),
        json!({
            "queue_name": queue.name,
            "queue_url": url,
            "queue_arn": arn,
            "fifo_queue": flag(attributes, &QueueAttributeName::FifoQueue),
            "approximate_number_of_messages": number(
                attributes,
                &QueueAttributeName::ApproximateNumberOfMessages,
            ),
            "approximate_number_of_messages_not_visible": number(
                attributes,
                &QueueAttributeName::ApproximateNumberOfMessagesNotVisible,
            ),
            "visibility_timeout_seconds": number(attributes, &QueueAttributeName::VisibilityTimeout),
            "created_timestamp": number(attributes, &QueueAttributeName::CreatedTimestamp),
            "kms_master_key_id": attribute(attributes, &QueueAttributeName::KmsMasterKeyId),
            "sqs_managed_sse_enabled": flag(attributes, &QueueAttributeName::SqsManagedSseEnabled),
            "redrive_policy_configured": attribute(attributes, &QueueAttributeName::RedrivePolicy)
                .is_some(),
            "redrive_max_receive_count": max_receive_count,
            "tags_collected": tags_collected,
        }),
        tags.unwrap_or_default(),
    )?;
    let mut edges = Vec::new();
    if let Some(target) = redrive_target.filter(|target| *target != resource.id) {
        let mut edge = ctx.edge(&resource, target, "DELIVERS_TO");
        edge.properties = json!({
            "evidence": "redrive_policy",
            "max_receive_count": max_receive_count,
        });
        edges.push(edge);
    }
    Ok(QueueObservation {
        resources: vec![resource],
        edges,
    })
}

/// Parses only the reviewed RedrivePolicy fields. Malformed or foreign values are
/// evidence absence, never a fabricated relationship.
fn redrive(attributes: &HashMap<QueueAttributeName, String>) -> (Option<String>, Option<u64>) {
    let Some(policy) = attribute(attributes, &QueueAttributeName::RedrivePolicy) else {
        return (None, None);
    };
    let Ok(value) = serde_json::from_str::<Value>(policy) else {
        return (None, None);
    };
    let target = value
        .get("deadLetterTargetArn")
        .and_then(Value::as_str)
        .filter(|arn| parse_queue_arn(arn).is_ok())
        .map(str::to_owned);
    let count = match value.get("maxReceiveCount") {
        Some(Value::String(text)) => text.parse().ok(),
        Some(number) => number.as_u64(),
        None => None,
    };
    (target, count)
}

fn attribute<'a>(
    attributes: &'a HashMap<QueueAttributeName, String>,
    name: &QueueAttributeName,
) -> Option<&'a str> {
    attributes.get(name).map(String::as_str)
}

fn number(
    attributes: &HashMap<QueueAttributeName, String>,
    name: &QueueAttributeName,
) -> Option<u64> {
    attribute(attributes, name).and_then(|value| value.parse().ok())
}

fn flag(
    attributes: &HashMap<QueueAttributeName, String>,
    name: &QueueAttributeName,
) -> Option<bool> {
    attribute(attributes, name).map(|value| value == "true")
}

fn invalid(message: impl Into<String>) -> AwsScanError {
    AwsScanError::InvalidData(message.into())
}
