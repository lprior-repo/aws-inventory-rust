use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "DescribePatchGroupState",
        Decision::Deferred(
            "Required selectors PatchGroup are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Instances, InstancesWithInstalledPatches, InstancesWithInstalledOtherPatches, InstancesWithInstalledPendingRebootPatches, InstancesWithInstalledRejectedPatches, InstancesWithMissingPatches, InstancesWithFailedPatches, InstancesWithNotApplicablePatches, InstancesWithUnreportedNotApplicablePatches, InstancesWithCriticalNonCompliantPatches, InstancesWithSecurityNonCompliantPatches, InstancesWithOtherNonCompliantPatches, InstancesWithAvailableSecurityUpdates. Pinned purpose: Returns high-level aggregated patch compliance state information for a patch group.",
        ),
    ),
    (
        "DescribePatchProperties",
        Decision::Deferred(
            "Required selectors OperatingSystem, Property are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Properties, NextToken. Pinned purpose: Lists the properties of available patches organized by product, product family, classification, severity, and other properties of available patches. You can use the reported properties in the filters you specify in requests for operations such as Cre",
        ),
    ),
    (
        "DescribeSessions",
        Decision::Deferred(
            "Required selectors State are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Sessions, NextToken. Pinned purpose: Retrieves a list of all active sessions (both connected and disconnected) or terminated sessions from the past 30 days.",
        ),
    ),
    (
        "DisassociateOpsItemRelatedItem",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("GetAccessToken", Decision::Denied(DenyReason::Secret)),
    (
        "GetAutomationExecution",
        Decision::Deferred(
            "Required selectors AutomationExecutionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AutomationExecution. Pinned purpose: Get detailed information about a particular Automation execution.",
        ),
    ),
    (
        "GetCalendarState",
        Decision::Deferred(
            "Required selectors CalendarNames are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members State, AtTime, NextTransitionTime. Pinned purpose: Gets the state of a Amazon Web Services Systems Manager change calendar at the current time or a specified time. If you specify a time, GetCalendarState returns the state of the calendar at that specific time, and returns the next time that the chang",
        ),
    ),
    (
        "GetCloudConnector",
        Decision::Deferred(
            "Required selectors CloudConnectorId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members CloudConnectorArn, DisplayName, Description, RoleArn, Configuration, ConfigConnectorArn, CreatedAt, UpdatedAt. Pinned purpose: Returns detailed information about a cloud connector.",
        ),
    ),
    (
        "GetCommandInvocation",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "GetConnectionStatus",
        Decision::Deferred(
            "Required selectors Target are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Target, Status. Pinned purpose: Retrieves the Session Manager connection status for a managed node to determine whether it is running and ready to receive Session Manager connections.",
        ),
    ),
    (
        "GetDefaultPatchBaseline",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members BaselineId, OperatingSystem. Pinned request/ownership contract: Retrieves the default patch baseline. Amazon Web Services Systems Manager supports creating multiple default patch baselines. For example, you can create a default patch baseline for each operating system. If you don't specify an operating system value, the default patch baseline for Windows is returned.",
        ),
    ),
    (
        "GetDeployablePatchSnapshotForInstance",
        Decision::Deferred(
            "Required selectors InstanceId, SnapshotId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members InstanceId, SnapshotId, SnapshotDownloadUrl, Product. Pinned purpose: Retrieves the current snapshot for the patch baseline the managed node uses. This API is primarily used by the AWS-RunPatchBaseline Systems Manager document (SSM document). If you run the command locally, such as with the Command Line Interface (CLI)",
        ),
    ),
    ("GetDocument", Decision::Denied(DenyReason::Content)),
    (
        "GetExecutionPreview",
        Decision::Deferred(
            "Required selectors ExecutionPreviewId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ExecutionPreviewId, EndedAt, Status, StatusMessage, ExecutionPreview. Pinned purpose: Initiates the process of retrieving an existing preview that shows the effects that running a specified Automation runbook would have on the targeted resources.",
        ),
    ),
    ("GetInventory", Decision::Denied(DenyReason::Content)),
    ("GetInventorySchema", Decision::Denied(DenyReason::Content)),
    (
        "GetMaintenanceWindow",
        Decision::Deferred(
            "Required selectors WindowId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members WindowId, Name, Description, StartDate, EndDate, Schedule, ScheduleTimezone, ScheduleOffset, NextExecutionTime, Duration, Cutoff, AllowUnassociatedTargets, Enabled, CreatedDate, ModifiedDate. Pinned purpose: Retrieves a maintenance window.",
        ),
    ),
    (
        "GetMaintenanceWindowExecution",
        Decision::Deferred(
            "Required selectors WindowExecutionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members WindowExecutionId, TaskIds, Status, StatusDetails, StartTime, EndTime. Pinned purpose: Retrieves details about a specific a maintenance window execution.",
        ),
    ),
    (
        "GetMaintenanceWindowExecutionTask",
        Decision::Deferred(
            "Required selectors WindowExecutionId, TaskId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members WindowExecutionId, TaskExecutionId, TaskArn, ServiceRole, Type, TaskParameters, Priority, MaxConcurrency, MaxErrors, Status, StatusDetails, StartTime, EndTime, AlarmConfiguration, TriggeredAlarms. Pinned purpose: Retrieves the details about a specific task run as part of a maintenance window execution.",
        ),
    ),
    (
        "GetMaintenanceWindowExecutionTaskInvocation",
        Decision::Deferred(
            "Required selectors WindowExecutionId, TaskId, InvocationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members WindowExecutionId, TaskExecutionId, InvocationId, ExecutionId, TaskType, Parameters, Status, StatusDetails, StartTime, EndTime, OwnerInformation, WindowTargetId. Pinned purpose: Retrieves information about a specific task running on a specific target.",
        ),
    ),
    (
        "GetMaintenanceWindowTask",
        Decision::Deferred(
            "Required selectors WindowId, WindowTaskId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members WindowId, WindowTaskId, Targets, TaskArn, ServiceRoleArn, TaskType, TaskParameters, TaskInvocationParameters, Priority, MaxConcurrency, MaxErrors, LoggingInfo, Name, Description, CutoffBehavior, AlarmConfiguration. Pinned purpose: Retrieves the details of a maintenance window task. For maintenance window tasks without a specified target, you can't supply values for --max-errors and --max-concurrency . Instead, the system inserts a placeholder value of 1 , which may be reported",
        ),
    ),
    ("GetOpsItem", Decision::Denied(DenyReason::Content)),
    (
        "GetOpsMetadata",
        Decision::Deferred(
            "Required selectors OpsMetadataArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ResourceId, Metadata, NextToken. Pinned purpose: View operational metadata related to an application in Application Manager.",
        ),
    ),
    ("GetOpsSummary", Decision::Denied(DenyReason::Content)),
    ("GetParameter", Decision::Denied(DenyReason::Content)),
    ("GetParameterHistory", Decision::Denied(DenyReason::Content)),
    ("GetParameters", Decision::Denied(DenyReason::Content)),
    ("GetParametersByPath", Decision::Denied(DenyReason::Content)),
    (
        "GetPatchBaseline",
        Decision::Deferred(
            "Required selectors BaselineId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members BaselineId, Name, OperatingSystem, GlobalFilters, ApprovalRules, ApprovedPatches, ApprovedPatchesComplianceLevel, ApprovedPatchesEnableNonSecurity, RejectedPatches, RejectedPatchesAction, PatchGroups, CreatedDate, ModifiedDate, Description, Sources, AvailableSecurityUpdatesComplianceStatus. Pinned purpose: Retrieves information about a patch baseline.",
        ),
    ),
    (
        "GetPatchBaselineForPatchGroup",
        Decision::Deferred(
            "Required selectors PatchGroup are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members BaselineId, PatchGroup, OperatingSystem. Pinned purpose: Retrieves the patch baseline that should be used for the specified patch group.",
        ),
    ),
    (
        "GetResourcePolicies",
        Decision::Deferred(
            "Required selectors ResourceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NextToken, Policies. Pinned purpose: Returns an array of the Policy object.",
        ),
    ),
    (
        "GetServiceSetting",
        Decision::Deferred(
            "Required selectors SettingId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ServiceSetting. Pinned purpose: ServiceSetting is an account-level setting for an Amazon Web Services service. This setting defines how a user interacts with or uses a service or a feature of a service. For example, if an Amazon Web Services service charges money to the account bas",
        ),
    ),
    (
        "LabelParameterVersion",
        Decision::Deferred(
            "Required selectors Name, Labels are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members InvalidLabels, ParameterVersion. Pinned purpose: A parameter label is a user-defined alias to help you manage different versions of a parameter. When you modify a parameter, Amazon Web Services Systems Manager automatically saves a new version and increments the version number by one. A label can h",
        ),
    ),
    (
        "ListAssociationVersions",
        Decision::Deferred(
            "Required selectors AssociationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AssociationVersions, NextToken. Pinned purpose: Retrieves all versions of an association for a specific association ID.",
        ),
    ),
    (
        "ListCommandInvocations",
        Decision::Denied(DenyReason::Content),
    ),
    ("ListCommands", Decision::Denied(DenyReason::Content)),
    ("ListComplianceItems", Decision::Denied(DenyReason::Content)),
    (
        "ListDocumentMetadataHistory",
        Decision::Deferred(
            "Required selectors Name, Metadata are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Name, DocumentVersion, Author, Metadata, NextToken. Pinned purpose: Amazon Web Services Systems Manager Change Manager is no longer open to new customers. Existing customers can continue to use the service as normal. For more information, see Amazon Web Services Systems Manager Change Manager availability change . In",
        ),
    ),
    (
        "ListDocumentVersions",
        Decision::Deferred(
            "Required selectors Name are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DocumentVersions, NextToken. Pinned purpose: List all versions for a document.",
        ),
    ),
    (
        "ListInventoryEntries",
        Decision::Deferred(
            "Required selectors InstanceId, TypeName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members TypeName, InstanceId, SchemaVersion, CaptureTime, Entries, NextToken. Pinned purpose: A list of inventory items returned by the request.",
        ),
    ),
    (
        "ListNodes",
        Decision::Deferred(
            "Nodes carry Owner and Region and can span an organization. Current per-operation account/Region scope cannot route each node to its actual owner/Region.",
        ),
    ),
    (
        "ListNodesSummary",
        Decision::Deferred(
            "Required selectors Aggregators are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Summary, NextToken. Pinned purpose: Generates a summary of managed instance/node metadata based on the filters and aggregators you specify. Results are grouped by the input aggregator you specify.",
        ),
    ),
    ("ListOpsItemEvents", Decision::Denied(DenyReason::Content)),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: ResourceType, ResourceId. Pinned purpose: Returns a list of the tags assigned to the specified resource. For information about the ID format for each supported resource type, see AddTagsToResource .",
        ),
    ),
    (
        "ModifyDocumentPermission",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("PutComplianceItems", Decision::Denied(DenyReason::Mutating)),
    ("PutInventory", Decision::Denied(DenyReason::Mutating)),
    ("PutParameter", Decision::Denied(DenyReason::Mutating)),
    ("PutResourcePolicy", Decision::Denied(DenyReason::Mutating)),
    (
        "RegisterDefaultHostManagementConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "RegisterDefaultPatchBaseline",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "RegisterPatchBaselineForPatchGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "RegisterTargetWithMaintenanceWindow",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "RegisterTaskWithMaintenanceWindow",
        Decision::Denied(DenyReason::Mutating),
    ),
];
