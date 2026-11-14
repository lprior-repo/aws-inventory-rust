use crate::common::Context;

pub(super) const MAX_ARN_BYTES: usize = 512;
pub(super) const MAX_ENDPOINT_BYTES: usize = 4_096;
const ACCOUNT_DIGITS: usize = 12;
const MAX_TOPIC_NAME_BYTES: usize = 256;
const MAX_SUBSCRIPTION_ID_BYTES: usize = 128;

pub(super) struct ArnRef<'a> {
    pub partition: &'a str,
    pub region: &'a str,
    pub account: &'a str,
    pub resource: &'a str,
}

pub(super) fn split_sns_arn(arn: &str) -> Result<ArnRef<'_>, String> {
    if arn.is_empty() || arn.len() > MAX_ARN_BYTES {
        return Err("SNS ARN is empty or oversized".into());
    }
    let mut parts = arn.splitn(6, ':');
    let scheme = parts.next();
    let partition = parts.next();
    let service = parts.next();
    let region = parts.next();
    let account = parts.next();
    let resource = parts.next();
    let (Some("arn"), Some(partition), Some("sns"), Some(region), Some(account), Some(resource)) =
        (scheme, partition, service, region, account, resource)
    else {
        return Err("ARN does not identify an SNS resource".into());
    };
    if partition.is_empty() || region.is_empty() || resource.is_empty() {
        return Err("SNS ARN lacks a partition, region or resource".into());
    }
    if account.len() != ACCOUNT_DIGITS || !account.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("SNS ARN account segment is not 12 digits".into());
    }
    Ok(ArnRef {
        partition,
        region,
        account,
        resource,
    })
}

pub(super) fn split_subscription_arn(arn: &str) -> Result<(ArnRef<'_>, &str, &str), String> {
    let parts = split_sns_arn(arn)?;
    let (topic, id) = parts
        .resource
        .rsplit_once(':')
        .ok_or_else(|| "SNS subscription ARN lacks a topic/id resource".to_owned())?;
    validate_topic_name(topic)?;
    if id.is_empty()
        || id.len() > MAX_SUBSCRIPTION_ID_BYTES
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err("SNS subscription ARN identifier is invalid".into());
    }
    Ok((parts, topic, id))
}

pub(super) fn validate_topic_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.len() > MAX_TOPIC_NAME_BYTES
        || name.starts_with('.')
        || name.ends_with('.')
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err("SNS topic name is invalid".into());
    }
    Ok(())
}

pub(super) fn is_in_scope(parts: &ArnRef<'_>, ctx: &Context) -> bool {
    parts.partition == ctx.partition && parts.region == ctx.region && parts.account == ctx.account
}

pub(super) fn endpoint(value: Option<&str>) -> Result<Option<&str>, String> {
    match value {
        Some(value) if value.len() > MAX_ENDPOINT_BYTES || value.contains('\0') => {
            Err("SNS subscription endpoint is invalid".into())
        }
        Some("") => Ok(None),
        other => Ok(other),
    }
}
