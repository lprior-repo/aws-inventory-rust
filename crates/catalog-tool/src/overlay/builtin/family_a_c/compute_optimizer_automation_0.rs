// Pinned source: .local/aws-models/models/compute-optimizer-automation/service/2025-09-22/compute-optimizer-automation-2025-09-22.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "AssociateAccounts",
            deferred(
                "Model requires accountIds. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates one or more member accounts with your organization's management account, enabling centralized implementation of optimization actions across those accounts. Once associated, the management account (or a delegated…",
            ),
        ),
        (
            "CreateAutomationRule",
            deferred(
                "Model requires name, ruleType, recommendedActionTypes, schedule, status. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new automation rule to apply recommended actions to resources based on specified criteria.",
            ),
        ),
        (
            "DeleteAutomationRule",
            deferred(
                "Model requires ruleArn, ruleRevision. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing automation rule.",
            ),
        ),
        (
            "DisassociateAccounts",
            deferred(
                "Model requires accountIds. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Disassociates member accounts from your organization's management account, removing centralized automation capabilities. Once disassociated, organization rules no longer apply to the member account, and the management account (or…",
            ),
        ),
        (
            "GetAutomationEvent",
            deferred(
                "Model requires eventId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves details about a specific automation event.",
            ),
        ),
        (
            "GetAutomationRule",
            deferred(
                "Model requires ruleArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves details about a specific automation rule.",
            ),
        ),
        (
            "GetEnrollmentConfiguration",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is status, statusReason, organizationRuleMode, lastUpdatedTimestamp; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Retrieves the current enrollment configuration for Compute Optimizer Automation.",
            ),
        ),
        (
            "ListAccounts",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is accounts; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists the accounts in your organization that are enrolled in Compute Optimizer and whether they have enabled Automation. Only the management account or a delegated administrator can perform this action.",
            ),
        ),
        (
            "ListAutomationEventSteps",
            deferred(
                "Model requires eventId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the steps for a specific automation event. You can only list steps for events created within the past year.",
            ),
        ),
        (
            "ListAutomationEventSummaries",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is automationEventSummaries; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Provides a summary of automation events based on specified filters. Only events created within the past year will be included in the summary.",
            ),
        ),
        // Model: Lists automation events based on specified filters. You can retrieve events that were created within the past year.
        (
            "ListAutomationEvents",
            OperationOverlay {
                queryable: vec![path("eventId")?],
                ..read(
                    "automationEvents",
                    Identity::Key(vec![path("eventId")?]),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListAutomationRulePreview",
            deferred(
                "Model requires ruleType, recommendedActionTypes. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a preview of the recommended actions that match your Automation rule's configuration and criteria.",
            ),
        ),
        (
            "ListAutomationRulePreviewSummaries",
            deferred(
                "Model requires ruleType, recommendedActionTypes. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a summary of the recommended actions that match your rule preview configuration and criteria.",
            ),
        ),
        // Model: Lists the automation rules that match specified filters.
        (
            "ListAutomationRules",
            OperationOverlay {
                queryable: vec![path("ruleArn")?],
                ..read(
                    "automationRules",
                    Identity::Arn(path("ruleArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListRecommendedActionSummaries",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is recommendedActionSummaries; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Provides a summary of recommended actions based on specified filters. Management accounts and delegated administrators can retrieve recommended actions that include associated member accounts. You can associate a member account…",
            ),
        ),
        (
            "ListRecommendedActions",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is recommendedActions; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists the recommended actions based that match specified filters. Management accounts and delegated administrators can retrieve recommended actions that include associated member accounts. You can associate a member account using…",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the tags for a specified resource.",
            ),
        ),
        (
            "RollbackAutomationEvent",
            deferred(
                "Model requires eventId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Initiates a rollback for a completed automation event. Management accounts and delegated administrators can only initiate a rollback for events belonging to associated member accounts. You can associate a member account using…",
            ),
        ),
        (
            "StartAutomationEvent",
            deferred(
                "Model requires recommendedActionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Initiates a one-time, on-demand automation for the specified recommended action. Management accounts and delegated administrators can only initiate recommended actions for associated member accounts. You can associate a member…",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceArn, ruleRevision, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds tags to the specified resource.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceArn, ruleRevision, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes tags from the specified resource.",
            ),
        ),
        (
            "UpdateAutomationRule",
            deferred(
                "Model requires ruleArn, ruleRevision. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing automation rule.",
            ),
        ),
        (
            "UpdateEnrollmentConfiguration",
            deferred(
                "Model requires status. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates your account’s Compute Optimizer Automation enrollment configuration.",
            ),
        ),
    ])
}
