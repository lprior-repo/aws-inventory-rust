use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![(
        "ListRecommendedActions",
        deferred(
            "Recommendation, cost-analysis, or enrollment data is not an owned resource enumeration; automatic enrollment, exports, or recommendation generation are not authorized. Model: Returns a list of recommended actions that match the filter criteria.",
        ),
    )])
}
