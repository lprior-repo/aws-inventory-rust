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
