//! Bounded observations of VPC configuration; these relationships are not a
//! reachability analysis. In particular, route destinations are not targets.

use std::collections::{BTreeMap, BTreeSet};

use aws_sdk_ec2::{
    Client,
    types::{
        InternetGateway, NatGateway, NetworkAcl, Route, RouteTable, Tag, VpcEndpoint,
        VpcPeeringConnection, VpcPeeringConnectionVpcInfo,
    },
};
use aws_smithy_types::DateTime;
use inventory_core::{Edge, MAX_PAGE_EDGES, Resource, Source};
use serde_json::{Value, json};

use crate::{
    AwsScanError,
    common::{Context, PageBody, paginate, require_budget, required, sdk_error},
};

const MAX_COMPONENTS_PER_RESOURCE: usize = 4_096;
const MAX_COMPONENTS_PER_PAGE: usize = 65_536;

#[tracing::instrument(skip_all)]
pub(crate) async fn collect(ctx: &Context) -> Result<(), AwsScanError> {
    let client = Client::new(&ctx.config);
    tokio::try_join!(
        route_tables(ctx, &client),
        internet_gateways(ctx, &client),
        nat_gateways(ctx, &client),
        endpoints(ctx, &client),
        network_acls(ctx, &client),
        peering_connections(ctx, &client),
    )?;
    Ok(())
}

#[derive(Clone, Copy)]
struct TargetKind {
    arn_type: &'static str,
    resource_kind: &'static str,
}

const VPC: TargetKind = TargetKind {
    arn_type: "vpc",
    resource_kind: "aws_vpc",
};
const SUBNET: TargetKind = TargetKind {
    arn_type: "subnet",
    resource_kind: "aws_vpc_subnet",
};
const ROUTE_TABLE: TargetKind = TargetKind {
    arn_type: "route-table",
    resource_kind: "aws_vpc_route_table",
};
const IGW: TargetKind = TargetKind {
    arn_type: "internet-gateway",
    resource_kind: "aws_vpc_internet_gateway",
};
const NAT: TargetKind = TargetKind {
    arn_type: "natgateway",
    resource_kind: "aws_vpc_nat_gateway",
};
const ENDPOINT: TargetKind = TargetKind {
    arn_type: "vpc-endpoint",
    resource_kind: "aws_vpc_endpoint",
};
const ACL: TargetKind = TargetKind {
    arn_type: "network-acl",
    resource_kind: "aws_vpc_network_acl",
};
const PEERING: TargetKind = TargetKind {
    arn_type: "vpc-peering-connection",
    resource_kind: "aws_vpc_peering_connection",
};
const ENI: TargetKind = TargetKind {
    arn_type: "network-interface",
    resource_kind: "aws_ec2_network_interface",
};
const GROUP: TargetKind = TargetKind {
    arn_type: "security-group",
    resource_kind: "aws_vpc_security_group",
};
const INSTANCE: TargetKind = TargetKind {
    arn_type: "instance",
    resource_kind: "aws_ec2_instance",
};
const EIP: TargetKind = TargetKind {
    arn_type: "elastic-ip",
    resource_kind: "aws_vpc_eip",
};
const VGW: TargetKind = TargetKind {
    arn_type: "vpn-gateway",
    resource_kind: "aws_vpc_vpn_gateway",
};
const EGRESS_IGW: TargetKind = TargetKind {
    arn_type: "egress-only-internet-gateway",
    resource_kind: "aws_vpc_egress_only_internet_gateway",
};
const TRANSIT_GATEWAY: TargetKind = TargetKind {
    arn_type: "transit-gateway",
    resource_kind: "aws_ec2_transit_gateway",
};
const LOCAL_GATEWAY: TargetKind = TargetKind {
    arn_type: "local-gateway",
    resource_kind: "aws_ec2_local_gateway",
};
const CARRIER_GATEWAY: TargetKind = TargetKind {
    arn_type: "carrier-gateway",
    resource_kind: "aws_vpc_carrier_gateway",
};

struct Reference<'a> {
    kind: TargetKind,
    id: &'a str,
    owner: Option<&'a str>,
    region: Option<&'a str>,
    owner_resolution: &'static str,
}

impl<'a> Reference<'a> {
    fn new(kind: TargetKind, id: Option<&'a str>) -> Option<Self> {
        id.filter(|id| !id.is_empty()).map(|id| Self {
            kind,
            id,
            owner: None,
            region: None,
            owner_resolution: "assumed_scanning_account",
        })
    }

    fn owner(mut self, owner: Option<&'a str>, resolution: &'static str) -> Self {
        self.owner = owner.filter(|owner| !owner.is_empty());
        if self.owner.is_some() {
            self.owner_resolution = resolution;
        }
        self
    }

    fn region(mut self, region: Option<&'a str>) -> Self {
        self.region = region.filter(|region| !region.is_empty());
        self
    }

    fn resolve(self, ctx: &Context) -> (String, Value) {
        let owner = self.owner.unwrap_or(&ctx.account);
        let region = self.region.unwrap_or(&ctx.region);
        let target = format!(
            "arn:{}:ec2:{region}:{owner}:{}/{}",
            ctx.partition, self.kind.arn_type, self.id
        );
        let properties = json!({
            "target_resource_id": self.id,
            "target_kind": self.kind.resource_kind,
            "target_account_id": owner,
            "target_region": region,
            "owner_resolution": self.owner_resolution,
            "region_resolution": if self.region.is_some() { "aws_response" } else { "scanning_region" },
        });
        (target, properties)
    }
}

/// Deduplication is page-local. The store's edge key excludes properties, so
/// multiple route destinations to one target must share an evidence array.
#[derive(Default)]
struct NetworkPage {
    resources: Vec<Resource>,
    edges: BTreeMap<(String, String, String), Edge>,
    components: usize,
}

impl NetworkPage {
    fn components(&mut self, counts: &[usize]) -> Result<(), AwsScanError> {
        let count = counts
            .iter()
            .try_fold(0usize, |total, count| total.checked_add(*count))
            .ok_or_else(|| AwsScanError::InvalidData("network component count overflow".into()))?;
        if count > MAX_COMPONENTS_PER_RESOURCE {
            return Err(AwsScanError::InvalidData(format!(
                "network resource exceeds {MAX_COMPONENTS_PER_RESOURCE} nested components"
            )));
        }
        self.components = self.components.checked_add(count).ok_or_else(|| {
            AwsScanError::InvalidData("network page component count overflow".into())
        })?;
        if self.components > MAX_COMPONENTS_PER_PAGE {
            return Err(AwsScanError::InvalidData(format!(
                "network page exceeds {MAX_COMPONENTS_PER_PAGE} nested components"
            )));
        }
        Ok(())
    }

    fn link(
        &mut self,
        ctx: &Context,
        resource: &Resource,
        reference: Option<Reference<'_>>,
        relation: &str,
        evidence: Value,
    ) -> Result<(), AwsScanError> {
        let Some(reference) = reference else {
            return Ok(());
        };
        let (target, properties) = reference.resolve(ctx);
        self.link_resolved(ctx, resource, target, relation, properties, evidence)
    }

    fn arn_link(
        &mut self,
        ctx: &Context,
        resource: &Resource,
        arn: Option<&str>,
        relation: &str,
        evidence: Value,
    ) -> Result<(), AwsScanError> {
        let Some(arn) = arn.filter(|arn| !arn.is_empty()) else {
            return Ok(());
        };
        if !arn.starts_with("arn:") {
            return Err(AwsScanError::InvalidData(
                "AWS returned a non-ARN network target in an ARN field".into(),
            ));
        }
        self.link_resolved(
            ctx,
            resource,
            arn.to_owned(),
            relation,
            json!({
                "target_resource_id": arn,
                "target_account_id": arn.split(':').nth(4),
                "target_region": arn.split(':').nth(3),
                "owner_resolution": "aws_returned_arn",
                "region_resolution": "aws_returned_arn",
            }),
            evidence,
        )
    }

    fn link_resolved(
        &mut self,
        ctx: &Context,
        resource: &Resource,
        target: String,
        relation: &str,
        properties: Value,
        evidence: Value,
    ) -> Result<(), AwsScanError> {
        let key = (resource.id.clone(), target.clone(), relation.to_owned());
        if let Some(edge) = self.edges.get_mut(&key) {
            let observations = edge
                .properties
                .get_mut("evidence")
                .and_then(Value::as_array_mut)
                .ok_or_else(|| {
                    AwsScanError::InvalidData("network edge evidence is not an array".into())
                })?;
            if observations.len() >= MAX_COMPONENTS_PER_RESOURCE {
                return Err(AwsScanError::InvalidData(
                    "network edge evidence capacity exceeded".into(),
                ));
            }
            observations.push(evidence);
        } else {
            if self.edges.len() >= MAX_PAGE_EDGES {
                return Err(AwsScanError::InvalidData(
                    "network page edge capacity exceeded".into(),
                ));
            }
            let mut edge = ctx.edge(resource, target, relation);
            edge.properties = properties;
            let object = edge.properties.as_object_mut().ok_or_else(|| {
                AwsScanError::InvalidData("network reference evidence is not an object".into())
            })?;
            object.insert("evidence".into(), json!([evidence]));
            self.edges.insert(key, edge);
        }
        Ok(())
    }

    fn finish(
        self,
        next_token: Option<&str>,
        listed_count: usize,
    ) -> Result<PageBody, AwsScanError> {
        {
            let mut resource_ids = BTreeSet::new();
            for resource in &self.resources {
                if !resource_ids.insert(resource.id.as_str()) {
                    return Err(AwsScanError::InvalidData(
                        "network page repeats a resource identity".into(),
                    ));
                }
            }
        }
        let edges = self.edges.into_values().collect::<Vec<_>>();
        for edge in &edges {
            edge.validate()?;
        }
        Ok(PageBody {
            resources: self.resources,
            edges,
            next_token: next_token
                .filter(|token| !token.is_empty())
                .map(str::to_owned),
            listed_count,
        })
    }
}

fn resource(
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

fn millis(value: Option<&DateTime>) -> Result<Option<i64>, AwsScanError> {
    value
        .map(|value| value.to_millis())
        .transpose()
        .map_err(|error| AwsScanError::InvalidData(format!("network timestamp: {error}")))
}

fn route_json(route: &Route) -> Value {
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

fn gateway_reference<'a>(id: Option<&'a str>, igw_owner: Option<&'a str>) -> Option<Reference<'a>> {
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

fn route_evidence(route: &Route, index: usize, field: &str) -> Value {
    json!({
        "route_index": index, "target_field": field,
        "destination_cidr_block": route.destination_cidr_block(),
        "destination_ipv6_cidr_block": route.destination_ipv6_cidr_block(),
        "destination_prefix_list_id": route.destination_prefix_list_id(),
        "state": route.state().map(|value| value.as_str()),
        "origin": route.origin().map(|value| value.as_str()),
    })
}

fn route_links(
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

fn map_route_table(
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

fn map_internet_gateway(
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

fn map_nat_gateway(
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

fn map_endpoint(
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

fn map_network_acl(
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

fn peer_json(info: &VpcPeeringConnectionVpcInfo) -> Value {
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

fn map_peering(
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

#[tracing::instrument(skip_all)]
async fn route_tables(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    paginate(
        ctx,
        ctx.scope(ROUTE_TABLE.resource_kind, None, Source::Direct),
        |token, budget| async move {
            let output = ctx
                .execute("ec2", "read", || async {
                    client
                        .describe_route_tables()
                        .max_results(ctx.page_size.clamp(5, 100))
                        .set_next_token(token.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            require_budget(output.route_tables().len(), budget)?;
            let mut page = NetworkPage::default();
            for table in output.route_tables() {
                map_route_table(ctx, &mut page, table)?;
            }
            page.finish(output.next_token(), output.route_tables().len())
        },
    )
    .await
}

#[tracing::instrument(skip_all)]
async fn internet_gateways(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    paginate(
        ctx,
        ctx.scope(IGW.resource_kind, None, Source::Direct),
        |token, budget| async move {
            let output = ctx
                .execute("ec2", "read", || async {
                    client
                        .describe_internet_gateways()
                        .max_results(ctx.page_size.clamp(5, 1_000))
                        .set_next_token(token.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            require_budget(output.internet_gateways().len(), budget)?;
            let mut page = NetworkPage::default();
            for gateway in output.internet_gateways() {
                map_internet_gateway(ctx, &mut page, gateway)?;
            }
            page.finish(output.next_token(), output.internet_gateways().len())
        },
    )
    .await
}

#[tracing::instrument(skip_all)]
async fn nat_gateways(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    paginate(
        ctx,
        ctx.scope(NAT.resource_kind, None, Source::Direct),
        |token, budget| async move {
            let output = ctx
                .execute("ec2", "read", || async {
                    client
                        .describe_nat_gateways()
                        .max_results(ctx.page_size.clamp(5, 1_000))
                        .set_next_token(token.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            require_budget(output.nat_gateways().len(), budget)?;
            let mut page = NetworkPage::default();
            for gateway in output.nat_gateways() {
                map_nat_gateway(ctx, &mut page, gateway)?;
            }
            page.finish(output.next_token(), output.nat_gateways().len())
        },
    )
    .await
}

#[tracing::instrument(skip_all)]
async fn endpoints(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    paginate(
        ctx,
        ctx.scope(ENDPOINT.resource_kind, None, Source::Direct),
        |token, budget| async move {
            let output = ctx
                .execute("ec2", "read", || async {
                    client
                        .describe_vpc_endpoints()
                        .max_results(ctx.page_size.clamp(1, 1_000))
                        .set_next_token(token.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            require_budget(output.vpc_endpoints().len(), budget)?;
            let mut page = NetworkPage::default();
            for endpoint in output.vpc_endpoints() {
                map_endpoint(ctx, &mut page, endpoint)?;
            }
            page.finish(output.next_token(), output.vpc_endpoints().len())
        },
    )
    .await
}

#[tracing::instrument(skip_all)]
async fn network_acls(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    paginate(
        ctx,
        ctx.scope(ACL.resource_kind, None, Source::Direct),
        |token, budget| async move {
            let output = ctx
                .execute("ec2", "read", || async {
                    client
                        .describe_network_acls()
                        .max_results(ctx.page_size.clamp(5, 1_000))
                        .set_next_token(token.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            require_budget(output.network_acls().len(), budget)?;
            let mut page = NetworkPage::default();
            for acl in output.network_acls() {
                map_network_acl(ctx, &mut page, acl)?;
            }
            page.finish(output.next_token(), output.network_acls().len())
        },
    )
    .await
}

#[tracing::instrument(skip_all)]
async fn peering_connections(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    paginate(
        ctx,
        ctx.scope(PEERING.resource_kind, None, Source::Direct),
        |token, budget| async move {
            let output = ctx
                .execute("ec2", "read", || async {
                    client
                        .describe_vpc_peering_connections()
                        .max_results(ctx.page_size.clamp(5, 1_000))
                        .set_next_token(token.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            require_budget(output.vpc_peering_connections().len(), budget)?;
            let mut page = NetworkPage::default();
            for peering in output.vpc_peering_connections() {
                map_peering(ctx, &mut page, peering)?;
            }
            page.finish(output.next_token(), output.vpc_peering_connections().len())
        },
    )
    .await
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use aws_sdk_ec2::types::{
        AttachmentStatus, AvailabilityMode, IcmpTypeCode, InternetGatewayAttachment,
        NatGatewayAddress, NatGatewayState, NetworkAclAssociation, NetworkAclEntry, PortRange,
        RouteOrigin, RouteState, RouteTableAssociation, RouteTableAssociationState,
        RouteTableAssociationStateCode, RuleAction, SecurityGroupIdentifier,
        VpcPeeringConnectionStateReason, VpcPeeringConnectionStateReasonCode,
    };
    use inventory_core::CollectionEvent;
    use inventory_engine::limiter::{ApiError, ApiExecutor, ExecutorConfig};
    use tokio::sync::mpsc;

    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn context() -> Result<(Context, mpsc::Receiver<CollectionEvent>), ApiError> {
        let (sender, receiver) = mpsc::channel(8);
        Ok((
            Context {
                config: aws_types::SdkConfig::builder().build(),
                executor: Arc::new(ApiExecutor::new(ExecutorConfig::default())?),
                sender,
                account: "111111111111".into(),
                partition: "aws".into(),
                region: "us-east-1".into(),
                max_pages: 3,
                page_size: 100,
            },
            receiver,
        ))
    }

    #[test]
    fn routes_keep_destinations_states_and_one_edge_per_target() -> TestResult {
        let (ctx, _receiver) = context()?;
        let table = RouteTable::builder()
            .route_table_id("rtb-one")
            .vpc_id("vpc-one")
            .owner_id("222222222222")
            .routes(
                Route::builder()
                    .destination_cidr_block("10.0.0.0/16")
                    .gateway_id("local")
                    .state(RouteState::Active)
                    .build(),
            )
            .routes(
                Route::builder()
                    .destination_cidr_block("0.0.0.0/0")
                    .gateway_id("igw-one")
                    .state(RouteState::Active)
                    .origin(RouteOrigin::CreateRoute)
                    .build(),
            )
            .routes(
                Route::builder()
                    .destination_ipv6_cidr_block("::/0")
                    .gateway_id("igw-one")
                    .state(RouteState::Blackhole)
                    .origin(RouteOrigin::CreateRoute)
                    .build(),
            )
            .routes(
                Route::builder()
                    .destination_prefix_list_id("pl-one")
                    .gateway_id("igw-one")
                    .state(RouteState::Active)
                    .build(),
            )
            .build();
        let mut page = NetworkPage::default();
        map_route_table(&ctx, &mut page, &table)?;
        let body = page.finish(None, 1)?;
        let resource = body.resources.first().ok_or("missing route table")?;
        assert_eq!(resource.account_id, "222222222222");
        assert!(resource.id.contains(":222222222222:route-table/"));
        assert_eq!(
            resource.properties.pointer("/routes/0/gateway_id"),
            Some(&json!("local"))
        );
        assert_eq!(
            resource.properties.pointer("/routes/2/state"),
            Some(&json!("blackhole"))
        );
        let targets = body
            .edges
            .iter()
            .filter(|edge| edge.relation == "ROUTES_TO")
            .collect::<Vec<_>>();
        assert_eq!(targets.len(), 1);
        let target = targets.first().ok_or("missing gateway target")?;
        assert!(target.to_id.ends_with("internet-gateway/igw-one"));
        assert_eq!(
            target.to_id,
            "arn:aws:ec2:us-east-1:222222222222:internet-gateway/igw-one"
        );
        assert_eq!(
            target.properties.get("owner_resolution"),
            Some(&json!("same_vpc_owner_from_response"))
        );
        assert_eq!(
            target
                .properties
                .get("evidence")
                .and_then(Value::as_array)
                .map(Vec::len),
            Some(3)
        );
        assert_eq!(
            target.properties.pointer("/evidence/1/state"),
            Some(&json!("blackhole"))
        );
        assert_eq!(
            target
                .properties
                .pointer("/evidence/2/destination_prefix_list_id"),
            Some(&json!("pl-one"))
        );
        assert!(
            body.edges
                .iter()
                .all(|edge| !edge.to_id.ends_with("/local"))
        );
        Ok(())
    }

    #[test]
    fn main_association_does_not_fabricate_subnets_and_explicit_refs_deduplicate() -> TestResult {
        let (ctx, _receiver) = context()?;
        let explicit = RouteTableAssociation::builder()
            .route_table_association_id("rtbassoc-explicit")
            .route_table_id("rtb-one")
            .subnet_id("subnet-one")
            .main(false)
            .build();
        let table = RouteTable::builder()
            .route_table_id("rtb-one")
            .vpc_id("vpc-one")
            .owner_id("222222222222")
            .associations(
                RouteTableAssociation::builder()
                    .route_table_association_id("rtbassoc-main")
                    .main(true)
                    .association_state(
                        RouteTableAssociationState::builder()
                            .state(RouteTableAssociationStateCode::Associated)
                            .build(),
                    )
                    .build(),
            )
            .associations(explicit.clone())
            .associations(explicit)
            .associations(
                RouteTableAssociation::builder()
                    .route_table_association_id("rtbassoc-gateway")
                    .gateway_id("igw-one")
                    .build(),
            )
            .build();
        let mut page = NetworkPage::default();
        map_route_table(&ctx, &mut page, &table)?;
        let body = page.finish(None, 1)?;
        assert_eq!(
            body.edges
                .iter()
                .filter(|edge| edge.relation == "ASSOCIATED_WITH_SUBNET")
                .count(),
            1
        );
        let subnet = body
            .edges
            .iter()
            .find(|edge| edge.relation == "ASSOCIATED_WITH_SUBNET")
            .ok_or("missing explicit subnet association")?;
        assert_eq!(
            subnet.to_id,
            "arn:aws:ec2:us-east-1:222222222222:subnet/subnet-one"
        );
        assert_eq!(
            subnet.properties.get("owner_resolution"),
            Some(&json!("same_vpc_owner_from_response"))
        );
        let gateway = body
            .edges
            .iter()
            .find(|edge| edge.relation == "ASSOCIATED_WITH_GATEWAY")
            .ok_or("missing gateway association")?;
        assert_eq!(
            gateway.to_id,
            "arn:aws:ec2:us-east-1:222222222222:internet-gateway/igw-one"
        );
        assert_eq!(
            gateway.properties.get("owner_resolution"),
            Some(&json!("same_vpc_owner_from_response"))
        );
        let main = body
            .edges
            .iter()
            .find(|edge| edge.relation == "MAIN_ROUTE_TABLE_FOR")
            .ok_or("missing main association")?;
        assert_eq!(main.to_id, "arn:aws:ec2:us-east-1:222222222222:vpc/vpc-one");
        assert_eq!(
            main.properties.pointer("/evidence/0/state"),
            Some(&json!("associated"))
        );
        assert_eq!(
            body.resources
                .first()
                .and_then(|resource| resource.properties.get("is_main")),
            Some(&json!(true))
        );
        Ok(())
    }

    #[test]
    fn route_target_owner_arn_and_unrecognized_gateway_are_preserved() -> TestResult {
        let (ctx, _receiver) = context()?;
        let core_arn = "arn:aws:networkmanager::333333333333:core-network/core-network-one";
        let table = RouteTable::builder()
            .route_table_id("rtb-one")
            .routes(
                Route::builder()
                    .destination_cidr_block("10.2.0.0/16")
                    .instance_id("i-one")
                    .instance_owner_id("222222222222")
                    .build(),
            )
            .routes(
                Route::builder()
                    .destination_cidr_block("10.3.0.0/16")
                    .core_network_arn(core_arn)
                    .build(),
            )
            .routes(
                Route::builder()
                    .destination_cidr_block("10.4.0.0/16")
                    .gateway_id("future-target-one")
                    .build(),
            )
            .build();
        let mut page = NetworkPage::default();
        map_route_table(&ctx, &mut page, &table)?;
        let body = page.finish(None, 1)?;
        assert_eq!(body.edges.len(), 2);
        let instance = body
            .edges
            .iter()
            .find(|edge| edge.to_id.ends_with("instance/i-one"))
            .ok_or("missing instance")?;
        assert_eq!(
            instance.to_id,
            "arn:aws:ec2:us-east-1:222222222222:instance/i-one"
        );
        let core = body
            .edges
            .iter()
            .find(|edge| edge.to_id == core_arn)
            .ok_or("missing core network")?;
        assert_eq!(
            core.properties.get("owner_resolution"),
            Some(&json!("aws_returned_arn"))
        );
        assert_eq!(
            body.resources
                .first()
                .and_then(|resource| resource.properties.pointer("/routes/2/gateway_id")),
            Some(&json!("future-target-one"))
        );
        Ok(())
    }

    #[test]
    fn internet_gateway_uses_reported_owner_and_attachment_state() -> TestResult {
        let (ctx, _receiver) = context()?;
        let gateway = InternetGateway::builder()
            .internet_gateway_id("igw-one")
            .owner_id("222222222222")
            .attachments(
                InternetGatewayAttachment::builder()
                    .vpc_id("vpc-one")
                    .state(AttachmentStatus::from("available"))
                    .build(),
            )
            .build();
        let mut page = NetworkPage::default();
        map_internet_gateway(&ctx, &mut page, &gateway)?;
        let body = page.finish(None, 1)?;
        assert_eq!(
            body.resources
                .first()
                .map(|resource| resource.account_id.as_str()),
            Some("222222222222")
        );
        let attachment = body.edges.first().ok_or("missing attachment")?;
        assert_eq!(attachment.relation, "ATTACHED_TO_VPC");
        assert_eq!(
            attachment.to_id,
            "arn:aws:ec2:us-east-1:222222222222:vpc/vpc-one"
        );
        assert_eq!(
            attachment.properties.pointer("/evidence/0/state"),
            Some(&json!("available"))
        );
        Ok(())
    }

    #[test]
    fn regional_nat_allows_absent_subnet_and_maps_address_targets() -> TestResult {
        let (ctx, _receiver) = context()?;
        let gateway = NatGateway::builder()
            .nat_gateway_id("nat-one")
            .vpc_id("vpc-one")
            .availability_mode(AvailabilityMode::from("regional"))
            .state(NatGatewayState::Available)
            .route_table_id("nat-one")
            .nat_gateway_addresses(
                NatGatewayAddress::builder()
                    .allocation_id("eipalloc-one")
                    .network_interface_id("eni-one")
                    .private_ip("10.0.1.4")
                    .public_ip("203.0.113.10")
                    .is_primary(true)
                    .build(),
            )
            .build();
        let mut page = NetworkPage::default();
        map_nat_gateway(&ctx, &mut page, &gateway)?;
        let body = page.finish(None, 1)?;
        let nat = body.resources.first().ok_or("missing NAT")?;
        assert!(nat.id.ends_with("natgateway/nat-one"));
        assert_eq!(nat.properties.get("subnet_id"), Some(&Value::Null));
        assert_eq!(
            nat.properties.get("availability_mode"),
            Some(&json!("regional"))
        );
        assert_eq!(
            nat.properties.pointer("/addresses/0/is_primary"),
            Some(&json!(true))
        );
        assert!(body.edges.iter().all(|edge| edge.relation != "IN_SUBNET"));
        assert!(
            body.edges
                .iter()
                .all(|edge| edge.relation != "USES_ROUTE_TABLE")
        );
        assert!(
            body.edges
                .iter()
                .any(|edge| edge.relation == "USES_ELASTIC_IP"
                    && edge.to_id.ends_with("elastic-ip/eipalloc-one"))
        );
        assert!(
            body.edges
                .iter()
                .any(|edge| edge.relation == "USES_NETWORK_INTERFACE"
                    && edge.to_id.ends_with("network-interface/eni-one"))
        );
        Ok(())
    }

    #[test]
    fn endpoint_shared_network_refs_stay_uncertain_and_duplicate_refs_share_edges() -> TestResult {
        let (ctx, _receiver) = context()?;
        let service_arn = "arn:aws:vpc-lattice:us-west-2:333333333333:servicenetwork/sn-one";
        let endpoint = VpcEndpoint::builder()
            .vpc_endpoint_id("vpce-one")
            .owner_id("222222222222")
            .vpc_id("vpc-shared")
            .subnet_ids("subnet-one")
            .subnet_ids("subnet-one")
            .route_table_ids("rtb-one")
            .network_interface_ids("eni-one")
            .groups(
                SecurityGroupIdentifier::builder()
                    .group_id("sg-one")
                    .group_name("endpoint")
                    .build(),
            )
            .groups(
                SecurityGroupIdentifier::builder()
                    .group_id("sg-one")
                    .group_name("endpoint")
                    .build(),
            )
            .service_network_arn(service_arn)
            .service_name("com.amazonaws.us-east-1.s3")
            .build();
        let mut page = NetworkPage::default();
        map_endpoint(&ctx, &mut page, &endpoint)?;
        let body = page.finish(None, 1)?;
        let endpoint = body.resources.first().ok_or("missing endpoint")?;
        assert_eq!(endpoint.account_id, "222222222222");
        assert_eq!(
            body.edges
                .iter()
                .filter(|edge| edge.relation == "USES_SUBNET")
                .count(),
            1
        );
        assert_eq!(
            body.edges
                .iter()
                .filter(|edge| edge.relation == "USES_SECURITY_GROUP")
                .count(),
            1
        );
        let vpc = body
            .edges
            .iter()
            .find(|edge| edge.relation == "IN_VPC")
            .ok_or("missing VPC ref")?;
        assert_eq!(
            vpc.properties.get("owner_resolution"),
            Some(&json!("assumed_scanning_account"))
        );
        assert!(body.edges.iter().any(|edge| edge.to_id == service_arn && edge.relation == "TARGETS_SERVICE_NETWORK"));
        assert!(
            body.edges
                .iter()
                .all(|edge| !edge.to_id.contains("com.amazonaws"))
        );
        Ok(())
    }

    #[test]
    fn acl_keeps_order_direction_protocol_and_deny_without_access_edges() -> TestResult {
        let (ctx, _receiver) = context()?;
        let acl = NetworkAcl::builder()
            .network_acl_id("acl-one")
            .vpc_id("vpc-one")
            .owner_id("222222222222")
            .is_default(false)
            .entries(
                NetworkAclEntry::builder()
                    .rule_number(100)
                    .egress(false)
                    .protocol("6")
                    .rule_action(RuleAction::Allow)
                    .cidr_block("10.0.0.0/16")
                    .port_range(PortRange::builder().from(443).to(443).build())
                    .build(),
            )
            .entries(
                NetworkAclEntry::builder()
                    .rule_number(200)
                    .egress(true)
                    .protocol("58")
                    .rule_action(RuleAction::Deny)
                    .ipv6_cidr_block("::/0")
                    .icmp_type_code(IcmpTypeCode::builder().r#type(1).code(-1).build())
                    .build(),
            )
            .entries(
                NetworkAclEntry::builder()
                    .rule_number(32767)
                    .egress(false)
                    .protocol("-1")
                    .rule_action(RuleAction::Deny)
                    .cidr_block("0.0.0.0/0")
                    .build(),
            )
            .associations(
                NetworkAclAssociation::builder()
                    .network_acl_association_id("aclassoc-one")
                    .network_acl_id("acl-one")
                    .subnet_id("subnet-one")
                    .build(),
            )
            .build();
        let mut page = NetworkPage::default();
        map_network_acl(&ctx, &mut page, &acl)?;
        let body = page.finish(None, 1)?;
        let acl = body.resources.first().ok_or("missing ACL")?;
        assert_eq!(
            acl.properties.pointer("/entries/0/port_range/from"),
            Some(&json!(443))
        );
        assert_eq!(
            acl.properties.pointer("/entries/1/egress"),
            Some(&json!(true))
        );
        assert_eq!(
            acl.properties.pointer("/entries/1/icmp_type_code/code"),
            Some(&json!(-1))
        );
        assert_eq!(
            acl.properties.pointer("/entries/2/rule_number"),
            Some(&json!(32767))
        );
        assert_eq!(
            acl.properties.pointer("/entries/2/rule_action"),
            Some(&json!("deny"))
        );
        let subnet = body
            .edges
            .iter()
            .find(|edge| edge.relation == "ASSOCIATED_WITH_SUBNET")
            .ok_or("missing ACL subnet association")?;
        assert_eq!(
            subnet.to_id,
            "arn:aws:ec2:us-east-1:222222222222:subnet/subnet-one"
        );
        assert_eq!(
            subnet.properties.get("owner_resolution"),
            Some(&json!("same_vpc_owner_from_response"))
        );
        assert!(
            body.edges
                .iter()
                .all(|edge| matches!(edge.relation.as_str(), "IN_VPC" | "ASSOCIATED_WITH_SUBNET"))
        );
        Ok(())
    }

    #[test]
    fn peering_uses_both_reported_owners_and_regions_without_reversing_roles() -> TestResult {
        let (ctx, _receiver) = context()?;
        let peering = VpcPeeringConnection::builder()
            .vpc_peering_connection_id("pcx-one")
            .requester_vpc_info(
                VpcPeeringConnectionVpcInfo::builder()
                    .vpc_id("vpc-requester")
                    .owner_id("222222222222")
                    .region("eu-west-1")
                    .cidr_block("10.0.0.0/16")
                    .build(),
            )
            .accepter_vpc_info(
                VpcPeeringConnectionVpcInfo::builder()
                    .vpc_id("vpc-accepter")
                    .owner_id("333333333333")
                    .region("us-west-2")
                    .cidr_block("10.1.0.0/16")
                    .build(),
            )
            .status(
                VpcPeeringConnectionStateReason::builder()
                    .code(VpcPeeringConnectionStateReasonCode::Active)
                    .message("Active")
                    .build(),
            )
            .build();
        let mut page = NetworkPage::default();
        map_peering(&ctx, &mut page, &peering)?;
        let body = page.finish(None, 1)?;
        let connection = body.resources.first().ok_or("missing connection")?;
        assert_eq!(connection.account_id, "111111111111");
        assert_eq!(connection.properties.get("owner_id"), Some(&Value::Null));
        assert_eq!(
            connection.properties.get("identity_scope"),
            Some(&json!("scanning_account_and_region"))
        );
        let requester = body
            .edges
            .iter()
            .find(|edge| edge.relation == "REQUESTER_VPC")
            .ok_or("missing requester")?;
        let accepter = body
            .edges
            .iter()
            .find(|edge| edge.relation == "ACCEPTER_VPC")
            .ok_or("missing accepter")?;
        assert_eq!(
            requester.to_id,
            "arn:aws:ec2:eu-west-1:222222222222:vpc/vpc-requester"
        );
        assert_eq!(
            accepter.to_id,
            "arn:aws:ec2:us-west-2:333333333333:vpc/vpc-accepter"
        );
        assert_eq!(
            requester.properties.get("owner_resolution"),
            Some(&json!("aws_response"))
        );
        assert_eq!(
            accepter.properties.get("target_region"),
            Some(&json!("us-west-2"))
        );
        Ok(())
    }

    #[test]
    fn repeated_resource_identity_fails_before_storage() -> TestResult {
        let (ctx, _receiver) = context()?;
        let table = RouteTable::builder().route_table_id("rtb-one").build();
        let mut page = NetworkPage::default();
        map_route_table(&ctx, &mut page, &table)?;
        map_route_table(&ctx, &mut page, &table)?;
        assert!(matches!(
            page.finish(None, 2),
            Err(AwsScanError::InvalidData(_))
        ));
        Ok(())
    }

    #[test]
    fn nested_capacity_is_checked_before_resource_mapping() -> TestResult {
        let (ctx, _receiver) = context()?;
        let count = MAX_COMPONENTS_PER_RESOURCE
            .checked_add(1)
            .ok_or("test count overflow")?;
        let table = RouteTable::builder()
            .route_table_id("rtb-one")
            .set_routes(Some(vec![
                Route::builder().gateway_id("local").build();
                count
            ]))
            .build();
        let mut page = NetworkPage::default();
        assert!(matches!(
            map_route_table(&ctx, &mut page, &table),
            Err(AwsScanError::InvalidData(_))
        ));
        assert!(page.resources.is_empty());
        assert!(page.edges.is_empty());
        Ok(())
    }

    #[test]
    fn absent_identifier_is_an_error_and_empty_token_is_terminal() -> TestResult {
        let (ctx, _receiver) = context()?;
        let mut page = NetworkPage::default();
        assert!(matches!(
            map_internet_gateway(&ctx, &mut page, &InternetGateway::builder().build()),
            Err(AwsScanError::InvalidData(_))
        ));
        assert!(
            NetworkPage::default()
                .finish(Some(""), 0)?
                .next_token
                .is_none()
        );
        Ok(())
    }

    #[tokio::test]
    async fn network_scope_retains_nonterminal_page_then_records_denial() -> TestResult {
        let (ctx, mut receiver) = context()?;
        let scope = ctx.scope(ROUTE_TABLE.resource_kind, None, Source::Direct);
        paginate(&ctx, scope.clone(), |token, _budget| async move {
            if token.is_none() {
                NetworkPage::default().finish(Some("continue"), 0)
            } else {
                Err(AwsScanError::Api(ApiError::Denied(
                    "test access denial".into(),
                )))
            }
        })
        .await?;
        let first = receiver.recv().await.ok_or("missing nonterminal page")?;
        let second = receiver.recv().await.ok_or("missing failure")?;
        assert!(
            matches!(first, CollectionEvent::Page(ref page) if page.scope == scope && page.next_token.as_deref() == Some("continue"))
        );
        assert!(
            matches!(second, CollectionEvent::Failure(ref failure) if failure.scope == scope && failure.category == "access_denied")
        );
        assert!(receiver.try_recv().is_err());
        Ok(())
    }

    #[tokio::test]
    async fn network_scope_page_cap_emits_failure_without_terminal_success() -> TestResult {
        let (mut ctx, mut receiver) = context()?;
        ctx.max_pages = 1;
        paginate(
            &ctx,
            ctx.scope(ACL.resource_kind, None, Source::Direct),
            |_token, _budget| async { NetworkPage::default().finish(Some("more"), 0) },
        )
        .await?;
        let first = receiver.recv().await.ok_or("missing capped page")?;
        let second = receiver.recv().await.ok_or("missing cap failure")?;
        assert!(
            matches!(first, CollectionEvent::Page(ref page) if page.next_token.as_deref() == Some("more"))
        );
        assert!(
            matches!(second, CollectionEvent::Failure(ref failure) if failure.category == "page_limit")
        );
        assert!(receiver.try_recv().is_err());
        Ok(())
    }
}
