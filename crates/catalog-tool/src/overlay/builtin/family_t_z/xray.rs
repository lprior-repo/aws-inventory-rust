use super::super::service;
use super::{defer, deny, xray_plans_1};
use crate::{
    classes::DenyReason,
    overlay::{NameError, ServiceOverlay},
};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    service(
        "xray",
        vec![
            ("BatchGetTraces", deny(DenyReason::Content)),
            ("CancelTraceRetrieval", deny(DenyReason::Mutating)),
            ("CreateGroup", deny(DenyReason::Mutating)),
            ("CreateSamplingRule", deny(DenyReason::Mutating)),
            ("DeleteGroup", deny(DenyReason::Mutating)),
            ("DeleteResourcePolicy", deny(DenyReason::Mutating)),
            ("DeleteSamplingRule", deny(DenyReason::Mutating)),
            (
                "GetEncryptionConfig",
                defer(
                    "The documented response is aggregate, reference, operational history, or a singleton without an exact returned resource identity; it is not an executable resource enumeration.",
                ),
            ),
            ("GetGroup", xray_plans_1::get_group()?),
            ("GetGroups", xray_plans_1::get_groups()?),
            ("GetIndexingRules", xray_plans_1::get_indexing_rules()?),
            ("GetInsight", deny(DenyReason::Content)),
            ("GetInsightEvents", deny(DenyReason::Content)),
            ("GetInsightImpactGraph", deny(DenyReason::Content)),
            ("GetInsightSummaries", deny(DenyReason::Content)),
            ("GetRetrievedTracesGraph", deny(DenyReason::Content)),
            ("GetSamplingRules", xray_plans_1::get_sampling_rules()?),
            (
                "GetSamplingStatisticSummaries",
                defer(
                    "The documented response is aggregate, reference, operational history, or a singleton without an exact returned resource identity; it is not an executable resource enumeration.",
                ),
            ),
            (
                "GetSamplingTargets",
                defer(
                    "Missing executable inventory binding for SamplingStatisticsDocuments: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            ("GetServiceGraph", deny(DenyReason::Content)),
            ("GetTimeSeriesServiceStatistics", deny(DenyReason::Content)),
            ("GetTraceGraph", deny(DenyReason::Content)),
            (
                "GetTraceSegmentDestination",
                defer(
                    "The documented response is aggregate, reference, operational history, or a singleton without an exact returned resource identity; it is not an executable resource enumeration.",
                ),
            ),
            ("GetTraceSummaries", deny(DenyReason::Content)),
            (
                "ListResourcePolicies",
                xray_plans_1::list_resource_policies()?,
            ),
            ("ListRetrievedTraces", deny(DenyReason::Content)),
            (
                "ListTagsForResource",
                defer(
                    "Tag-only response omits the resource identity; the runtime does not yet merge bound-parent identity into complete tag-only observations.",
                ),
            ),
            ("PutEncryptionConfig", deny(DenyReason::Mutating)),
            ("PutResourcePolicy", deny(DenyReason::Mutating)),
            ("PutTelemetryRecords", deny(DenyReason::Mutating)),
            ("PutTraceSegments", deny(DenyReason::Mutating)),
            ("StartTraceRetrieval", deny(DenyReason::Mutating)),
            ("TagResource", deny(DenyReason::Mutating)),
            ("UntagResource", deny(DenyReason::Mutating)),
            ("UpdateGroup", deny(DenyReason::Mutating)),
            ("UpdateIndexingRule", deny(DenyReason::Mutating)),
            ("UpdateSamplingRule", deny(DenyReason::Mutating)),
            ("UpdateTraceSegmentDestination", deny(DenyReason::Mutating)),
        ],
    )
}
