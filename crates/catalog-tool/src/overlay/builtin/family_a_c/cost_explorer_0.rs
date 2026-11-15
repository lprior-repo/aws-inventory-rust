// Pinned source: .local/aws-models/models/cost-explorer/service/2017-10-25/cost-explorer-2017-10-25.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        // Model: Creates a new cost anomaly detection monitor with the requested type and monitor specification.
        ("CreateAnomalyMonitor", denied(DenyReason::Paid)),
        // Model: Adds an alert subscription to a cost anomaly detection monitor. You can use each subscription to define subscribers with email or SNS notifications. Email subscribers can set an absolute or…
        ("CreateAnomalySubscription", denied(DenyReason::Paid)),
        // Model: Creates a new cost category with the requested name and rules.
        ("CreateCostCategoryDefinition", denied(DenyReason::Paid)),
        // Model: Deletes a cost anomaly monitor.
        ("DeleteAnomalyMonitor", denied(DenyReason::Paid)),
        // Model: Deletes a cost anomaly subscription.
        ("DeleteAnomalySubscription", denied(DenyReason::Paid)),
        // Model: Deletes a cost category. Expenses from this month going forward will no longer be categorized with this cost category.
        ("DeleteCostCategoryDefinition", denied(DenyReason::Paid)),
        // Model: Returns the name, Amazon Resource Name (ARN), rules, definition, and effective dates of a cost category that's defined in the account. You have the option to use EffectiveOn to return a cost category…
        ("DescribeCostCategoryDefinition", denied(DenyReason::Paid)),
        // Model: Retrieves all of the cost anomalies detected on your account during the time period that's specified by the DateInterval object. Anomalies are available for up to 90 days.
        ("GetAnomalies", denied(DenyReason::Paid)),
        // Model: Retrieves the cost anomaly monitor definitions for your account. You can filter using a list of cost anomaly monitor Amazon Resource Names (ARNs).
        ("GetAnomalyMonitors", denied(DenyReason::Paid)),
        // Model: Retrieves the cost anomaly subscription objects for your account. You can filter using a list of cost anomaly monitor Amazon Resource Names (ARNs).
        ("GetAnomalySubscriptions", denied(DenyReason::Paid)),
        // Model: Retrieves estimated usage records for hourly granularity or resource-level data at daily granularity.
        ("GetApproximateUsageRecords", denied(DenyReason::Paid)),
        // Model: Retrieves a commitment purchase analysis result based on the AnalysisId .
        ("GetCommitmentPurchaseAnalysis", denied(DenyReason::Paid)),
        // Model: Retrieves cost and usage metrics for your account. You can specify which cost and usage-related metric that you want the request to return. For example, you can specify BlendedCosts or UsageQuantity…
        ("GetCostAndUsage", denied(DenyReason::Paid)),
        // Model: Retrieves cost and usage comparisons for your account between two periods within the last 13 months. If you have enabled multi-year data at monthly granularity, you can go back up to 38 months.
        ("GetCostAndUsageComparisons", denied(DenyReason::Paid)),
        // Model: Retrieves cost and usage metrics with resources for your account. You can specify which cost and usage-related metric, such as BlendedCosts or UsageQuantity , that you want the request to return. You…
        ("GetCostAndUsageWithResources", denied(DenyReason::Paid)),
        // Model: Retrieves an array of cost category names and values incurred cost. If some cost category names and values are not associated with any cost, they will not be returned by this API.
        ("GetCostCategories", denied(DenyReason::Paid)),
        // Model: Retrieves key factors driving cost changes between two time periods within the last 13 months, such as usage changes, discount changes, and commitment-based savings. If you have enabled multi-year…
        ("GetCostComparisonDrivers", denied(DenyReason::Paid)),
        // Model: Retrieves a forecast for how much Amazon Web Services predicts that you will spend over the forecast time period that you select, based on your past costs.
        ("GetCostForecast", denied(DenyReason::Paid)),
        // Model: Retrieves all available filter values for a specified filter over a period of time. You can search the dimension values for an arbitrary string.
        ("GetDimensionValues", denied(DenyReason::Paid)),
        // Model: Retrieves the reservation coverage for your account, which you can use to see how much of your Amazon Elastic Compute Cloud, Amazon ElastiCache, Amazon Relational Database Service, or Amazon Redshift…
        ("GetReservationCoverage", denied(DenyReason::Paid)),
        // Model: Gets recommendations for reservation purchases. These recommendations might help you to reduce your costs. Reservations provide a discounted hourly rate (up to 75%) compared to On-Demand pricing.…
        ("GetReservationPurchaseRecommendation", denied(DenyReason::Paid)),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        // Model: Retrieves the reservation utilization for your account. Management account in an organization have access to member accounts. You can filter data by dimensions in a time period. You can use…
        ("GetReservationUtilization", denied(DenyReason::Paid)),
        // Model: Creates recommendations that help you save cost by identifying idle and underutilized Amazon EC2 instances. Recommendations are generated to either downsize or terminate instances, along with…
        ("GetRightsizingRecommendation", denied(DenyReason::Paid)),
        // Model: Retrieves the details for a Savings Plan recommendation. These details include the hourly data-points that construct the cost, coverage, and utilization charts.
        ("GetSavingsPlanPurchaseRecommendationDetails", denied(DenyReason::Paid)),
        // Model: Retrieves the Savings Plans covered for your account. This enables you to see how much of your cost is covered by a Savings Plan. An organization’s management account can see the coverage of the…
        ("GetSavingsPlansCoverage", denied(DenyReason::Paid)),
        // Model: Retrieves the Savings Plans recommendations for your account. First use StartSavingsPlansPurchaseRecommendationGeneration to generate a new set of recommendations, and then use…
        ("GetSavingsPlansPurchaseRecommendation", denied(DenyReason::Paid)),
        // Model: Retrieves the Savings Plans utilization for your account across date ranges with daily or monthly granularity. Management account in an organization have access to member accounts. You can use…
        ("GetSavingsPlansUtilization", denied(DenyReason::Paid)),
        // Model: Retrieves attribute data along with aggregate utilization and savings data for a given time period. This doesn't support granular or grouped data (daily/monthly) in response. You can't retrieve data…
        ("GetSavingsPlansUtilizationDetails", denied(DenyReason::Paid)),
        // Model: Queries for available tag keys and tag values for a specified period. You can search the tag values for an arbitrary string.
        ("GetTags", denied(DenyReason::Paid)),
        // Model: Retrieves a forecast for how much Amazon Web Services predicts that you will use over the forecast time period that you select, based on your past usage.
        ("GetUsageForecast", denied(DenyReason::Paid)),
        // Model: Lists the commitment purchase analyses for your account.
        ("ListCommitmentPurchaseAnalyses", denied(DenyReason::Paid)),
        // Model: Retrieves a list of your historical cost allocation tag backfill requests.
        ("ListCostAllocationTagBackfillHistory", denied(DenyReason::Paid)),
        // Model: Get a list of cost allocation tags. All inputs in the API are optional and serve as filters. By default, all cost allocation tags are returned.
        ("ListCostAllocationTags", denied(DenyReason::Paid)),
        // Model: Returns the name, Amazon Resource Name (ARN), NumberOfRules and effective dates of all cost categories defined in the account. You have the option to use EffectiveOn and SupportedResourceTypes to…
        ("ListCostCategoryDefinitions", denied(DenyReason::Paid)),
        // Model: Returns resource associations of all cost categories defined in the account. You have the option to use CostCategoryArn to get the association for a specific cost category.…
        ("ListCostCategoryResourceAssociations", denied(DenyReason::Paid)),
        // Model: Retrieves a list of your historical recommendation generations within the past 30 days.
        ("ListSavingsPlansPurchaseRecommendationGeneration", denied(DenyReason::Paid)),
        // Model: Returns a list of resource tags associated with the resource specified by the Amazon Resource Name (ARN).
        ("ListTagsForResource", denied(DenyReason::Paid)),
        // Model: Modifies the feedback property of a given cost anomaly.
        ("ProvideAnomalyFeedback", denied(DenyReason::Paid)),
        // Model: Specifies the parameters of a planned commitment purchase and starts the generation of the analysis. This enables you to estimate the cost, coverage, and utilization impact of your planned commitment…
        ("StartCommitmentPurchaseAnalysis", denied(DenyReason::Paid)),
        // Model: Request a cost allocation tag backfill. This will backfill the activation status (either active or inactive ) for all tag keys from para:BackfillFrom up to the time this request is made. You can…
        ("StartCostAllocationTagBackfill", denied(DenyReason::Paid)),
        // Model: Requests a Savings Plans recommendation generation. This enables you to calculate a fresh set of Savings Plans recommendations that takes your latest usage data and current Savings Plans inventory…
        ("StartSavingsPlansPurchaseRecommendationGeneration", denied(DenyReason::Paid)),
        // Model: An API operation for adding one or more tags (key-value pairs) to a resource. You can use the TagResource operation with a resource that already has tags. If you specify a new tag key for the…
        ("TagResource", denied(DenyReason::Paid)),
    ])
}

pub(super) fn part_2() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        // Model: Removes one or more tags from a resource. Specify only tag keys in your request. Don't specify the value.
        ("UntagResource", denied(DenyReason::Paid)),
        // Model: Updates an existing cost anomaly monitor. The changes made are applied going forward, and doesn't change anomalies detected in the past.
        ("UpdateAnomalyMonitor", denied(DenyReason::Paid)),
        // Model: Updates an existing cost anomaly subscription. Specify the fields that you want to update. Omitted fields are unchanged. The JSON below describes the generic construct for each type. See Request…
        ("UpdateAnomalySubscription", denied(DenyReason::Paid)),
        // Model: Updates status for cost allocation tags in bulk, with maximum batch size of 20. If the tag status that's updated is the same as the existing tag status, the request doesn't fail. Instead, it doesn't…
        ("UpdateCostAllocationTagsStatus", denied(DenyReason::Paid)),
        // Model: Updates an existing cost category. Changes made to the cost category rules will be used to categorize the current month’s expenses and future expenses. This won’t change categorization for the…
        ("UpdateCostCategoryDefinition", denied(DenyReason::Paid)),
    ])
}
