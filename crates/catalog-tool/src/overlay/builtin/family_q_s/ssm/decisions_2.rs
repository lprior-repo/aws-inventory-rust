// Pinned operation purposes: .local/aws-models/models/ssm/service/2014-11-06/ssm-2014-11-06.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Removes tag keys from the specified resource.
    (
        "RemoveTagsFromResource",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: ServiceSetting is an account-level setting for an Amazon Web Services service. This setting defines how a user interacts with or uses a service or a feature of a service. For example, if an Amazon Web Services service charges money to the account based on feature or service usage, then the Amazon Web Services service team might create a default setting of "false". This means the user can't use thi
    (
        "ResetServiceSetting",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Reconnects a session to a managed node after it has been disconnected. Connections can be resumed for disconnected sessions, but not terminated sessions. This command is primarily for use by client machines to automatically reconnect during intermittent network issues. It isn't intended for any other use.
    ("ResumeSession", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Sends a signal to an Automation execution to change the current behavior or status of the execution.
    (
        "SendAutomationSignal",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Runs commands on one or more managed nodes.
    ("SendCommand", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Starts the workflow for just-in-time node access sessions.
    ("StartAccessRequest", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Runs an association immediately and only one time. This operation can be helpful when troubleshooting associations.
    (
        "StartAssociationsOnce",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Initiates execution of an Automation runbook.
    (
        "StartAutomationExecution",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Amazon Web Services Systems Manager Change Manager is no longer open to new customers. Existing customers can continue to use the service as normal. For more information, see Amazon Web Services Systems Manager Change Manager availability change . Creates a change request for Change Manager. The Automation runbooks specified in the change request run only after all required approvals for the chang
    (
        "StartChangeRequestExecution",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Initiates the process of creating a preview showing the effects that running a specified Automation runbook would have on the targeted resources.
    (
        "StartExecutionPreview",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Initiates a connection to a target (for example, a managed node) for a Session Manager session. Returns a URL and token that can be used to open a WebSocket connection for sending input and receiving outputs. Amazon Web Services CLI usage: start-session is an interactive command that requires the Session Manager plugin to be installed on the client machine making the call. For information, see Ins
    ("StartSession", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Stop an Automation that is currently running.
    (
        "StopAutomationExecution",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Permanently ends a session and closes the data connection between the Session Manager client and SSM Agent on the managed node. A terminated session can't be resumed.
    ("TerminateSession", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Remove a label or labels from a parameter. Parameter names can't contain spaces. The service removes any spaces specified for the beginning or end of a parameter name. If the specified name for a parameter contains spaces between characters, the request fails with a ValidationException error.
    (
        "UnlabelParameterVersion",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an association. You can update the association name and version, the document version, schedule, parameters, and Amazon Simple Storage Service (Amazon S3) output. When you call UpdateAssociation , the system removes all optional parameters from the request and overwrites the association with null values for those parameters. This is by design. You must specify all optional parameters in th
    ("UpdateAssociation", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the status of the Amazon Web Services Systems Manager document (SSM document) associated with the specified managed node. UpdateAssociationStatus is primarily used by the Amazon Web Services Systems Manager Agent (SSM Agent) to report status updates about your associations and is only used for associations created with the InstanceId legacy parameter.
    (
        "UpdateAssociationStatus",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing cloud connector with new configuration details.
    (
        "UpdateCloudConnector",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates one or more values for an SSM document.
    ("UpdateDocument", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Set the default version of a document. If you change a document version for a State Manager association, Systems Manager immediately runs the association unless you previously specifed the apply-only-at-cron-interval parameter.
    (
        "UpdateDocumentDefaultVersion",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Amazon Web Services Systems Manager Change Manager is no longer open to new customers. Existing customers can continue to use the service as normal. For more information, see Amazon Web Services Systems Manager Change Manager availability change . Updates information related to approval reviews for a specific version of a change template in Change Manager.
    (
        "UpdateDocumentMetadata",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing maintenance window. Only specified parameters are modified. The value you specify for Duration determines the specific end time for the maintenance window based on the time it begins. No maintenance window tasks are permitted to start after the resulting endtime minus the number of hours you specify for Cutoff . For example, if the maintenance window starts at 3 PM, the duratio
    (
        "UpdateMaintenanceWindow",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Modifies the target of an existing maintenance window. You can change the following: Name Description Owner IDs for an ID target Tags for a Tag target From any supported tag type to another. The three supported tag types are ID target, Tag target, and resource group. For more information, see Target . If a parameter is null, then the corresponding field isn't modified.
    (
        "UpdateMaintenanceWindowTarget",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Modifies a task assigned to a maintenance window. You can't change the task type, but you can change the following values: TaskARN . For example, you can change a RUN_COMMAND task from AWS-RunPowerShellScript to AWS-RunShellScript . ServiceRoleArn TaskInvocationParameters Priority MaxConcurrency MaxErrors One or more targets must be specified for maintenance window Run Command-type tasks. Dependin
    (
        "UpdateMaintenanceWindowTask",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Changes the Identity and Access Management (IAM) role that is assigned to the on-premises server, edge device, or virtual machines (VM). IAM roles are first assigned to these hybrid nodes during the activation process. For more information, see CreateActivation .
    (
        "UpdateManagedInstanceRole",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Edit or change an OpsItem. You must have permission in Identity and Access Management (IAM) to update an OpsItem. For more information, see Set up OpsCenter in the Amazon Web Services Systems Manager User Guide . Operations engineers and IT professionals use Amazon Web Services Systems Manager OpsCenter to view, investigate, and remediate operational issues impacting the performance and health of
    ("UpdateOpsItem", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Amazon Web Services Systems Manager calls this API operation when you edit OpsMetadata in Application Manager.
    ("UpdateOpsMetadata", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Modifies an existing patch baseline. Fields not specified in the request are left unchanged. For information about valid key-value pairs in PatchFilters for each supported operating system type, see PatchFilter .
    (
        "UpdatePatchBaseline",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Update a resource data sync. After you create a resource data sync for a Region, you can't change the account options for that sync. For example, if you create a sync in the us-east-2 (Ohio) Region and you choose the Include only the current account option, you can't edit that sync later and choose the Include all accounts from my Organizations configuration option. Instead, you must delete the fi
    (
        "UpdateResourceDataSync",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: ServiceSetting is an account-level setting for an Amazon Web Services service. This setting defines how a user interacts with or uses a service or a feature of a service. For example, if an Amazon Web Services service charges money to the account based on feature or service usage, then the Amazon Web Services service team might create a default setting of "false". This means the user can't use thi
    (
        "UpdateServiceSetting",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ValidateCloudConnector",
        Decision::Deferred(
            "Required selectors CloudConnectorId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ValidationFindings, NextToken. Pinned purpose: Validates the configuration and connectivity of a cloud connector.",
        ),
    ),
];
