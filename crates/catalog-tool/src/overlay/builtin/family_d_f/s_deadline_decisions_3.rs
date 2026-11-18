// Explicit dispositions from .local/aws-models/models/deadline/service/2023-10-12/deadline-2023-10-12.json; never a runtime read-name approval rule.
pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "UpdateBudget",
        "Requires farmId, budgetId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a budget that sets spending thresholds for rendering activity.",
    ),
    (
        "UpdateFarm",
        "Requires farmId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a farm.",
    ),
    (
        "UpdateFleet",
        "Requires farmId, fleetId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a fleet.",
    ),
    (
        "UpdateJob",
        "Requires farmId, queueId, jobId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a job.",
    ),
    (
        "UpdateLimit",
        "Requires farmId, limitId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the properties of the specified limit.",
    ),
    (
        "UpdateMonitor",
        "Requires monitorId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies the settings for a Deadline Cloud monitor.",
    ),
    (
        "UpdateMonitorSettings",
        "Requires monitorId, settings; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the settings for a Deadline Cloud monitor.",
    ),
    (
        "UpdateQueue",
        "Requires farmId, queueId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a queue.",
    ),
    (
        "UpdateQueueEnvironment",
        "Requires farmId, queueId, queueEnvironmentId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the queue environment.",
    ),
    (
        "UpdateQueueFleetAssociation",
        "Requires farmId, queueId, fleetId, status; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a queue-fleet association.",
    ),
    (
        "UpdateQueueLimitAssociation",
        "Requires farmId, queueId, limitId, status; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the status of the queue.",
    ),
    (
        "UpdateSession",
        "Requires farmId, queueId, jobId, sessionId, targetLifecycleStatus; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a session.",
    ),
    (
        "UpdateStep",
        "Requires farmId, queueId, jobId, stepId, targetTaskRunStatus; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a step.",
    ),
    (
        "UpdateStorageProfile",
        "Requires farmId, storageProfileId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a storage profile.",
    ),
    (
        "UpdateTask",
        "Requires farmId, queueId, jobId, stepId, taskId, targetRunStatus; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a task.",
    ),
    (
        "UpdateWorker",
        "Requires farmId, fleetId, workerId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a worker.",
    ),
    (
        "UpdateWorkerSchedule",
        "Requires farmId, fleetId, workerId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the schedule for a worker.",
    ),
];
