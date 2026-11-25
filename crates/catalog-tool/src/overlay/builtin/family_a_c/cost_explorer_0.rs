use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        ("CreateAnomalyMonitor", denied(DenyReason::Paid)),
        ("CreateAnomalySubscription", denied(DenyReason::Paid)),
        ("CreateCostCategoryDefinition", denied(DenyReason::Paid)),
        ("DeleteAnomalyMonitor", denied(DenyReason::Paid)),
        ("DeleteAnomalySubscription", denied(DenyReason::Paid)),
        ("DeleteCostCategoryDefinition", denied(DenyReason::Paid)),
        ("DescribeCostCategoryDefinition", denied(DenyReason::Paid)),
        ("GetAnomalies", denied(DenyReason::Paid)),
        ("GetAnomalyMonitors", denied(DenyReason::Paid)),
        ("GetAnomalySubscriptions", denied(DenyReason::Paid)),
        ("GetApproximateUsageRecords", denied(DenyReason::Paid)),
        ("GetCommitmentPurchaseAnalysis", denied(DenyReason::Paid)),
        ("GetCostAndUsage", denied(DenyReason::Paid)),
        ("GetCostAndUsageComparisons", denied(DenyReason::Paid)),
        ("GetCostAndUsageWithResources", denied(DenyReason::Paid)),
        ("GetCostCategories", denied(DenyReason::Paid)),
        ("GetCostComparisonDrivers", denied(DenyReason::Paid)),
        ("GetCostForecast", denied(DenyReason::Paid)),
        ("GetDimensionValues", denied(DenyReason::Paid)),
        ("GetReservationCoverage", denied(DenyReason::Paid)),
        (
            "GetReservationPurchaseRecommendation",
            denied(DenyReason::Paid),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        ("GetReservationUtilization", denied(DenyReason::Paid)),
        ("GetRightsizingRecommendation", denied(DenyReason::Paid)),
        (
            "GetSavingsPlanPurchaseRecommendationDetails",
            denied(DenyReason::Paid),
        ),
        ("GetSavingsPlansCoverage", denied(DenyReason::Paid)),
        (
            "GetSavingsPlansPurchaseRecommendation",
            denied(DenyReason::Paid),
        ),
        ("GetSavingsPlansUtilization", denied(DenyReason::Paid)),
        (
            "GetSavingsPlansUtilizationDetails",
            denied(DenyReason::Paid),
        ),
        ("GetTags", denied(DenyReason::Paid)),
        ("GetUsageForecast", denied(DenyReason::Paid)),
        ("ListCommitmentPurchaseAnalyses", denied(DenyReason::Paid)),
        (
            "ListCostAllocationTagBackfillHistory",
            denied(DenyReason::Paid),
        ),
        ("ListCostAllocationTags", denied(DenyReason::Paid)),
        ("ListCostCategoryDefinitions", denied(DenyReason::Paid)),
        (
            "ListCostCategoryResourceAssociations",
            denied(DenyReason::Paid),
        ),
        (
            "ListSavingsPlansPurchaseRecommendationGeneration",
            denied(DenyReason::Paid),
        ),
        ("ListTagsForResource", denied(DenyReason::Paid)),
        ("ProvideAnomalyFeedback", denied(DenyReason::Paid)),
        ("StartCommitmentPurchaseAnalysis", denied(DenyReason::Paid)),
        ("StartCostAllocationTagBackfill", denied(DenyReason::Paid)),
        (
            "StartSavingsPlansPurchaseRecommendationGeneration",
            denied(DenyReason::Paid),
        ),
        ("TagResource", denied(DenyReason::Paid)),
    ])
}

pub(super) fn part_2() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        ("UntagResource", denied(DenyReason::Paid)),
        ("UpdateAnomalyMonitor", denied(DenyReason::Paid)),
        ("UpdateAnomalySubscription", denied(DenyReason::Paid)),
        ("UpdateCostAllocationTagsStatus", denied(DenyReason::Paid)),
        ("UpdateCostCategoryDefinition", denied(DenyReason::Paid)),
    ])
}
