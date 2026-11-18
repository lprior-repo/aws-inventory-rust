// Explicit dispositions from .local/aws-models/models/emr/service/2009-03-31/emr-2009-03-31.json; never a runtime read-name approval rule.
pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "ListSteps",
        "Requires ClusterId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Provides a list of steps for the cluster in reverse order unless you specify stepIds with the request or filter by StepStates .",
    ),
    (
        "ListSupportedInstanceTypes",
        "Requires ReleaseLabel; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: A list of the instance types that Amazon EMR supports.",
    ),
    (
        "ModifyCluster",
        "Requires ClusterId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies the number of steps that can be executed concurrently for the cluster specified using ClusterID.",
    ),
    (
        "ModifyInstanceFleet",
        "Requires ClusterId, InstanceFleet; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies the target On-Demand and target Spot capacities for the instance fleet with the specified InstanceFleetID within the cluster specified using ClusterID.",
    ),
    (
        "ModifyInstanceGroups",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: ModifyInstanceGroups modifies the number of nodes and configuration settings of an instance group.",
    ),
    (
        "PutAutoScalingPolicy",
        "Requires ClusterId, InstanceGroupId, AutoScalingPolicy; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates or updates an automatic scaling policy for a core instance group or task instance group in an Amazon EMR cluster.",
    ),
    (
        "PutAutoTerminationPolicy",
        "Requires ClusterId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Auto-termination is supported in Amazon EMR releases 5.30.0 and 6.1.0 and later.",
    ),
    (
        "PutBlockPublicAccessConfiguration",
        "Requires BlockPublicAccessConfiguration; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates or updates an Amazon EMR block public access configuration for your Amazon Web Services account in the current Region.",
    ),
    (
        "PutManagedScalingPolicy",
        "Requires ClusterId, ManagedScalingPolicy; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates or updates a managed scaling policy for an Amazon EMR cluster.",
    ),
    (
        "RemoveAutoScalingPolicy",
        "Requires ClusterId, InstanceGroupId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes an automatic scaling policy from a specified instance group within an Amazon EMR cluster.",
    ),
    (
        "RemoveAutoTerminationPolicy",
        "Requires ClusterId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes an auto-termination policy from an Amazon EMR cluster.",
    ),
    (
        "RemoveManagedScalingPolicy",
        "Requires ClusterId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes a managed scaling policy from a specified Amazon EMR cluster.",
    ),
    (
        "RemoveTags",
        "Requires ResourceId, TagKeys; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes tags from an Amazon EMR resource, such as a cluster or Amazon EMR Studio.",
    ),
    (
        "RunJobFlow",
        "Requires Name, Instances; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: RunJobFlow creates and starts running a new cluster (job flow).",
    ),
    (
        "SetKeepJobFlowAliveWhenNoSteps",
        "Requires JobFlowIds, KeepJobFlowAliveWhenNoSteps; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: You can use the SetKeepJobFlowAliveWhenNoSteps to configure a cluster (job flow) to terminate after the step execution, i.e., all your steps are executed.",
    ),
    (
        "SetTerminationProtection",
        "Requires JobFlowIds, TerminationProtected; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: SetTerminationProtection locks a cluster (job flow) so the Amazon EC2 instances in the cluster cannot be terminated by user intervention, an API call, or in the event of a job-flow error.",
    ),
    (
        "SetUnhealthyNodeReplacement",
        "Requires JobFlowIds, UnhealthyNodeReplacement; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Specify whether to enable unhealthy node replacement, which lets Amazon EMR gracefully replace core nodes on a cluster if any nodes become unhealthy.",
    ),
    (
        "SetVisibleToAllUsers",
        "Requires JobFlowIds, VisibleToAllUsers; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: The SetVisibleToAllUsers parameter is no longer supported.",
    ),
    (
        "StartNotebookExecution",
        "Requires ExecutionEngine, ServiceRole; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Starts a notebook execution.",
    ),
    (
        "StartSession",
        "Requires ClusterId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates and starts a new Spark Connect session on the specified cluster.",
    ),
    (
        "StopNotebookExecution",
        "Requires NotebookExecutionId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Stops a notebook execution.",
    ),
    (
        "TerminateJobFlows",
        "Requires JobFlowIds; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: TerminateJobFlows shuts a list of clusters (job flows) down.",
    ),
    (
        "TerminateSession",
        "Requires ClusterId, SessionId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Terminates an active session.",
    ),
    (
        "UpdateStudio",
        "Requires StudioId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates an Amazon EMR Studio configuration, including attributes such as name, description, and subnets.",
    ),
    (
        "UpdateStudioSessionMapping",
        "Requires StudioId, IdentityType, SessionPolicyArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the session policy attached to the user or group for the specified Amazon EMR Studio.",
    ),
];
