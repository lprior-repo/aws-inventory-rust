use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "ActivateSubscription",
            deferred(
                "Model requires domainId, subscriptionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Activates a Subscription to enable billing for a user.",
            ),
        ),
        (
            "CreateDomain",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new Domain for managing HealthAgent resources.",
            ),
        ),
        (
            "CreateSubscription",
            deferred(
                "Model requires domainId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new Subscription within a Domain for billing and user management.",
            ),
        ),
        (
            "DeactivateSubscription",
            deferred(
                "Model requires domainId, subscriptionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deactivates a Subscription to stop billing for a user.",
            ),
        ),
        (
            "DeleteDomain",
            deferred(
                "Model requires domainId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a Domain and all associated resources.",
            ),
        ),
        (
            "GetDomain",
            deferred(
                "Model requires domainId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about a Domain.",
            ),
        ),
        (
            "GetMedicalScribeListeningSession",
            deferred(
                "Model requires sessionId, domainId, subscriptionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves details about an existing Medical Scribe listening session",
            ),
        ),
        (
            "GetPatientInsightsJob",
            deferred(
                "Model requires domainId, jobId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Get details of a started patient insights job.",
            ),
        ),
        (
            "GetSubscription",
            deferred(
                "Model requires domainId, subscriptionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about a Subscription.",
            ),
        ),
        (
            "ListDomains",
            OperationOverlay {
                queryable: vec![path("arn")?],
                ..read(
                    "domains",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListSubscriptions",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("domainId")?,
                    Binding::From {
                        operation: name("ListDomains")?,
                        path: path("domainId")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("arn")?],
                ..read(
                    "subscriptions",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the tags associated with the specified resource",
            ),
        ),
        (
            "StartMedicalScribeListeningSession",
            deferred(
                "Model requires sessionId, domainId, subscriptionId, languageCode, mediaSampleRateHertz, mediaEncoding. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts a new Medical Scribe listening session for real-time audio transcription",
            ),
        ),
        (
            "StartPatientInsightsJob",
            deferred(
                "Model requires domainId, patientContext, insightsContext, encounterContext, userContext, inputDataConfig, outputDataConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts a new patient insights job.",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceArn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates the specified tags with the specified resource",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceArn, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes the specified tags from the specified resource",
            ),
        ),
    ])
}
