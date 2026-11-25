use super::super::{empty, name, path};
use crate::classes::DenyReason;
use crate::overlay::{
    Binding, Identity, ItemSelector, NameError, OperationOverlay, OverlayClass, Paginator, Scope,
    TokenPair,
};

pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("DescribeVpcBlockPublicAccessOptions", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("$")?),
        identity: Some(Identity::Key(vec![path("VpcBlockPublicAccessOptions.AwsAccountId")?, path("VpcBlockPublicAccessOptions.AwsRegion")?])),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("DescribeVpcClassicLink", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Vpcs")?),
        identity: Some(Identity::Key(vec![path("VpcId")?])),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("DescribeVpcClassicLinkDnsSupport", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Vpcs")?),
        identity: Some(Identity::Key(vec![path("VpcId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeVpcEncryptionControls", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("VpcEncryptionControls")?),
        identity: Some(Identity::Key(vec![path("VpcEncryptionControlId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeVpcEndpointAssociations", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("VpcEndpointAssociations")?),
        identity: Some(Identity::Key(vec![path("Id")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeVpcEndpointConnectionNotifications", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("ConnectionNotificationSet")?),
        identity: Some(Identity::Key(vec![path("ConnectionNotificationId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeVpcEndpointConnections", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("VpcEndpointConnections")?),
        identity: Some(Identity::Key(vec![path("VpcEndpointConnectionId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeVpcEndpointServiceConfigurations", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("ServiceConfigurations")?),
        identity: Some(Identity::Key(vec![path("ServiceId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeVpcEndpointServicePermissions", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("AllowedPrincipals")?),
        identity: Some(Identity::Key(vec![path("ServiceId")?, path("ServicePermissionId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        bind: [(path("ServiceId")?, Binding::From { operation: name("DescribeVpcEndpointServiceConfigurations")?, path: path("ServiceId")? })].into_iter().collect(),
        ..empty()
    }),
("DescribeVpcEndpointServices", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("DescribeVpnConcentrators", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("VpnConcentrators")?),
        identity: Some(Identity::Key(vec![path("VpnConcentratorId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeVpnConnections", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("DescribeVpnGateways", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("VpnGateways")?),
        identity: Some(Identity::Key(vec![path("VpnGatewayId")?])),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("GetActiveVpnTunnelStatus", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires VpnConnectionId and VpnTunnelOutsideIpAddress; the only complete tunnel discovery parent DescribeVpnConnections exposes PSKs and remains denied, so a safe tunnel-ID projection is required.".to_owned()),
        ..empty()
    }),
("GetAllowedImagesSettings", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("State, ImageCriteria[] and ManagedBy describe an account/region singleton without an output resource identifier; requires context-derived identity.".to_owned()),
        ..empty()
    }),
("GetAssociatedEnclaveCertificateIamRoles", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires CertificateArn from ACM/IAM rather than an EC2 enumeration; AssociatedRoles[] also omits the certificate parent needed for association identity.".to_owned()),
        ..empty()
    }),
("GetAssociatedIpv6PoolCidrs", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("PoolId can bind from DescribeIpv6Pools.PoolId but Ipv6CidrAssociations[] omits PoolId; requires parent-context association identity.".to_owned()),
        ..empty()
    }),
("GetAwsNetworkPerformanceData", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetCapacityManagerAttributes", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("CapacityManagerStatus, OrganizationsAccess and IngestionStatus are mutable account settings without an output identifier; requires contextual singleton identity and home-region handling.".to_owned()),
        ..empty()
    }),
("GetCapacityManagerMetricData", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    })
    ])
}
