// Pinned Smithy source: .local/aws-models/models/networkmanager/service/2019-07-05/networkmanager-2019-07-05.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "AcceptAttachment",
            "AssociateConnectPeer",
            "AssociateCustomerGateway",
            "AssociateLink",
            "AssociateTransitGatewayConnectPeer",
            "CreateConnectAttachment",
            "CreateConnectPeer",
            "CreateConnection",
            "CreateCoreNetwork",
            "CreateCoreNetworkPrefixListAssociation",
            "CreateDevice",
            "CreateDirectConnectGatewayAttachment",
            "CreateGlobalNetwork",
            "CreateLink",
            "CreateSite",
            "CreateSiteToSiteVpnAttachment",
            "CreateTransitGatewayPeering",
            "CreateTransitGatewayRouteTableAttachment",
            "CreateVpcAttachment",
            "DeleteAttachment",
            "DeleteConnectPeer",
            "DeleteConnection",
            "DeleteCoreNetwork",
            "DeleteCoreNetworkPolicyVersion",
            "DeleteCoreNetworkPrefixListAssociation",
            "DeleteDevice",
            "DeleteGlobalNetwork",
            "DeleteLink",
            "DeletePeering",
            "DeleteResourcePolicy",
            "DeleteSite",
            "DeregisterTransitGateway",
            "DisassociateConnectPeer",
            "DisassociateCustomerGateway",
            "DisassociateLink",
            "DisassociateTransitGatewayConnectPeer",
            "ExecuteCoreNetworkChangeSet",
            "PutAttachmentRoutingPolicyLabel",
            "PutCoreNetworkPolicy",
            "PutResourcePolicy",
            "RegisterTransitGateway",
            "RejectAttachment",
            "RemoveAttachmentRoutingPolicyLabel",
            "RestoreCoreNetworkPolicyVersion",
            "StartOrganizationServiceAccessUpdate",
            "StartRouteAnalysis",
            "TagResource",
            "UntagResource",
            "UpdateAttachmentNatConfiguration",
            "UpdateConnection",
            "UpdateCoreNetwork",
            "UpdateDevice",
            "UpdateDirectConnectGatewayAttachment",
            "UpdateGlobalNetwork",
            "UpdateLink",
            "UpdateNetworkResourceMetadata",
            "UpdateSite",
            "UpdateVpcAttachment",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetConnectAttachment",
            "GetDirectConnectGatewayAttachment",
            "GetSiteToSiteVpnAttachment",
            "GetTransitGatewayRouteTableAttachment",
            "GetVpcAttachment",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (AttachmentId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetConnectPeer"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ConnectPeerId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "GetConnectPeerAssociations",
            "GetCustomerGatewayAssociations",
            "GetLinkAssociations",
            "GetNetworkResourceCounts",
            "GetNetworkResourceRelationships",
            "GetNetworkResources",
            "GetNetworkTelemetry",
            "GetTransitGatewayConnectPeerAssociations",
            "GetTransitGatewayRegistrations",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (GlobalNetworkId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "GetCoreNetwork",
            "GetCoreNetworkPolicy",
            "ListAttachmentRoutingPolicyAssociations",
            "ListCoreNetworkPolicyVersions",
            "ListCoreNetworkPrefixListAssociations",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (CoreNetworkId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetCoreNetworkChangeEvents", "GetCoreNetworkChangeSet"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (CoreNetworkId, PolicyVersionId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetNetworkRoutes"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (GlobalNetworkId, RouteTableIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetResourcePolicy", "ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ResourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetRouteAnalysis"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (GlobalNetworkId, RouteAnalysisId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetTransitGatewayPeering"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (PeeringId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "ListAttachmentNatConfigurations",
            "ListOrganizationServiceAccessStatus",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract.",
        ),
    },
    Group {
        operations: &["ListCoreNetworkRoutingInformation"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (CoreNetworkId, SegmentName, EdgeLocation); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListCoreNetworks"],
        class: OverlayClass::Defer,
        reason: Some(
            "The modeled collection includes owned and shared core networks without an account-owner request filter; no owned-only selector is available.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    // Describes one or more global networks. By default, all global networks are described. To describe the objects in your global network, you must use the appropriate Get* action. For
    Resource {
        operation: "DescribeGlobalNetworks",
        items: "GlobalNetworks",
        id: Id::Arn("GlobalNetworkArn"),
        scope: Scope::Account,
        tokens: Some(("NextToken", "NextToken")),
        query: &["GlobalNetworkArn", "GlobalNetworkId", "State"],
        tags: Some("Tags"),
        bindings: &[],
        fixed: &[],
    },
    // Returns a list of core network attachments.
    Resource {
        operation: "ListAttachments",
        items: "Attachments",
        id: Id::Key(&["AttachmentId"]),
        scope: Scope::Account,
        tokens: Some(("NextToken", "NextToken")),
        query: &[
            "AttachmentId",
            "CoreNetworkId",
            "CoreNetworkArn",
            "OwnerAccountId",
            "AttachmentType",
            "State",
            "ResourceArn",
            "SegmentName",
            "NetworkFunctionGroupName",
        ],
        tags: Some("Tags"),
        bindings: &[],
        fixed: &[],
    },
    // Returns a list of core network Connect peers.
    Resource {
        operation: "ListConnectPeers",
        items: "ConnectPeers",
        id: Id::Key(&["ConnectPeerId"]),
        scope: Scope::Account,
        tokens: Some(("NextToken", "NextToken")),
        query: &[
            "ConnectPeerId",
            "CoreNetworkId",
            "ConnectAttachmentId",
            "ConnectPeerState",
            "SubnetArn",
        ],
        tags: Some("Tags"),
        bindings: &[],
        fixed: &[],
    },
    // Lists the peerings for a core network.
    Resource {
        operation: "ListPeerings",
        items: "Peerings",
        id: Id::Key(&["PeeringId"]),
        scope: Scope::Account,
        tokens: Some(("NextToken", "NextToken")),
        query: &[
            "PeeringId",
            "CoreNetworkId",
            "CoreNetworkArn",
            "OwnerAccountId",
            "PeeringType",
            "State",
            "ResourceArn",
        ],
        tags: Some("Tags"),
        bindings: &[],
        fixed: &[],
    },
    // Gets information about one or more of your sites in a global network.
    Resource {
        operation: "GetSites",
        items: "Sites",
        id: Id::Arn("SiteArn"),
        scope: Scope::Account,
        tokens: Some(("NextToken", "NextToken")),
        query: &["SiteArn", "SiteId", "GlobalNetworkId", "State"],
        tags: Some("Tags"),
        bindings: &[(
            "GlobalNetworkId",
            "DescribeGlobalNetworks",
            "GlobalNetworkId",
        )],
        fixed: &[],
    },
    // Gets information about one or more of your devices in a global network.
    Resource {
        operation: "GetDevices",
        items: "Devices",
        id: Id::Arn("DeviceArn"),
        scope: Scope::Account,
        tokens: Some(("NextToken", "NextToken")),
        query: &[
            "DeviceArn",
            "DeviceId",
            "GlobalNetworkId",
            "Type",
            "SiteId",
            "State",
        ],
        tags: Some("Tags"),
        bindings: &[(
            "GlobalNetworkId",
            "DescribeGlobalNetworks",
            "GlobalNetworkId",
        )],
        fixed: &[],
    },
    // Gets information about one or more links in a specified global network. If you specify the site ID, you cannot specify the type or provider in the same request. You can specify the
    Resource {
        operation: "GetLinks",
        items: "Links",
        id: Id::Arn("LinkArn"),
        scope: Scope::Account,
        tokens: Some(("NextToken", "NextToken")),
        query: &[
            "LinkArn",
            "LinkId",
            "GlobalNetworkId",
            "SiteId",
            "Type",
            "State",
        ],
        tags: Some("Tags"),
        bindings: &[(
            "GlobalNetworkId",
            "DescribeGlobalNetworks",
            "GlobalNetworkId",
        )],
        fixed: &[],
    },
    // Gets information about one or more of your connections in a global network.
    Resource {
        operation: "GetConnections",
        items: "Connections",
        id: Id::Arn("ConnectionArn"),
        scope: Scope::Account,
        tokens: Some(("NextToken", "NextToken")),
        query: &[
            "ConnectionArn",
            "ConnectionId",
            "GlobalNetworkId",
            "DeviceId",
            "ConnectedDeviceId",
            "LinkId",
            "ConnectedLinkId",
            "State",
        ],
        tags: Some("Tags"),
        bindings: &[(
            "GlobalNetworkId",
            "DescribeGlobalNetworks",
            "GlobalNetworkId",
        )],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("networkmanager", GROUPS, RESOURCES)
}
