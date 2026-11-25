use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "BatchDeleteInstrumentationConfigurations",
            deferred(
                "Model requires DeletionTarget. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes multiple instrumentation configurations in a single request. Supports two mutually exclusive selection methods: - By scope: Delete all configurations matching a Service + Environment + InstrumentationType - By ARN list:…",
            ),
        ),
        (
            "BatchGetServiceLevelObjectiveBudgetReport",
            deferred(
                "Model requires Timestamp, SloIds. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Use this operation to retrieve one or more service level objective (SLO) budget reports . An error budget is the amount of time or requests in an unhealthy state that your service can accumulate during an interval before your…",
            ),
        ),
        (
            "BatchUpdateExclusionWindows",
            deferred(
                "Model requires SloIds. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Add or remove time window exclusions for one or more Service Level Objectives (SLOs).",
            ),
        ),
        (
            "CreateInstrumentationConfiguration",
            deferred(
                "Model requires InstrumentationType, Service, Environment, SignalType, Location, CaptureConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a dynamic instrumentation configuration for a specific code or endpoint location within a service and environment. Configurations are immutable after creation. For BREAKPOINT type configurations, they expire after 24…",
            ),
        ),
        (
            "CreateServiceLevelObjective",
            deferred(
                "Model requires Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a service level objective (SLO), which can help you ensure that your critical business operations are meeting customer expectations. Use SLOs to set and track specific target levels for the reliability and availability of…",
            ),
        ),
        (
            "DeleteGroupingConfiguration",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is not an inventory record; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Deletes the grouping configuration for this account. This removes all custom grouping attribute definitions that were previously configured.",
            ),
        ),
        (
            "DeleteInstrumentationConfiguration",
            deferred(
                "Model requires InstrumentationType, Service, Environment, SignalType, LocationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified instrumentation configuration. SDKs remove the instrumentation during their next sync after the configuration is deleted or expires.",
            ),
        ),
        (
            "DeleteServiceLevelObjective",
            deferred(
                "Model requires Id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified service level objective.",
            ),
        ),
        (
            "GetInstrumentationConfiguration",
            deferred(
                "Model requires InstrumentationType, Service, Environment, SignalType, LocationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the details of a single instrumentation configuration identified by service, environment, signal type, and location. Use this to audit or display configuration details.",
            ),
        ),
        (
            "GetInstrumentationConfigurationStatus",
            deferred(
                "Model requires InstrumentationType, Service, Environment, SignalType, LocationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the status history for a single instrumentation configuration during a specified time range. The response lists when the configuration was ACTIVE, READY, ERROR, or DISABLED. If no status or time window is provided, the…",
            ),
        ),
        (
            "GetService",
            deferred(
                "Model requires StartTime, EndTime, KeyAttributes. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about a service discovered by Application Signals.",
            ),
        ),
        (
            "GetServiceLevelObjective",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("Id")?,
                    Binding::From {
                        operation: name("ListServiceLevelObjectives")?,
                        path: path("Arn")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("Arn")?],
                ..read(
                    "Slo",
                    Identity::Arn(path("Arn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "ListAuditFindings",
            deferred(
                "Model requires StartTime, EndTime, AuditTargets. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of audit findings that provide automated analysis of service behavior and root cause analysis. These findings help identify the most significant observations about your services, including performance issues,…",
            ),
        ),
        (
            "ListEntityEvents",
            deferred(
                "Model requires Entity, StartTime, EndTime. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of change events for a specific entity, such as deployments, configuration changes, or other state-changing activities. This operation helps track the history of changes that may have affected service performance.",
            ),
        ),
        (
            "ListGroupingAttributeDefinitions",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is GroupingAttributeDefinitions, UpdatedAt; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns the current grouping configuration for this account, including all custom grouping attribute definitions that have been configured. These definitions determine how services are logically grouped based on telemetry…",
            ),
        ),
        (
            "ListInstrumentationConfigurations",
            deferred(
                "Model requires Service, Environment, InstrumentationType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns all active instrumentation configurations for a service and environment. SDKs use this operation to sync configurations and apply client-side filters locally. Include the previous SyncedAt value to perform incremental…",
            ),
        ),
        (
            "ListServiceDependencies",
            deferred(
                "Model requires StartTime, EndTime, KeyAttributes. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of service dependencies of the service that you specify. A dependency is an infrastructure component that an operation of this service connects with. Dependencies can include Amazon Web Services services, Amazon…",
            ),
        ),
        (
            "ListServiceDependents",
            deferred(
                "Model requires StartTime, EndTime, KeyAttributes. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the list of dependents that invoked the specified service during the provided time range. Dependents include other services, CloudWatch Synthetics canaries, and clients that are instrumented with CloudWatch RUM app…",
            ),
        ),
        (
            "ListServiceLevelObjectiveExclusionWindows",
            deferred(
                "Model requires Id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves all exclusion windows configured for a specific SLO.",
            ),
        ),
        (
            "ListServiceLevelObjectives",
            OperationOverlay {
                queryable: vec![path("Arn")?],
                ..read(
                    "SloSummaries",
                    Identity::Arn(path("Arn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListServiceOperations",
            deferred(
                "Model requires StartTime, EndTime, KeyAttributes. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of the operations of this service that have been discovered by Application Signals. Only the operations that were invoked during the specified time range are returned.",
            ),
        ),
        (
            "ListServiceStates",
            deferred(
                "Model requires StartTime, EndTime. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about the last deployment and other change states of services. This API provides visibility into recent changes that may have affected service performance, helping with troubleshooting and change correlation.",
            ),
        ),
        (
            "ListServices",
            deferred(
                "Model requires StartTime, EndTime. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of services that have been discovered by Application Signals. A service represents a minimum logical and transactional unit that completes a business function. Services are discovered through Application Signals…",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires ResourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Displays the tags associated with a CloudWatch resource. Tags can be assigned to service level objectives.",
            ),
        ),
        (
            "PutGroupingConfiguration",
            deferred(
                "Model requires GroupingAttributeDefinitions. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates or updates the grouping configuration for this account. This operation allows you to define custom grouping attributes that determine how services are logically grouped based on telemetry attributes, Amazon Web Services…",
            ),
        ),
        (
            "ReportInstrumentationConfigurationStatus",
            deferred(
                "Model requires Service, Environment, Configurations. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Reports the status of one or more instrumentation configurations from SDK instances. Use this to record when configurations become ready, hit errors, become active, or are disabled by limits. Report READY , ERROR , and DISABLED…",
            ),
        ),
        (
            "StartDiscovery",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is not an inventory record; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Enables this Amazon Web Services account to be able to use CloudWatch Application Signals by creating the AWSServiceRoleForCloudWatchApplicationSignals service-linked role. This service- linked role has the following permissions:…",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires ResourceArn, Tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Assigns one or more tags (key-value pairs) to the specified CloudWatch resource, such as a service level objective. Tags can help you organize and categorize your resources. You can also use them to scope user permissions by…",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires ResourceArn, TagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes one or more tags from the specified resource.",
            ),
        ),
        (
            "UpdateServiceLevelObjective",
            deferred(
                "Model requires Id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing service level objective (SLO). If you omit parameters, the previous values of those parameters are retained. You cannot change from a period-based SLO to a request-based SLO, or change from a request-based SLO…",
            ),
        ),
    ])
}
