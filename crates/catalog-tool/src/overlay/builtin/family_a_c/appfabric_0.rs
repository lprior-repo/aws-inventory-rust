// Pinned source: .local/aws-models/models/appfabric/service/2023-05-19/appfabric-2023-05-19.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;
use crate::overlay::ContextSource;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "BatchGetUserAccessTasks",
            deferred(
                "Model requires appBundleIdentifier, taskIdList. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets user access details in a batch request. This action polls data from the tasks that are kicked off by the StartUserAccessTasks action.",
            ),
        ),
        (
            "ConnectAppAuthorization",
            deferred(
                "Model requires appBundleIdentifier, appAuthorizationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Establishes a connection between Amazon Web Services AppFabric and an application, which allows AppFabric to call the APIs of the application.",
            ),
        ),
        (
            "CreateAppAuthorization",
            deferred(
                "Model requires appBundleIdentifier, app, credential, tenant, authType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an app authorization within an app bundle, which allows AppFabric to connect to an application.",
            ),
        ),
        (
            "CreateAppBundle",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is appBundle; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Creates an app bundle to collect data from an application using AppFabric.",
            ),
        ),
        (
            "CreateIngestion",
            deferred(
                "Model requires appBundleIdentifier, app, tenantId, ingestionType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a data ingestion for an application.",
            ),
        ),
        (
            "CreateIngestionDestination",
            deferred(
                "Model requires appBundleIdentifier, ingestionIdentifier, processingConfiguration, destinationConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an ingestion destination, which specifies how an application's ingested data is processed by Amazon Web Services AppFabric and where it's delivered.",
            ),
        ),
        (
            "DeleteAppAuthorization",
            deferred(
                "Model requires appBundleIdentifier, appAuthorizationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an app authorization. You must delete the associated ingestion before you can delete an app authorization.",
            ),
        ),
        (
            "DeleteAppBundle",
            deferred(
                "Model requires appBundleIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an app bundle. You must delete all associated app authorizations before you can delete an app bundle.",
            ),
        ),
        (
            "DeleteIngestion",
            deferred(
                "Model requires appBundleIdentifier, ingestionIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an ingestion. You must stop (disable) the ingestion and you must delete all associated ingestion destinations before you can delete an app ingestion.",
            ),
        ),
        (
            "DeleteIngestionDestination",
            deferred(
                "Model requires appBundleIdentifier, ingestionIdentifier, ingestionDestinationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an ingestion destination. This deletes the association between an ingestion and it's destination. It doesn't delete previously ingested data or the storage destination, such as the Amazon S3 bucket where the data is…",
            ),
        ),
        (
            "GetAppAuthorization",
            deferred(
                "Model requires appBundleIdentifier, appAuthorizationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about an app authorization.",
            ),
        ),
        // Model: Returns information about an app bundle.
        (
            "GetAppBundle",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("appBundleIdentifier")?,
                    Binding::From {
                        operation: name("ListAppBundles")?,
                        path: path("arn")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("arn")?],
                ..read(
                    "appBundle",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "GetIngestion",
            deferred(
                "Model requires appBundleIdentifier, ingestionIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about an ingestion.",
            ),
        ),
        (
            "GetIngestionDestination",
            deferred(
                "Model requires appBundleIdentifier, ingestionIdentifier, ingestionDestinationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about an ingestion destination.",
            ),
        ),
        // Model: Returns a list of all app authorizations configured for an app bundle.
        (
            "ListAppAuthorizations",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("appBundleIdentifier")?,
                    Binding::From {
                        operation: name("ListAppBundles")?,
                        path: path("arn")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("appAuthorizationArn")?],
                ..read(
                    "appAuthorizationSummaryList",
                    Identity::Arn(path("appAuthorizationArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // Model: Returns a list of app bundles.
        (
            "ListAppBundles",
            OperationOverlay {
                queryable: vec![path("arn")?],
                ..read(
                    "appBundleSummaryList",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // Reviewed full response: destination ARNs and continuation only; inherit the bundle from this ingestion's request.
        (
            "ListIngestionDestinations",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [
                    (
                        path("ingestionIdentifier")?,
                        Binding::From {
                            operation: name("ListIngestions")?,
                            path: path("arn")?,
                        },
                    ),
                    (
                        path("appBundleIdentifier")?,
                        Binding::Context {
                            operation: name("ListIngestions")?,
                            source: ContextSource::Request,
                            path: path("appBundleIdentifier")?,
                        },
                    ),
                ]
                .into_iter()
                .collect(),
                queryable: vec![path("arn")?],
                ..read(
                    "ingestionDestinations",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // Model: Returns a list of all ingestions configured for an app bundle.
        (
            "ListIngestions",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("appBundleIdentifier")?,
                    Binding::From {
                        operation: name("ListAppBundles")?,
                        path: path("arn")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("arn")?],
                ..read(
                    "ingestions",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of tags for a resource.",
            ),
        ),
        (
            "StartIngestion",
            deferred(
                "Model requires ingestionIdentifier, appBundleIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts (enables) an ingestion, which collects data from an application.",
            ),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "StartUserAccessTasks",
            deferred(
                "Model requires appBundleIdentifier, email. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts the tasks to search user access status for a specific email address. The tasks are stopped when the user access status data is found. The tasks are terminated when the API calls to the application time out.",
            ),
        ),
        (
            "StopIngestion",
            deferred(
                "Model requires ingestionIdentifier, appBundleIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops (disables) an ingestion.",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceArn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Assigns one or more tags (key-value pairs) to the specified resource.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceArn, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes a tag or tags from a resource.",
            ),
        ),
        (
            "UpdateAppAuthorization",
            deferred(
                "Model requires appBundleIdentifier, appAuthorizationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an app authorization within an app bundle, which allows AppFabric to connect to an application. If the app authorization was in a connected state, updating the app authorization will set it back to a PendingConnect state.",
            ),
        ),
        (
            "UpdateIngestionDestination",
            deferred(
                "Model requires appBundleIdentifier, ingestionIdentifier, ingestionDestinationIdentifier, destinationConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an ingestion destination, which specifies how an application's ingested data is processed by Amazon Web Services AppFabric and where it's delivered.",
            ),
        ),
    ])
}
