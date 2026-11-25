use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "BatchCreateRumMetricDefinitions",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchDeleteRumMetricDefinitions",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchGetRumMetricDefinitions",
        Decision::Deferred(
            "Required selectors AppMonitorName, Destination are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members MetricDefinitions, NextToken. Pinned purpose: Retrieves the list of metrics and dimensions that a RUM app monitor is sending to a single destination.",
        ),
    ),
    ("CreateAppMonitor", Decision::Denied(DenyReason::Mutating)),
    ("DeleteAppMonitor", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteResourcePolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteRumMetricsDestination",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetAppMonitor",
        Decision::Deferred(
            "Required selectors Name are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members AppMonitor. Pinned purpose: Retrieves the complete configuration information for one app monitor.",
        ),
    ),
    (
        "GetAppMonitorData",
        Decision::Deferred(
            "Required selectors Name, TimeRange are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Events, NextToken. Pinned purpose: Retrieves the raw performance events that RUM has collected from your web application, so that you can do your own processing or analysis of this data.",
        ),
    ),
    (
        "GetResourcePolicy",
        Decision::Deferred(
            "Required selectors Name are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members PolicyDocument, PolicyRevisionId. Pinned purpose: Use this operation to retrieve information about a resource-based policy that is attached to an app monitor.",
        ),
    ),
    (
        "ListRumMetricsDestinations",
        Decision::Deferred(
            "Required selectors AppMonitorName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Destinations, NextToken. Pinned purpose: Returns a list of destinations that you have created to receive RUM extended metrics, for the specified app monitor. For more information about extended metrics, see AddRumMetrics .",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: ResourceArn. Pinned purpose: Displays the tags associated with a CloudWatch RUM resource.",
        ),
    ),
    ("PutResourcePolicy", Decision::Denied(DenyReason::Mutating)),
    ("PutRumEvents", Decision::Denied(DenyReason::Mutating)),
    (
        "PutRumMetricsDestination",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateAppMonitor", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateRumMetricDefinition",
        Decision::Denied(DenyReason::Mutating),
    ),
];
