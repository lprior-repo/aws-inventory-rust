use super::super::{empty, path};
use crate::overlay::{
    FixedValue, Identity, ItemSelector, NameError, NestedItems, OperationOverlay, OverlayClass,
    Paginator, Scope, TokenPair,
};

pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("DescribeVpcs", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Vpcs")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("VpcId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeSubnets", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Subnets")?),
        tags: Some(path("Tags")?),
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
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::Nested(NestedItems {
            parents: path("Reservations")?,
            items: path("Instances")?,
            owner: path("OwnerId")?,
        })),
        page_size: Some(1000),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("InstanceId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeInternetGateways", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("InternetGateways")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("InternetGatewayId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeNatGateways", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("NatGateways")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("NatGatewayId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeNetworkAcls", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("NetworkAcls")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("NetworkAclId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeNetworkInterfaces", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("NetworkInterfaces")?),
        tags: Some(path("TagSet")?),
        fixed: [(path("IncludeManagedResources")?, FixedValue::Boolean(true))].into_iter().collect(),
        identity: Some(Identity::Key(vec![path("NetworkInterfaceId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeRouteTables", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("RouteTables")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("RouteTableId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeSecurityGroups", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("SecurityGroups")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("GroupId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeVolumes", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Volumes")?),
        tags: Some(path("Tags")?),
        fixed: [(path("IncludeManagedResources")?, FixedValue::Boolean(true))].into_iter().collect(),
        identity: Some(Identity::Key(vec![path("VolumeId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeVpcEndpoints", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("VpcEndpoints")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("VpcEndpointId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeVpcPeeringConnections", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("VpcPeeringConnections")?),
        tags: Some(path("Tags")?),
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
        page_size: Some(10),
        identity: Some(Identity::Key(vec![path("AllocationId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeAddresses", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Addresses")?),
        tags: Some(path("Tags")?),
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
