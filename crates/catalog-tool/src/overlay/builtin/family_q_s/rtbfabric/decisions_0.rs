// Pinned operation purposes: .local/aws-models/models/rtbfabric/service/2023-05-15/rtbfabric-2023-05-15.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Accepts a link request between gateways. When a requester gateway requests to link with a responder gateway, the responder can use this operation to accept the link request and establish the connection.
    ("AcceptLink", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Associates an ACM certificate with a responder gateway.
    (
        "AssociateCertificate",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an inbound external link.
    (
        "CreateInboundExternalLink",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new link between gateways. Establishes a connection that allows gateways to communicate and exchange bid requests and responses.
    ("CreateLink", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a routing rule for a link. Routing rules use priority-based evaluation where lower priority numbers are evaluated first. Each rule specifies conditions that must all match for the rule to apply.
    (
        "CreateLinkRoutingRule",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an outbound external link.
    (
        "CreateOutboundExternalLink",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a requester gateway.
    (
        "CreateRequesterGateway",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a responder gateway. A domain name or managed endpoint is required.
    (
        "CreateResponderGateway",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an inbound external link.
    (
        "DeleteInboundExternalLink",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a link between gateways. Permanently removes the connection between gateways. This action cannot be undone.
    ("DeleteLink", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a routing rule from a link.
    (
        "DeleteLinkRoutingRule",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an outbound external link.
    (
        "DeleteOutboundExternalLink",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a requester gateway.
    (
        "DeleteRequesterGateway",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a responder gateway.
    (
        "DeleteResponderGateway",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Removes a certificate association from a responder gateway.
    (
        "DisassociateCertificate",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetCertificateAssociation",
        Decision::Deferred(
            "Required selectors gatewayId, acmCertificateArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members gatewayId, acmCertificateArn, status, associatedAt, updatedAt. Pinned purpose: Retrieves the details of a certificate association with a responder gateway.",
        ),
    ),
    (
        "GetInboundExternalLink",
        Decision::Deferred(
            "Required selectors gatewayId, linkId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members gatewayId, linkId, status, domainName, flowModules, pendingFlowModules, attributes, createdAt, updatedAt, tags, logSettings, connectivityType. Pinned purpose: Retrieves information about an inbound external link.",
        ),
    ),
    (
        "GetLink",
        Decision::Deferred(
            "Required selectors gatewayId, linkId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members gatewayId, peerGatewayId, status, createdAt, updatedAt, direction, flowModules, pendingFlowModules, attributes, logSettings, connectivityType, linkId, tags, httpResponderAllowed, timeoutInMillis. Pinned purpose: Retrieves information about a link between gateways. Returns detailed information about the link configuration, status, and associated gateways.",
        ),
    ),
    (
        "GetLinkRoutingRule",
        Decision::Deferred(
            "Required selectors gatewayId, linkId, ruleId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members gatewayId, linkId, ruleId, priority, conditions, status, createdAt, updatedAt, tags. Pinned purpose: Retrieves the details of a routing rule for a link.",
        ),
    ),
    (
        "GetOutboundExternalLink",
        Decision::Deferred(
            "Required selectors gatewayId, linkId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members gatewayId, linkId, status, publicEndpoint, flowModules, pendingFlowModules, attributes, createdAt, updatedAt, tags, logSettings, connectivityType. Pinned purpose: Retrieves information about an outbound external link.",
        ),
    ),
    (
        "GetRequesterGateway",
        Decision::Deferred(
            "Required selectors gatewayId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members status, domainName, description, createdAt, updatedAt, vpcId, subnetIds, securityGroupIds, gatewayId, tags, activeLinksCount, totalLinksCount. Pinned purpose: Retrieves information about a requester gateway.",
        ),
    ),
    (
        "GetResponderGateway",
        Decision::Deferred(
            "Required selectors gatewayId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members vpcId, subnetIds, securityGroupIds, status, description, createdAt, updatedAt, domainName, port, protocol, listenerConfig, trustStoreConfiguration, managedEndpointConfiguration, gatewayId, tags, activeLinksCount, totalLinksCount, linksRequestedCount, gatewayType, externalInboundEndpoint, clientRoutingPolicy. Pinned purpose: Retrieves information about a responder gateway.",
        ),
    ),
    (
        "ListCertificateAssociations",
        Decision::Deferred(
            "Required selectors gatewayId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members certificateAssociations, nextToken. Pinned purpose: Lists the certificate associations for a responder gateway.",
        ),
    ),
    (
        "ListLinkRoutingRules",
        Decision::Deferred(
            "Required selectors gatewayId, linkId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members rules, nextToken. Pinned purpose: Lists the routing rules for a link.",
        ),
    ),
    (
        "ListLinks",
        Decision::Deferred(
            "Required selectors gatewayId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members links, nextToken. Pinned purpose: Lists links associated with gateways. Returns a list of all links for the specified gateways, including their status and configuration details.",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: Lists tags for a resource.",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Rejects a link request between gateways. When a requester gateway requests to link with a responder gateway, the responder can use this operation to decline the link request.
    ("RejectLink", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Assigns one or more tags (key-value pairs) to the specified resource.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes a tag or tags from a resource.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the configuration of a link between gateways. Allows you to modify settings and parameters for an existing link.
    ("UpdateLink", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a link module flow.
    (
        "UpdateLinkModuleFlow",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a routing rule for a link.
    (
        "UpdateLinkRoutingRule",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a requester gateway.
    (
        "UpdateRequesterGateway",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the description, Auto Scaling group managed endpoint configuration, trust store configuration, and client routing policy of a responder gateway. This operation also updates the protocols list in the listener configuration. You cannot change the domainName , port , and protocol values that you set when you create a responder gateway. To change any of them, delete the gateway and create a ne
    (
        "UpdateResponderGateway",
        Decision::Denied(DenyReason::Mutating),
    ),
];
