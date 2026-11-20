// Pinned source: .local/aws-models/models/cloudformation/service/2010-05-15/cloudformation-2010-05-15.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "ActivateOrganizationsAccess",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is not an inventory record; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Activate trusted access with Organizations. With trusted access between StackSets and Organizations activated, the management account has permissions to create and manage StackSets for your organization.",
            ),
        ),
        (
            "ActivateType",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is Arn; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Activates a public third-party extension, such as a resource or module, to make it available for use in stack templates in your current account and Region. It can also create CloudFormation Hooks, which allow you to evaluate…",
            ),
        ),
        (
            "BatchDescribeTypeConfigurations",
            deferred(
                "Model requires TypeConfigurationIdentifiers. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns configuration data for the specified CloudFormation extensions, from the CloudFormation registry in your current account and Region. For more information, see Edit configuration data for extensions in your account in the…",
            ),
        ),
        (
            "CancelUpdateStack",
            deferred(
                "Model requires StackName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Cancels an update on the specified stack. If the call completes successfully, the stack rolls back the update and reverts to the previous stack configuration. You can cancel only stacks that are in the UPDATE_IN_PROGRESS state.",
            ),
        ),
        (
            "ContinueUpdateRollback",
            deferred(
                "Model requires StackName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Continues rolling back a stack from UPDATE_ROLLBACK_FAILED to UPDATE_ROLLBACK_COMPLETE state. Depending on the cause of the failure, you can manually fix the error and continue the rollback. By continuing the rollback, you can…",
            ),
        ),
        (
            "CreateChangeSet",
            deferred(
                "Model requires StackName, ChangeSetName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a list of changes that will be applied to a stack so that you can review the changes before executing them. You can create a change set for a stack that doesn't exist or an existing stack. If you create a change set for a…",
            ),
        ),
        (
            "CreateGeneratedTemplate",
            deferred(
                "Model requires GeneratedTemplateName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a template from existing resources that are not already managed with CloudFormation. You can check the status of the template generation using the DescribeGeneratedTemplate API action.",
            ),
        ),
        (
            "CreateStack",
            deferred(
                "Model requires StackName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a stack as specified in the template. After the call completes successfully, the stack creation starts. You can check the status of the stack through the DescribeStacks operation. For more information about creating a…",
            ),
        ),
        (
            "CreateStackInstances",
            deferred(
                "Model requires StackSetName, Regions. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates stack instances for the specified accounts, within the specified Amazon Web Services Regions. A stack instance refers to a stack in a specific account and Region. You must specify at least one value for either Accounts or…",
            ),
        ),
        (
            "CreateStackRefactor",
            deferred(
                "Model requires StackDefinitions. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a refactor across multiple stacks, with the list of stacks and resources that are affected.",
            ),
        ),
        (
            "CreateStackSet",
            deferred(
                "Model requires StackSetName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a StackSet.",
            ),
        ),
        (
            "DeactivateOrganizationsAccess",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is not an inventory record; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Deactivates trusted access with Organizations. If trusted access is deactivated, the management account does not have permissions to create and manage service-managed StackSets for your organization.",
            ),
        ),
        (
            "DeactivateType",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is not an inventory record; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Deactivates a public third-party extension, such as a resource or module, or a CloudFormation Hook when you no longer use it. Deactivating an extension deletes the configuration details that are associated with it. To temporarily…",
            ),
        ),
        (
            "DeleteChangeSet",
            deferred(
                "Model requires ChangeSetName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified change set. Deleting change sets ensures that no one executes the wrong change set. If the call successfully completes, CloudFormation successfully deleted the change set. If IncludeNestedStacks specifies…",
            ),
        ),
        (
            "DeleteGeneratedTemplate",
            deferred(
                "Model requires GeneratedTemplateName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deleted a generated template.",
            ),
        ),
        (
            "DeleteStack",
            deferred(
                "Model requires StackName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a specified stack. Once the call completes successfully, stack deletion starts. Deleted stacks don't show up in the DescribeStacks operation if the deletion has been completed successfully. For more information about…",
            ),
        ),
        (
            "DeleteStackInstances",
            deferred(
                "Model requires StackSetName, Regions, RetainStacks. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes stack instances for the specified accounts, in the specified Amazon Web Services Regions. The maximum number of organizational unit (OUs) supported by a DeleteStackInstances operation is 50. If you need more than 50,…",
            ),
        ),
        (
            "DeleteStackSet",
            deferred(
                "Model requires StackSetName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a StackSet. Before you can delete a StackSet, all its member stack instances must be deleted. For more information about how to complete this, see DeleteStackInstances .",
            ),
        ),
        (
            "DeregisterType",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is not an inventory record; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Marks an extension or extension version as DEPRECATED in the CloudFormation registry, removing it from active use. Deprecated extensions or extension versions cannot be used in CloudFormation operations. To deregister an entire…",
            ),
        ),
        (
            "DescribeAccountLimits",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is AccountLimits; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Retrieves your account's CloudFormation limits, such as the maximum number of stacks that you can create in your account. For more information about account limits, see Understand CloudFormation quotas in the CloudFormation User…",
            ),
        ),
        (
            "DescribeChangeSet",
            deferred(
                "Model requires ChangeSetName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the inputs for the change set and a list of changes that CloudFormation will make if you execute the change set. For more information, see Update CloudFormation stacks using change sets in the CloudFormation User Guide .",
            ),
        ),
        (
            "DescribeChangeSetHooks",
            deferred(
                "Model requires ChangeSetName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns Hook-related information for the change set and a list of changes that CloudFormation makes when you run the change set.",
            ),
        ),
        (
            "DescribeEvents",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is OperationEvents; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns CloudFormation events based on flexible query criteria. Groups events by operation ID, enabling you to focus on individual stack operations during deployment. An operation is any action performed on a stack, including…",
            ),
        ),
        (
            "DescribeGeneratedTemplate",
            deferred(
                "Model requires GeneratedTemplateName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes a generated template. The output includes details about the progress of the creation of a generated template started by a CreateGeneratedTemplate API action or the update of a generated template started with an…",
            ),
        ),
        (
            "DescribeOrganizationsAccess",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is Status; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Retrieves information about the account's OrganizationAccess status. This API can be called either by the management account or the delegated administrator by using the CallAs parameter. This API can also be called without the…",
            ),
        ),
        (
            "DescribePublisher",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is PublisherId, PublisherStatus, IdentityProvider, PublisherProfile; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns information about a CloudFormation extension publisher. If you don't supply a PublisherId , and you have registered as an extension publisher, DescribePublisher returns information about your own publisher account. For…",
            ),
        ),
        (
            "DescribeResourceScan",
            deferred(
                "Model requires ResourceScanId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes details of a resource scan.",
            ),
        ),
        (
            "DescribeStackDriftDetectionStatus",
            deferred(
                "Model requires StackDriftDetectionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about a stack drift detection operation. A stack drift detection operation detects whether a stack's actual configuration differs, or has drifted , from its expected configuration, as defined in the stack…",
            ),
        ),
        (
            "DescribeStackEvents",
            deferred(
                "Model requires StackName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns all stack related events for a specified stack in reverse chronological order. For more information about a stack's event history, see Understand CloudFormation stack creation events in the CloudFormation User Guide . You…",
            ),
        ),
        (
            "DescribeStackInstance",
            deferred(
                "Model requires StackSetName, StackInstanceAccount, StackInstanceRegion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the stack instance that's associated with the specified StackSet, Amazon Web Services account, and Amazon Web Services Region. For a list of stack instances that are associated with a specific StackSet, use…",
            ),
        ),
        (
            "DescribeStackRefactor",
            deferred(
                "Model requires StackRefactorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes the stack refactor status.",
            ),
        ),
        (
            "DescribeStackResource",
            deferred(
                "Model requires StackName, LogicalResourceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a description of the specified resource in the specified stack. For deleted stacks, DescribeStackResource returns resource information for up to 90 days after the stack has been deleted.",
            ),
        ),
        (
            "DescribeStackResourceDrifts",
            deferred(
                "Model requires StackName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns drift information for the resources that have been checked for drift in the specified stack. This includes actual and expected configuration values for resources where CloudFormation detects configuration drift. For a…",
            ),
        ),
        (
            "DescribeStackResources",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is StackResources; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns Amazon Web Services resource descriptions for running and deleted stacks. If StackName is specified, all the associated resources that are part of the stack are returned. If PhysicalResourceId is specified, the associated…",
            ),
        ),
        (
            "DescribeStackSet",
            deferred(
                "Model requires StackSetName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the description of the specified StackSet. This API provides strongly consistent reads meaning it will always return the most up-to-date data.",
            ),
        ),
        (
            "DescribeStackSetOperation",
            deferred(
                "Model requires StackSetName, OperationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the description of the specified StackSet operation. This API provides strongly consistent reads meaning it will always return the most up-to-date data.",
            ),
        ),
        (
            "DescribeStacks",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is Stacks; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns the description for the specified stack; if no stack name was specified, then it returns the description for all the stacks created. For more information about a stack's event history, see Understand CloudFormation stack…",
            ),
        ),
        (
            "DescribeType",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is Arn, Type, TypeName, DefaultVersionId, IsDefaultVersion, TypeTestsStatus, TypeTestsStatusDescription, Description, Schema, ProvisioningType, DeprecatedStatus, LoggingConfig, RequiredActivatedTypes, ExecutionRoleArn, Visibility, SourceUrl, DocumentationUrl, LastUpdated, TimeCreated, ConfigurationSchema, PublisherId, OriginalTypeName, OriginalTypeArn, PublicVersionNumber, LatestPublicVersion, IsActivated, AutoUpdate; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns detailed information about an extension from the CloudFormation registry in your current account and Region. If you specify a VersionId , DescribeType returns information about that specific extension version. Otherwise,…",
            ),
        ),
        (
            "DescribeTypeRegistration",
            deferred(
                "Model requires RegistrationToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about an extension's registration, including its current status and type and version identifiers. When you initiate a registration request using RegisterType , you can then use DescribeTypeRegistration to…",
            ),
        ),
        (
            "DetectStackDrift",
            deferred(
                "Model requires StackName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Detects whether a stack's actual configuration differs, or has drifted , from its expected configuration, as defined in the stack template and any values specified as template parameters. For each resource in the stack that…",
            ),
        ),
        (
            "DetectStackResourceDrift",
            deferred(
                "Model requires StackName, LogicalResourceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about whether a resource's actual configuration differs, or has drifted , from its expected configuration, as defined in the stack template and any values specified as template parameters. This information…",
            ),
        ),
        (
            "DetectStackSetDrift",
            deferred(
                "Model requires StackSetName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Detect drift on a StackSet. When CloudFormation performs drift detection on a StackSet, it performs drift detection on the stack associated with each stack instance in the StackSet. For more information, see Performing drift…",
            ),
        ),
    ])
}

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

pub(super) fn part_2() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        // Model: Returns the summary information for stacks whose status matches the specified StackStatusFilter . Summary information for stacks that have been deleted is kept for 90 days after the stack is deleted.…
        (
            "ListStacks",
            OperationOverlay {
                fixed: [(
                    path("StackStatusFilter")?,
                    FixedValue::Strings(vec![
                        "CREATE_IN_PROGRESS".to_owned(),
                        "CREATE_FAILED".to_owned(),
                        "CREATE_COMPLETE".to_owned(),
                        "ROLLBACK_IN_PROGRESS".to_owned(),
                        "ROLLBACK_FAILED".to_owned(),
                        "ROLLBACK_COMPLETE".to_owned(),
                        "DELETE_IN_PROGRESS".to_owned(),
                        "DELETE_FAILED".to_owned(),
                        "UPDATE_IN_PROGRESS".to_owned(),
                        "UPDATE_COMPLETE_CLEANUP_IN_PROGRESS".to_owned(),
                        "UPDATE_COMPLETE".to_owned(),
                        "UPDATE_FAILED".to_owned(),
                        "UPDATE_ROLLBACK_IN_PROGRESS".to_owned(),
                        "UPDATE_ROLLBACK_FAILED".to_owned(),
                        "UPDATE_ROLLBACK_COMPLETE_CLEANUP_IN_PROGRESS".to_owned(),
                        "UPDATE_ROLLBACK_COMPLETE".to_owned(),
                        "REVIEW_IN_PROGRESS".to_owned(),
                        "IMPORT_IN_PROGRESS".to_owned(),
                        "IMPORT_COMPLETE".to_owned(),
                        "IMPORT_ROLLBACK_IN_PROGRESS".to_owned(),
                        "IMPORT_ROLLBACK_FAILED".to_owned(),
                        "IMPORT_ROLLBACK_COMPLETE".to_owned(),
                    ]),
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("StackId")?],
                ..read(
                    "StackSummaries",
                    Identity::Arn(path("StackId")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListTypeRegistrations",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is not an inventory record; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns a list of registration tokens for the specified extension(s).",
            ),
        ),
        (
            "ListTypeVersions",
            OperationOverlay {
                queryable: vec![path("Arn")?],
                ..read(
                    "TypeVersionSummaries",
                    Identity::Arn(path("Arn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListTypes",
            OperationOverlay {
                queryable: vec![path("TypeArn")?],
                ..read(
                    "TypeSummaries",
                    Identity::Arn(path("TypeArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "PublishType",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is PublicTypeArn; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Publishes the specified extension to the CloudFormation registry as a public extension in this Region. Public extensions are available for use by all CloudFormation users. For more information about publishing extensions, see…",
            ),
        ),
        (
            "RecordHandlerProgress",
            deferred(
                "Model requires BearerToken, OperationStatus. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Reports progress of a resource handler to CloudFormation. Reserved for use by the CloudFormation CLI . Don't use this API in your code.",
            ),
        ),
        (
            "RegisterPublisher",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is PublisherId; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Registers your account as a publisher of public extensions in the CloudFormation registry. Public extensions are available for use by all CloudFormation users. This publisher ID applies to your account in all Amazon Web Services…",
            ),
        ),
        (
            "RegisterType",
            deferred(
                "Model requires TypeName, SchemaHandlerPackage. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Registers an extension with the CloudFormation service. Registering an extension makes it available for use in CloudFormation templates in your Amazon Web Services account, and includes: Validating the extension schema.…",
            ),
        ),
        (
            "RemoveAlias",
            deferred(
                "Model requires AliasName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes an alias from a stack. After the alias is removed successfully, it can no longer be used to reference the stack in API calls.",
            ),
        ),
        (
            "RollbackStack",
            deferred(
                "Model requires StackName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: When specifying RollbackStack , you preserve the state of previously provisioned resources when an operation fails. You can check the status of the stack through the DescribeStacks operation. Rolls back the specified stack to the…",
            ),
        ),
        (
            "SetAlias",
            deferred(
                "Model requires StackName, AliasName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Assigns an alias to an existing stack. An alias is an alternative name for a stack that can be used in place of the stack name in any CloudFormation API call that accepts the StackName parameter. This enables you to effectively…",
            ),
        ),
        (
            "SetStackPolicy",
            deferred(
                "Model requires StackName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Sets a stack policy for a specified stack.",
            ),
        ),
        (
            "SetTypeConfiguration",
            deferred(
                "Model requires Configuration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Specifies the configuration data for a CloudFormation extension, such as a resource or Hook, in the given account and Region. For more information, see Edit configuration data for extensions in your account in the CloudFormation…",
            ),
        ),
        (
            "SetTypeDefaultVersion",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is not an inventory record; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Specify the default version of an extension. The default version of an extension will be used in CloudFormation operations.",
            ),
        ),
        (
            "SignalResource",
            deferred(
                "Model requires StackName, LogicalResourceId, UniqueId, Status. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Sends a signal to the specified resource with a success or failure status. You can use the SignalResource operation in conjunction with a creation policy or update policy. CloudFormation doesn't proceed with a stack creation or…",
            ),
        ),
        (
            "StartResourceScan",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is ResourceScanId; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Starts a scan of the resources in this account in this Region. You can the status of a scan using the ListResourceScans API action.",
            ),
        ),
        (
            "StopStackSetOperation",
            deferred(
                "Model requires StackSetName, OperationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops an in-progress operation on a StackSet and its associated stack instances. StackSets will cancel all the unstarted stack instance deployments and wait for those are in-progress to complete.",
            ),
        ),
        (
            "TestType",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is TypeVersionArn; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Tests a registered extension to make sure it meets all necessary requirements for being published in the CloudFormation registry. For resource types, this includes passing all contracts tests defined for the type. For modules,…",
            ),
        ),
        (
            "UpdateGeneratedTemplate",
            deferred(
                "Model requires GeneratedTemplateName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a generated template. This can be used to change the name, add and remove resources, refresh resources, and change the DeletionPolicy and UpdateReplacePolicy settings. You can check the status of the update to the…",
            ),
        ),
        (
            "UpdateStack",
            deferred(
                "Model requires StackName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a stack as specified in the template. After the call completes successfully, the stack update starts. You can check the status of the stack through the DescribeStacks action. To get a copy of the template for an existing…",
            ),
        ),
        (
            "UpdateStackInstances",
            deferred(
                "Model requires StackSetName, Regions. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the parameter values for stack instances for the specified accounts, within the specified Amazon Web Services Regions. A stack instance refers to a stack in a specific account and Region. You can only update stack…",
            ),
        ),
        (
            "UpdateStackSet",
            deferred(
                "Model requires StackSetName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the StackSet and associated stack instances in the specified accounts and Amazon Web Services Regions. Even if the StackSet operation created by updating the StackSet fails (completely or partially, below or above a…",
            ),
        ),
        (
            "UpdateTerminationProtection",
            deferred(
                "Model requires EnableTerminationProtection, StackName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates termination protection for the specified stack. If a user attempts to delete a stack with termination protection enabled, the operation fails and the stack remains unchanged. For more information, see Protect a…",
            ),
        ),
        (
            "ValidateTemplate",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is Parameters, Description, Capabilities, CapabilitiesReason, DeclaredTransforms; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Validates a specified template. CloudFormation first checks if the template is valid JSON. If it isn't, CloudFormation checks if the template is valid YAML. If both these checks fail, CloudFormation returns a template validation…",
            ),
        ),
    ])
}
