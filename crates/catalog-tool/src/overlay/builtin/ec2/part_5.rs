use super::super::{empty, name, path};
use crate::overlay::{
    Binding, FixedValue, Identity, ItemSelector, NameError, OperationOverlay, OverlayClass,
    Paginator, Scope, TokenPair,
};

pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("DescribeLaunchTemplates", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("LaunchTemplates")?),
        tags: Some(path("Tags")?),
        fixed: [(path("IncludeManagedResources")?, FixedValue::Boolean(true))].into_iter().collect(),
        identity: Some(Identity::Key(vec![path("LaunchTemplateId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeLocalGatewayRouteTableVirtualInterfaceGroupAssociations", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("LocalGatewayRouteTableVirtualInterfaceGroupAssociations")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("LocalGatewayRouteTableVirtualInterfaceGroupAssociationId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeLocalGatewayRouteTableVpcAssociations", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("LocalGatewayRouteTableVpcAssociations")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("LocalGatewayRouteTableVpcAssociationId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeLocalGatewayRouteTables", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("LocalGatewayRouteTables")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("LocalGatewayRouteTableId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeLocalGatewayVirtualInterfaceGroups", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("LocalGatewayVirtualInterfaceGroups")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("LocalGatewayVirtualInterfaceGroupId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeLocalGatewayVirtualInterfaces", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("LocalGatewayVirtualInterfaces")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("LocalGatewayVirtualInterfaceId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeLocalGateways", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("LocalGateways")?),
        tags: Some(path("Tags")?),
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
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("MacModificationTaskId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeManagedPrefixLists", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("PrefixLists")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("PrefixListId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeMetricCategories", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("MetricCategories")?),
        tags: Some(path("Tags")?),
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
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("NetworkInsightsAccessScopeAnalysisId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeNetworkInsightsAccessScopes", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("NetworkInsightsAccessScopes")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("NetworkInsightsAccessScopeId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeNetworkInsightsAnalyses", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("NetworkInsightsAnalyses")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("NetworkInsightsAnalysisId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        bind: [(path("NetworkInsightsPathId")?, Binding::From { operation: name("DescribeNetworkInsightsPaths")?, path: path("NetworkInsightsPathId")? })].into_iter().collect(),
        ..empty()
    }),
("DescribeNetworkInsightsPaths", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("NetworkInsightsPaths")?),
        tags: Some(path("Tags")?),
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
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("FcaPolicyId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    })
    ])
}
