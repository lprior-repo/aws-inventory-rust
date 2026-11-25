use super::super::{empty, path};
use crate::overlay::{
    FixedValue, Identity, ItemSelector, NameError, OperationOverlay, OverlayClass, Paginator,
    Scope, TokenPair,
};

pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("DescribeVpcs", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Vpcs")?),
        identity: Some(Identity::Key(vec![path("VpcId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeSubnets", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Subnets")?),
        identity: Some(Identity::Key(vec![path("SubnetId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeRegions", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("Regions")?),
        fixed: [(path("AllRegions")?, FixedValue::Boolean(false))].into_iter().collect(),
        identity: Some(Identity::Key(vec![path("RegionName")?])),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("DescribeInstances", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Reservations[].Instances[] requires flattening with Reservations[].OwnerId retained for cross-account ownership; selecting Reservations incorrectly inventories reservation wrappers, not instances.".to_owned()),
        ..empty()
    }),
("DescribeInternetGateways", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("InternetGateways")?),
        identity: Some(Identity::Key(vec![path("InternetGatewayId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeNatGateways", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("NatGateways")?),
        identity: Some(Identity::Key(vec![path("NatGatewayId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeNetworkAcls", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("NetworkAcls")?),
        identity: Some(Identity::Key(vec![path("NetworkAclId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeNetworkInterfaces", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("NetworkInterfaces")?),
        fixed: [(path("IncludeManagedResources")?, FixedValue::Boolean(true))].into_iter().collect(),
        identity: Some(Identity::Key(vec![path("NetworkInterfaceId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeRouteTables", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("RouteTables")?),
        identity: Some(Identity::Key(vec![path("RouteTableId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeSecurityGroups", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("SecurityGroups")?),
        identity: Some(Identity::Key(vec![path("GroupId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeVolumes", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Volumes")?),
        fixed: [(path("IncludeManagedResources")?, FixedValue::Boolean(true))].into_iter().collect(),
        identity: Some(Identity::Key(vec![path("VolumeId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeVpcEndpoints", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("VpcEndpoints")?),
        identity: Some(Identity::Key(vec![path("VpcEndpointId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeVpcPeeringConnections", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("VpcPeeringConnections")?),
        identity: Some(Identity::Key(vec![path("VpcPeeringConnectionId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeAccountAttributes", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("AccountAttributes")?),
        identity: Some(Identity::Key(vec![path("AttributeName")?])),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("DescribeAccountVpcEncryptionControl", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("AccountVpcEncryptionControl contains State, Mode and Exclusions but no stable identifier; requires account/region-context identity rather than hashing mutable settings.".to_owned()),
        ..empty()
    }),
("DescribeAddressTransfers", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("AddressTransfers")?),
        identity: Some(Identity::Key(vec![path("AllocationId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeAddresses", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Addresses")?),
        identity: Some(Identity::Key(vec![path("PublicIp")?])),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("DescribeAddressesAttribute", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Addresses")?),
        identity: Some(Identity::Key(vec![path("AllocationId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeAggregateIdFormat", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Statuses[] needs Resource identity while UseLongIdsAggregated is a separate account-level result; requires independently identified projections to retain both without content-derived identity.".to_owned()),
        ..empty()
    }),
("DescribeApplicationStatus", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("ApplicationStatuses.Instances[] requires nested selection plus instance/application-check identity review; selecting the ApplicationStatuses wrapper does not enumerate instances.".to_owned()),
        ..empty()
    })
    ])
}
