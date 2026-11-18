use super::super::service;
use super::{defer, deny};
use crate::{
    classes::DenyReason,
    overlay::{NameError, ServiceOverlay},
};

// Reviewed against pinned Smithy documentation, not operation-name read approval.
pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    service(
        "wafv2",
        vec![
            ("AssociateWebACL", deny(DenyReason::Mutating)),
            (
                "CheckCapacity",
                defer(
                    "Resource calls require both REGIONAL and CLOUDFRONT scope variants with different regional dispatch; one operation cannot currently emit both.",
                ),
            ),
            ("CreateAPIKey", deny(DenyReason::Mutating)),
            ("CreateIPSet", deny(DenyReason::Mutating)),
            ("CreateRegexPatternSet", deny(DenyReason::Mutating)),
            ("CreateRuleGroup", deny(DenyReason::Mutating)),
            ("CreateWebACL", deny(DenyReason::Mutating)),
            ("DeleteAPIKey", deny(DenyReason::Mutating)),
            (
                "DeleteFirewallManagerRuleGroups",
                deny(DenyReason::Mutating),
            ),
            ("DeleteIPSet", deny(DenyReason::Mutating)),
            ("DeleteLoggingConfiguration", deny(DenyReason::Mutating)),
            ("DeletePermissionPolicy", deny(DenyReason::Mutating)),
            ("DeleteRegexPatternSet", deny(DenyReason::Mutating)),
            ("DeleteRuleGroup", deny(DenyReason::Mutating)),
            ("DeleteWebACL", deny(DenyReason::Mutating)),
            (
                "DescribeAllManagedProducts",
                deny(DenyReason::ReferenceData),
            ),
            (
                "DescribeManagedProductsByVendor",
                deny(DenyReason::ReferenceData),
            ),
            ("DescribeManagedRuleGroup", deny(DenyReason::ReferenceData)),
            ("DisassociateWebACL", deny(DenyReason::Mutating)),
            ("GenerateMobileSdkReleaseUrl", deny(DenyReason::Mutating)),
            ("GetDecryptedAPIKey", deny(DenyReason::Secret)),
            (
                "GetIPSet",
                defer(
                    "Resource calls require both REGIONAL and CLOUDFRONT scope variants with different regional dispatch; one operation cannot currently emit both.",
                ),
            ),
            (
                "GetLoggingConfiguration",
                defer(
                    "Resource calls require both REGIONAL and CLOUDFRONT scope variants with different regional dispatch; one operation cannot currently emit both.",
                ),
            ),
            (
                "GetManagedRuleSet",
                defer(
                    "Resource calls require both REGIONAL and CLOUDFRONT scope variants with different regional dispatch; one operation cannot currently emit both.",
                ),
            ),
            ("GetMobileSdkRelease", deny(DenyReason::ReferenceData)),
            (
                "GetPermissionPolicy",
                defer(
                    "Resource calls require both REGIONAL and CLOUDFRONT scope variants with different regional dispatch; one operation cannot currently emit both.",
                ),
            ),
            (
                "GetRateBasedStatementManagedKeys",
                deny(DenyReason::Content),
            ),
            (
                "GetRegexPatternSet",
                defer(
                    "Resource calls require both REGIONAL and CLOUDFRONT scope variants with different regional dispatch; one operation cannot currently emit both.",
                ),
            ),
            ("GetRevenueStatistics", deny(DenyReason::Content)),
            ("GetRevenueStatisticsSummary", deny(DenyReason::Content)),
            ("GetRevenueStatisticsTimeSeries", deny(DenyReason::Content)),
            (
                "GetRuleGroup",
                defer(
                    "Resource calls require both REGIONAL and CLOUDFRONT scope variants with different regional dispatch; one operation cannot currently emit both.",
                ),
            ),
            ("GetSampledRequests", deny(DenyReason::Content)),
            ("GetTopPathStatisticsByTraffic", deny(DenyReason::Content)),
            (
                "GetWebACL",
                defer(
                    "Resource calls require both REGIONAL and CLOUDFRONT scope variants with different regional dispatch; one operation cannot currently emit both.",
                ),
            ),
            (
                "GetWebACLForResource",
                defer(
                    "Resource calls require both REGIONAL and CLOUDFRONT scope variants with different regional dispatch; one operation cannot currently emit both.",
                ),
            ),
            ("ListAPIKeys", deny(DenyReason::Secret)),
            (
                "ListAvailableManagedRuleGroups",
                deny(DenyReason::ReferenceData),
            ),
            (
                "ListAvailableManagedRuleGroupVersions",
                deny(DenyReason::ReferenceData),
            ),
            (
                "ListIPSets",
                defer(
                    "Resource calls require both REGIONAL and CLOUDFRONT scope variants with different regional dispatch; one operation cannot currently emit both.",
                ),
            ),
            (
                "ListLoggingConfigurations",
                defer(
                    "Resource calls require both REGIONAL and CLOUDFRONT scope variants with different regional dispatch; one operation cannot currently emit both.",
                ),
            ),
            (
                "ListManagedRuleSets",
                defer(
                    "Resource calls require both REGIONAL and CLOUDFRONT scope variants with different regional dispatch; one operation cannot currently emit both.",
                ),
            ),
            ("ListMobileSdkReleases", deny(DenyReason::ReferenceData)),
            (
                "ListRegexPatternSets",
                defer(
                    "Resource calls require both REGIONAL and CLOUDFRONT scope variants with different regional dispatch; one operation cannot currently emit both.",
                ),
            ),
            (
                "ListResourcesForWebACL",
                defer(
                    "Resource calls require both REGIONAL and CLOUDFRONT scope variants with different regional dispatch; one operation cannot currently emit both.",
                ),
            ),
            (
                "ListRuleGroups",
                defer(
                    "Resource calls require both REGIONAL and CLOUDFRONT scope variants with different regional dispatch; one operation cannot currently emit both.",
                ),
            ),
            ("ListSettlementRecords", deny(DenyReason::Content)),
            (
                "ListTagsForResource",
                defer(
                    "Resource calls require both REGIONAL and CLOUDFRONT scope variants with different regional dispatch; one operation cannot currently emit both.",
                ),
            ),
            (
                "ListWebACLs",
                defer(
                    "Resource calls require both REGIONAL and CLOUDFRONT scope variants with different regional dispatch; one operation cannot currently emit both.",
                ),
            ),
            ("PutLoggingConfiguration", deny(DenyReason::Mutating)),
            ("PutManagedRuleSetVersions", deny(DenyReason::Mutating)),
            ("PutPermissionPolicy", deny(DenyReason::Mutating)),
            ("TagResource", deny(DenyReason::Mutating)),
            ("UntagResource", deny(DenyReason::Mutating)),
            ("UpdateIPSet", deny(DenyReason::Mutating)),
            (
                "UpdateManagedRuleSetVersionExpiryDate",
                deny(DenyReason::Mutating),
            ),
            ("UpdateRegexPatternSet", deny(DenyReason::Mutating)),
            ("UpdateRuleGroup", deny(DenyReason::Mutating)),
            ("UpdateWebACL", deny(DenyReason::Mutating)),
        ],
    )
}
