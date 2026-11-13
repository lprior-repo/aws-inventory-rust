//! access operations.
use super::*;

pub(super) fn map_network_acl(
    ctx: &Context,
    page: &mut NetworkPage,
    acl: &NetworkAcl,
) -> Result<(), AwsScanError> {
    page.components(&[
        acl.entries().len(),
        acl.associations().len(),
        acl.tags().len(),
    ])?;
    let resource = resource(
        ctx,
        ACL,
        acl.network_acl_id(),
        acl.owner_id(),
        acl.tags(),
        json!({
            "network_acl_id": acl.network_acl_id(), "vpc_id": acl.vpc_id(), "is_default": acl.is_default(),
            "entries": acl.entries().iter().map(|entry| json!({
                "rule_number": entry.rule_number(), "egress": entry.egress(), "protocol": entry.protocol(),
                "rule_action": entry.rule_action().map(|action| action.as_str()),
                "cidr_block": entry.cidr_block(), "ipv6_cidr_block": entry.ipv6_cidr_block(),
                "port_range": entry.port_range().map(|range| json!({"from":range.from(),"to":range.to()})),
                "icmp_type_code": entry.icmp_type_code().map(|value| json!({"type":value.r#type(),"code":value.code()})),
            })).collect::<Vec<_>>(),
            "associations": acl.associations().iter().map(|association| json!({
                "network_acl_association_id": association.network_acl_association_id(),
                "network_acl_id": association.network_acl_id(), "subnet_id": association.subnet_id(),
            })).collect::<Vec<_>>(),
        }),
    )?;
    let vpc = Reference::new(VPC, acl.vpc_id())
        .map(|reference| reference.owner(acl.owner_id(), "same_vpc_owner_from_response"));
    page.link(ctx, &resource, vpc, "IN_VPC", json!({"field":"vpc_id"}))?;
    for association in acl.associations() {
        let subnet = Reference::new(SUBNET, association.subnet_id())
            .map(|reference| reference.owner(acl.owner_id(), "same_vpc_owner_from_response"));
        page.link(ctx, &resource, subnet, "ASSOCIATED_WITH_SUBNET", json!({
            "network_acl_association_id": association.network_acl_association_id(), "network_acl_id":association.network_acl_id(),
        }))?;
    }
    page.resources.push(resource);
    Ok(())
}

pub(super) fn peer_json(info: &VpcPeeringConnectionVpcInfo) -> Value {
    json!({
        "vpc_id": info.vpc_id(), "owner_id": info.owner_id(), "region": info.region(), "cidr_block": info.cidr_block(),
        "cidr_block_set": info.cidr_block_set().iter().filter_map(|block| block.cidr_block()).collect::<Vec<_>>(),
        "ipv6_cidr_block_set": info.ipv6_cidr_block_set().iter().filter_map(|block| block.ipv6_cidr_block()).collect::<Vec<_>>(),
        "peering_options": info.peering_options().map(|options| json!({
            "allow_dns_resolution_from_remote_vpc": options.allow_dns_resolution_from_remote_vpc(),
            "allow_egress_from_local_classic_link_to_remote_vpc": options.allow_egress_from_local_classic_link_to_remote_vpc(),
            "allow_egress_from_local_vpc_to_remote_classic_link": options.allow_egress_from_local_vpc_to_remote_classic_link(),
        })),
    })
}

pub(super) fn map_peering(
    ctx: &Context,
    page: &mut NetworkPage,
    peering: &VpcPeeringConnection,
) -> Result<(), AwsScanError> {
    let requester = peering.requester_vpc_info();
    let accepter = peering.accepter_vpc_info();
    page.components(&[
        peering.tags().len(),
        requester.map_or(0, |info| info.cidr_block_set().len()),
        requester.map_or(0, |info| info.ipv6_cidr_block_set().len()),
        accepter.map_or(0, |info| info.cidr_block_set().len()),
        accepter.map_or(0, |info| info.ipv6_cidr_block_set().len()),
    ])?;
    // DescribeVpcPeeringConnections returns ownership for each participant,
    // but no authoritative owner/ARN for the connection observation itself.
    let resource = resource(
        ctx,
        PEERING,
        peering.vpc_peering_connection_id(),
        None,
        peering.tags(),
        json!({
            "vpc_peering_connection_id": peering.vpc_peering_connection_id(),
            "identity_scope": "scanning_account_and_region",
            "status_code": peering.status().and_then(|status| status.code()).map(|code| code.as_str()),
            "status_message": peering.status().and_then(|status| status.message()),
            "expiration_time": millis(peering.expiration_time())?,
            "requester_vpc_id": requester.and_then(|info| info.vpc_id()),
            "requester_owner_id": requester.and_then(|info| info.owner_id()),
            "requester_region": requester.and_then(|info| info.region()),
            "accepter_vpc_id": accepter.and_then(|info| info.vpc_id()),
            "accepter_owner_id": accepter.and_then(|info| info.owner_id()),
            "accepter_region": accepter.and_then(|info| info.region()),
            "requester": requester.map(peer_json), "accepter": accepter.map(peer_json),
        }),
    )?;
    for (info, relation, role) in [
        (requester, "REQUESTER_VPC", "requester"),
        (accepter, "ACCEPTER_VPC", "accepter"),
    ] {
        if let Some(info) = info {
            let reference = Reference::new(VPC, info.vpc_id()).map(|reference| {
                reference
                    .owner(info.owner_id(), "aws_response")
                    .region(info.region())
            });
            page.link(ctx, &resource, reference, relation, json!({"participant":role,"status_code":peering.status().and_then(|status| status.code()).map(|code| code.as_str())}))?;
        }
    }
    page.resources.push(resource);
    Ok(())
}
