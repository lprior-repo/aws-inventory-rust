use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("AddTagsToResource", Decision::Denied(DenyReason::Mutating)),
    (
        "AssociateOpsItemRelatedItem",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CancelCommand", Decision::Denied(DenyReason::Mutating)),
    (
        "CancelMaintenanceWindowExecution",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateActivation", Decision::Denied(DenyReason::Secret)),
    ("CreateAssociation", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateAssociationBatch",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateCloudConnector",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateDocument", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateMaintenanceWindow",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateOpsItem", Decision::Denied(DenyReason::Mutating)),
    ("CreateOpsMetadata", Decision::Denied(DenyReason::Mutating)),
    (
        "CreatePatchBaseline",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateResourceDataSync",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteActivation", Decision::Denied(DenyReason::Mutating)),
    ("DeleteAssociation", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteCloudConnector",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteDocument", Decision::Denied(DenyReason::Mutating)),
    ("DeleteInventory", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteMaintenanceWindow",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteOpsItem", Decision::Denied(DenyReason::Mutating)),
    ("DeleteOpsMetadata", Decision::Denied(DenyReason::Mutating)),
    ("DeleteParameter", Decision::Denied(DenyReason::Mutating)),
    ("DeleteParameters", Decision::Denied(DenyReason::Mutating)),
    (
        "DeletePatchBaseline",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteResourceDataSync",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteResourcePolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeregisterDefaultHostManagementConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeregisterManagedInstance",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeregisterPatchBaselineForPatchGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeregisterTargetFromMaintenanceWindow",
        Decision::Denied(DenyReason::Mutating),
    ),
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
    ("DescribeOpsItems", Decision::Denied(DenyReason::Content)),
];
