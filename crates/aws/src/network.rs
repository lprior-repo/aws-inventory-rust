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

#[cfg(test)]
mod tests;

mod route_evidence;
use route_evidence::{gateway_reference, route_json, route_links};
mod routing;
use routing::{map_internet_gateway, map_route_table};
mod gateways;
use gateways::{map_endpoint, map_nat_gateway};
mod access;
use access::{map_network_acl, map_peering};
mod collection;
use collection::{
    endpoints, internet_gateways, nat_gateways, network_acls, peering_connections, route_tables,
};
mod resource_mapping;
use resource_mapping::{millis, resource};

mod page;
