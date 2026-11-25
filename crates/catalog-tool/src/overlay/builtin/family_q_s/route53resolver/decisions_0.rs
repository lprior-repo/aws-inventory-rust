use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "AssociateFirewallRuleGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "AssociateResolverEndpointIpAddress",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "AssociateResolverQueryLogConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "AssociateResolverRule",
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
    (
        "CreateFirewallDomainList",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateFirewallRule", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateFirewallRuleGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateOutpostResolver",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateResolverEndpoint",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateResolverQueryLogConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateResolverRule", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteFirewallDomainList",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteFirewallRule", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteFirewallRuleGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteOutpostResolver",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteResolverEndpoint",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteResolverQueryLogConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteResolverRule", Decision::Denied(DenyReason::Mutating)),
    (
        "DisassociateFirewallRuleGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisassociateResolverEndpointIpAddress",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisassociateResolverQueryLogConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisassociateResolverRule",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetFirewallConfig",
        Decision::Deferred(
            "Required selectors ResourceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members FirewallConfig. Pinned purpose: Retrieves the configuration of the firewall behavior provided by DNS Firewall for a single VPC from Amazon Virtual Private Cloud (Amazon VPC).",
        ),
    ),
    (
        "GetFirewallDomainList",
        Decision::Deferred(
            "Required selectors FirewallDomainListId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members FirewallDomainList. Pinned purpose: Retrieves the specified firewall domain list.",
        ),
    ),
    (
        "GetFirewallRuleGroup",
        Decision::Deferred(
            "Required selectors FirewallRuleGroupId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members FirewallRuleGroup. Pinned purpose: Retrieves the specified firewall rule group.",
        ),
    ),
    (
        "GetFirewallRuleGroupAssociation",
        Decision::Deferred(
            "Required selectors FirewallRuleGroupAssociationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members FirewallRuleGroupAssociation. Pinned purpose: Retrieves a firewall rule group association, which enables DNS filtering for a VPC with one rule group. A VPC can have more than one firewall rule group association, and a rule group can be associated with more than one VPC.",
        ),
    ),
    (
        "GetFirewallRuleGroupPolicy",
        Decision::Deferred(
            "Required selectors Arn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members FirewallRuleGroupPolicy. Pinned purpose: Returns the Identity and Access Management (Amazon Web Services IAM) policy for sharing the specified rule group. You can use the policy to share the rule group using Resource Access Manager (RAM).",
        ),
    ),
    (
        "GetOutpostResolver",
        Decision::Deferred(
            "Required selectors Id are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members OutpostResolver. Pinned purpose: Gets information about a specified Resolver on the Outpost, such as its instance count and type, name, and the current status of the Resolver.",
        ),
    ),
    (
        "GetResolverConfig",
        Decision::Deferred(
            "Required selectors ResourceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ResolverConfig. Pinned purpose: Retrieves the behavior configuration of Route 53 Resolver behavior for a single VPC from Amazon Virtual Private Cloud.",
        ),
    ),
    (
        "GetResolverDnssecConfig",
        Decision::Deferred(
            "Required selectors ResourceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ResolverDNSSECConfig. Pinned purpose: Gets DNSSEC validation information for a specified resource.",
        ),
    ),
    (
        "GetResolverEndpoint",
        Decision::Deferred(
            "Required selectors ResolverEndpointId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ResolverEndpoint. Pinned purpose: Gets information about a specified Resolver endpoint, such as whether it's an inbound or an outbound Resolver endpoint, and the current status of the endpoint.",
        ),
    ),
    (
        "GetResolverQueryLogConfig",
        Decision::Deferred(
            "Required selectors ResolverQueryLogConfigId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ResolverQueryLogConfig. Pinned purpose: Gets information about a specified Resolver query logging configuration, such as the number of VPCs that the configuration is logging queries for and the location that logs are sent to.",
        ),
    ),
    (
        "GetResolverQueryLogConfigAssociation",
        Decision::Deferred(
            "Required selectors ResolverQueryLogConfigAssociationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ResolverQueryLogConfigAssociation. Pinned purpose: Gets information about a specified association between a Resolver query logging configuration and an Amazon VPC. When you associate a VPC with a query logging configuration, Resolver logs DNS queries that originate in that VPC.",
        ),
    ),
    (
        "GetResolverQueryLogConfigPolicy",
        Decision::Deferred(
            "Required selectors Arn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ResolverQueryLogConfigPolicy. Pinned purpose: Gets information about a query logging policy. A query logging policy specifies the Resolver query logging operations and resources that you want to allow another Amazon Web Services account to be able to use.",
        ),
    ),
    (
        "GetResolverRule",
        Decision::Deferred(
            "Required selectors ResolverRuleId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ResolverRule. Pinned purpose: Gets information about a specified Resolver rule, such as the domain name that the rule forwards DNS queries for and the ID of the outbound Resolver endpoint that the rule is associated with.",
        ),
    ),
    (
        "GetResolverRuleAssociation",
        Decision::Deferred(
            "Required selectors ResolverRuleAssociationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ResolverRuleAssociation. Pinned purpose: Gets information about an association between a specified Resolver rule and a VPC. You associate a Resolver rule and a VPC using AssociateResolverRule .",
        ),
    ),
    (
        "GetResolverRulePolicy",
        Decision::Deferred(
            "Required selectors Arn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ResolverRulePolicy. Pinned purpose: Gets information about the Resolver rule policy for a specified rule. A Resolver rule policy includes the rule that you want to share with another account, the account that you want to share the rule with, and the Resolver operations that you want to",
        ),
    ),
    (
        "ImportFirewallDomains",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ListFirewallDomains",
        Decision::Deferred(
            "Required selectors FirewallDomainListId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NextToken, Domains. Pinned purpose: Retrieves the domains that you have defined for the specified firewall domain list. A single call might return only a partial list of the domains. For information, see MaxResults .",
        ),
    ),
    (
        "ListFirewallRuleTypes",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "ListFirewallRules",
        Decision::Deferred(
            "Required selectors FirewallRuleGroupId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NextToken, FirewallRules. Pinned purpose: Retrieves the firewall rules that you have defined for the specified firewall rule group. DNS Firewall uses the rules in a rule group to filter DNS network traffic for a VPC. A single call might return only a partial list of the rules. For informatio",
        ),
    ),
    (
        "ListResolverEndpointIpAddresses",
        Decision::Deferred(
            "Required selectors ResolverEndpointId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NextToken, MaxResults, IpAddresses. Pinned purpose: Gets the IP addresses for a specified Resolver endpoint.",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: ResourceArn. Pinned purpose: Lists the tags that you associated with the specified resource.",
        ),
    ),
    (
        "PutFirewallRuleGroupPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutResolverQueryLogConfigPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutResolverRulePolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateFirewallConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateFirewallDomains",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateFirewallRule", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateFirewallRuleGroupAssociation",
        Decision::Denied(DenyReason::Mutating),
    ),
];
