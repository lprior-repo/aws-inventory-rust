// Explicit dispositions from .local/aws-models/models/deadline/service/2023-10-12/deadline-2023-10-12.json; never a runtime read-name approval rule.
pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "DeleteLicenseEndpoint",
        "Requires licenseEndpointId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a license endpoint.",
    ),
    (
        "DeleteLimit",
        "Requires farmId, limitId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes a limit from the specified farm.",
    ),
    (
        "DeleteMeteredProduct",
        "Requires licenseEndpointId, productId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a metered product.",
    ),
    (
        "DeleteMonitor",
        "Requires monitorId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes a Deadline Cloud monitor.",
    ),
    (
        "DeleteQueue",
        "Requires farmId, queueId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a queue.",
    ),
    (
        "DeleteQueueEnvironment",
        "Requires farmId, queueId, queueEnvironmentId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a queue environment.",
    ),
    (
        "DeleteQueueFleetAssociation",
        "Requires farmId, queueId, fleetId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a queue-fleet association.",
    ),
    (
        "DeleteQueueLimitAssociation",
        "Requires farmId, queueId, limitId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes the association between a queue and a limit.",
    ),
    (
        "DeleteStorageProfile",
        "Requires farmId, storageProfileId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a storage profile.",
    ),
    (
        "DeleteVolume",
        "Requires farmId, fleetId, volumeId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a persistent volume.",
    ),
    (
        "DeleteWorker",
        "Requires farmId, fleetId, workerId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a worker.",
    ),
    (
        "DisassociateMemberFromFarm",
        "Requires farmId, principalId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Disassociates a member from a farm.",
    ),
    (
        "DisassociateMemberFromFleet",
        "Requires farmId, fleetId, principalId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Disassociates a member from a fleet.",
    ),
    (
        "DisassociateMemberFromJob",
        "Requires farmId, queueId, jobId, principalId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Disassociates a member from a job.",
    ),
    (
        "DisassociateMemberFromQueue",
        "Requires farmId, queueId, principalId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Disassociates a member from a queue.",
    ),
    (
        "GetBudget",
        "Requires farmId, budgetId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Get a budget.",
    ),
    (
        "GetFarm",
        "Requires farmId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Get a farm.",
    ),
    (
        "GetFleet",
        "Requires farmId, fleetId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Get a fleet.",
    ),
    (
        "GetJob",
        "Requires farmId, queueId, jobId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets a Deadline Cloud job.",
    ),
    (
        "GetLicenseEndpoint",
        "Requires licenseEndpointId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets a licence endpoint.",
    ),
    (
        "GetLimit",
        "Requires farmId, limitId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets information about a specific limit.",
    ),
    (
        "GetMonitor",
        "Requires monitorId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets information about the specified monitor.",
    ),
    (
        "GetMonitorSettings",
        "Requires monitorId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets the settings for a Deadline Cloud monitor.",
    ),
    (
        "GetQueue",
        "Requires farmId, queueId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets a queue.",
    ),
    (
        "GetQueueEnvironment",
        "Requires farmId, queueId, queueEnvironmentId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets a queue environment.",
    ),
    (
        "GetQueueFleetAssociation",
        "Requires farmId, queueId, fleetId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets a queue-fleet association.",
    ),
    (
        "GetQueueLimitAssociation",
        "Requires farmId, queueId, limitId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets information about a specific association between a queue and a limit.",
    ),
    (
        "GetSession",
        "Requires farmId, queueId, jobId, sessionId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets a session.",
    ),
    (
        "GetSessionAction",
        "Requires farmId, queueId, jobId, sessionActionId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets a session action for the job.",
    ),
    (
        "GetSessionsStatisticsAggregation",
        "Requires farmId, aggregationId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets a set of statistics for queues or farms.",
    ),
    (
        "GetStep",
        "Requires farmId, queueId, jobId, stepId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets a step.",
    ),
    (
        "GetStorageProfile",
        "Requires farmId, storageProfileId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets a storage profile.",
    ),
    (
        "GetStorageProfileForQueue",
        "Requires farmId, queueId, storageProfileId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets a storage profile for a queue.",
    ),
    (
        "GetTask",
        "Requires farmId, queueId, jobId, stepId, taskId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets a task.",
    ),
    (
        "GetVolume",
        "Requires farmId, fleetId, volumeId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets a persistent volume.",
    ),
];
