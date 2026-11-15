use crate::overlay::{NameError, OperationOverlay, Binding, FixedValue, Identity, ItemSelector, OverlayClass, Paginator, Scope, TokenPair};
use super::super::{empty, path, name};

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("DescribeLaunchTemplates", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("LaunchTemplates")?),
        fixed: [(path("IncludeManagedResources")?, FixedValue::Boolean(true))].into_iter().collect(),
        identity: Some(Identity::Key(vec![path("LaunchTemplateId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeLocalGatewayRouteTableVirtualInterfaceGroupAssociations", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("LocalGatewayRouteTableVirtualInterfaceGroupAssociations")?),
        identity: Some(Identity::Key(vec![path("LocalGatewayRouteTableVirtualInterfaceGroupAssociationId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeLocalGatewayRouteTableVpcAssociations", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("LocalGatewayRouteTableVpcAssociations")?),
        identity: Some(Identity::Key(vec![path("LocalGatewayRouteTableVpcAssociationId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeLocalGatewayRouteTables", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("LocalGatewayRouteTables")?),
        identity: Some(Identity::Key(vec![path("LocalGatewayRouteTableId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeLocalGatewayVirtualInterfaceGroups", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("LocalGatewayVirtualInterfaceGroups")?),
        identity: Some(Identity::Key(vec![path("LocalGatewayVirtualInterfaceGroupId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeLocalGatewayVirtualInterfaces", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("LocalGatewayVirtualInterfaces")?),
        identity: Some(Identity::Key(vec![path("LocalGatewayVirtualInterfaceId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeLocalGateways", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("LocalGateways")?),
        identity: Some(Identity::Key(vec![path("LocalGatewayId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeLockedSnapshots", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Snapshots")?),
        identity: Some(Identity::Key(vec![path("SnapshotId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeMacHosts", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("MacHosts")?),
        identity: Some(Identity::Key(vec![path("HostId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeMacModificationTasks", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("MacModificationTasks")?),
        identity: Some(Identity::Key(vec![path("MacModificationTaskId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeManagedPrefixLists", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("PrefixLists")?),
        identity: Some(Identity::Key(vec![path("PrefixListId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeMetricCategories", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("MetricCategories")?),
        identity: Some(Identity::Key(vec![path("MetricCategoryId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeMovingAddresses", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("MovingAddressStatuses")?),
        identity: Some(Identity::Key(vec![path("PublicIp")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeNetworkInsightsAccessScopeAnalyses", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("NetworkInsightsAccessScopeAnalyses")?),
        identity: Some(Identity::Key(vec![path("NetworkInsightsAccessScopeAnalysisId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeNetworkInsightsAccessScopes", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("NetworkInsightsAccessScopes")?),
        identity: Some(Identity::Key(vec![path("NetworkInsightsAccessScopeId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeNetworkInsightsAnalyses", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("NetworkInsightsAnalyses")?),
        identity: Some(Identity::Key(vec![path("NetworkInsightsAnalysisId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        bind: [(path("NetworkInsightsPathId")?, Binding::From { operation: name("DescribeNetworkInsightsPaths")?, path: path("NetworkInsightsPathId")? })].into_iter().collect(),
        ..empty()
    }),
("DescribeNetworkInsightsPaths", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("NetworkInsightsPaths")?),
        identity: Some(Identity::Key(vec![path("NetworkInsightsPathId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeNetworkInterfaceAttribute", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("NetworkInterfaceId can bind from DescribeNetworkInterfaces, but Attribute is required by its documentation despite lacking smithy.required; description/groupSet/sourceDestCheck/attachment/associatePublicIpAddress require bounded enum-input fanout rather than one selected attribute.".to_owned()),
        ..empty()
    }),
("DescribeNetworkInterfacePermissions", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("NetworkInterfacePermissions")?),
        identity: Some(Identity::Key(vec![path("NetworkInterfacePermissionId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeNetworkingFlexCostAllocationPolicies", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("FlexCostAllocationPolicies")?),
        identity: Some(Identity::Key(vec![path("FcaPolicyId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    })
    ])
}
