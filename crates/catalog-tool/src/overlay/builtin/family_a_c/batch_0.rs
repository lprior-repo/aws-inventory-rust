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
