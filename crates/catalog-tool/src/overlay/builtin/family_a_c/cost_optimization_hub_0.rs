use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "GetPreferences",
            deferred(
                "Recommendation, cost-analysis, or enrollment data is not an owned resource enumeration; automatic enrollment, exports, or recommendation generation are not authorized. Model: Returns a set of preferences for an account in order to add account-specific preferences into the service. These preferences impact how the savings associated with recommendations are presented—estimated savings after discounts…",
            ),
        ),
        (
            "GetRecommendation",
            deferred(
                "Recommendation, cost-analysis, or enrollment data is not an owned resource enumeration; automatic enrollment, exports, or recommendation generation are not authorized. Model: Returns both the current and recommended resource configuration and the estimated cost impact for a recommendation. The recommendationId is only valid for up to a maximum of 24 hours as recommendations are refreshed daily. To…",
            ),
        ),
        (
            "ListEfficiencyMetrics",
            deferred(
                "Recommendation, cost-analysis, or enrollment data is not an owned resource enumeration; automatic enrollment, exports, or recommendation generation are not authorized. Model: Returns cost efficiency metrics aggregated over time and optionally grouped by a specified dimension. The metrics provide insights into your cost optimization progress by tracking estimated savings, spending, and measures how…",
            ),
        ),
        (
            "ListEnrollmentStatuses",
            deferred(
                "Recommendation, cost-analysis, or enrollment data is not an owned resource enumeration; automatic enrollment, exports, or recommendation generation are not authorized. Model: Retrieves the enrollment status for an account. It can also return the list of accounts that are enrolled under the organization.",
            ),
        ),
        (
            "ListRecommendationSummaries",
            deferred(
                "Recommendation, cost-analysis, or enrollment data is not an owned resource enumeration; automatic enrollment, exports, or recommendation generation are not authorized. Model: Returns a concise representation of savings estimates for resources. Also returns de-duped savings across different types of recommendations. The following filters are not supported for this API: recommendationIds , resourceArns…",
            ),
        ),
        (
            "ListRecommendations",
            deferred(
                "Recommendation, cost-analysis, or enrollment data is not an owned resource enumeration; automatic enrollment, exports, or recommendation generation are not authorized. Model: Returns a list of recommendations.",
            ),
        ),
        (
            "UpdateEnrollmentStatus",
            deferred(
                "Recommendation, cost-analysis, or enrollment data is not an owned resource enumeration; automatic enrollment, exports, or recommendation generation are not authorized. Model: Updates the enrollment (opt in and opt out) status of an account to the Cost Optimization Hub service. If the account is a management account of an organization, this action can also be used to enroll member accounts of the…",
            ),
        ),
        (
            "UpdatePreferences",
            deferred(
                "Recommendation, cost-analysis, or enrollment data is not an owned resource enumeration; automatic enrollment, exports, or recommendation generation are not authorized. Model: Updates a set of preferences for an account in order to add account-specific preferences into the service. These preferences impact how the savings associated with recommendations are presented.",
            ),
        ),
    ])
}
