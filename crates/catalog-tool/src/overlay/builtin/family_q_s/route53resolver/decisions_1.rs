// Pinned operation purposes: .local/aws-models/models/route53resolver/service/2018-04-01/route53resolver-2018-04-01.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: You can use UpdateOutpostResolver to update the instance count, type, or name of a Resolver on an Outpost. This operation applies to first-generation Outposts only. On second-generation Outposts, Resolver is managed automatically by Amazon Web Services and can't be updated directly.
    ("UpdateOutpostResolver", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the behavior configuration of Route 53 Resolver behavior for a single VPC from Amazon Virtual Private Cloud.
    ("UpdateResolverConfig", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing DNSSEC validation configuration. If there is no existing DNSSEC validation configuration, one is created.
    ("UpdateResolverDnssecConfig", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the name, or endpoint type for an inbound or an outbound Resolver endpoint. You can only update between IPV4 and DUALSTACK, IPV6 endpoint type can't be updated to other type.
    ("UpdateResolverEndpoint", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates settings for a specified Resolver rule. ResolverRuleId is required, and all other parameters are optional. If you don't specify a parameter, it retains its current value.
    ("UpdateResolverRule", Decision::Denied(DenyReason::Mutating)),
];
