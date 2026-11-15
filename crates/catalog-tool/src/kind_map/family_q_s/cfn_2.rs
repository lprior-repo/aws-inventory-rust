use crate::kind_map::CfnMapping;

pub(super) const ENTRIES: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::RolesAnywhere::TrustAnchor"),
    CfnMapping::canonical("AWS::Route53Profiles::Profile"),
    CfnMapping::canonical("AWS::Route53Profiles::ProfileAssociation"),
    CfnMapping::canonical("AWS::Route53Profiles::ProfileResourceAssociation"),
    CfnMapping::canonical("AWS::Route53RecoveryControl::Cluster"),
    CfnMapping::canonical("AWS::Route53RecoveryControl::ControlPanel"),
    CfnMapping::canonical("AWS::Route53RecoveryControl::RoutingControl"),
    CfnMapping::canonical("AWS::Route53Resolver::FirewallDomainList"),
    CfnMapping::canonical("AWS::Route53Resolver::FirewallRuleGroup"),
    CfnMapping::canonical("AWS::Route53Resolver::FirewallRuleGroupAssociation"),
    CfnMapping::canonical("AWS::Route53Resolver::ResolverDNSSECConfig"),
    CfnMapping::canonical("AWS::Route53Resolver::ResolverEndpoint"),
    CfnMapping::canonical("AWS::Route53Resolver::ResolverQueryLoggingConfig"),
    CfnMapping::canonical("AWS::Route53Resolver::ResolverQueryLoggingConfigAssociation"),
    CfnMapping::canonical("AWS::Route53Resolver::ResolverRule"),
    CfnMapping::canonical("AWS::Route53Resolver::ResolverRuleAssociation"),
    CfnMapping::canonical("AWS::S3::AccessGrant"),
    CfnMapping::canonical("AWS::S3::AccessGrantsInstance"),
    CfnMapping::canonical("AWS::S3::AccessGrantsLocation"),
    CfnMapping::canonical("AWS::S3::AccessPoint"),
];
