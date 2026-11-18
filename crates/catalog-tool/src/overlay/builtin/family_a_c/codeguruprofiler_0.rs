// Pinned source: .local/aws-models/models/codeguruprofiler/service/2019-07-18/codeguruprofiler-2019-07-18.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "AddNotificationChannels",
            deferred(
                "Model requires profilingGroupName, channels. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Add up to 2 anomaly notifications channels for a profiling group.",
            ),
        ),
        (
            "BatchGetFrameMetricData",
            deferred(
                "Model requires profilingGroupName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the time series of values for a requested list of frame metrics from a time period.",
            ),
        ),
        (
            "ConfigureAgent",
            deferred(
                "Model requires profilingGroupName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Used by profiler agents to report their current state and to receive remote configuration updates. For example, ConfigureAgent can be used to tell an agent whether to profile or not and for how long to return profiling data.",
            ),
        ),
        (
            "CreateProfilingGroup",
            deferred(
                "Model requires profilingGroupName, clientToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a profiling group.",
            ),
        ),
        (
            "DeleteProfilingGroup",
            deferred(
                "Model requires profilingGroupName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a profiling group.",
            ),
        ),
        // Model: Returns a ProfilingGroupDescription object that contains information about the requested profiling group.
        (
            "DescribeProfilingGroup",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("profilingGroupName")?,
                    Binding::From {
                        operation: name("ListProfilingGroups")?,
                        path: path("name")?,
                    },
                )]
                .into_iter()
                .collect(),
                tags: Some(path("tags")?),
                queryable: vec![path("arn")?],
                ..read(
                    "profilingGroup",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "GetFindingsReportAccountSummary",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is reportSummaries; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns a list of FindingsReportSummary objects that contain analysis results for all profiling groups in your AWS account.",
            ),
        ),
        (
            "GetNotificationConfiguration",
            deferred(
                "Model requires profilingGroupName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Get the current configuration for anomaly notifications for a profiling group.",
            ),
        ),
        (
            "GetPolicy",
            deferred(
                "Model requires profilingGroupName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the JSON-formatted resource-based policy on a profiling group.",
            ),
        ),
        (
            "GetProfile",
            deferred(
                "Model requires profilingGroupName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the aggregated profile of a profiling group for a specified time range. Amazon CodeGuru Profiler collects posted agent profiles for a profiling group into aggregated profiles. Because aggregated profiles expire over time…",
            ),
        ),
        (
            "GetRecommendations",
            deferred(
                "Model requires profilingGroupName, startTime, endTime. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of Recommendation objects that contain recommendations for a profiling group for a given time period. A list of Anomaly objects that contains details about anomalies detected in the profiling group for the same…",
            ),
        ),
        (
            "ListFindingsReports",
            deferred(
                "Model requires profilingGroupName, startTime, endTime. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: List the available reports for a given profiling group and time range.",
            ),
        ),
        (
            "ListProfileTimes",
            deferred(
                "Model requires profilingGroupName, startTime, endTime, period. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the start times of the available aggregated profiles of a profiling group for an aggregation period within the specified time range.",
            ),
        ),
        // Model: Returns a list of profiling groups. The profiling groups are returned as ProfilingGroupDescription objects.
        (
            "ListProfilingGroups",
            OperationOverlay {
                fixed: [(path("includeDescription")?, FixedValue::Boolean(true))]
                    .into_iter()
                    .collect(),
                tags: Some(path("tags")?),
                queryable: vec![path("arn")?],
                ..read(
                    "profilingGroups",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of the tags that are assigned to a specified resource.",
            ),
        ),
        (
            "PostAgentProfile",
            deferred(
                "Model requires profilingGroupName, agentProfile, contentType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Submits profiling data to an aggregated profile of a profiling group. To get an aggregated profile that is created with this profiling data, use GetProfile .",
            ),
        ),
        (
            "PutPermission",
            deferred(
                "Model requires profilingGroupName, actionGroup, principals. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds permissions to a profiling group's resource-based policy that are provided using an action group. If a profiling group doesn't have a resource-based policy, one is created for it using the permissions in the action group and…",
            ),
        ),
        (
            "RemoveNotificationChannel",
            deferred(
                "Model requires profilingGroupName, channelId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Remove one anomaly notifications channel for a profiling group.",
            ),
        ),
        (
            "RemovePermission",
            deferred(
                "Model requires profilingGroupName, actionGroup, revisionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes permissions from a profiling group's resource-based policy that are provided using an action group. The one supported action group that can be removed is agentPermission which grants ConfigureAgent and PostAgent…",
            ),
        ),
        (
            "SubmitFeedback",
            deferred(
                "Model requires profilingGroupName, anomalyInstanceId, type. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Sends feedback to CodeGuru Profiler about whether the anomaly detected by the analysis is useful or not.",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceArn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Use to assign one or more tags to a resource.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceArn, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Use to remove one or more tags from a resource.",
            ),
        ),
        (
            "UpdateProfilingGroup",
            deferred(
                "Model requires profilingGroupName, agentOrchestrationConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a profiling group.",
            ),
        ),
    ])
}
