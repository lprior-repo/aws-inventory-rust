// Pinned source: .local/aws-models/models/auto-scaling-plans/service/2018-01-06/auto-scaling-plans-2018-01-06.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        ("CreateScalingPlan", deferred("Model requires ScalingPlanName, ApplicationSource, ScalingInstructions. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a scaling plan.")),
        ("DeleteScalingPlan", deferred("Model requires ScalingPlanName, ScalingPlanVersion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified scaling plan. Deleting a scaling plan deletes the underlying ScalingInstruction for all of the scalable resources that are covered by the plan. If the plan has launched resources or has scaling activities in…")),
        ("DescribeScalingPlanResources", deferred("Model requires ScalingPlanName, ScalingPlanVersion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes the scalable resources in the specified scaling plan.")),
        // Model: Describes one or more of your scaling plans.
        ("DescribeScalingPlans", OperationOverlay {
            queryable: vec![path("ScalingPlanName")?, path("ScalingPlanVersion")?],
            ..read("ScalingPlans", Identity::Key(vec![path("ScalingPlanName")?, path("ScalingPlanVersion")?]), Scope::Region, token("NextToken", "NextToken")?)?
        }),
        ("GetScalingPlanResourceForecastData", deferred("Model requires ScalingPlanName, ScalingPlanVersion, ServiceNamespace, ResourceId, ScalableDimension, ForecastDataType, StartTime, EndTime. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the forecast data for a scalable resource. Capacity forecasts are represented as predicted values, or data points, that are calculated using historical data points from a specified CloudWatch load metric. Data points…")),
        ("UpdateScalingPlan", deferred("Model requires ScalingPlanName, ScalingPlanVersion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the specified scaling plan. You cannot update a scaling plan if it is in the process of being created, updated, or deleted.")),
    ])
}
