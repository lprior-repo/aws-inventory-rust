// Pinned CloudFormation operation decisions continued.
use super::support::*;
pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "EstimateTemplateCost",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is Url; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns the estimated monthly cost of a template. The return value is an Amazon Web Services Simple Monthly Calculator URL with a query string that describes the resources required to run the template.",
            ),
        ),
        (
            "ExecuteChangeSet",
            deferred(
                "Model requires ChangeSetName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a stack using the input information that was provided when the specified change set was created. After the call successfully completes, CloudFormation starts updating the stack. Use the DescribeStacks action to view the…",
            ),
        ),
        (
            "ExecuteStackRefactor",
            deferred(
                "Model requires StackRefactorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Executes the stack refactor operation.",
            ),
        ),
        (
            "GetGeneratedTemplate",
            deferred(
                "Model requires GeneratedTemplateName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a generated template. If the template is in an InProgress or Pending status then the template returned will be the template when the template was last in a Complete status. If the template has not yet been in a Complete…",
            ),
        ),
        (
            "GetHookResult",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is HookResultId, InvocationPoint, FailureMode, TypeName, OriginalTypeName, TypeVersionId, TypeConfigurationVersionId, TypeArn, Status, HookStatusReason, InvokedAt, Target, Annotations; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Retrieves detailed information and remediation guidance for a Hook invocation result. If the Hook uses a KMS key to encrypt annotations, callers of the GetHookResult operation must have kms:Decrypt permissions. For more…",
            ),
        ),
        (
            "GetStackPolicy",
            deferred(
                "Model requires StackName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the stack policy for a specified stack. If a stack doesn't have a policy, a null value is returned.",
            ),
        ),
        (
            "GetTemplate",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is TemplateBody, StagesAvailable; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns the template body for a specified stack. You can get the template for running or deleted stacks. For deleted stacks, GetTemplate returns the template for up to 90 days after the stack has been deleted. If the template…",
            ),
        ),
        (
            "GetTemplateSummary",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is Parameters, Description, Capabilities, CapabilitiesReason, ResourceTypes, Version, Metadata, DeclaredTransforms, ResourceIdentifierSummaries, Warnings; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns information about a new or existing template. The GetTemplateSummary action is useful for viewing parameter information, such as default parameter values and parameter types, before you create or update a stack or…",
            ),
        ),
        (
            "ImportStacksToStackSet",
            deferred(
                "Model requires StackSetName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Import existing stacks into a new StackSets. Use the stack import operation to import up to 10 stacks into a new StackSet in the same account as the source stack or in a different administrator account and Region, by specifying…",
            ),
        ),
        (
            "ListChangeSets",
            deferred(
                "Model requires StackName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the ID and status of each active change set for a stack. For example, CloudFormation lists change sets that are in the CREATE_IN_PROGRESS or CREATE_PENDING state.",
            ),
        ),
        (
            "ListExports",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is Exports; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists all exported output values in the account and Region in which you call this action. Use this action to see the exported output values that you can import into other stacks. To import values, use the Fn::ImportValue…",
            ),
        ),
        // Model: Lists your generated templates in this Region.
        (
            "ListGeneratedTemplates",
            OperationOverlay {
                queryable: vec![path("GeneratedTemplateId")?],
                ..read(
                    "Summaries",
                    Identity::Key(vec![path("GeneratedTemplateId")?]),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListHookResults",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is TargetType, TargetId, HookResults; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns summaries of invoked Hooks. For more information, see View invocation summaries for CloudFormation Hooks in the CloudFormation Hooks User Guide . This operation supports the following parameter combinations: No…",
            ),
        ),
        (
            "ListImports",
            deferred(
                "Model requires ExportName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all stacks that are importing an exported output value. To modify or remove an exported output value, first use this action to see which stacks are using it. To see the exported output values in your account, see…",
            ),
        ),
        (
            "ListResourceScanRelatedResources",
            deferred(
                "Model requires ResourceScanId, Resources. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the related resources for a list of resources from a resource scan. The response indicates whether each returned resource is already managed by CloudFormation.",
            ),
        ),
        (
            "ListResourceScanResources",
            deferred(
                "Model requires ResourceScanId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the resources from a resource scan. The results can be filtered by resource identifier, resource type prefix, tag key, and tag value. Only resources that match all specified filters are returned. The response indicates…",
            ),
        ),
        // Model: List the resource scans from newest to oldest. By default it will return up to 10 resource scans.
        (
            "ListResourceScans",
            OperationOverlay {
                queryable: vec![path("ResourceScanId")?],
                ..read(
                    "ResourceScanSummaries",
                    Identity::Key(vec![path("ResourceScanId")?]),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListStackInstanceResourceDrifts",
            deferred(
                "Model requires StackSetName, StackInstanceAccount, StackInstanceRegion, OperationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns drift information for resources in a stack instance. ListStackInstanceResourceDrifts returns drift information for the most recent drift detection operation. If an operation is in progress, it may only return partial…",
            ),
        ),
        (
            "ListStackInstances",
            deferred(
                "Model requires StackSetName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns summary information about stack instances that are associated with the specified StackSet. You can filter for stack instances that are associated with a specific Amazon Web Services account name or Region, or that have a…",
            ),
        ),
        (
            "ListStackRefactorActions",
            deferred(
                "Model requires StackRefactorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the stack refactor actions that will be taken after calling the ExecuteStackRefactor action.",
            ),
        ),
        (
            "ListStackRefactors",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is StackRefactorSummaries; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists all account stack refactor operations and their statuses.",
            ),
        ),
        (
            "ListStackResources",
            deferred(
                "Model requires StackName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns descriptions of all resources of the specified stack. For deleted stacks, ListStackResources returns resource information for up to 90 days after the stack has been deleted.",
            ),
        ),
        (
            "ListStackSetAutoDeploymentTargets",
            deferred(
                "Model requires StackSetName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns summary information about deployment targets for a StackSet.",
            ),
        ),
        (
            "ListStackSetOperationResults",
            deferred(
                "Model requires StackSetName, OperationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns summary information about the results of a StackSet operation. This API provides eventually consistent reads meaning it may take some time but will eventually return the most up-to-date data.",
            ),
        ),
        (
            "ListStackSetOperations",
            deferred(
                "Model requires StackSetName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns summary information about operations performed on a StackSet. This API provides eventually consistent reads meaning it may take some time but will eventually return the most up-to-date data.",
            ),
        ),
        // Model: Returns summary information about StackSets that are associated with the user. This API provides strongly consistent reads meaning it will always return the most up-to-date data. [Self-managed…
        (
            "ListStackSets",
            OperationOverlay {
                queryable: vec![path("StackSetId")?],
                ..read(
                    "Summaries",
                    Identity::Key(vec![path("StackSetId")?]),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
    ])
}
