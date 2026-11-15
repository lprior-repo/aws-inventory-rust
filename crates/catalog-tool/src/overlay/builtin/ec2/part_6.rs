use crate::overlay::{NameError, OperationOverlay, Identity, ItemSelector, OverlayClass, Paginator, Scope, TokenPair};
use super::super::{empty, path};
use crate::classes::DenyReason;

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("DescribeNetworkingFlexCostAllocationPolicyAssociations", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("FcaPolicyAssociations")?),
        identity: Some(Identity::Key(vec![path("FcaPolicyAssociationId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeOutpostLags", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("OutpostLags")?),
        identity: Some(Identity::Key(vec![path("OutpostLagId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribePlacementGroups", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("PlacementGroups")?),
        identity: Some(Identity::Key(vec![path("GroupId")?])),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("DescribePrefixLists", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("DescribePrincipalIdFormat", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Principals")?),
        identity: Some(Identity::Key(vec![path("Arn")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribePublicIpv4Pools", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("PublicIpv4Pools")?),
        identity: Some(Identity::Key(vec![path("PoolId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeReplaceRootVolumeTasks", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("ReplaceRootVolumeTasks")?),
        identity: Some(Identity::Key(vec![path("ReplaceRootVolumeTaskId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeReservedInstances", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("ReservedInstances")?),
        identity: Some(Identity::Key(vec![path("ReservedInstancesId")?])),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("DescribeReservedInstancesListings", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("ReservedInstancesListings")?),
        identity: Some(Identity::Key(vec![path("ReservedInstancesListingId")?])),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("DescribeReservedInstancesModifications", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("ReservedInstancesModifications")?),
        identity: Some(Identity::Key(vec![path("ReservedInstancesModificationId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeReservedInstancesOfferings", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("DescribeRouteServerEndpoints", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("RouteServerEndpoints")?),
        identity: Some(Identity::Key(vec![path("RouteServerEndpointId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeRouteServerPeers", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("RouteServerPeers")?),
        identity: Some(Identity::Key(vec![path("RouteServerPeerId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeRouteServers", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("RouteServers")?),
        identity: Some(Identity::Key(vec![path("RouteServerId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeScheduledInstanceAvailability", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("DescribeScheduledInstances", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("ScheduledInstanceSet")?),
        identity: Some(Identity::Key(vec![path("ScheduledInstanceId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeSecondaryInterfaces", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("SecondaryInterfaces")?),
        identity: Some(Identity::Key(vec![path("SecondaryInterfaceId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeSecondaryNetworks", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("SecondaryNetworks")?),
        identity: Some(Identity::Key(vec![path("SecondaryNetworkId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeSecondarySubnets", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("SecondarySubnets")?),
        identity: Some(Identity::Key(vec![path("SecondarySubnetId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeSecurityGroupReferences", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required GroupId is a list of group IDs; SecurityGroupReferenceSet[] also needs referencing VPC/peering/transit gateway conditional identity and source-to-list batching.".to_owned()),
        ..empty()
    })
    ])
}
