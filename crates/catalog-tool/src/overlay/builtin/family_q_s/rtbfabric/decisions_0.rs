use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("AcceptLink", Decision::Denied(DenyReason::Mutating)),
    (
        "AssociateCertificate",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateInboundExternalLink",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateLink", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateLinkRoutingRule",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateOutboundExternalLink",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateRequesterGateway",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateResponderGateway",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteInboundExternalLink",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteLink", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteLinkRoutingRule",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteOutboundExternalLink",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteRequesterGateway",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteResponderGateway",
        Decision::Denied(DenyReason::Mutating),
    ),
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
    ("RejectLink", Decision::Denied(DenyReason::Mutating)),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateLink", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateLinkModuleFlow",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateLinkRoutingRule",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateRequesterGateway",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateResponderGateway",
        Decision::Denied(DenyReason::Mutating),
    ),
];
