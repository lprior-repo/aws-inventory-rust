// Pinned operation purposes: .local/aws-models/models/ssm/service/2014-11-06/ssm-2014-11-06.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Adds or overwrites one or more tags for the specified resource. Tags are metadata that you can assign to your automations, documents, managed nodes, maintenance windows, Parameter Store parameters, and patch baselines. Tags enable you to categorize your resources in different ways, for example, by purpose, owner, or environment. Each tag consists of a key and an optional value, both of which you d
    ("AddTagsToResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Associates a related item to a Systems Manager OpsCenter OpsItem. For example, you can associate an Incident Manager incident or analysis with an OpsItem. Incident Manager and OpsCenter are tools in Amazon Web Services Systems Manager.
    (
        "AssociateOpsItemRelatedItem",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Attempts to cancel the command specified by the Command ID. There is no guarantee that the command will be terminated and the underlying process stopped.
    ("CancelCommand", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Stops a maintenance window execution that is already in progress and cancels any tasks in the window that haven't already starting running. Tasks already in progress will continue to completion.
    (
        "CancelMaintenanceWindowExecution",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Full modeled response contains authentication/secret material ActivationCode. Pinned purpose: Generates an activation code and activation ID you can use to register your on-premises servers, edge devices, or virtual machine (VM) with Amazon Web Services Systems Manager. Registering these machines with Systems Manager makes it possible to manage them using Systems Manager tools. You use the a. Projection cannot authorize this call.
    ("CreateActivation", Decision::Denied(DenyReason::Secret)),
    // Changes service state rather than reading inventory. Pinned purpose: A State Manager association defines the state that you want to maintain on your managed nodes. For example, an association can specify that anti-virus software must be installed and running on your managed nodes, or that certain ports must be closed. For static targets, the association specifies a schedule for when the configuration is reapplied. For dynamic targets, such as an Amazon Web Services
    ("CreateAssociation", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Associates the specified Amazon Web Services Systems Manager document (SSM document) with the specified managed nodes or targets. When you associate a document with one or more managed nodes using IDs or tags, Amazon Web Services Systems Manager Agent (SSM Agent) running on the managed node processes the document and configures the node as specified. If you associate a document with a managed node
    (
        "CreateAssociationBatch",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a cloud connector that establishes a connection between Systems Manager and a third-party cloud environment.
    (
        "CreateCloudConnector",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a Amazon Web Services Systems Manager (SSM document). An SSM document defines the actions that Systems Manager performs on your managed nodes. For more information about SSM documents, including information about supported schemas, features, and syntax, see Amazon Web Services Systems Manager Documents in the Amazon Web Services Systems Manager User Guide .
    ("CreateDocument", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new maintenance window. The value you specify for Duration determines the specific end time for the maintenance window based on the time it begins. No maintenance window tasks are permitted to start after the resulting endtime minus the number of hours you specify for Cutoff . For example, if the maintenance window starts at 3 PM, the duration is three hours, and the value you specify fo
    (
        "CreateMaintenanceWindow",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new OpsItem. You must have permission in Identity and Access Management (IAM) to create a new OpsItem. For more information, see Set up OpsCenter in the Amazon Web Services Systems Manager User Guide . Operations engineers and IT professionals use Amazon Web Services Systems Manager OpsCenter to view, investigate, and remediate operational issues impacting the performance and health of t
    ("CreateOpsItem", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: If you create a new application in Application Manager, Amazon Web Services Systems Manager calls this API operation to specify information about the new application, including the application type.
    ("CreateOpsMetadata", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a patch baseline. For information about valid key-value pairs in PatchFilters for each supported operating system type, see PatchFilter .
    (
        "CreatePatchBaseline",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: A resource data sync helps you view data from multiple sources in a single location. Amazon Web Services Systems Manager offers two types of resource data sync: SyncToDestination and SyncFromSource . You can configure Systems Manager Inventory to use the SyncToDestination type to synchronize Inventory data from multiple Amazon Web Services Regions to a single Amazon Simple Storage Service (Amazon
    (
        "CreateResourceDataSync",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an activation. You aren't required to delete an activation. If you delete an activation, you can no longer use it to register additional managed nodes. Deleting an activation doesn't de-register managed nodes. You must manually de-register managed nodes.
    ("DeleteActivation", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Disassociates the specified Amazon Web Services Systems Manager document (SSM document) from the specified managed node. If you created the association by using the Targets parameter, then you must delete the association by using the association ID. When you disassociate a document from a managed node, it doesn't change the configuration of the node. To change the configuration state of a managed
    ("DeleteAssociation", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a cloud connector.
    (
        "DeleteCloudConnector",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the Amazon Web Services Systems Manager document (SSM document) and all managed node associations to the document. Before you delete the document, we recommend that you use DeleteAssociation to disassociate all managed nodes that are associated with the document.
    ("DeleteDocument", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Delete a custom inventory type or the data associated with a custom Inventory type. Deleting a custom inventory type is also referred to as deleting a custom inventory schema.
    ("DeleteInventory", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a maintenance window.
    (
        "DeleteMaintenanceWindow",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Delete an OpsItem. You must have permission in Identity and Access Management (IAM) to delete an OpsItem. Note the following important information about this operation. Deleting an OpsItem is irreversible. You can't restore a deleted OpsItem. This operation uses an eventual consistency model , which means the system can take a few minutes to complete this operation. If you delete an OpsItem and im
    ("DeleteOpsItem", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Delete OpsMetadata related to an application.
    ("DeleteOpsMetadata", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Delete a parameter from the system. After deleting a parameter, wait for at least 30 seconds to create a parameter with the same name. This operation is authorized against the ssm:DeleteParameter action. Denying that action alone doesn't restrict DeleteParameters , which is authorized against its own action. For more information, see Restricting access to specific parameters and paths in the Amazo
    ("DeleteParameter", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Delete a list of parameters. After deleting a parameter, wait for at least 30 seconds to create a parameter with the same name. This operation is authorized against the ssm:DeleteParameters action, not against ssm:DeleteParameter . Denying ssm:DeleteParameter for a parameter doesn't prevent a principal from deleting that parameter with this operation. For more information, see Restricting access t
    ("DeleteParameters", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a patch baseline.
    (
        "DeletePatchBaseline",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a resource data sync configuration. After the configuration is deleted, changes to data on managed nodes are no longer synced to or from the target. Deleting a sync configuration doesn't delete data.
    (
        "DeleteResourceDataSync",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a Systems Manager resource policy. A resource policy helps you to define the IAM entity (for example, an Amazon Web Services account) that can manage your Systems Manager resources. The following resources support Systems Manager resource policies. OpsItemGroup - The resource policy for OpsItemGroup enables Amazon Web Services accounts to view and interact with OpsCenter operational work i
    (
        "DeleteResourcePolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Removes the Default Host Management Configuration (DHMC) for an Amazon Web Services account and Amazon Web Services Region. After you deregister DHMC, Amazon EC2 instances are no longer automatically managed as managed nodes by Amazon Web Services Systems Manager. To use this operation, you must have the following permissions: You need the ssm:GetServiceSetting and ssm:ResetServiceSetting permissi
    (
        "DeregisterDefaultHostManagementConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Removes the server or virtual machine from the list of registered servers. If you want to reregister an on-premises server, edge device, or VM, you must use a different Activation Code and Activation ID than used to register the machine previously. The Activation Code and Activation ID must not have already been used on the maximum number of activations specified when they were created. For more i
    (
        "DeregisterManagedInstance",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Removes a patch group from a patch baseline.
    (
        "DeregisterPatchBaselineForPatchGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Removes a target from a maintenance window.
    (
        "DeregisterTargetFromMaintenanceWindow",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Removes a task from a maintenance window.
    (
        "DeregisterTaskFromMaintenanceWindow",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DescribeAssociation",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members AssociationDescription. Pinned request/ownership contract: Describes the association for the specified target or managed node. If you created the association by using the Targets parameter, then you must retrieve the association by using the association ID.",
        ),
    ),
    (
        "DescribeAssociationExecutionTargets",
        Decision::Deferred(
            "Required selectors AssociationId, ExecutionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AssociationExecutionTargets, NextToken. Pinned purpose: Views information about a specific execution of a specific association.",
        ),
    ),
    (
        "DescribeAssociationExecutions",
        Decision::Deferred(
            "Required selectors AssociationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AssociationExecutions, NextToken. Pinned purpose: Views all executions for a specific association ID.",
        ),
    ),
    (
        "DescribeAutomationExecutions",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members AutomationExecutionMetadataList, NextToken. Pinned request/ownership contract: Provides details about all active and terminated Automation executions.",
        ),
    ),
    (
        "DescribeAutomationStepExecutions",
        Decision::Deferred(
            "Required selectors AutomationExecutionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members StepExecutions, NextToken. Pinned purpose: Information about all active and terminated step executions in an Automation workflow.",
        ),
    ),
    // Public/service reference catalog or supported-capability enumeration, not account-owned resource inventory. Pinned purpose: Lists all patches eligible to be included in a patch baseline. Currently, DescribeAvailablePatches supports only the Amazon Linux 1, Amazon Linux 2, and Windows Server operating systems.
    (
        "DescribeAvailablePatches",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "DescribeDocument",
        Decision::Deferred(
            "Required selectors Name are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Document. Pinned purpose: Describes the specified Amazon Web Services Systems Manager document (SSM document).",
        ),
    ),
    (
        "DescribeDocumentPermission",
        Decision::Deferred(
            "Required selectors Name, PermissionType are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AccountIds, AccountSharingInfoList, NextToken. Pinned purpose: Describes the permissions for a Amazon Web Services Systems Manager document (SSM document). If you created the document, you are the owner. If a document is shared, it can either be shared privately (by specifying a user's Amazon Web Services accoun",
        ),
    ),
    (
        "DescribeEffectiveInstanceAssociations",
        Decision::Deferred(
            "Required selectors InstanceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Associations, NextToken. Pinned purpose: All associations for the managed nodes.",
        ),
    ),
    (
        "DescribeEffectivePatchesForPatchBaseline",
        Decision::Deferred(
            "Required selectors BaselineId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members EffectivePatches, NextToken. Pinned purpose: Retrieves the current effective patches (the patch and the approval state) for the specified patch baseline. Applies to patch baselines for Windows only.",
        ),
    ),
    (
        "DescribeInstanceAssociationsStatus",
        Decision::Deferred(
            "Required selectors InstanceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members InstanceAssociationStatusInfos, NextToken. Pinned purpose: The status of the associations for the managed nodes.",
        ),
    ),
    (
        "DescribeInstancePatchStates",
        Decision::Deferred(
            "Required selectors InstanceIds are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members InstancePatchStates, NextToken. Pinned purpose: Retrieves the high-level patch state of one or more managed nodes.",
        ),
    ),
    (
        "DescribeInstancePatchStatesForPatchGroup",
        Decision::Deferred(
            "Required selectors PatchGroup are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members InstancePatchStates, NextToken. Pinned purpose: Retrieves the high-level patch state for the managed nodes in the specified patch group.",
        ),
    ),
    (
        "DescribeInstancePatches",
        Decision::Deferred(
            "Required selectors InstanceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Patches, NextToken. Pinned purpose: Retrieves information about the patches on the specified managed node and their state relative to the patch baseline being used for the node.",
        ),
    ),
    (
        "DescribeInstanceProperties",
        Decision::Deferred(
            "Instance properties duplicate managed-instance inventory with different optional identity alternatives. DescribeInstanceInformation provides a stable InstanceId root; properties require an explicit documented join.",
        ),
    ),
    (
        "DescribeMaintenanceWindowExecutionTaskInvocations",
        Decision::Deferred(
            "Required selectors WindowExecutionId, TaskId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members WindowExecutionTaskInvocationIdentities, NextToken. Pinned purpose: Retrieves the individual task executions (one per target) for a particular task run as part of a maintenance window execution.",
        ),
    ),
    (
        "DescribeMaintenanceWindowExecutionTasks",
        Decision::Deferred(
            "Required selectors WindowExecutionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members WindowExecutionTaskIdentities, NextToken. Pinned purpose: For a given maintenance window execution, lists the tasks that were run.",
        ),
    ),
    (
        "DescribeMaintenanceWindowExecutions",
        Decision::Deferred(
            "Required selectors WindowId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members WindowExecutions, NextToken. Pinned purpose: Lists the executions of a maintenance window. This includes information about when the maintenance window was scheduled to be active, and information about tasks registered and run with the maintenance window.",
        ),
    ),
    (
        "DescribeMaintenanceWindowSchedule",
        Decision::Deferred(
            "Returns projected future scheduled execution times; these are derived occurrences rather than stable configured resources. DescribeMaintenanceWindows is the root.",
        ),
    ),
    (
        "DescribeMaintenanceWindowTasks",
        Decision::Deferred(
            "Required selectors WindowId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Tasks, NextToken. Pinned purpose: Lists the tasks in a maintenance window. For maintenance window tasks without a specified target, you can't supply values for --max-errors and --max-concurrency . Instead, the system inserts a placeholder value of 1 , which may be reported in the res",
        ),
    ),
    (
        "DescribeMaintenanceWindowsForTarget",
        Decision::Deferred(
            "Required selectors Targets, ResourceType are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members WindowIdentities, NextToken. Pinned purpose: Retrieves information about the maintenance window targets or tasks that a managed node is associated with.",
        ),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Query a set of OpsItems. You must have permission in Identity and Access Management (IAM) to query a list of OpsItems. For more information, see Set up OpsCenter in the Amazon Web Services Systems Manager User Guide . Operations engineers and IT professionals use Amazon Web Services Systems Manager OpsCenter to view, investigate, and remediate operational issues impacting the performance and healt
    ("DescribeOpsItems", Decision::Denied(DenyReason::Content)),
];
