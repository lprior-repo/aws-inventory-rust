// Pinned CloudFormation operation decisions continued.
use super::support::*;
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
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("Arn")?,
                    Binding::From {
                        operation: name("ListTypes")?,
                        path: path("TypeArn")?,
                    },
                )]
                .into_iter()
                .collect(),
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
                fixed: [(
                    path("Visibility")?,
                    FixedValue::String("PRIVATE".to_owned()),
                )]
                .into_iter()
                .collect(),
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
