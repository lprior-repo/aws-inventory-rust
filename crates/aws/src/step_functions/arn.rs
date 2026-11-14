use crate::common::Context;

pub(super) const MAX_ARN_BYTES: usize = 2_048;
const ACCOUNT_DIGITS: usize = 12;
const MAX_NAME_BYTES: usize = 256;
const KEY_ID_BYTES: usize = 36;
const MULTI_REGION_PREFIX: &str = "mrk-";
const MULTI_REGION_ID_BYTES: usize = 32;

pub(super) struct ArnRef<'a> {
    pub partition: &'a str,
    pub region: &'a str,
    pub account: &'a str,
    pub name: &'a str,
}

pub(super) fn split_state_machine_arn(arn: &str) -> Result<ArnRef<'_>, String> {
    split_states_arn(arn, "stateMachine:")
}

pub(super) fn split_activity_arn(arn: &str) -> Result<ArnRef<'_>, String> {
    split_states_arn(arn, "activity:")
}

fn split_states_arn<'a>(arn: &'a str, prefix: &str) -> Result<ArnRef<'a>, String> {
    let (partition, region, account, resource) = segments(arn)?;
    let name = resource
        .strip_prefix(prefix)
        .ok_or_else(|| format!("Step Functions ARN resource is not {prefix}<name>"))?;
    validate_name(name)?;
    Ok(ArnRef {
        partition,
        region,
        account,
        name,
    })
}

fn segments(arn: &str) -> Result<(&str, &str, &str, &str), String> {
    if arn.is_empty() || arn.len() > MAX_ARN_BYTES {
        return Err("Step Functions ARN is empty or oversized".into());
    }
    let mut parts = arn.splitn(6, ':');
    let (Some("arn"), Some(partition), Some("states"), Some(region), Some(account), Some(resource)) = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) else {
        return Err("ARN does not identify a Step Functions resource".into());
    };
    if partition.is_empty() || region.is_empty() {
        return Err("Step Functions ARN lacks a partition or region".into());
    }
    if account.len() != ACCOUNT_DIGITS || !account.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("Step Functions ARN account segment is not 12 digits".into());
    }
    Ok((partition, region, account, resource))
}

fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.len() > MAX_NAME_BYTES
        || name.contains(':')
        || name
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace())
    {
        return Err("Step Functions resource name is invalid".into());
    }
    Ok(())
}

pub(super) fn is_in_scope(parts: &ArnRef<'_>, ctx: &Context) -> bool {
    parts.partition == ctx.partition && parts.region == ctx.region && parts.account == ctx.account
}

pub(super) fn role_target(arn: &str, ctx: &Context) -> Option<String> {
    let mut parts = arn.splitn(6, ':');
    let scheme = parts.next();
    let partition = parts.next();
    let service = parts.next();
    let region = parts.next();
    let account = parts.next();
    let resource = parts.next();
    let (Some("arn"), Some(partition), Some("iam"), Some(""), Some(account), Some(resource)) =
        (scheme, partition, service, region, account, resource)
    else {
        return None;
    };
    if partition != ctx.partition || account != ctx.account || arn.len() > MAX_ARN_BYTES {
        return None;
    }
    let name = resource.strip_prefix("role/")?;
    if name.is_empty() || name.contains('\0') || name.bytes().any(|byte| byte == b' ') {
        return None;
    }
    Some(arn.to_owned())
}

pub(super) fn kms_target(value: &str, ctx: &Context) -> Option<String> {
    let mut parts = value.splitn(6, ':');
    let scheme = parts.next();
    let partition = parts.next();
    let service = parts.next();
    let region = parts.next();
    let account = parts.next();
    let resource = parts.next();
    let (Some("arn"), Some(partition), Some("kms"), Some(region), Some(account), Some(resource)) =
        (scheme, partition, service, region, account, resource)
    else {
        return None;
    };
    if partition != ctx.partition || region != ctx.region || account != ctx.account {
        return None;
    }
    let key_id = resource.strip_prefix("key/")?;
    validate_key_id(key_id).then(|| value.to_owned())
}

fn validate_key_id(key_id: &str) -> bool {
    if key_id.len() == KEY_ID_BYTES && is_uuid(key_id.as_bytes()) {
        return true;
    }
    key_id
        .strip_prefix(MULTI_REGION_PREFIX)
        .is_some_and(|value| {
            value.len() == MULTI_REGION_ID_BYTES
                && value.bytes().all(|byte| byte.is_ascii_hexdigit())
        })
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
