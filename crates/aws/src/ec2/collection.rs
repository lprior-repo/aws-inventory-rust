//! collection operations.
use super::*;

#[tracing::instrument(skip_all)]
pub(super) async fn instances(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
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
pub(super) async fn vpcs(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
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
pub(super) async fn subnets(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
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

pub(super) fn permissions(values: &[IpPermission]) -> Vec<Value> {
    values.iter().map(|permission| json!({
        "ip_protocol": permission.ip_protocol(), "from_port": permission.from_port(), "to_port": permission.to_port(),
        "ipv4_ranges": permission.ip_ranges().iter().map(|range| json!({"cidr_ip":range.cidr_ip(),"description":range.description()})).collect::<Vec<_>>(),
        "ipv6_ranges": permission.ipv6_ranges().iter().map(|range| json!({"cidr_ipv6":range.cidr_ipv6(),"description":range.description()})).collect::<Vec<_>>(),
        "security_groups": permission.user_id_group_pairs().iter().map(|group| json!({"group_id":group.group_id(),"user_id":group.user_id(),"vpc_id":group.vpc_id()})).collect::<Vec<_>>(),
        "prefix_list_ids": permission.prefix_list_ids().iter().filter_map(|list| list.prefix_list_id()).collect::<Vec<_>>(),
    })).collect()
}

#[tracing::instrument(skip_all)]
pub(super) async fn security_groups(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
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
pub(super) async fn interfaces(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
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
pub(super) async fn volumes(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
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
