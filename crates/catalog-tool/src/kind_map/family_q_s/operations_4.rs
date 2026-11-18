use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "route53profiles",
        operation: "ListProfiles",
        kind: "AWS::Route53Profiles::Profile",
    },
    OperationMapping {
        service: "route53profiles",
        operation: "ListProfileAssociations",
        kind: "AWS::Route53Profiles::ProfileAssociation",
    },
    OperationMapping {
        service: "route53profiles",
        operation: "ListProfileResourceAssociations",
        kind: "AWS::Route53Profiles::ProfileResourceAssociation",
    },
    OperationMapping {
        service: "route53resolver",
        operation: "ListResolverEndpoints",
        kind: "AWS::Route53Resolver::ResolverEndpoint",
    },
    OperationMapping {
        service: "route53resolver",
        operation: "ListResolverRules",
        kind: "AWS::Route53Resolver::ResolverRule",
    },
    OperationMapping {
        service: "route53resolver",
        operation: "ListResolverRuleAssociations",
        kind: "AWS::Route53Resolver::ResolverRuleAssociation",
    },
    OperationMapping {
        service: "route53resolver",
        operation: "ListResolverQueryLogConfigs",
        kind: "AWS::Route53Resolver::ResolverQueryLoggingConfig",
    },
    OperationMapping {
        service: "route53resolver",
        operation: "ListResolverQueryLogConfigAssociations",
        kind: "AWS::Route53Resolver::ResolverQueryLoggingConfigAssociation",
    },
    OperationMapping {
        service: "route53resolver",
        operation: "ListFirewallDomainLists",
        kind: "AWS::Route53Resolver::FirewallDomainList",
    },
    OperationMapping {
        service: "route53resolver",
        operation: "ListFirewallRuleGroups",
        kind: "AWS::Route53Resolver::FirewallRuleGroup",
    },
    OperationMapping {
        service: "route53resolver",
        operation: "ListFirewallRuleGroupAssociations",
        kind: "AWS::Route53Resolver::FirewallRuleGroupAssociation",
    },
    OperationMapping {
        service: "route53resolver",
        operation: "ListOutpostResolvers",
        kind: "AWS::ApiOnly::Route53OutpostResolver",
    },
    OperationMapping {
        service: "route53resolver",
        operation: "ListFirewallConfigs",
        kind: "AWS::ApiOnly::Route53FirewallConfig",
    },
    OperationMapping {
        service: "route53resolver",
        operation: "ListResolverConfigs",
        kind: "AWS::ApiOnly::Route53ResolverConfig",
    },
    OperationMapping {
        service: "route53resolver",
        operation: "ListResolverDnssecConfigs",
        kind: "AWS::Route53Resolver::ResolverDNSSECConfig",
    },
    OperationMapping {
        service: "rtbfabric",
        operation: "ListRequesterGateways",
        kind: "AWS::RTBFabric::RequesterGateway",
    },
    OperationMapping {
        service: "rtbfabric",
        operation: "ListResponderGateways",
        kind: "AWS::RTBFabric::ResponderGateway",
    },
    OperationMapping {
        service: "rum",
        operation: "ListAppMonitors",
        kind: "AWS::RUM::AppMonitor",
    },
    OperationMapping {
        service: "s3-control",
        operation: "ListAccessPoints",
        kind: "AWS::S3::AccessPoint",
    },
    OperationMapping {
        service: "s3-control",
        operation: "ListAccessPointsForObjectLambda",
        kind: "AWS::S3ObjectLambda::AccessPoint",
    },
];
