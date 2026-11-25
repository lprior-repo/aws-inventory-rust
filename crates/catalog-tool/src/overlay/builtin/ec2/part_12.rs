use super::super::{empty, name, path};
use crate::classes::DenyReason;
use crate::overlay::{
    Binding, Identity, ItemSelector, NameError, OperationOverlay, OverlayClass, Paginator, Scope,
    TokenPair,
};

pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("GetIpamRoutingPolicyRegistrationDeltas", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required registry association ID has a source, but IpamRoutingPolicyRegistrationDeltas[].DeltaId needs registry context and stable change-history semantics.".to_owned()),
        ..empty()
    }),
("GetIpamRoutingPolicyRegistrations", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required registry association ID has a source, but IpamRoutingPolicyRegistrations[].Cidr omits its registry parent; requires association/CIDR/ASN composite identity.".to_owned()),
        ..empty()
    }),
("GetLaunchTemplateData", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetManagedPrefixListAssociations", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("PrefixListId can bind from DescribeManagedPrefixLists, but PrefixListAssociations[].ResourceId omits PrefixListId; needs parent-aware association identity and ResourceOwner propagation.".to_owned()),
        ..empty()
    }),
("GetManagedPrefixListEntries", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("PrefixListId can bind from DescribeManagedPrefixLists, but Entries[].Cidr omits PrefixListId; using CIDR alone collides across lists and identity cannot reference request context.".to_owned()),
        ..empty()
    }),
("GetManagedResourceVisibility", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Visibility is mutable regional account configuration without an output identifier; requires account/region-context singleton identity.".to_owned()),
        ..empty()
    }),
("GetNetworkInsightsAccessScopeAnalysisFindings", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("AnalysisFindings")?),
        identity: Some(Identity::Key(vec![path("NetworkInsightsAccessScopeAnalysisId")?, path("FindingId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        bind: [(path("NetworkInsightsAccessScopeAnalysisId")?, Binding::From { operation: name("DescribeNetworkInsightsAccessScopeAnalyses")?, path: path("NetworkInsightsAccessScopeAnalysisId")? })].into_iter().collect(),
        ..empty()
    }),
("GetNetworkInsightsAccessScopeContent", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("$")?),
        identity: Some(Identity::Key(vec![path("NetworkInsightsAccessScopeContent.NetworkInsightsAccessScopeId")?])),
        paginator: Some(Paginator::None),
        bind: [(path("NetworkInsightsAccessScopeId")?, Binding::From { operation: name("DescribeNetworkInsightsAccessScopes")?, path: path("NetworkInsightsAccessScopeId")? })].into_iter().collect(),
        ..empty()
    }),
("GetNetworkingFlexCostAllocationPolicyVersions", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("FcaPolicyVersions")?),
        identity: Some(Identity::Key(vec![path("FcaPolicyId")?, path("Version")?])),
        paginator: Some(Paginator::None),
        bind: [(path("FcaPolicyId")?, Binding::From { operation: name("DescribeNetworkingFlexCostAllocationPolicies")?, path: path("FcaPolicyId")? })].into_iter().collect(),
        ..empty()
    }),
("GetReservedInstancesExchangeQuote", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("GetRouteServerAssociations", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("RouteServerAssociations")?),
        identity: Some(Identity::Key(vec![path("RouteServerId")?, path("VpcId")?])),
        paginator: Some(Paginator::None),
        bind: [(path("RouteServerId")?, Binding::From { operation: name("DescribeRouteServers")?, path: path("RouteServerId")? })].into_iter().collect(),
        ..empty()
    }),
("GetRouteServerPropagations", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("RouteServerPropagations")?),
        identity: Some(Identity::Key(vec![path("RouteServerId")?, path("RouteTableId")?])),
        paginator: Some(Paginator::None),
        bind: [(path("RouteServerId")?, Binding::From { operation: name("DescribeRouteServers")?, path: path("RouteServerId")? })].into_iter().collect(),
        ..empty()
    }),
("GetRouteServerRoutingDatabase", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("RouteServerId can bind from DescribeRouteServers, but Routes[] omits it and contains route/source-path variants; requires parent-aware route identity and full multi-page handling.".to_owned()),
        ..empty()
    }),
("GetSecurityGroupsForVpc", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("SecurityGroupForVpcs")?),
        identity: Some(Identity::Key(vec![path("GroupId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        bind: [(path("VpcId")?, Binding::From { operation: name("DescribeVpcs")?, path: path("VpcId")? })].into_iter().collect(),
        ..empty()
    }),
("GetSerialConsoleAccessStatus", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("SerialConsoleAccessEnabled and ManagedBy are regional account settings without an output identifier; requires account/region-context singleton identity.".to_owned()),
        ..empty()
    }),
("GetSnapshotBlockPublicAccessState", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("State and ManagedBy are regional account settings without an output identifier; requires account/region-context singleton identity.".to_owned()),
        ..empty()
    }),
("GetSpotPlacementScores", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("GetSubnetCidrReservations", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("SubnetId can bind from DescribeSubnets, but output splits SubnetIpv4CidrReservations[] and SubnetIpv6CidrReservations[]; one items selector cannot retain both address families.".to_owned()),
        ..empty()
    }),
("GetTransitGatewayAttachmentNatConfiguration", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("$")?),
        identity: Some(Identity::Key(vec![path("TransitGatewayAttachmentId")?])),
        paginator: Some(Paginator::None),
        bind: [(path("TransitGatewayAttachmentId")?, Binding::From { operation: name("DescribeTransitGatewayAttachments")?, path: path("TransitGatewayAttachmentId")? })].into_iter().collect(),
        ..empty()
    }),
("GetTransitGatewayAttachmentPropagations", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Attachment ID can bind from DescribeTransitGatewayAttachments, but TransitGatewayAttachmentPropagations[].TransitGatewayRouteTableId omits the attachment parent required for association identity.".to_owned()),
        ..empty()
    })
    ])
}
