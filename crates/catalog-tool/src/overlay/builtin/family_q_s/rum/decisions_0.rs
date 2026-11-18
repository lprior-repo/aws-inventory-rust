// Pinned operation purposes: .local/aws-models/models/rum/service/2018-05-10/rum-2018-05-10.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Specifies the extended metrics and custom metrics that you want a CloudWatch RUM app monitor to send to a destination. Valid destinations include CloudWatch and Evidently. By default, RUM app monitors send some metrics to CloudWatch. These default metrics are listed in CloudWatch metrics that you can collect with CloudWatch RUM . In addition to these default metrics, you can choose to send extende
    (
        "BatchCreateRumMetricDefinitions",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Removes the specified metrics from being sent to an extended metrics destination. If some metric definition IDs specified in a BatchDeleteRumMetricDefinitions operations are not valid, those metric definitions fail and return errors, but all valid metric definition IDs in the same operation are still deleted. The maximum number of metric definitions that you can specify in one BatchDeleteRumMetric
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
    // Changes service state rather than reading inventory. Pinned purpose: Creates a Amazon CloudWatch RUM app monitor, which collects telemetry data from your application and sends that data to RUM. The data includes performance and reliability information such as page load time, client-side errors, and user behavior. You use this operation only to create a new app monitor. To update an existing app monitor, use UpdateAppMonitor instead. After you create an app monitor,
    ("CreateAppMonitor", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an existing app monitor. This immediately stops the collection of data.
    ("DeleteAppMonitor", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes the association of a resource-based policy from an app monitor.
    (
        "DeleteResourcePolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a destination for CloudWatch RUM extended metrics, so that the specified app monitor stops sending extended metrics to that destination.
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
    // Changes service state rather than reading inventory. Pinned purpose: Use this operation to assign a resource-based policy to a CloudWatch RUM app monitor to control access to it. Each app monitor can have one resource-based policy. The maximum size of the policy is 4 KB. To learn more about using resource policies with RUM, see Using resource-based policies with CloudWatch RUM .
    ("PutResourcePolicy", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Sends telemetry events about your application performance and user behavior to CloudWatch RUM. The code snippet that RUM generates for you to add to your application includes PutRumEvents operations to send this data to RUM. Each PutRumEvents operation can send a batch of events from one user session.
    ("PutRumEvents", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates or updates a destination to receive extended metrics from CloudWatch RUM. You can send extended metrics to CloudWatch or to a CloudWatch Evidently experiment. For more information about extended metrics, see BatchCreateRumMetricDefinitions .
    (
        "PutRumMetricsDestination",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Assigns one or more tags (key-value pairs) to the specified CloudWatch RUM resource. Currently, the only resources that can be tagged app monitors. Tags can help you organize and categorize your resources. You can also use them to scope user permissions by granting a user permission to access or change only resources with certain tag values. Tags don't have any semantic meaning to Amazon Web Servi
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes one or more tags from the specified resource.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the configuration of an existing app monitor. When you use this operation, only the parts of the app monitor configuration that you specify in this operation are changed. For any parameters that you omit, the existing values are kept. You can't use this operation to change the tags of an existing app monitor. To change the tags of an existing app monitor, use TagResource . To create a new
    ("UpdateAppMonitor", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Modifies one existing metric definition for CloudWatch RUM extended metrics. For more information about extended metrics, see BatchCreateRumMetricsDefinitions .
    (
        "UpdateRumMetricDefinition",
        Decision::Denied(DenyReason::Mutating),
    ),
];
