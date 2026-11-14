use super::*;

pub(super) fn checked_text(value: &str, what: &'static str) -> Result<String, AwsScanError> {
    inventory_core::validate_identifier(value, what)?;
    Ok(value.to_owned())
}

pub(super) fn own_arn(
    ctx: &Context,
    arn: &str,
    service: &str,
    kind: &str,
) -> Result<String, AwsScanError> {
    inventory_core::validate_identifier(arn, "global resource ARN")?;
    let prefix = format!("arn:{}:{service}::{}:{kind}/", ctx.partition, ctx.account);
    if !arn
        .strip_prefix(&prefix)
        .is_some_and(|suffix| !suffix.is_empty())
    {
        return Err(AwsScanError::InvalidData(format!(
            "unexpected {service} {kind} resource ARN"
        )));
    }
    Ok(arn.to_owned())
}

pub(super) fn reference(
    ctx: &Context,
    source: &Resource,
    target: &str,
    relation: &str,
    evidence: Value,
) -> Result<Edge, AwsScanError> {
    inventory_core::validate_identifier(target, "global reference target")?;
    let mut edge = ctx.edge(source, target.to_owned(), relation);
    let properties = edge.properties.as_object_mut().ok_or_else(|| {
        AwsScanError::InvalidData("reference properties are not an object".into())
    })?;
    let evidence = evidence
        .as_object()
        .ok_or_else(|| AwsScanError::InvalidData("reference evidence is not an object".into()))?;
    properties.extend(evidence.clone());
    if !properties.contains_key("owner_resolution")
        && let Some(owner) = properties.get("target_owner_resolution").cloned()
    {
        properties.insert("owner_resolution".to_owned(), owner);
    }
    edge.validate()?;
    Ok(edge)
}

pub(super) fn digest_identity(namespace: &str, fields: &[&str]) -> Result<String, AwsScanError> {
    if fields.is_empty() || fields.len() > 16 {
        return Err(AwsScanError::InvalidData(
            "compound identity requires 1 to 16 components".into(),
        ));
    }
    let mut hash = Sha256::new();
    for field in fields {
        inventory_core::validate_identifier(field, "identity component")?;
        hash.update(
            u64::try_from(field.len())
                .map_err(|_| AwsScanError::InvalidData("identity length overflow".into()))?
                .to_be_bytes(),
        );
        hash.update(field.as_bytes());
    }
    Ok(format!(
        "aws-inventory:{namespace}:{:x}",
        base16ct::HexDisplay(&hash.finalize())
    ))
}

pub(super) fn dns_reference(name: &str) -> Result<String, AwsScanError> {
    let name = name.trim_end_matches('.').to_ascii_lowercase();
    if name.is_empty()
        || name.len() > 1_024
        || name.contains(['/', '@', ':'])
        || name
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err(AwsScanError::InvalidData(
            "invalid DNS reference name".into(),
        ));
    }
    Ok(format!("dns:{name}"))
}
