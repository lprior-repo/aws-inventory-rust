use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        ("DescribeScalableTargets", OperationOverlay {
            queryable: vec![path("ServiceNamespace")?, path("ScalableDimension")?],
            ..read("ScalableTargets", Identity::Arn(path("ScalableTargetARN")?), Scope::Region, token("NextToken", "NextToken")?)?
        }),
        ("DescribeScalingActivities", OperationOverlay {
            queryable: vec![path("ServiceNamespace")?],
            ..read("ScalingActivities", Identity::Correlated(path("ResourceId")?, path("ScalableDimension")?), Scope::Region, token("NextToken", "NextToken")?)?
        }),
        ("DescribeScalingPolicies", OperationOverlay {
            queryable: vec![path("ServiceNamespace")?],
            ..read("ScalingPolicies", Identity::Arn(path("PolicyARN")?), Scope::Region, token("NextToken", "NextToken")?)?
        }),
        ("DescribeScheduledActions", OperationOverlay {
            queryable: vec![path("ServiceNamespace")?],
            ..read("ScheduledActions", Identity::Arn(path("ScheduledActionARN")?), Scope::Region, token("NextToken", "NextToken")?)?
        }),
        ("DeleteScalingPolicy", deferred("Mutation: changes scaling policy configuration. Pinned operation: Deletes a scaling policy.")),
        ("DeleteScheduledAction", deferred("Mutation: changes scheduled action configuration. Pinned operation: Deletes a scheduled action.")),
        ("DeregisterScalableTarget", deferred("Mutation: changes scalable target registration. Pinned operation: Deregisters a scalable target.")),
        ("GetPredictiveScalingForecast", deferred("Returns forecast data, not a resource collection. No reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets a load and capacity forecast for a scaling policy.")),
        ("ListTagsForResource", deferred("Requires resource ARN that is not enumerable from a list operation. No reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Lists tags for a specific resource.")),
        ("PutScalingPolicy", deferred("Mutation: changes scaling policy configuration. Pinned operation: Creates or updates a scaling policy.")),
        ("PutScheduledAction", deferred("Mutation: changes scheduled action configuration. Pinned operation: Creates or updates a scheduled action.")),
        ("RegisterScalableTarget", deferred("Mutation: changes scalable target registration. Pinned operation: Registers a scalable target.")),
        ("TagResource", deferred("Mutation: changes resource tags. Pinned operation: Tags a resource.")),
        ("UntagResource", deferred("Mutation: changes resource tags. Pinned operation: Untags a resource.")),
    ])
}

