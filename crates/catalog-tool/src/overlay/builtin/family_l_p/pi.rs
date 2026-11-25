use super::policy::{Group, compile};
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreatePerformanceAnalysisReport",
            "DeletePerformanceAnalysisReport",
            "TagResource",
            "UntagResource",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "DescribeDimensionKeys",
            "GetDimensionKeyDetails",
            "GetPerformanceAnalysisReport",
            "GetResourceMetrics",
        ],
        class: OverlayClass::Deny(DenyReason::Content),
        reason: None,
    },
    Group {
        operations: &[
            "GetResourceMetadata",
            "ListAvailableResourceDimensions",
            "ListAvailableResourceMetrics",
            "ListPerformanceAnalysisReportRecommendations",
            "ListPerformanceAnalysisReports",
            "ListTagsForResource",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Performance Insights reads SQL/dimension values and metrics for an externally supplied database identifier; it is not infrastructure discovery.",
        ),
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("pi", GROUPS, &[])
}
