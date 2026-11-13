//! resource mapping operations.
use super::*;

pub(super) fn resource(
    ctx: &Context,
    kind: TargetKind,
    id: Option<&str>,
    owner: Option<&str>,
    values: &[Tag],
    mut properties: Value,
) -> Result<Resource, AwsScanError> {
    let id = required(id, kind.resource_kind)?;
    let owner = owner.filter(|owner| !owner.is_empty());
    let identity_owner = owner.unwrap_or(&ctx.account);
    let tags: BTreeMap<String, String> = values
        .iter()
        .filter_map(|tag| {
            Some((
                tag.key()?.to_owned(),
                tag.value().unwrap_or_default().to_owned(),
            ))
        })
        .collect();
    let object = properties.as_object_mut().ok_or_else(|| {
        AwsScanError::InvalidData("network resource properties are not an object".into())
    })?;
    object.insert("owner_id".into(), json!(owner));
    object.insert(
        "owner_resolution".into(),
        json!(if owner.is_some() {
            "aws_response"
        } else {
            "assumed_scanning_account"
        }),
    );
    let arn = format!(
        "arn:{}:ec2:{}:{identity_owner}:{}/{id}",
        ctx.partition, ctx.region, kind.arn_type
    );
    let mut resource = ctx.resource(
        kind.resource_kind,
        arn,
        tags.get("Name").cloned(),
        properties,
        tags,
    )?;
    resource.account_id = identity_owner.to_owned();
    resource.validate()?;
    Ok(resource)
}

pub(super) fn millis(value: Option<&DateTime>) -> Result<Option<i64>, AwsScanError> {
    value
        .map(|value| value.to_millis())
        .transpose()
        .map_err(|error| AwsScanError::InvalidData(format!("network timestamp: {error}")))
}
