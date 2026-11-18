// Pinned source: .local/aws-models/models/application-insights/service/2018-11-25/application-insights-2018-11-25.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "AddWorkload",
            deferred(
                "Model requires ResourceGroupName, ComponentName, WorkloadConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds a workload to a component. Each component can have at most five workloads.",
            ),
        ),
        (
            "CreateApplication",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is ApplicationInfo; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Adds an application that is created from a resource group.",
            ),
        ),
        (
            "CreateComponent",
            deferred(
                "Model requires ResourceGroupName, ComponentName, ResourceList. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a custom component by grouping similar standalone instances to monitor.",
            ),
        ),
        (
            "CreateLogPattern",
            deferred(
                "Model requires ResourceGroupName, PatternSetName, PatternName, Pattern, Rank. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds an log pattern to a LogPatternSet .",
            ),
        ),
        (
            "DeleteApplication",
            deferred(
                "Model requires ResourceGroupName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes the specified application from monitoring. Does not delete the application.",
            ),
        ),
        (
            "DeleteComponent",
            deferred(
                "Model requires ResourceGroupName, ComponentName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Ungroups a custom component. When you ungroup custom components, all applicable monitors that are set up for the component are removed and the instances revert to their standalone status.",
            ),
        ),
        (
            "DeleteLogPattern",
            deferred(
                "Model requires ResourceGroupName, PatternSetName, PatternName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes the specified log pattern from a LogPatternSet .",
            ),
        ),
        // Model: Describes the application.
        (
            "DescribeApplication",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("ResourceGroupName")?,
                    Binding::From {
                        operation: name("ListApplications")?,
                        path: path("ResourceGroupName")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("ResourceGroupName")?],
                ..read(
                    "ApplicationInfo",
                    Identity::Key(vec![path("ResourceGroupName")?]),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "DescribeComponent",
            deferred(
                "Model requires ResourceGroupName, ComponentName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes a component and lists the resources that are grouped together in a component.",
            ),
        ),
        (
            "DescribeComponentConfiguration",
            deferred(
                "Model requires ResourceGroupName, ComponentName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes the monitoring configuration of the component.",
            ),
        ),
        (
            "DescribeComponentConfigurationRecommendation",
            deferred(
                "Model requires ResourceGroupName, ComponentName, Tier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes the recommended monitoring configuration of the component.",
            ),
        ),
        (
            "DescribeLogPattern",
            deferred(
                "Model requires ResourceGroupName, PatternSetName, PatternName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describe a specific log pattern from a LogPatternSet .",
            ),
        ),
        (
            "DescribeObservation",
            deferred(
                "Model requires ObservationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes an anomaly or error with the application.",
            ),
        ),
        (
            "DescribeProblem",
            deferred(
                "Model requires ProblemId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes an application problem.",
            ),
        ),
        (
            "DescribeProblemObservations",
            deferred(
                "Model requires ProblemId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes the anomalies or errors associated with the problem.",
            ),
        ),
        (
            "DescribeWorkload",
            deferred(
                "Model requires ResourceGroupName, ComponentName, WorkloadId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes a workload and its configuration.",
            ),
        ),
        // Model: Lists the IDs of the applications that you are monitoring.
        (
            "ListApplications",
            OperationOverlay {
                queryable: vec![path("ResourceGroupName")?],
                ..read(
                    "ApplicationInfoList",
                    Identity::Key(vec![path("ResourceGroupName")?]),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListComponents",
            deferred(
                "Model requires ResourceGroupName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the auto-grouped, standalone, and custom components of the application.",
            ),
        ),
        (
            "ListConfigurationHistory",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is EventList; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists the INFO, WARN, and ERROR events for periodic configuration updates performed by Application Insights. Examples of events represented are: INFO: creating a new alarm or updating an alarm threshold. WARN: alarm not created…",
            ),
        ),
        (
            "ListLogPatternSets",
            deferred(
                "Model requires ResourceGroupName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the log pattern sets in the specific application.",
            ),
        ),
        (
            "ListLogPatterns",
            deferred(
                "Model requires ResourceGroupName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the log patterns in the specific log LogPatternSet .",
            ),
        ),
        (
            "ListProblems",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is ProblemList, ResourceGroupName, AccountId; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists the problems with your application.",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires ResourceARN. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieve a list of the tags (keys and values) that are associated with a specified application. A tag is a label that you optionally define and associate with an application. Each tag consists of a required tag key and an…",
            ),
        ),
        (
            "ListWorkloads",
            deferred(
                "Model requires ResourceGroupName, ComponentName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the workloads that are configured on a given component.",
            ),
        ),
        (
            "RemoveWorkload",
            deferred(
                "Model requires ResourceGroupName, ComponentName, WorkloadId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Remove workload from a component.",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires ResourceARN, Tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Add one or more tags (keys and values) to a specified application. A tag is a label that you optionally define and associate with an application. Tags can help you categorize and manage application in different ways, such as by…",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires ResourceARN, TagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Remove one or more tags (keys and values) from a specified application.",
            ),
        ),
        (
            "UpdateApplication",
            deferred(
                "Model requires ResourceGroupName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the application.",
            ),
        ),
        (
            "UpdateComponent",
            deferred(
                "Model requires ResourceGroupName, ComponentName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the custom component name and/or the list of resources that make up the component.",
            ),
        ),
        (
            "UpdateComponentConfiguration",
            deferred(
                "Model requires ResourceGroupName, ComponentName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the monitoring configurations for the component. The configuration input parameter is an escaped JSON of the configuration and should match the schema of what is returned by DescribeComponentConfigurationRecommendation .",
            ),
        ),
        (
            "UpdateLogPattern",
            deferred(
                "Model requires ResourceGroupName, PatternSetName, PatternName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds a log pattern to a LogPatternSet .",
            ),
        ),
        (
            "UpdateProblem",
            deferred(
                "Model requires ProblemId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the visibility of the problem or specifies the problem as RESOLVED .",
            ),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![(
        "UpdateWorkload",
        deferred(
            "Model requires ResourceGroupName, ComponentName, WorkloadConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds a workload to a component. Each component can have at most five workloads.",
        ),
    )])
}
