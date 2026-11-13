//! gateways operations.
use super::*;

pub(super) fn map_nat_gateway(
    ctx: &Context,
    page: &mut NetworkPage,
    gateway: &NatGateway,
) -> Result<(), AwsScanError> {
    page.components(&[
        gateway.nat_gateway_addresses().len(),
        gateway.attached_appliances().len(),
        gateway.tags().len(),
    ])?;
    let resource = resource(
        ctx,
        NAT,
        gateway.nat_gateway_id(),
        None,
        gateway.tags(),
        json!({
            "nat_gateway_id": gateway.nat_gateway_id(), "vpc_id": gateway.vpc_id(), "subnet_id": gateway.subnet_id(),
            "state": gateway.state().map(|state| state.as_str()),
            "connectivity_type": gateway.connectivity_type().map(|value| value.as_str()),
            "availability_mode": gateway.availability_mode().map(|value| value.as_str()),
            "auto_scaling_ips": gateway.auto_scaling_ips().map(|value| value.as_str()),
            "auto_provision_zones": gateway.auto_provision_zones().map(|value| value.as_str()),
            "create_time": millis(gateway.create_time())?, "delete_time": millis(gateway.delete_time())?,
            "failure_code": gateway.failure_code(), "failure_message": gateway.failure_message(),
            "route_table_id": gateway.route_table_id(),
            "addresses": gateway.nat_gateway_addresses().iter().map(|address| json!({
                "allocation_id": address.allocation_id(), "network_interface_id": address.network_interface_id(),
                "private_ip": address.private_ip(), "public_ip": address.public_ip(),
                "association_id": address.association_id(), "is_primary": address.is_primary(),
                "status": address.status().map(|state| state.as_str()), "failure_message": address.failure_message(),
                "availability_zone": address.availability_zone(), "availability_zone_id": address.availability_zone_id(),
            })).collect::<Vec<_>>(),
            "attached_appliances": gateway.attached_appliances().iter().map(|appliance| json!({
                "type": appliance.r#type().map(|value| value.as_str()), "appliance_arn": appliance.appliance_arn(),
                "vpc_endpoint_id": appliance.vpc_endpoint_id(), "attachment_state": appliance.attachment_state().map(|value| value.as_str()),
                "modification_state": appliance.modification_state().map(|value| value.as_str()),
                "failure_code": appliance.failure_code(), "failure_message": appliance.failure_message(),
            })).collect::<Vec<_>>(),
        }),
    )?;
    for (kind, id, relation, field) in [
        (VPC, gateway.vpc_id(), "IN_VPC", "vpc_id"),
        (SUBNET, gateway.subnet_id(), "IN_SUBNET", "subnet_id"),
        (
            ROUTE_TABLE,
            gateway.route_table_id().filter(|id| id.starts_with("rtb-")),
            "USES_ROUTE_TABLE",
            "route_table_id",
        ),
    ] {
        page.link(
            ctx,
            &resource,
            Reference::new(kind, id),
            relation,
            json!({"field":field}),
        )?;
    }
    for (index, address) in gateway.nat_gateway_addresses().iter().enumerate() {
        let evidence =
            json!({"address_index":index,"status":address.status().map(|state| state.as_str())});
        page.link(
            ctx,
            &resource,
            Reference::new(ENI, address.network_interface_id()),
            "USES_NETWORK_INTERFACE",
            evidence.clone(),
        )?;
        page.link(
            ctx,
            &resource,
            Reference::new(EIP, address.allocation_id()),
            "USES_ELASTIC_IP",
            evidence,
        )?;
    }
    // Appliance details are retained without inventing a target kind or
    // asserting the appliance is in the data path while its attachment changes.
    page.resources.push(resource);
    Ok(())
}

pub(super) fn map_endpoint(
    ctx: &Context,
    page: &mut NetworkPage,
    endpoint: &VpcEndpoint,
) -> Result<(), AwsScanError> {
    page.components(&[
        endpoint.route_table_ids().len(),
        endpoint.subnet_ids().len(),
        endpoint.groups().len(),
        endpoint.network_interface_ids().len(),
        endpoint.dns_entries().len(),
        endpoint.tags().len(),
    ])?;
    let resource = resource(
        ctx,
        ENDPOINT,
        endpoint.vpc_endpoint_id(),
        endpoint.owner_id(),
        endpoint.tags(),
        json!({
            "vpc_endpoint_id": endpoint.vpc_endpoint_id(), "vpc_endpoint_type": endpoint.vpc_endpoint_type().map(|value| value.as_str()),
            "vpc_id": endpoint.vpc_id(), "service_name": endpoint.service_name(), "service_region": endpoint.service_region(),
            "state": endpoint.state().map(|state| state.as_str()), "private_dns_enabled": endpoint.private_dns_enabled(),
            "requester_managed": endpoint.requester_managed(), "ip_address_type": endpoint.ip_address_type().map(|value| value.as_str()),
            "route_table_ids": endpoint.route_table_ids(), "subnet_ids": endpoint.subnet_ids(),
            "security_group_ids": endpoint.groups().iter().filter_map(|group| group.group_id()).collect::<Vec<_>>(),
            "security_groups": endpoint.groups().iter().map(|group| json!({"group_id":group.group_id(),"group_name":group.group_name()})).collect::<Vec<_>>(),
            "network_interface_ids": endpoint.network_interface_ids(), "policy_document": endpoint.policy_document(),
            "dns_entries": endpoint.dns_entries().iter().map(|entry| json!({"dns_name":entry.dns_name(),"hosted_zone_id":entry.hosted_zone_id()})).collect::<Vec<_>>(),
            "dns_options": endpoint.dns_options().map(|options| json!({
                "dns_record_ip_type": options.dns_record_ip_type().map(|value| value.as_str()),
                "private_dns_only_for_inbound_resolver_endpoint": options.private_dns_only_for_inbound_resolver_endpoint(),
            })),
            "creation_timestamp": millis(endpoint.creation_timestamp())?,
            "service_network_arn": endpoint.service_network_arn(), "resource_configuration_arn": endpoint.resource_configuration_arn(),
            "failure_reason": endpoint.failure_reason(),
            "last_error": endpoint.last_error().map(|error| json!({"code":error.code(),"message":error.message()})),
        }),
    )?;
    // Endpoint ownership does not establish ownership of a shared VPC/subnet.
    page.link(
        ctx,
        &resource,
        Reference::new(VPC, endpoint.vpc_id()),
        "IN_VPC",
        json!({"field":"vpc_id"}),
    )?;
    for (kind, ids, relation, field) in [
        (
            ROUTE_TABLE,
            endpoint.route_table_ids(),
            "USES_ROUTE_TABLE",
            "route_table_ids",
        ),
        (SUBNET, endpoint.subnet_ids(), "USES_SUBNET", "subnet_ids"),
        (
            ENI,
            endpoint.network_interface_ids(),
            "USES_NETWORK_INTERFACE",
            "network_interface_ids",
        ),
    ] {
        for id in ids {
            page.link(
                ctx,
                &resource,
                Reference::new(kind, Some(id)),
                relation,
                json!({"field":field}),
            )?;
        }
    }
    for group in endpoint.groups() {
        page.link(
            ctx,
            &resource,
            Reference::new(GROUP, group.group_id()),
            "USES_SECURITY_GROUP",
            json!({"field":"groups","group_name":group.group_name()}),
        )?;
    }
    page.arn_link(
        ctx,
        &resource,
        endpoint.service_network_arn(),
        "TARGETS_SERVICE_NETWORK",
        json!({"field":"service_network_arn"}),
    )?;
    page.arn_link(
        ctx,
        &resource,
        endpoint.resource_configuration_arn(),
        "TARGETS_RESOURCE_CONFIGURATION",
        json!({"field":"resource_configuration_arn"}),
    )?;
    page.resources.push(resource);
    Ok(())
}
