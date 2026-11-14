use super::*;

pub(super) fn arn_parts(arn: &str) -> Result<ArnParts<'_>, AwsScanError> {
    validate_identifier(arn, "task-definition related ARN")?;
    let mut parts = arn.splitn(6, ':');
    let (Some("arn"), Some(partition), Some(service), Some(region), Some(account), Some(resource)) = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) else {
        return Err(AwsScanError::InvalidData(
            "task-definition response contains an invalid ARN".into(),
        ));
    };
    if partition.is_empty() || service.is_empty() || account.is_empty() || resource.is_empty() {
        return Err(AwsScanError::InvalidData(
            "task-definition response contains an incomplete ARN".into(),
        ));
    }
    Ok(ArnParts {
        service,
        region,
        account,
        resource,
    })
}

pub(super) fn definition_identity(arn: &str) -> Result<DefinitionIdentity<'_>, AwsScanError> {
    let parts = arn_parts(arn)?;
    let (family, revision) = parts
        .resource
        .strip_prefix("task-definition/")
        .and_then(|value| value.rsplit_once(':'))
        .ok_or_else(|| {
            AwsScanError::InvalidData(
                "task-definition identity lacks family and explicit revision".into(),
            )
        })?;
    let revision = revision.parse::<i32>().map_err(|_| {
        AwsScanError::InvalidData("task-definition revision is not a positive integer".into())
    })?;
    if parts.service != "ecs"
        || parts.region.is_empty()
        || family.is_empty()
        || family.contains(':')
        || revision <= 0
    {
        return Err(AwsScanError::InvalidData(
            "invalid ECS task-definition revision ARN".into(),
        ));
    }
    Ok(DefinitionIdentity {
        account: parts.account,
        region: parts.region,
        family,
        revision,
    })
}

pub(super) fn consume(budget: &mut usize, count: usize) -> Result<(), AwsScanError> {
    *budget = budget.checked_sub(count).ok_or_else(|| {
        AwsScanError::InvalidData("task-definition mapped component capacity exceeded".into())
    })?;
    Ok(())
}

pub(super) fn millis(value: Option<&DateTime>) -> Result<Option<i64>, AwsScanError> {
    value
        .map(|value| value.to_millis())
        .transpose()
        .map_err(|error| AwsScanError::InvalidData(format!("task-definition timestamp: {error}")))
}

pub(super) fn option_keys(values: Option<&HashMap<String, String>>) -> Vec<&str> {
    let mut keys: Vec<_> = values
        .into_iter()
        .flat_map(|values| values.keys().map(String::as_str))
        .collect();
    keys.sort_unstable();
    keys
}

pub(super) fn secret_references(values: &[Secret]) -> Vec<Value> {
    values
        .iter()
        .map(|value| json!({"name":value.name(),"value_from":value.value_from()}))
        .collect()
}
