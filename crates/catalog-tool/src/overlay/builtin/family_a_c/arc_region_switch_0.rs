// Pinned source: .local/aws-models/models/arc-region-switch/service/2022-07-26/arc-region-switch-2022-07-26.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "ApprovePlanExecutionStep",
            deferred(
                "Model requires planArn, executionId, stepName, approval. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Approves a step in a plan execution that requires manual approval. When you create a plan, you can include approval steps that require manual intervention before the execution can proceed. This operation allows you to provide…",
            ),
        ),
        (
            "CancelPlanExecution",
            deferred(
                "Model requires planArn, executionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Cancels an in-progress plan execution. This operation stops the execution of the plan and prevents any further steps from being processed. You must specify the plan ARN and execution ID. You can also provide an optional comment…",
            ),
        ),
        (
            "CreatePlan",
            deferred(
                "Model requires workflows, executionRole, name, regions, recoveryApproach. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new Region switch plan. A plan defines the steps required to shift traffic from one Amazon Web Services Region to another. You must specify a name for the plan, the primary Region, and at least one additional Region.…",
            ),
        ),
        (
            "DeletePlan",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a Region switch plan. You must specify the ARN of the plan to delete. You cannot delete a plan that has an active execution in progress.",
            ),
        ),
        (
            "GetPlan",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves detailed information about a Region switch plan. You must specify the ARN of the plan.",
            ),
        ),
        (
            "GetPlanEvaluationStatus",
            deferred(
                "Model requires planArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the evaluation status of a Region switch plan. The evaluation status provides information about the last time the plan was evaluated and any warnings or issues detected.",
            ),
        ),
        (
            "GetPlanExecution",
            deferred(
                "Model requires planArn, executionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves detailed information about a specific plan execution. You must specify the plan ARN and execution ID.",
            ),
        ),
        (
            "GetPlanInRegion",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about a Region switch plan in a specific Amazon Web Services Region. This operation is useful for getting Region-specific information about a plan.",
            ),
        ),
        (
            "ListPlanExecutionEvents",
            deferred(
                "Model requires planArn, executionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the events that occurred during a plan execution. These events provide a detailed timeline of the execution process.",
            ),
        ),
        (
            "ListPlanExecutions",
            deferred(
                "Model requires planArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the executions of a Region switch plan. This operation returns information about both current and historical executions.",
            ),
        ),
        // Model: Lists all Region switch plans in your Amazon Web Services account.
        (
            "ListPlans",
            OperationOverlay {
                queryable: vec![path("arn")?],
                ..read(
                    "plans",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListPlansInRegion",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is plans; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists all Region switch plans in your Amazon Web Services account that are available in the current Amazon Web Services Region.",
            ),
        ),
        (
            "ListRoute53HealthChecks",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: List the Amazon Route 53 health checks.",
            ),
        ),
        (
            "ListRoute53HealthChecksInRegion",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: List the Amazon Route 53 health checks in a specific Amazon Web Services Region.",
            ),
        ),
        (
            "ListServiceQuotaWarnings",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is serviceQuotaWarningSummaries; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists the service quota warnings for the plans that you can access. Region switch creates a warning when the applied quota value in one Region of a plan is lower than the value required for the matching resource in another Region…",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the tags attached to a Region switch resource.",
            ),
        ),
        (
            "StartPlanExecution",
            deferred(
                "Model requires planArn, targetRegion, action. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts the execution of a Region switch plan. You can execute a plan in either graceful or ungraceful mode. Specifing ungraceful mode either changes the behavior of the execution blocks in a workflow or skips specific execution…",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires arn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds or updates tags for a Region switch resource. You can assign metadata to your resources in the form of tags, which are key-value pairs.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires arn, resourceTagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes tags from a Region switch resource.",
            ),
        ),
        (
            "UpdatePlan",
            deferred(
                "Model requires arn, workflows, executionRole. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing Region switch plan. You can modify the plan's description, workflows, execution role, recovery time objective, associated alarms, and triggers.",
            ),
        ),
        (
            "UpdatePlanExecution",
            deferred(
                "Model requires planArn, executionId, action. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an in-progress plan execution. This operation allows you to modify certain aspects of the execution, such as adding a comment or changing the action.",
            ),
        ),
        (
            "UpdatePlanExecutionStep",
            deferred(
                "Model requires planArn, executionId, comment, stepName, actionToTake. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a specific step in an in-progress plan execution. This operation allows you to modify the step's comment or action.",
            ),
        ),
    ])
}
