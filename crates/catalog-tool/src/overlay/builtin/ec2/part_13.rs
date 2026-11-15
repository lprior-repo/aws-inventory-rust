use crate::overlay::{NameError, OperationOverlay, Binding, Identity, ItemSelector, OverlayClass, Paginator, Scope, TokenPair};
use super::super::{empty, path, name};
use crate::classes::DenyReason;

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("GetTransitGatewayMeteringPolicyEntries", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Metering policy ID can bind from DescribeTransitGatewayMeteringPolicies, but TransitGatewayMeteringPolicyEntries[] omits the policy parent and needs parent-scoped rule identity.".to_owned()),
        ..empty()
    }),
("GetTransitGatewayMulticastDomainAssociations", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Domain ID can bind from DescribeTransitGatewayMulticastDomains, but MulticastDomainAssociations[] omits it and has nested Subnet identity; requires parent-aware association flattening.".to_owned()),
        ..empty()
    }),
("GetTransitGatewayPolicyTableAssociations", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Associations")?),
        identity: Some(Identity::Key(vec![path("TransitGatewayPolicyTableId")?, path("TransitGatewayAttachmentId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        bind: [(path("TransitGatewayPolicyTableId")?, Binding::From { operation: name("DescribeTransitGatewayPolicyTables")?, path: path("TransitGatewayPolicyTableId")? })].into_iter().collect(),
        ..empty()
    }),
("GetTransitGatewayPolicyTableEntries", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Policy table ID can bind from DescribeTransitGatewayPolicyTables, but TransitGatewayPolicyTableEntries[] omits it and has rule-number/match variants requiring contextual identity.".to_owned()),
        ..empty()
    }),
("GetTransitGatewayPrefixListReferences", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("TransitGatewayPrefixListReferences")?),
        identity: Some(Identity::Key(vec![path("TransitGatewayRouteTableId")?, path("PrefixListId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        bind: [(path("TransitGatewayRouteTableId")?, Binding::From { operation: name("DescribeTransitGatewayRouteTables")?, path: path("TransitGatewayRouteTableId")? })].into_iter().collect(),
        ..empty()
    }),
("GetTransitGatewayRouteTableAssociations", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Route table ID can bind from DescribeTransitGatewayRouteTables, but Associations[].TransitGatewayAttachmentId omits that table; requires table/attachment association identity.".to_owned()),
        ..empty()
    }),
("GetTransitGatewayRouteTablePropagations", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Route table ID can bind from DescribeTransitGatewayRouteTables, but TransitGatewayRouteTablePropagations[] omits the table and includes attachment/announcement variants requiring contextual identity.".to_owned()),
        ..empty()
    }),
("GetVerifiedAccessEndpointPolicy", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Endpoint ID can bind from DescribeVerifiedAccessEndpoints, but PolicyEnabled and PolicyDocument omit it; requires parent-context singleton identity without hashing mutable policy contents.".to_owned()),
        ..empty()
    }),
("GetVerifiedAccessEndpointTargets", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Endpoint ID can bind from DescribeVerifiedAccessEndpoints, but VerifiedAccessEndpointTargets[] contains multiple destination variants; endpoint ID alone would collapse targets and requires variant-aware destination identity.".to_owned()),
        ..empty()
    }),
("GetVerifiedAccessGroupPolicy", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Group ID can bind from DescribeVerifiedAccessGroups, but PolicyEnabled and PolicyDocument omit it; requires parent-context singleton identity without hashing mutable policy contents.".to_owned()),
        ..empty()
    }),
("GetVpcResourcesBlockingEncryptionEnforcement", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("NonCompliantResources")?),
        identity: Some(Identity::Key(vec![path("Id")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        bind: [(path("VpcId")?, Binding::From { operation: name("DescribeVpcs")?, path: path("VpcId")? })].into_iter().collect(),
        ..empty()
    }),
("GetVpnConnectionDeviceSampleConfiguration", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("GetVpnConnectionDeviceTypes", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("GetVpnTunnelReplacementStatus", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires VpnConnectionId and VpnTunnelOutsideIpAddress; complete tunnel discovery currently traverses denied DescribeVpnConnections output containing PSKs, so needs a safe discovery projection first.".to_owned()),
        ..empty()
    }),
("ListImagesInRecycleBin", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Images")?),
        identity: Some(Identity::Key(vec![path("ImageId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("ListSnapshotsInRecycleBin", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Snapshots")?),
        identity: Some(Identity::Key(vec![path("SnapshotId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("ListVolumesInRecycleBin", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Volumes")?),
        identity: Some(Identity::Key(vec![path("VolumeId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("SearchLocalGatewayRoutes", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Local gateway route table ID has a source, but Routes[] can use destination CIDR or DestinationPrefixListId and optional next-hop variants; requires conditional route identity, not one arbitrarily selected route form.".to_owned()),
        ..empty()
    }),
("SearchTransitGatewayMulticastGroups", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Domain ID has a source, but MulticastGroups[] omits it and combines group/source/member/network-interface variants; requires parent-aware multicast membership identity.".to_owned()),
        ..empty()
    }),
("SearchTransitGatewayRoutes", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires TransitGatewayRouteTableId and Filters; a complete unfiltered inventory search is not represented by an arbitrary mandatory filter, and AdditionalRoutesAvailable must be honored independently of NextToken.".to_owned()),
        ..empty()
    })
    ])
}
