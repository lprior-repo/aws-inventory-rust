use super::super::{empty, name, path};
use crate::overlay::{
    Binding, Identity, ItemSelector, NameError, OperationOverlay, OverlayClass, Paginator, Scope,
    TokenPair,
};

pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("DescribeClientVpnAuthorizationRules", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("AuthorizationRules[] identity needs ClientVpnEndpointId, DestinationCidr and either GroupId or AccessAll; all-users rules omit GroupId and overlay cannot express conditional identity.".to_owned()),
        ..empty()
    }),
("DescribeClientVpnConnections", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Connections")?),
        identity: Some(Identity::Key(vec![path("ClientVpnEndpointId")?, path("ConnectionId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        bind: [(path("ClientVpnEndpointId")?, Binding::From { operation: name("DescribeClientVpnEndpoints")?, path: path("ClientVpnEndpointId")? })].into_iter().collect(),
        ..empty()
    }),
("DescribeClientVpnEndpoints", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("ClientVpnEndpoints")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("ClientVpnEndpointId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeClientVpnRoutes", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Routes[] identity depends on ClientVpnEndpointId, DestinationCidr and either TargetSubnet or TransitGatewayAttachmentId; requires conditional identity for both route target kinds.".to_owned()),
        ..empty()
    }),
("DescribeClientVpnTargetNetworks", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("ClientVpnTargetNetworks")?),
        identity: Some(Identity::Key(vec![path("AssociationId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        bind: [(path("ClientVpnEndpointId")?, Binding::From { operation: name("DescribeClientVpnEndpoints")?, path: path("ClientVpnEndpointId")? })].into_iter().collect(),
        ..empty()
    }),
("DescribeCoipPools", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("CoipPools")?),
        tags: Some(path("Tags")?),
        page_size: Some(255),
        identity: Some(Identity::Key(vec![path("PoolId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeCustomerGateways", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("CustomerGateways")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("CustomerGatewayId")?])),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("DescribeDeclarativePoliciesReports", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Reports")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("ReportId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeDhcpOptions", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("DhcpOptions")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("DhcpOptionsId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeEgressOnlyInternetGateways", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("EgressOnlyInternetGateways")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("EgressOnlyInternetGatewayId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeElasticGpus", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("ElasticGpuSet")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("ElasticGpuId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeExportImageTasks", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("ExportImageTasks")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("ExportImageTaskId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeExportTasks", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("ExportTasks")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("ExportTaskId")?])),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("DescribeFastLaunchImages", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("FastLaunchImages")?),
        identity: Some(Identity::Key(vec![path("ImageId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeFastSnapshotRestores", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("FastSnapshotRestores")?),
        identity: Some(Identity::Key(vec![path("SnapshotId")?, path("AvailabilityZone")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeFleetHistory", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required FleetId lacks a complete discovery source because DescribeFleets omits instant fleets without supplied IDs; StartTime additionally needs a history-window policy and HistoryRecords[] lacks a stable event ID.".to_owned()),
        ..empty()
    }),
("DescribeFleetInstances", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required FleetId has only a parent DescribeFleets that cannot fully enumerate instant fleets without supplied IDs; ActiveInstances[] also omits the fleet context needed to retain memberships.".to_owned()),
        ..empty()
    }),
("DescribeFleets", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("FleetIds is model-optional, but its documentation explicitly excludes instant fleets unless their IDs are supplied; no complete instant-fleet ID enumeration source exists, so enabling an unbound list would silently omit that fleet class.".to_owned()),
        ..empty()
    }),
("DescribeFlowLogs", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("FlowLogs")?),
        tags: Some(path("Tags")?),
        identity: Some(Identity::Key(vec![path("FlowLogId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeFpgaImageAttribute", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires FpgaImageId and Attribute; safe permission/product-code/description attributes require enum-input fanout and contextual singleton identity rather than selecting one attribute only.".to_owned()),
        ..empty()
    })
    ])
}
