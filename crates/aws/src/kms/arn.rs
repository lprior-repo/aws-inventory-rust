//! Shared KMS ARN validation. Key and alias ARNs share the partition, region,
//! account and service segments; only the `key/` or `alias/` resource differs.

use crate::common::Context;

pub(super) const MAX_ARN_BYTES: usize = 1_600;
const ACCOUNT_DIGITS: usize = 12;
const KEY_ID_BYTES: usize = 36;
const MULTI_REGION_PREFIX: &str = "mrk-";
const MULTI_REGION_ID_BYTES: usize = 32;
const MAX_ALIAS_NAME_BYTES: usize = 256;

pub(super) struct ArnRef<'a> {
    pub partition: &'a str,
    pub region: &'a str,
    pub account: &'a str,
    pub resource: &'a str,
}

pub(super) fn split_kms_arn(arn: &str) -> Result<ArnRef<'_>, String> {
    if arn.is_empty() || arn.len() > MAX_ARN_BYTES {
        return Err("KMS ARN is empty or oversized".into());
    }
    let mut parts = arn.splitn(6, ':');
    let scheme = parts.next();
    let partition = parts.next();
    let service = parts.next();
    let region = parts.next();
    let account = parts.next();
    let resource = parts.next();
    let (Some("arn"), Some(partition), Some("kms"), Some(region), Some(account), Some(resource)) =
        (scheme, partition, service, region, account, resource)
    else {
        return Err("ARN does not identify a KMS resource".into());
    };
    if partition.is_empty() || region.is_empty() || resource.is_empty() {
        return Err("KMS ARN lacks a partition, region or resource".into());
    }
    if account.len() != ACCOUNT_DIGITS || !account.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("KMS ARN account segment is not 12 digits".into());
    }
    Ok(ArnRef {
        partition,
        region,
        account,
        resource,
    })
}

/// Returns the parsed ARN and its validated key id.
pub(super) fn split_key_arn(arn: &str) -> Result<(ArnRef<'_>, &str), String> {
    let parts = split_kms_arn(arn)?;
    let key_id = parts
        .resource
        .strip_prefix("key/")
        .ok_or_else(|| "KMS key ARN resource is not key/<id>".to_owned())?;
    validate_key_id(key_id)?;
    Ok((parts, key_id))
}

/// Returns the parsed ARN and the alias name, which keeps its `alias/` prefix.
pub(super) fn split_alias_arn(arn: &str) -> Result<(ArnRef<'_>, &str), String> {
    let parts = split_kms_arn(arn)?;
    let name = parts.resource;
    validate_alias_name(name)?;
    Ok((parts, name))
}

pub(super) fn validate_key_id(key_id: &str) -> Result<(), String> {
    if key_id.len() == KEY_ID_BYTES && is_uuid(key_id.as_bytes()) {
        return Ok(());
    }
    let digest = key_id
        .strip_prefix(MULTI_REGION_PREFIX)
        .filter(|value| value.len() == MULTI_REGION_ID_BYTES);
    if digest.is_some_and(|value| value.bytes().all(|byte| byte.is_ascii_hexdigit())) {
        return Ok(());
    }
    Err("KMS key id is not a key UUID or multi-Region key id".into())
}

fn is_uuid(value: &[u8]) -> bool {
    value.iter().enumerate().all(|(index, byte)| {
        if matches!(index, 8 | 13 | 18 | 23) {
            *byte == b'-'
        } else {
            byte.is_ascii_hexdigit()
        }
    })
}

pub(super) fn validate_alias_name(name: &str) -> Result<(), String> {
    let suffix = name
        .strip_prefix("alias/")
        .ok_or_else(|| "KMS alias name does not start with alias/".to_owned())?;
    if suffix.is_empty()
        || name.len() > MAX_ALIAS_NAME_BYTES
        || suffix.starts_with('/')
        || suffix.ends_with('/')
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'_' | b'-' | b':'))
    {
        return Err("KMS alias name is invalid".into());
    }
    Ok(())
}

/// `ListAliases` may return a bare key id or a key ARN as the target. Both forms
/// are checked; an ARN target must stay inside the scan scope.
pub(super) fn validate_target_key_id(value: &str, ctx: &Context) -> Result<(), String> {
    if !value.starts_with("arn:") {
        return validate_key_id(value);
    }
    let (parts, _) = split_key_arn(value)?;
    if !is_in_scope(&parts, ctx) {
        return Err("KMS alias target ARN is outside the scan scope".into());
    }
    Ok(())
}

pub(super) fn is_in_scope(parts: &ArnRef<'_>, ctx: &Context) -> bool {
    parts.partition == ctx.partition && parts.region == ctx.region && parts.account == ctx.account
}
