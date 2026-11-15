use crate::overlay::{NameError, OperationOverlay, FixedValue, Identity, ItemSelector, OverlayClass, Paginator, Scope, TokenPair};
use super::super::{empty, path};

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("DescribeTransitGatewayConnectPeers", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("TransitGatewayConnectPeers")?),
        identity: Some(Identity::Key(vec![path("TransitGatewayConnectPeerId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeTransitGatewayConnects", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("TransitGatewayConnects")?),
        identity: Some(Identity::Key(vec![path("TransitGatewayAttachmentId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeTransitGatewayMeteringPolicies", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("TransitGatewayMeteringPolicies")?),
        identity: Some(Identity::Key(vec![path("TransitGatewayMeteringPolicyId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeTransitGatewayMulticastDomains", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("TransitGatewayMulticastDomains")?),
        identity: Some(Identity::Key(vec![path("TransitGatewayMulticastDomainId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeTransitGatewayPeeringAttachments", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("TransitGatewayPeeringAttachments")?),
        identity: Some(Identity::Key(vec![path("TransitGatewayAttachmentId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeTransitGatewayPolicyTables", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("TransitGatewayPolicyTables")?),
        identity: Some(Identity::Key(vec![path("TransitGatewayPolicyTableId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeTransitGatewayRouteTableAnnouncements", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("TransitGatewayRouteTableAnnouncements")?),
        identity: Some(Identity::Key(vec![path("TransitGatewayRouteTableAnnouncementId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeTransitGatewayRouteTables", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("TransitGatewayRouteTables")?),
        identity: Some(Identity::Key(vec![path("TransitGatewayRouteTableId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeTransitGatewayVpcAttachments", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("TransitGatewayVpcAttachments")?),
        identity: Some(Identity::Key(vec![path("TransitGatewayAttachmentId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeTransitGateways", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("TransitGateways")?),
        identity: Some(Identity::Key(vec![path("TransitGatewayId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeTrunkInterfaceAssociations", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("InterfaceAssociations")?),
        identity: Some(Identity::Key(vec![path("AssociationId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeVerifiedAccessEndpoints", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("VerifiedAccessEndpoints")?),
        identity: Some(Identity::Key(vec![path("VerifiedAccessEndpointId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeVerifiedAccessGroups", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("VerifiedAccessGroups")?),
        identity: Some(Identity::Key(vec![path("VerifiedAccessGroupId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeVerifiedAccessInstanceLoggingConfigurations", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("LoggingConfigurations")?),
        identity: Some(Identity::Key(vec![path("VerifiedAccessInstanceId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeVerifiedAccessInstances", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("VerifiedAccessInstances")?),
        identity: Some(Identity::Key(vec![path("VerifiedAccessInstanceId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeVolumeAttribute", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires VolumeId and Attribute; autoEnableIO and productCodes require bounded attribute-input fanout and per-attribute identity to avoid dropping one result.".to_owned()),
        ..empty()
    }),
("DescribeVolumeStatus", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("VolumeStatuses")?),
        fixed: [(path("IncludeManagedResources")?, FixedValue::Boolean(true))].into_iter().collect(),
        identity: Some(Identity::Key(vec![path("VolumeId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeVolumesModifications", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("VolumesModifications")?),
        fixed: [(path("IncludeManagedResources")?, FixedValue::Boolean(true))].into_iter().collect(),
        identity: Some(Identity::Key(vec![path("VolumeId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeVpcAttribute", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires VpcId and Attribute; enableDnsSupport, enableDnsHostnames and enableNetworkAddressUsageMetrics require bounded enum-input fanout and contextual attribute identity.".to_owned()),
        ..empty()
    }),
("DescribeVpcBlockPublicAccessExclusions", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("VpcBlockPublicAccessExclusions")?),
        identity: Some(Identity::Key(vec![path("ExclusionId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    })
    ])
}
