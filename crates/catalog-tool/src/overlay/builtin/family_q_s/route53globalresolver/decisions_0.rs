use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "AssociateHostedZone",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchCreateFirewallRule",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchDeleteFirewallRule",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchUpdateFirewallRule",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateAccessSource", Decision::Denied(DenyReason::Mutating)),
    ("CreateAccessToken", Decision::Denied(DenyReason::Mutating)),
    ("CreateDNSView", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateFirewallDomainList",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateFirewallRule", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateGlobalResolver",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteAccessSource", Decision::Denied(DenyReason::Mutating)),
    ("DeleteAccessToken", Decision::Denied(DenyReason::Mutating)),
    ("DeleteDNSView", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteFirewallDomainList",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteFirewallRule", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteGlobalResolver",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DisableDNSView", Decision::Denied(DenyReason::Mutating)),
    (
        "DisassociateHostedZone",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("EnableDNSView", Decision::Denied(DenyReason::Mutating)),
    (
        "GetAccessSource",
        Decision::Deferred(
            "Required selectors accessSourceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members arn, cidr, createdAt, id, ipAddressType, name, dnsViewId, protocol, status, updatedAt. Pinned purpose: Retrieves information about an access source. Route 53 Global Resolver is a global service that supports resolvers in multiple Amazon Web Services Regions but you must specify the US East (Ohio) Region to create, update, or otherwise work with Route ",
        ),
    ),
    (
        "GetAccessToken",
        Decision::Deferred(
            "Required selectors accessTokenId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members id, arn, clientToken, createdAt, dnsViewId, expiresAt, globalResolverId, name, status, updatedAt, value. Pinned purpose: Retrieves information about an access token. Route 53 Global Resolver is a global service that supports resolvers in multiple Amazon Web Services Regions but you must specify the US East (Ohio) Region to create, update, or otherwise work with Route 5",
        ),
    ),
    (
        "GetDNSView",
        Decision::Deferred(
            "Required selectors dnsViewId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members id, arn, clientToken, dnssecValidation, ednsClientSubnet, firewallRulesFailOpen, name, description, globalResolverId, createdAt, updatedAt, status. Pinned purpose: Retrieves information about a DNS view. Route 53 Global Resolver is a global service that supports resolvers in multiple Amazon Web Services Regions but you must specify the US East (Ohio) Region to create, update, or otherwise work with Route 53 Glo",
        ),
    ),
    (
        "GetFirewallDomainList",
        Decision::Deferred(
            "Required selectors firewallDomainListId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members arn, globalResolverId, clientToken, createdAt, description, domainCount, id, name, status, statusMessage, updatedAt. Pinned purpose: Retrieves information about a firewall domain list. Route 53 Global Resolver is a global service that supports resolvers in multiple Amazon Web Services Regions but you must specify the US East (Ohio) Region to create, update, or otherwise work with ",
        ),
    ),
    (
        "GetFirewallRule",
        Decision::Deferred(
            "Required selectors firewallRuleId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members action, blockOverrideDnsType, blockOverrideDomain, blockOverrideTtl, blockResponse, confidenceThreshold, createdAt, description, dnsAdvancedProtection, firewallDomainListId, id, name, priority, dnsViewId, queryType, status, updatedAt. Pinned purpose: Retrieves information about a DNS firewall rule. Route 53 Global Resolver is a global service that supports resolvers in multiple Amazon Web Services Regions but you must specify the US East (Ohio) Region to create, update, or otherwise work with Rou",
        ),
    ),
    (
        "GetGlobalResolver",
        Decision::Deferred(
            "Required selectors globalResolverId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members id, arn, clientToken, dnsName, observabilityRegion, name, description, regions, createdAt, updatedAt, status, ipv4Addresses, ipv6Addresses, ipAddressType. Pinned purpose: Retrieves information about a Route 53 Global Resolver instance. Route 53 Global Resolver is a global service that supports resolvers in multiple Amazon Web Services Regions but you must specify the US East (Ohio) Region to create, update, or otherwi",
        ),
    ),
    (
        "GetHostedZoneAssociation",
        Decision::Deferred(
            "Required selectors hostedZoneAssociationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members id, resourceArn, hostedZoneId, hostedZoneName, name, createdAt, updatedAt, status. Pinned purpose: Retrieves information about a hosted zone association. Route 53 Global Resolver is a global service that supports resolvers in multiple Amazon Web Services Regions but you must specify the US East (Ohio) Region to create, update, or otherwise work wi",
        ),
    ),
    (
        "GetManagedFirewallDomainList",
        Decision::Deferred(
            "Required selectors managedFirewallDomainListId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members description, id, name, managedListType. Pinned purpose: Retrieves information about an Amazon Web Services-managed firewall domain list. Managed domain lists contain domains associated with malicious activity, content categories, or specific threats. Route 53 Global Resolver is a global service that suppo",
        ),
    ),
    (
        "ImportFirewallDomains",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ListAccessTokens",
        Decision::Deferred(
            "Required selectors dnsViewId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members nextToken, accessTokens. Pinned purpose: Lists all access tokens for a DNS view with pagination support. Route 53 Global Resolver is a global service that supports resolvers in multiple Amazon Web Services Regions but you must specify the US East (Ohio) Region to create, update, or otherwis",
        ),
    ),
    (
        "ListFirewallDomains",
        Decision::Deferred(
            "Required selectors firewallDomainListId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members nextToken, domains. Pinned purpose: Lists all the domains in DNS Firewall domain list you have created. Route 53 Global Resolver is a global service that supports resolvers in multiple Amazon Web Services Regions but you must specify the US East (Ohio) Region to create, update, or othe",
        ),
    ),
    (
        "ListManagedFirewallDomainLists",
        Decision::Deferred(
            "Required selectors managedFirewallDomainListType are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members nextToken, managedFirewallDomainLists. Pinned purpose: Returns a paginated list of the Amazon Web Services Managed DNS Lists and the categories for DNS Firewall. The categories are either THREAT or CONTENT . Route 53 Global Resolver is a global service that supports resolvers in multiple Amazon Web Servi",
        ),
    ),
    (
        "ListSharedDNSViews",
        Decision::Deferred(
            "Shared DNS views are owned by another account and require fixed us-east-2 routing. Cannot relabel each foreign DNS view as a caller-owned regional resource.",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: Lists the tags associated with a Route 53 Global Resolver resource. Route 53 Global Resolver is a global service that supports resolvers in multiple Amazon Web Services Regions but you must specify the US East (Ohio) Region to create, update, or otherwise work with Route 53 Global Resolver resources",
        ),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateAccessSource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateAccessToken", Decision::Denied(DenyReason::Mutating)),
    ("UpdateDNSView", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateFirewallDomains",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateFirewallRule", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateGlobalResolver",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateHostedZoneAssociation",
        Decision::Denied(DenyReason::Mutating),
    ),
];
