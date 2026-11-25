use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "AcknowledgeJob",
            deferred(
                "Model requires jobId, nonce. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about a specified job and whether that job has been received by the job worker. Used for custom actions only.",
            ),
        ),
        (
            "AcknowledgeThirdPartyJob",
            deferred(
                "Model requires jobId, nonce, clientToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Confirms a job worker has received the specified job. Used for partner actions only.",
            ),
        ),
        (
            "CreateCustomActionType",
            deferred(
                "Model requires category, provider, version, inputArtifactDetails, outputArtifactDetails. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new custom action that can be used in all pipelines associated with the Amazon Web Services account. Only used for custom actions.",
            ),
        ),
        (
            "CreatePipeline",
            deferred(
                "Model requires pipeline. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a pipeline. In the pipeline structure, you must include either artifactStore or artifactStores in your pipeline, but you cannot use both. If you create a cross-region action in your pipeline, you must use artifactStores .",
            ),
        ),
        (
            "DeleteCustomActionType",
            deferred(
                "Model requires category, provider, version. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Marks a custom action as deleted. PollForJobs for the custom action fails after the action is marked for deletion. Used for custom actions only. To re-create a custom action after it has been deleted you must use a string in the…",
            ),
        ),
        (
            "DeletePipeline",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified pipeline.",
            ),
        ),
        (
            "DeleteWebhook",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a previously created webhook by name. Deleting the webhook stops CodePipeline from starting a pipeline every time an external event occurs. The API returns successfully when trying to delete a webhook that is already…",
            ),
        ),
        (
            "DeregisterWebhookWithThirdParty",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is not an inventory record; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Removes the connection between the webhook that was created by CodePipeline and the external tool with events to be detected. Currently supported only for webhooks that target an action type of GitHub.",
            ),
        ),
        (
            "DisableStageTransition",
            deferred(
                "Model requires pipelineName, stageName, transitionType, reason. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Prevents artifacts in a pipeline from transitioning to the next stage in the pipeline.",
            ),
        ),
        (
            "EnableStageTransition",
            deferred(
                "Model requires pipelineName, stageName, transitionType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Enables artifacts in a pipeline to transition to a stage in a pipeline.",
            ),
        ),
        (
            "GetActionType",
            deferred(
                "Model requires category, owner, provider, version. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about an action type created for an external provider, where the action is to be used by customers of the external provider. The action can be created with any supported integration model.",
            ),
        ),
        (
            "GetJobDetails",
            deferred(
                "Model requires jobId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about a job. Used for custom actions only. When this API is called, CodePipeline returns temporary credentials for the S3 bucket used to store artifacts for the pipeline, if the action requires access to that…",
            ),
        ),
        ("GetPipeline", denied(DenyReason::Secret)),
        ("GetPipelineExecution", denied(DenyReason::Secret)),
        (
            "GetPipelineState",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about the state of a pipeline, including the stages and actions. Values returned in the revisionId and revisionUrl fields indicate the source revision information, such as the commit ID, for the current state.",
            ),
        ),
        (
            "GetThirdPartyJobDetails",
            deferred(
                "Model requires jobId, clientToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Requests the details of a job for a third party action. Used for partner actions only. When this API is called, CodePipeline returns temporary credentials for the S3 bucket used to store artifacts for the pipeline, if the action…",
            ),
        ),
        (
            "ListActionExecutions",
            deferred(
                "Model requires pipelineName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the action executions that have occurred in a pipeline.",
            ),
        ),
        (
            "ListActionTypes",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is actionTypes; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Gets a summary of all CodePipeline action types associated with your account.",
            ),
        ),
        (
            "ListDeployActionExecutionTargets",
            deferred(
                "Model requires actionExecutionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the targets for the deploy action.",
            ),
        ),
        (
            "ListPipelineExecutions",
            deferred(
                "Model requires pipelineName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a summary of the most recent executions for a pipeline. When applying the filter for pipeline executions that have succeeded in the stage, the operation returns all executions in the current pipeline version beginning on…",
            ),
        ),
        (
            "ListPipelines",
            OperationOverlay {
                queryable: vec![path("name")?],
                ..read(
                    "pipelines",
                    Identity::Key(vec![path("name")?]),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListRuleExecutions",
            deferred(
                "Model requires pipelineName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the rule executions that have occurred in a pipeline configured for conditions with rules.",
            ),
        ),
        (
            "ListRuleTypes",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is ruleTypes; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists the rules for the condition. For more information about conditions, see Stage conditions and How do stage conditions work? .For more information about rules, see the CodePipeline rule reference .",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the set of key-value pairs (metadata) that are used to manage the resource.",
            ),
        ),
        (
            "ListWebhooks",
            OperationOverlay {
                tags: Some(path("tags")?),
                queryable: vec![path("arn")?],
                ..read(
                    "webhooks",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "OverrideStageCondition",
            deferred(
                "Model requires pipelineName, stageName, pipelineExecutionId, conditionType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Used to override a stage condition. For more information about conditions, see Stage conditions and How do stage conditions work? .",
            ),
        ),
        (
            "PollForJobs",
            deferred(
                "Model requires actionTypeId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about any jobs for CodePipeline to act on. PollForJobs is valid only for action types with \"Custom\" in the owner field. If the action type contains AWS or ThirdParty in the owner field, the PollForJobs action…",
            ),
        ),
        (
            "PollForThirdPartyJobs",
            deferred(
                "Model requires actionTypeId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Determines whether there are any third party jobs for a job worker to act on. Used for partner actions only. When this API is called, CodePipeline returns temporary credentials for the S3 bucket used to store artifacts for the…",
            ),
        ),
        (
            "PutActionRevision",
            deferred(
                "Model requires pipelineName, stageName, actionName, actionRevision. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Provides information to CodePipeline about new revisions to a source.",
            ),
        ),
        (
            "PutApprovalResult",
            deferred(
                "Model requires pipelineName, stageName, actionName, result, token. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Provides the response to a manual approval request to CodePipeline. Valid responses include Approved and Rejected.",
            ),
        ),
        (
            "PutJobFailureResult",
            deferred(
                "Model requires jobId, failureDetails. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Represents the failure of a job as returned to the pipeline by a job worker. Used for custom actions only.",
            ),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "PutJobSuccessResult",
            deferred(
                "Model requires jobId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Represents the success of a job as returned to the pipeline by a job worker. Used for custom actions only.",
            ),
        ),
        (
            "PutThirdPartyJobFailureResult",
            deferred(
                "Model requires jobId, clientToken, failureDetails. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Represents the failure of a third party job as returned to the pipeline by a job worker. Used for partner actions only.",
            ),
        ),
        (
            "PutThirdPartyJobSuccessResult",
            deferred(
                "Model requires jobId, clientToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Represents the success of a third party job as returned to the pipeline by a job worker. Used for partner actions only.",
            ),
        ),
        (
            "PutWebhook",
            deferred(
                "Model requires webhook. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Defines a webhook and returns a unique webhook URL generated by CodePipeline. This URL can be supplied to third party source hosting providers to call every time there's a code change. When CodePipeline receives a POST request on…",
            ),
        ),
        (
            "RegisterWebhookWithThirdParty",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is not an inventory record; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Configures a connection between the webhook that was created and the external tool with events to be detected.",
            ),
        ),
        (
            "RetryStageExecution",
            deferred(
                "Model requires pipelineName, stageName, pipelineExecutionId, retryMode. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: You can retry a stage that has failed without having to run a pipeline again from the beginning. You do this by either retrying the failed actions in a stage or by retrying all actions in the stage starting from the first action…",
            ),
        ),
        (
            "RollbackStage",
            deferred(
                "Model requires pipelineName, stageName, targetPipelineExecutionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Rolls back a stage execution.",
            ),
        ),
        (
            "StartPipelineExecution",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts the specified pipeline. Specifically, it begins processing the latest commit to the source location specified as part of the pipeline.",
            ),
        ),
        (
            "StopPipelineExecution",
            deferred(
                "Model requires pipelineName, pipelineExecutionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops the specified pipeline execution. You choose to either stop the pipeline execution by completing in-progress actions without starting subsequent actions, or by abandoning in-progress actions. While completing or abandoning…",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceArn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds to or modifies the tags of the given resource. Tags are metadata that can be used to manage a resource.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceArn, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes tags from an Amazon Web Services resource.",
            ),
        ),
        (
            "UpdateActionType",
            deferred(
                "Model requires actionType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an action type that was created with any supported integration model, where the action type is to be used by customers of the action type provider. Use a JSON file with the action definition and UpdateActionType to…",
            ),
        ),
        (
            "UpdatePipeline",
            deferred(
                "Model requires pipeline. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a specified pipeline with edits or changes to its structure. Use a JSON file with the pipeline structure and UpdatePipeline to provide the full structure of the pipeline. Updating the pipeline increases the version number…",
            ),
        ),
    ])
}
