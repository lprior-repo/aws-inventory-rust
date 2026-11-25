pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "ListStagingAccounts",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Returns an array of staging accounts for existing extended source servers.",
    ),
    (
        "ListTagsForResource",
        "Requires resourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: List all tags for your Elastic Disaster Recovery resources.",
    ),
    (
        "PutLaunchAction",
        "Requires resourceId, actionCode, order, actionId, optional, active, name, actionVersion, category, description; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Puts a resource launch action.",
    ),
    (
        "ReorderRecoveryPlanSteps",
        "Requires recoveryPlanArn, orderedStepArns; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Reorders steps in a Recovery Plan.",
    ),
    (
        "RetryDataReplication",
        "Requires sourceServerID; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: WARNING: RetryDataReplication is deprecated.",
    ),
    (
        "RetryRecoveryPlanExecutionStep",
        "Requires recoveryPlanExecutionStepArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retries a failed SERVER type execution step.",
    ),
    (
        "ReverseReplication",
        "Requires recoveryInstanceID; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Start replication to origin / target region - applies only to protected instances that originated in EC2.",
    ),
    (
        "StartFailbackLaunch",
        "Requires recoveryInstanceIDs; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Initiates a Job for launching the machine that is being failed back to from the specified Recovery Instance.",
    ),
    (
        "StartRecovery",
        "Requires sourceServers; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Launches Recovery Instances for the specified Source Servers.",
    ),
    (
        "StartRecoveryPlanExecution",
        "Requires recoveryPlanArn, mode; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Starts executing a Recovery Plan in DRILL or RECOVERY mode.",
    ),
    (
        "StartReplication",
        "Requires sourceServerID; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Starts replication for a stopped Source Server.",
    ),
    (
        "StartSourceNetworkRecovery",
        "Requires sourceNetworks; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deploy VPC for the specified Source Network and modify launch templates to use this network.",
    ),
    (
        "StartSourceNetworkReplication",
        "Requires sourceNetworkID; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Starts replication for a Source Network.",
    ),
    (
        "StopFailback",
        "Requires recoveryInstanceID; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Stops the failback process for a specified Recovery Instance.",
    ),
    (
        "StopReplication",
        "Requires sourceServerID; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Stops replication for a Source Server.",
    ),
    (
        "StopSourceNetworkReplication",
        "Requires sourceNetworkID; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Stops replication for a Source Network.",
    ),
    (
        "TagResource",
        "Requires resourceArn, tags; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Adds or overwrites only the specified tags for the specified Elastic Disaster Recovery resource or resources.",
    ),
    (
        "TerminateRecoveryInstances",
        "Requires recoveryInstanceIDs; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Initiates a Job for terminating the EC2 resources associated with the specified Recovery Instances, and then will delete the Recovery Instances from the Elastic Disaster Recovery service.",
    ),
    (
        "UntagResource",
        "Requires resourceArn, tagKeys; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified set of tags from the specified set of Elastic Disaster Recovery resources.",
    ),
    (
        "UpdateFailbackReplicationConfiguration",
        "Requires recoveryInstanceID; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Allows you to update the failback replication configuration of a Recovery Instance by ID.",
    ),
    (
        "UpdateLaunchConfiguration",
        "Requires sourceServerID; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a LaunchConfiguration by Source Server ID.",
    ),
    (
        "UpdateLaunchConfigurationTemplate",
        "Requires launchConfigurationTemplateID; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates an existing Launch Configuration Template by ID.",
    ),
    (
        "UpdateRecoveryPlan",
        "Requires recoveryPlanArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a Recovery Plan's name or description.",
    ),
    (
        "UpdateRecoveryPlanExecutionStep",
        "Requires recoveryPlanExecutionStepArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates an execution step.",
    ),
    (
        "UpdateRecoveryPlanStep",
        "Requires recoveryPlanStepArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a Recovery Plan step's name or configuration.",
    ),
    (
        "UpdateReplicationConfiguration",
        "Requires sourceServerID; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Allows you to update a ReplicationConfiguration by Source Server ID.",
    ),
    (
        "UpdateReplicationConfigurationTemplate",
        "Requires replicationConfigurationTemplateID; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a ReplicationConfigurationTemplate by ID.",
    ),
];
