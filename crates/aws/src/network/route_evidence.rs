//! route evidence operations.
use super::*;

pub(super) fn route_json(route: &Route) -> Value {
    json!({
        "destination_cidr_block": route.destination_cidr_block(),
        "destination_ipv6_cidr_block": route.destination_ipv6_cidr_block(),
        "destination_prefix_list_id": route.destination_prefix_list_id(),
        "gateway_id": route.gateway_id(),
        "egress_only_internet_gateway_id": route.egress_only_internet_gateway_id(),
        "nat_gateway_id": route.nat_gateway_id(),
        "transit_gateway_id": route.transit_gateway_id(),
        "local_gateway_id": route.local_gateway_id(),
        "carrier_gateway_id": route.carrier_gateway_id(),
        "instance_id": route.instance_id(),
        "instance_owner_id": route.instance_owner_id(),
        "network_interface_id": route.network_interface_id(),
        "vpc_peering_connection_id": route.vpc_peering_connection_id(),
        "core_network_arn": route.core_network_arn(),
        "odb_network_arn": route.odb_network_arn(),
        "ip_address": route.ip_address(),
        "state": route.state().map(|value| value.as_str()),
        "origin": route.origin().map(|value| value.as_str()),
    })
}

pub(super) fn gateway_reference<'a>(
    id: Option<&'a str>,
    igw_owner: Option<&'a str>,
) -> Option<Reference<'a>> {
    let id = id?;
    if id.starts_with("igw-") {
        return Reference::new(IGW, Some(id))
            .map(|reference| reference.owner(igw_owner, "same_vpc_owner_from_response"));
    }
    let kind = if id.starts_with("vgw-") {
        VGW
    } else if id.starts_with("vpce-") {
        ENDPOINT
    } else {
        return None;
    };
    Reference::new(kind, Some(id))
}

pub(super) fn route_evidence(route: &Route, index: usize, field: &str) -> Value {
    json!({
        "route_index": index, "target_field": field,
        "destination_cidr_block": route.destination_cidr_block(),
        "destination_ipv6_cidr_block": route.destination_ipv6_cidr_block(),
        "destination_prefix_list_id": route.destination_prefix_list_id(),
        "state": route.state().map(|value| value.as_str()),
        "origin": route.origin().map(|value| value.as_str()),
    })
}

pub(super) fn route_links(
    ctx: &Context,
    page: &mut NetworkPage,
    resource: &Resource,
    route: &Route,
    index: usize,
    table_owner: Option<&str>,
) -> Result<(), AwsScanError> {
    // The literal "local" and unknown gateway types remain in routes JSON.
    // Neither is assigned an invented gateway ARN.
    if let Some(reference) = gateway_reference(route.gateway_id(), table_owner) {
        page.link(
            ctx,
            resource,
            Some(reference),
            "ROUTES_TO",
            route_evidence(route, index, "gateway_id"),
        )?;
    }
    let targets = [
        (
            EGRESS_IGW,
            route.egress_only_internet_gateway_id(),
            "egress_only_internet_gateway_id",
        ),
        (NAT, route.nat_gateway_id(), "nat_gateway_id"),
        (
            TRANSIT_GATEWAY,
            route.transit_gateway_id(),
            "transit_gateway_id",
        ),
        (LOCAL_GATEWAY, route.local_gateway_id(), "local_gateway_id"),
        (
            CARRIER_GATEWAY,
            route.carrier_gateway_id(),
            "carrier_gateway_id",
        ),
        (ENI, route.network_interface_id(), "network_interface_id"),
        (
            PEERING,
            route.vpc_peering_connection_id(),
            "vpc_peering_connection_id",
        ),
    ];
    for (kind, id, field) in targets {
        if let Some(reference) = Reference::new(kind, id) {
            page.link(
                ctx,
                resource,
                Some(reference),
                "ROUTES_TO",
                route_evidence(route, index, field),
            )?;
        }
    }
    if let Some(reference) = Reference::new(INSTANCE, route.instance_id()) {
        page.link(
            ctx,
            resource,
            Some(reference.owner(
                route.instance_owner_id(),
                "instance_owner_from_aws_response",
            )),
            "ROUTES_TO",
            route_evidence(route, index, "instance_id"),
        )?;
    }
    for (arn, field) in [
        (route.core_network_arn(), "core_network_arn"),
        (route.odb_network_arn(), "odb_network_arn"),
    ] {
        if arn.is_some() {
            page.arn_link(
                ctx,
                resource,
                arn,
                "ROUTES_TO",
                route_evidence(route, index, field),
            )?;
        }
    }
    Ok(())
}
