//! routing operations.
use super::*;

pub(super) fn map_route_table(
    ctx: &Context,
    page: &mut NetworkPage,
    table: &RouteTable,
) -> Result<(), AwsScanError> {
    page.components(&[
        table.routes().len(),
        table.associations().len(),
        table.propagating_vgws().len(),
        table.tags().len(),
    ])?;
    let associations = table.associations().iter().map(|association| json!({
        "route_table_association_id": association.route_table_association_id(),
        "route_table_id": association.route_table_id(), "main": association.main(),
        "subnet_id": association.subnet_id(), "gateway_id": association.gateway_id(),
        "state": association.association_state().and_then(|state| state.state()).map(|state| state.as_str()),
        "status_message": association.association_state().and_then(|state| state.status_message()),
    })).collect::<Vec<_>>();
    let resource = resource(
        ctx,
        ROUTE_TABLE,
        table.route_table_id(),
        table.owner_id(),
        table.tags(),
        json!({
            "route_table_id": table.route_table_id(), "vpc_id": table.vpc_id(),
            "is_main": table.associations().iter().any(|association| association.main() == Some(true)),
            "routes": table.routes().iter().map(route_json).collect::<Vec<_>>(),
            "associations": associations,
            "propagating_vgw_ids": table.propagating_vgws().iter().filter_map(|gateway| gateway.gateway_id()).collect::<Vec<_>>(),
        }),
    )?;
    let vpc = || {
        Reference::new(VPC, table.vpc_id())
            .map(|reference| reference.owner(table.owner_id(), "same_vpc_owner_from_response"))
    };
    page.link(ctx, &resource, vpc(), "IN_VPC", json!({"field":"vpc_id"}))?;
    for association in table.associations() {
        let evidence = json!({
            "route_table_association_id": association.route_table_association_id(),
            "main": association.main(),
            "state": association.association_state().and_then(|state| state.state()).map(|state| state.as_str()),
            "status_message": association.association_state().and_then(|state| state.status_message()),
        });
        if association.main() == Some(true) {
            page.link(
                ctx,
                &resource,
                vpc(),
                "MAIN_ROUTE_TABLE_FOR",
                evidence.clone(),
            )?;
        }
        page.link(
            ctx,
            &resource,
            Reference::new(SUBNET, association.subnet_id())
                .map(|reference| reference.owner(table.owner_id(), "same_vpc_owner_from_response")),
            "ASSOCIATED_WITH_SUBNET",
            evidence.clone(),
        )?;
        page.link(
            ctx,
            &resource,
            gateway_reference(association.gateway_id(), table.owner_id()),
            "ASSOCIATED_WITH_GATEWAY",
            evidence,
        )?;
    }
    for (index, route) in table.routes().iter().enumerate() {
        route_links(ctx, page, &resource, route, index, table.owner_id())?;
    }
    for gateway in table.propagating_vgws() {
        page.link(
            ctx,
            &resource,
            Reference::new(VGW, gateway.gateway_id()),
            "PROPAGATES_FROM",
            json!({"field":"propagating_vgws"}),
        )?;
    }
    page.resources.push(resource);
    Ok(())
}

pub(super) fn map_internet_gateway(
    ctx: &Context,
    page: &mut NetworkPage,
    gateway: &InternetGateway,
) -> Result<(), AwsScanError> {
    page.components(&[gateway.attachments().len(), gateway.tags().len()])?;
    let resource = resource(
        ctx,
        IGW,
        gateway.internet_gateway_id(),
        gateway.owner_id(),
        gateway.tags(),
        json!({
            "internet_gateway_id": gateway.internet_gateway_id(),
            "attachments": gateway.attachments().iter().map(|attachment| json!({
                "vpc_id": attachment.vpc_id(), "state": attachment.state().map(|state| state.as_str()),
            })).collect::<Vec<_>>(),
        }),
    )?;
    for attachment in gateway.attachments() {
        let reference = Reference::new(VPC, attachment.vpc_id())
            .map(|reference| reference.owner(gateway.owner_id(), "gateway_owner_from_response"));
        page.link(
            ctx,
            &resource,
            reference,
            "ATTACHED_TO_VPC",
            json!({"state":attachment.state().map(|state| state.as_str())}),
        )?;
    }
    page.resources.push(resource);
    Ok(())
}
