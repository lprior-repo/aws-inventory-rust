//! evidence operations.
use super::*;

pub(super) fn network_reference_evidence(target: &str) -> Value {
    if target.split(':').nth(2) != Some("ec2") {
        return json!({});
    }
    let Some((kind, id)) = target
        .splitn(6, ':')
        .nth(5)
        .and_then(|value| value.split_once('/'))
    else {
        return json!({});
    };
    let target_kind = match kind {
        "vpc" => "aws_vpc",
        "subnet" => "aws_vpc_subnet",
        "security-group" => "aws_vpc_security_group",
        "network-interface" => "aws_ec2_network_interface",
        _ => return json!({}),
    };
    json!({"target_resource_id":id,"target_kind":target_kind,
        "target_region":target.split(':').nth(3),
        "owner_resolution":"assumed_scanning_account"})
}

pub(crate) fn now_millis() -> Result<i64, AwsScanError> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| AwsScanError::InvalidData(error.to_string()))?;
    i64::try_from(elapsed.as_millis()).map_err(|error| AwsScanError::InvalidData(error.to_string()))
}

pub(crate) fn required(value: Option<&str>, field: &str) -> Result<String, AwsScanError> {
    value
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| AwsScanError::InvalidData(format!("AWS response lacks {field}")))
}

pub(crate) fn require_budget(count: usize, remaining: usize) -> Result<(), AwsScanError> {
    if count > remaining {
        return Err(AwsScanError::ChildLimit);
    }
    if count > inventory_core::MAX_PAGE_RESOURCES {
        return Err(AwsScanError::InvalidData(
            "service page exceeds the maximum page resource count".into(),
        ));
    }
    Ok(())
}
