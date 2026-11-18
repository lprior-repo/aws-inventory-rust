// Pinned source: .local/aws-models/models/appflow/service/2020-08-23/appflow-2020-08-23.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "CancelFlowExecutions",
            deferred(
                "Model requires flowName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Cancels active runs for a flow. You can cancel all of the active runs for a flow, or you can cancel specific runs by providing their IDs. You can cancel a flow run only when the run is in progress. You can't cancel a run that has…",
            ),
        ),
        (
            "CreateConnectorProfile",
            deferred(
                "Model requires connectorProfileName, connectorType, connectionMode, connectorProfileConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new connector profile associated with your Amazon Web Services account. There is a soft quota of 100 connector profiles per Amazon Web Services account. If you need more connector profiles than this quota allows, you…",
            ),
        ),
        (
            "CreateFlow",
            deferred(
                "Model requires flowName, triggerConfig, sourceFlowConfig, destinationFlowConfigList, tasks. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Enables your application to create a new flow using Amazon AppFlow. You must create a connector profile before calling this API. Please note that the Request Syntax below shows syntax for multiple destinations, however, you can…",
            ),
        ),
        (
            "DeleteConnectorProfile",
            deferred(
                "Model requires connectorProfileName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Enables you to delete an existing connector profile.",
            ),
        ),
        (
            "DeleteFlow",
            deferred(
                "Model requires flowName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Enables your application to delete an existing flow. Before deleting the flow, Amazon AppFlow validates the request by checking the flow configuration and status. You can delete flows one at a time.",
            ),
        ),
        (
            "DescribeConnector",
            deferred(
                "Model requires connectorType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes the given custom connector registered in your Amazon Web Services account. This API can be used for custom connectors that are registered in your account and also for Amazon authored connectors.",
            ),
        ),
        (
            "DescribeConnectorEntity",
            deferred(
                "Model requires connectorEntityName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Provides details regarding the entity used with the connector, with a description of the data model for each field in that entity.",
            ),
        ),
        // Model: Returns a list of connector-profile details matching the provided connector-profile names and connector-types . Both input lists are optional, and you can use them to filter the result. If no names…
        (
            "DescribeConnectorProfiles",
            OperationOverlay {
                queryable: vec![path("connectorProfileArn")?],
                ..read(
                    "connectorProfileDetails",
                    Identity::Arn(path("connectorProfileArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "DescribeConnectors",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is connectorConfigurations, connectors; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Describes the connectors vended by Amazon AppFlow for specified connector types. If you don't specify a connector type, this operation describes all connectors vended by Amazon AppFlow. If there are more connectors than can be…",
            ),
        ),
        // Model: Provides a description of the specified flow.
        (
            "DescribeFlow",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("flowName")?,
                    Binding::From {
                        operation: name("ListFlows")?,
                        path: path("flowName")?,
                    },
                )]
                .into_iter()
                .collect(),
                tags: Some(path("tags")?),
                queryable: vec![path("flowArn")?],
                ..read(
                    "$",
                    Identity::Arn(path("flowArn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "DescribeFlowExecutionRecords",
            deferred(
                "Model requires flowName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Fetches the execution history of the flow.",
            ),
        ),
        (
            "ListConnectorEntities",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is connectorEntityMap; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns the list of available connector entities supported by Amazon AppFlow. For example, you can query Salesforce for Account and Opportunity entities, or query ServiceNow for the Incident entity.",
            ),
        ),
        // Model: Returns the list of all registered custom connectors in your Amazon Web Services account. This API lists only custom connectors registered in this account, not the Amazon Web Services authored…
        (
            "ListConnectors",
            OperationOverlay {
                queryable: vec![path("connectorLabel")?],
                ..read(
                    "connectors",
                    Identity::Key(vec![path("connectorLabel")?]),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // Model: Lists all of the flows associated with your account.
        (
            "ListFlows",
            OperationOverlay {
                tags: Some(path("tags")?),
                queryable: vec![path("flowArn")?],
                ..read(
                    "flows",
                    Identity::Arn(path("flowArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the tags that are associated with a specified flow.",
            ),
        ),
        (
            "RegisterConnector",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is connectorArn; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Registers a new custom connector with your Amazon Web Services account. Before you can register the connector, you must deploy the associated AWS lambda function in your account.",
            ),
        ),
        (
            "ResetConnectorMetadataCache",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is not an inventory record; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Resets metadata about your connector entities that Amazon AppFlow stored in its cache. Use this action when you want Amazon AppFlow to return the latest information about the data that you have in a source application. Amazon…",
            ),
        ),
        (
            "StartFlow",
            deferred(
                "Model requires flowName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Activates an existing flow. For on-demand flows, this operation runs the flow immediately. For schedule and event-triggered flows, this operation activates the flow.",
            ),
        ),
        (
            "StopFlow",
            deferred(
                "Model requires flowName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deactivates the existing flow. For on-demand flows, this operation returns an unsupportedOperationException error message. For schedule and event-triggered flows, this operation deactivates the flow.",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceArn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Applies a tag to the specified flow.",
            ),
        ),
        (
            "UnregisterConnector",
            deferred(
                "Model requires connectorLabel. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Unregisters the custom connector registered in your account that matches the connector label provided in the request.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceArn, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes a tag from the specified flow.",
            ),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "UpdateConnectorProfile",
            deferred(
                "Model requires connectorProfileName, connectionMode, connectorProfileConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a given connector profile associated with your account.",
            ),
        ),
        (
            "UpdateConnectorRegistration",
            deferred(
                "Model requires connectorLabel. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a custom connector that you've previously registered. This operation updates the connector with one of the following: The latest version of the AWS Lambda function that's assigned to the connector A new AWS Lambda…",
            ),
        ),
        (
            "UpdateFlow",
            deferred(
                "Model requires flowName, triggerConfig, sourceFlowConfig, destinationFlowConfigList, tasks. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing flow.",
            ),
        ),
    ])
}
