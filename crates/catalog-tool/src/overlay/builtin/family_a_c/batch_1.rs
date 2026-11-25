use super::support::*;
use crate::overlay::MemberPath;
pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
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
                fixed: all_jobs_filter()?,
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
        ("ListServiceJobs", service_jobs()?),
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

fn service_jobs() -> Result<OperationOverlay, NameError> {
    Ok(OperationOverlay {
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
        fixed: all_jobs_filter()?,
        queryable: vec![path("jobArn")?],
        ..read(
            "jobSummaryList",
            Identity::Arn(path("jobArn")?),
            Scope::Region,
            token("nextToken", "nextToken")?,
        )?
    })
}

fn all_jobs_filter() -> Result<std::collections::BTreeMap<MemberPath, FixedValue>, NameError> {
    let filter = FixedValue::Structure(
        [
            ("name".to_owned(), FixedValue::String("JOB_NAME".to_owned())),
            (
                "values".to_owned(),
                FixedValue::List(vec![FixedValue::String("*".to_owned())]),
            ),
        ]
        .into_iter()
        .collect(),
    );
    Ok([(path("filters")?, FixedValue::List(vec![filter]))]
        .into_iter()
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn given_service_jobs_when_planned_then_all_queues_and_statuses_are_covered()
    -> Result<(), Box<dyn std::error::Error>> {
        let compiled = super::super::consumer_fixture::compile(
            "batch",
            "2016-08-10",
            &["DescribeJobQueues", "ListServiceJobs"],
        )?;
        let catalog =
            inventory_core::catalog::runtime::RuntimeCatalog::from_bytes(&compiled.bytes)?;
        let plan = catalog
            .lookup("batch", "ListServiceJobs")
            .ok_or("service jobs missing")?;
        assert!(plan.bindings().iter().any(|binding| matches!(&binding.source,
            inventory_core::catalog::runtime::BindingSource::Operation { operation, .. } if operation.as_ref() == "DescribeJobQueues")));
        assert_eq!(service_jobs()?.fixed, all_jobs_filter()?);
        assert_eq!(service_jobs()?.class, Some(OverlayClass::Expand));
        Ok(())
    }
}
