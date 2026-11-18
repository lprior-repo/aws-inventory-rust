// Pinned source: .local/aws-models/models/batch/service/2016-08-10/batch-2016-08-10.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "CancelJob",
            deferred(
                "Model requires jobId, reason. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Cancels a job in an Batch job queue. Jobs that are in a SUBMITTED , PENDING , or RUNNABLE state are cancelled and the job status is updated to FAILED . A PENDING job is cancelled after all dependency jobs are completed.…",
            ),
        ),
        (
            "CancelJobs",
            deferred(
                "Model requires jobs, reason. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Cancels up to 50 jobs in an Batch job queue. This is a bulk version of CancelJob . Jobs that are in a SUBMITTED , PENDING , or RUNNABLE state are cancelled and the job status is updated to FAILED . A PENDING job is cancelled…",
            ),
        ),
        (
            "CreateComputeEnvironment",
            deferred(
                "Model requires computeEnvironmentName, type. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an Batch compute environment. You can create MANAGED or UNMANAGED compute environments. MANAGED compute environments can use Amazon EC2 or Fargate resources. UNMANAGED compute environments can only use EC2 resources. In a…",
            ),
        ),
        (
            "CreateConsumableResource",
            deferred(
                "Model requires consumableResourceName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an Batch consumable resource.",
            ),
        ),
        (
            "CreateJobQueue",
            deferred(
                "Model requires jobQueueName, priority. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an Batch job queue. When you create a job queue, you associate one or more compute environments to the queue and assign an order of preference for the compute environments. You also set a priority to the job queue that…",
            ),
        ),
        (
            "CreateQuotaShare",
            deferred(
                "Model requires quotaShareName, jobQueue, capacityLimits, resourceSharingConfiguration, preemptionConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an Batch quota share. Each quota share operates as a virtual queue with a configured compute capacity, resource sharing strategy, and borrow limits.",
            ),
        ),
        (
            "CreateSchedulingPolicy",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an Batch scheduling policy.",
            ),
        ),
        (
            "CreateServiceEnvironment",
            deferred(
                "Model requires serviceEnvironmentName, serviceEnvironmentType, capacityLimits. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a service environment for running service jobs. Service environments define capacity limits for specific service types such as SageMaker Training jobs.",
            ),
        ),
        (
            "DeleteComputeEnvironment",
            deferred(
                "Model requires computeEnvironment. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an Batch compute environment. Before you can delete a compute environment, you must set its state to DISABLED with the UpdateComputeEnvironment API operation and disassociate it from any job queues with the UpdateJobQueue…",
            ),
        ),
        (
            "DeleteConsumableResource",
            deferred(
                "Model requires consumableResource. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified consumable resource.",
            ),
        ),
        (
            "DeleteJobQueue",
            deferred(
                "Model requires jobQueue. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified job queue. You must first disable submissions for a queue with the UpdateJobQueue operation. All jobs in the queue are eventually terminated when you delete a job queue. It's not necessary to disassociate…",
            ),
        ),
        (
            "DeleteQuotaShare",
            deferred(
                "Model requires quotaShareArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified quota share. You must first disable submissions for the share by updating the state to DISABLED using the UpdateQuotaShare operation. All jobs in the share are eventually terminated when you delete a quota…",
            ),
        ),
        (
            "DeleteSchedulingPolicy",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified scheduling policy. You can't delete a scheduling policy that's used in any job queues.",
            ),
        ),
        (
            "DeleteServiceEnvironment",
            deferred(
                "Model requires serviceEnvironment. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a Service environment. Before you can delete a service environment, you must first set its state to DISABLED with the UpdateServiceEnvironment API operation and disassociate it from any job queues with the UpdateJobQueue…",
            ),
        ),
        (
            "DeregisterJobDefinition",
            deferred(
                "Model requires jobDefinition. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deregisters an Batch job definition. Job definitions are permanently deleted after 180 days.",
            ),
        ),
        // Model: Describes one or more of your compute environments. If you're using an unmanaged compute environment, you can use the DescribeComputeEnvironment operation to determine the ecsClusterArn that you…
        (
            "DescribeComputeEnvironments",
            OperationOverlay {
                tags: Some(path("tags")?),
                queryable: vec![path("computeEnvironmentArn")?],
                ..read(
                    "computeEnvironments",
                    Identity::Arn(path("computeEnvironmentArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "DescribeConsumableResource",
            deferred(
                "Model requires consumableResource. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a description of the specified consumable resource.",
            ),
        ),
        // Model: Describes a list of job definitions. You can specify a status (such as ACTIVE ) to only return job definitions that match that status.
        ("DescribeJobDefinitions", denied(DenyReason::Secret)),
        // Model: Describes one or more of your job queues.
        (
            "DescribeJobQueues",
            OperationOverlay {
                tags: Some(path("tags")?),
                queryable: vec![path("jobQueueArn")?],
                ..read(
                    "jobQueues",
                    Identity::Arn(path("jobQueueArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "DescribeJobs",
            deferred(
                "Model requires jobs. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes a list of Batch jobs.",
            ),
        ),
        (
            "DescribeQuotaShare",
            deferred(
                "Model requires quotaShareArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a description of the specified quota share.",
            ),
        ),
        (
            "DescribeSchedulingPolicies",
            deferred(
                "Model requires arns. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes one or more of your scheduling policies.",
            ),
        ),
        // Model: Describes one or more of your service environments.
        (
            "DescribeServiceEnvironments",
            OperationOverlay {
                tags: Some(path("tags")?),
                queryable: vec![path("serviceEnvironmentArn")?],
                ..read(
                    "serviceEnvironments",
                    Identity::Arn(path("serviceEnvironmentArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "DescribeServiceJob",
            deferred(
                "Model requires jobId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: The details of a service job.",
            ),
        ),
        (
            "GetJobQueueSnapshot",
            deferred(
                "Model requires jobQueue. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Provides a snapshot of job queue state, including ordering of RUNNABLE jobs, as well as capacity utilization for already dispatched jobs. The first 100 RUNNABLE jobs in the job queue are listed in order of dispatch. For job…",
            ),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        // Model: Returns a list of Batch consumable resources.
        (
            "ListConsumableResources",
            OperationOverlay {
                queryable: vec![path("consumableResourceArn")?],
                ..read(
                    "consumableResources",
                    Identity::Arn(path("consumableResourceArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // JOB_NAME's trailing wildcard covers every top-level job name and suppresses RUNNING-only defaults.
        (
            "ListJobs",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("jobQueue")?,
                    Binding::From {
                        operation: name("DescribeJobQueues")?,
                        path: path("jobQueueArn")?,
                    },
                )]
                .into_iter()
                .collect(),
                fixed: [(
                    path("filters")?,
                    FixedValue::List(vec![FixedValue::Structure(
                        [
                            ("name".to_owned(), FixedValue::String("JOB_NAME".to_owned())),
                            (
                                "values".to_owned(),
                                FixedValue::List(vec![FixedValue::String("*".to_owned())]),
                            ),
                        ]
                        .into_iter()
                        .collect(),
                    )]),
                )]
                .into_iter()
                .collect(),
                ..read(
                    "jobSummaryList",
                    Identity::Key(vec![path("jobId")?]),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListJobsByConsumableResource",
            deferred(
                "Model requires consumableResource. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of Batch jobs that require a specific consumable resource.",
            ),
        ),
        (
            "ListQuotaShares",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("jobQueue")?,
                    Binding::From {
                        operation: name("DescribeJobQueues")?,
                        path: path("jobQueueArn")?,
                    },
                )]
                .into_iter()
                .collect(),
                ..read(
                    "quotaShares",
                    Identity::Arn(path("quotaShareArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // Model: Returns a list of Batch scheduling policies.
        (
            "ListSchedulingPolicies",
            OperationOverlay {
                queryable: vec![path("arn")?],
                ..read(
                    "schedulingPolicies",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListServiceJobs",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is jobSummaryList; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns a list of service jobs for a specified job queue.",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the tags for an Batch resource. Batch resources that support tags are compute environments, jobs, job definitions, job queues, and scheduling policies. ARNs for child jobs of array and multi-node parallel (MNP) jobs aren't…",
            ),
        ),
        (
            "RegisterJobDefinition",
            deferred(
                "Model requires jobDefinitionName, type. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Registers an Batch job definition.",
            ),
        ),
        (
            "SubmitJob",
            deferred(
                "Model requires jobName, jobQueue, jobDefinition. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Submits an Batch job from a job definition. Parameters that are specified during SubmitJob override parameters defined in the job definition. vCPU and memory requirements that are specified in the resourceRequirements objects in…",
            ),
        ),
        (
            "SubmitServiceJob",
            deferred(
                "Model requires jobName, jobQueue, serviceRequestPayload, serviceJobType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Submits a service job to a specified job queue to run on SageMaker AI. A service job is a unit of work that you submit to Batch for execution on SageMaker AI.",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceArn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates the specified tags to a resource with the specified resourceArn . If existing tags on a resource aren't specified in the request parameters, they aren't changed. When a resource is deleted, the tags that are associated…",
            ),
        ),
        (
            "TerminateJob",
            deferred(
                "Model requires jobId, reason. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Terminates a job in a job queue. Jobs that are in the STARTING or RUNNING state are terminated, which causes them to transition to FAILED . Jobs that have not progressed to the STARTING state are cancelled.",
            ),
        ),
        (
            "TerminateJobs",
            deferred(
                "Model requires jobs, reason. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Terminates up to 50 jobs in a job queue. This is a bulk version of TerminateJob . Jobs that are in the STARTING or RUNNING state are terminated, which causes them to transition to FAILED . Jobs that have not progressed to the…",
            ),
        ),
        (
            "TerminateServiceJob",
            deferred(
                "Model requires jobId, reason. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Terminates a service job in a job queue.",
            ),
        ),
        (
            "TerminateServiceJobs",
            deferred(
                "Model requires jobs, reason. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Terminates up to 50 service jobs in a job queue. This is a bulk version of TerminateServiceJob . Batch reports the result for each service job individually in the response. Service jobs that were processed successfully are…",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceArn, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes specified tags from an Batch resource.",
            ),
        ),
        (
            "UpdateComputeEnvironment",
            deferred(
                "Model requires computeEnvironment. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an Batch compute environment.",
            ),
        ),
        (
            "UpdateConsumableResource",
            deferred(
                "Model requires consumableResource. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a consumable resource.",
            ),
        ),
        (
            "UpdateJobQueue",
            deferred(
                "Model requires jobQueue. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a job queue.",
            ),
        ),
        (
            "UpdateQuotaShare",
            deferred(
                "Model requires quotaShareArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a quota share.",
            ),
        ),
        (
            "UpdateSchedulingPolicy",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a scheduling policy.",
            ),
        ),
        (
            "UpdateServiceEnvironment",
            deferred(
                "Model requires serviceEnvironment. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a service environment. You can update the state of a service environment from ENABLED to DISABLED to prevent new service jobs from being placed in the service environment.",
            ),
        ),
        (
            "UpdateServiceJob",
            deferred(
                "Model requires jobId, schedulingPriority. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the priority of a specified service job in an Batch job queue.",
            ),
        ),
    ])
}
