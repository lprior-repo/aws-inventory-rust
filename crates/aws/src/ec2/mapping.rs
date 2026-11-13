//! mapping operations.
use super::*;

pub(super) fn tags(values: &[Tag]) -> BTreeMap<String, String> {
    values
        .iter()
        .filter_map(|tag| {
            Some((
                tag.key()?.to_owned(),
                tag.value().unwrap_or_default().to_owned(),
            ))
        })
        .collect()
}

pub(super) fn resource(
    ctx: &Context,
    kind: &str,
    arn_type: &str,
    id: Option<&str>,
    properties: Value,
    values: &[Tag],
) -> Result<Resource, AwsScanError> {
    let id = required(id, kind)?;
    let tags = tags(values);
    let owner = properties
        .get("owner_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or(&ctx.account)
        .to_owned();
    let arn = properties
        .get("arn")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| {
            format!(
                "arn:{}:ec2:{}:{}:{}/{}",
                ctx.partition, ctx.region, owner, arn_type, id
            )
        });
    let mut result = ctx.resource(kind, arn, tags.get("Name").cloned(), properties, tags)?;
    result.account_id = result
        .id
        .split(':')
        .nth(4)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or(owner);
    Ok(result)
}

pub(super) fn link(
    ctx: &Context,
    body: &mut PageBody,
    resource: &Resource,
    target: Option<&str>,
    kind: &str,
    relation: &str,
) {
    if let Some(id) = target.filter(|value| !value.is_empty()) {
        if resource.kind == "aws_vpc_subnet" && kind == "vpc" {
            let target = format!(
                "arn:{}:ec2:{}:{}:vpc/{}",
                ctx.partition, ctx.region, resource.account_id, id
            );
            let mut edge = ctx.edge(resource, target, relation);
            if let Some(properties) = edge.properties.as_object_mut() {
                properties.insert(
                    "owner_resolution".to_owned(),
                    json!("subnet_owner_from_aws_response"),
                );
            }
            body.edges.push(edge);
        } else {
            body.edges
                .push(ctx.edge(resource, ctx.ec2_arn(kind, id), relation));
        }
    }
}

pub(super) fn map_instance(
    ctx: &Context,
    body: &mut PageBody,
    instance: &Instance,
    owner: Option<&str>,
) -> Result<(), AwsScanError> {
    let resource = resource(
        ctx,
        "aws_ec2_instance",
        "instance",
        instance.instance_id(),
        json!({
            "instance_id": instance.instance_id(), "vpc_id": instance.vpc_id(), "subnet_id": instance.subnet_id(), "owner_id":owner,
            "instance_type": instance.instance_type().map(|value| value.as_str()),
            "state": instance.state().and_then(|state| state.name()).map(|value| value.as_str()),
            "private_ip_address": instance.private_ip_address(), "public_ip_address": instance.public_ip_address(),
            "image_id": instance.image_id(), "architecture": instance.architecture().map(|value| value.as_str()),
            "availability_zone": instance.placement().and_then(|placement| placement.availability_zone()),
            "security_group_ids": instance.security_groups().iter().filter_map(|group| group.group_id()).collect::<Vec<_>>(),
            "iam_instance_profile_arn": instance.iam_instance_profile().and_then(|profile| profile.arn()),
        }),
        instance.tags(),
    )?;
    link(ctx, body, &resource, instance.vpc_id(), "vpc", "IN_VPC");
    link(
        ctx,
        body,
        &resource,
        instance.subnet_id(),
        "subnet",
        "IN_SUBNET",
    );
    for group in instance.security_groups() {
        link(
            ctx,
            body,
            &resource,
            group.group_id(),
            "security-group",
            "USES_SECURITY_GROUP",
        );
    }
    if let Some(arn) = instance
        .iam_instance_profile()
        .and_then(|profile| profile.arn())
    {
        body.edges
            .push(ctx.edge(&resource, arn.to_owned(), "USES_INSTANCE_PROFILE"));
    }
    body.resources.push(resource);
    Ok(())
}
