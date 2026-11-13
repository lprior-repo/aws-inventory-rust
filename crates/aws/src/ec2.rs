use std::collections::BTreeMap;

use aws_sdk_ec2::{
    Client,
    types::{Instance, IpPermission, Tag},
};
use inventory_core::{Resource, Source};
use serde_json::{Value, json};

use crate::{
    AwsScanError,
    common::{Context, PageBody, paginate, require_budget, required, sdk_error},
};

#[tracing::instrument(skip_all)]
pub(crate) async fn collect(ctx: &Context) -> Result<(), AwsScanError> {
    let client = Client::new(&ctx.config);
    tokio::try_join!(
        instances(ctx, &client),
        vpcs(ctx, &client),
        subnets(ctx, &client),
        security_groups(ctx, &client),
        interfaces(ctx, &client),
        volumes(ctx, &client)
    )?;
    Ok(())
}

fn tags(values: &[Tag]) -> BTreeMap<String, String> {
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

fn resource(
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

fn link(
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

fn map_instance(
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

#[tracing::instrument(skip_all)]
async fn instances(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    paginate(
        ctx,
        ctx.scope("aws_ec2_instance", None, Source::Direct),
        |token, budget| async move {
            let output = ctx
                .execute("ec2", "read", || async {
                    client
                        .describe_instances()
                        .max_results(ctx.page_size.clamp(5, 1_000))
                        .set_next_token(token.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            let count = output
                .reservations()
                .iter()
                .fold(0usize, |count, reservation| {
                    count.saturating_add(reservation.instances().len())
                });
            require_budget(count, budget)?;
            let mut body = PageBody {
                next_token: output.next_token().map(str::to_owned),
                ..PageBody::default()
            };
            for reservation in output.reservations() {
                for instance in reservation.instances() {
                    map_instance(ctx, &mut body, instance, reservation.owner_id())?;
                }
            }
            body.listed_count = body.resources.len();
            Ok(body)
        },
    )
    .await
}

#[tracing::instrument(skip_all)]
async fn vpcs(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    paginate(ctx, ctx.scope("aws_vpc", None, Source::Direct), |token, budget| async move {
        let output = ctx.execute("ec2", "read", || async {
            client.describe_vpcs().max_results(ctx.page_size.clamp(5, 1_000))
                .set_next_token(token.clone()).send().await.map_err(sdk_error)
        }).await?;
        require_budget(output.vpcs().len(), budget)?;
        let resources = output.vpcs().iter().map(|vpc| resource(ctx, "aws_vpc", "vpc", vpc.vpc_id(), json!({
            "vpc_id": vpc.vpc_id(), "cidr_block": vpc.cidr_block(),
            "state": vpc.state().map(|value| value.as_str()), "is_default": vpc.is_default(),
            "dhcp_options_id": vpc.dhcp_options_id(), "owner_id": vpc.owner_id(),
            "instance_tenancy": vpc.instance_tenancy().map(|value| value.as_str()),
        }), vpc.tags())).collect::<Result<Vec<_>, _>>()?;
        Ok(PageBody { listed_count: resources.len(), resources, next_token: output.next_token().map(str::to_owned), ..PageBody::default() })
    }).await
}

#[tracing::instrument(skip_all)]
async fn subnets(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    paginate(ctx, ctx.scope("aws_vpc_subnet", None, Source::Direct), |token, budget| async move {
        let output = ctx.execute("ec2", "read", || async {
            client.describe_subnets().max_results(ctx.page_size.clamp(5, 1_000))
                .set_next_token(token.clone()).send().await.map_err(sdk_error)
        }).await?;
        require_budget(output.subnets().len(), budget)?;
        let mut body = PageBody { next_token: output.next_token().map(str::to_owned), listed_count: output.subnets().len(), ..PageBody::default() };
        for subnet in output.subnets() {
            let resource = resource(ctx, "aws_vpc_subnet", "subnet", subnet.subnet_id(), json!({
                "subnet_id": subnet.subnet_id(), "vpc_id": subnet.vpc_id(), "cidr_block": subnet.cidr_block(), "arn":subnet.subnet_arn(),
                "availability_zone": subnet.availability_zone(), "state": subnet.state().map(|value| value.as_str()),
                "available_ip_address_count": subnet.available_ip_address_count(),
                "map_public_ip_on_launch": subnet.map_public_ip_on_launch(),
                "default_for_az": subnet.default_for_az(), "owner_id": subnet.owner_id(),
            }), subnet.tags())?;
            link(ctx, &mut body, &resource, subnet.vpc_id(), "vpc", "IN_VPC");
            body.resources.push(resource);
        }
        Ok(body)
    }).await
}

fn permissions(values: &[IpPermission]) -> Vec<Value> {
    values.iter().map(|permission| json!({
        "ip_protocol": permission.ip_protocol(), "from_port": permission.from_port(), "to_port": permission.to_port(),
        "ipv4_ranges": permission.ip_ranges().iter().map(|range| json!({"cidr_ip":range.cidr_ip(),"description":range.description()})).collect::<Vec<_>>(),
        "ipv6_ranges": permission.ipv6_ranges().iter().map(|range| json!({"cidr_ipv6":range.cidr_ipv6(),"description":range.description()})).collect::<Vec<_>>(),
        "security_groups": permission.user_id_group_pairs().iter().map(|group| json!({"group_id":group.group_id(),"user_id":group.user_id(),"vpc_id":group.vpc_id()})).collect::<Vec<_>>(),
        "prefix_list_ids": permission.prefix_list_ids().iter().filter_map(|list| list.prefix_list_id()).collect::<Vec<_>>(),
    })).collect()
}

#[tracing::instrument(skip_all)]
async fn security_groups(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    paginate(ctx, ctx.scope("aws_vpc_security_group", None, Source::Direct), |token, budget| async move {
        let output = ctx.execute("ec2", "read", || async {
            client.describe_security_groups().max_results(ctx.page_size.clamp(5, 1_000))
                .set_next_token(token.clone()).send().await.map_err(sdk_error)
        }).await?;
        require_budget(output.security_groups().len(), budget)?;
        let mut body = PageBody { next_token: output.next_token().map(str::to_owned), listed_count: output.security_groups().len(), ..PageBody::default() };
        for group in output.security_groups() {
            let mut resource = resource(ctx, "aws_vpc_security_group", "security-group", group.group_id(), json!({
                "group_id": group.group_id(), "group_name": group.group_name(), "vpc_id": group.vpc_id(), "arn":group.security_group_arn(),
                "description": group.description(), "owner_id": group.owner_id(),
                "ip_permissions": permissions(group.ip_permissions()), "ip_permissions_egress": permissions(group.ip_permissions_egress()),
            }), group.tags())?;
            if resource.name.is_none() { resource.name = group.group_name().map(str::to_owned); }
            link(ctx, &mut body, &resource, group.vpc_id(), "vpc", "IN_VPC");
            body.resources.push(resource);
        }
        Ok(body)
    }).await
}

#[tracing::instrument(skip_all)]
async fn interfaces(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    paginate(ctx, ctx.scope("aws_ec2_network_interface", None, Source::Direct), |token, budget| async move {
        let output = ctx.execute("ec2", "read", || async {
            client.describe_network_interfaces().max_results(ctx.page_size.clamp(5, 1_000))
                .set_next_token(token.clone()).send().await.map_err(sdk_error)
        }).await?;
        require_budget(output.network_interfaces().len(), budget)?;
        let mut body = PageBody { next_token: output.next_token().map(str::to_owned), listed_count: output.network_interfaces().len(), ..PageBody::default() };
        for interface in output.network_interfaces() {
            let resource = resource(ctx, "aws_ec2_network_interface", "network-interface", interface.network_interface_id(), json!({
                "network_interface_id": interface.network_interface_id(), "vpc_id": interface.vpc_id(), "subnet_id": interface.subnet_id(), "owner_id":interface.owner_id(),
                "status": interface.status().map(|value| value.as_str()), "private_ip_address": interface.private_ip_address(),
                "interface_type": interface.interface_type().map(|value| value.as_str()), "description": interface.description(),
                "attachment": interface.attachment().map(|attachment| json!({"instance_id":attachment.instance_id(),"attachment_id":attachment.attachment_id(),"device_index":attachment.device_index()})),
                "security_group_ids": interface.groups().iter().filter_map(|group| group.group_id()).collect::<Vec<_>>(),
            }), interface.tag_set())?;
            link(ctx, &mut body, &resource, interface.vpc_id(), "vpc", "IN_VPC");
            link(ctx, &mut body, &resource, interface.subnet_id(), "subnet", "IN_SUBNET");
            link(ctx, &mut body, &resource, interface.attachment().and_then(|attachment| attachment.instance_id()), "instance", "ATTACHED_TO");
            for group in interface.groups() { link(ctx, &mut body, &resource, group.group_id(), "security-group", "USES_SECURITY_GROUP"); }
            body.resources.push(resource);
        }
        Ok(body)
    }).await
}

#[tracing::instrument(skip_all)]
async fn volumes(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    paginate(ctx, ctx.scope("aws_ebs_volume", None, Source::Direct), |token, budget| async move {
        let output = ctx.execute("ec2", "read", || async {
            client.describe_volumes().max_results(ctx.page_size.clamp(5, 500))
                .set_next_token(token.clone()).send().await.map_err(sdk_error)
        }).await?;
        require_budget(output.volumes().len(), budget)?;
        let mut body = PageBody { next_token: output.next_token().map(str::to_owned), listed_count: output.volumes().len(), ..PageBody::default() };
        for volume in output.volumes() {
            let resource = resource(ctx, "aws_ebs_volume", "volume", volume.volume_id(), json!({
                "volume_id": volume.volume_id(), "size": volume.size(), "state": volume.state().map(|value| value.as_str()),
                "volume_type": volume.volume_type().map(|value| value.as_str()), "encrypted": volume.encrypted(),
                "availability_zone": volume.availability_zone(), "iops": volume.iops(), "throughput": volume.throughput(),
                "kms_key_id": volume.kms_key_id(), "snapshot_id": volume.snapshot_id(),
                "attachments": volume.attachments().iter().map(|attachment| json!({"instance_id":attachment.instance_id(),"device":attachment.device(),"state":attachment.state().map(|value|value.as_str())})).collect::<Vec<_>>(),
            }), volume.tags())?;
            for attachment in volume.attachments() { link(ctx, &mut body, &resource, attachment.instance_id(), "instance", "ATTACHED_TO"); }
            body.resources.push(resource);
        }
        Ok(body)
    }).await
}

#[cfg(test)]
mod tests {
    use super::{Context, PageBody, link, resource};
    use inventory_engine::limiter::{ApiExecutor, ExecutorConfig};
    use serde_json::json;
    use std::sync::Arc;

    #[test]
    fn shared_subnet_uses_reported_owner_and_vpc_owner() -> Result<(), Box<dyn std::error::Error>> {
        let (sender, _receiver) = tokio::sync::mpsc::channel(1);
        let ctx = Context {
            config: aws_types::SdkConfig::builder().build(),
            executor: Arc::new(ApiExecutor::new(ExecutorConfig::default())?),
            sender,
            account: "111111111111".into(),
            partition: "aws".into(),
            region: "us-east-1".into(),
            max_pages: 1,
            page_size: 100,
        };
        let subnet = resource(
            &ctx,
            "aws_vpc_subnet",
            "subnet",
            Some("subnet-bbb"),
            json!({
                "owner_id":"222222222222", "arn":"arn:aws:ec2:us-east-1:222222222222:subnet/subnet-bbb"
            }),
            &[],
        )?;
        assert_eq!(subnet.account_id, "222222222222");
        assert_eq!(
            subnet.id,
            "arn:aws:ec2:us-east-1:222222222222:subnet/subnet-bbb"
        );
        let mut body = PageBody::default();
        link(&ctx, &mut body, &subnet, Some("vpc-aaa"), "vpc", "IN_VPC");
        let edge = body.edges.first().ok_or("missing subnet VPC edge")?;
        assert_eq!(edge.to_id, "arn:aws:ec2:us-east-1:222222222222:vpc/vpc-aaa");
        assert_eq!(
            edge.properties
                .get("owner_resolution")
                .and_then(serde_json::Value::as_str),
            Some("subnet_owner_from_aws_response")
        );
        let instance = resource(
            &ctx,
            "aws_ec2_instance",
            "instance",
            Some("i-abc"),
            json!({}),
            &[],
        )?;
        let unknown_owner = ctx.edge(&instance, ctx.ec2_arn("subnet", "subnet-bbb"), "IN_SUBNET");
        assert_eq!(
            unknown_owner
                .properties
                .get("owner_resolution")
                .and_then(serde_json::Value::as_str),
            Some("assumed_scanning_account")
        );
        assert_eq!(
            unknown_owner
                .properties
                .get("target_resource_id")
                .and_then(serde_json::Value::as_str),
            Some("subnet-bbb")
        );
        Ok(())
    }
}
