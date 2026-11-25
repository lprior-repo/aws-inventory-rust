use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "AssociateCollector",
            deferred(
                "Model requires environmentId, collectorArn, collectorDataAccessRoleArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates one Collector with this Environment and configures the customer role Continuum uses to authorize access to associated Collector data. The caller must have permission on the Environment and permission to pass the role.…",
            ),
        ),
        (
            "CreateCodeFinding",
            deferred(
                "Model requires environmentId, ocsfDocument. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Ingests a customer-supplied security finding into an environment. Continuum validates the document synchronously against the required overlay fields for the declared format; invalid documents are rejected with a…",
            ),
        ),
        (
            "CreateCodeScan",
            deferred(
                "Model requires environmentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an explicit scan request for one environment. CP creates durable work before any queue delivery occurs. Retrying a request that succeeded, with the same clientToken on the same environment, returns the original scan…",
            ),
        ),
        (
            "CreateCollector",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is collectorId, collectorArn, type, status, createdAt; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Creates the Collector for the caller's account. Provisions an AWS Config service-linked recorder using the caller's federated access session and begins recording infrastructure changes.",
            ),
        ),
        (
            "CreateEnvironment",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an environment and starts its validation and any requested provisioning. Storage provisioning and dependency checks may continue after the initial 201 response; use GetEnvironment to observe completion.",
            ),
        ),
        (
            "CreateIntegration",
            deferred(
                "Model requires name, providerConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new integration with an external source provider. Repeating an identical request with the same clientToken returns the original integrationId, integrationArn, and createdAt. Token records are retained for eight hours…",
            ),
        ),
        (
            "DeleteCollector",
            deferred(
                "Model requires collectorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the Collector for the caller's account. The Collector first enters PENDING_DELETION, then the service removes its schedule, AWS Config service-linked recorder, versioned S3 bucket, KMS key, and persisted record.",
            ),
        ),
        (
            "DeleteEnvironment",
            deferred(
                "Model requires environmentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an environment and its associated resources.",
            ),
        ),
        (
            "DeleteIntegration",
            deferred(
                "Model requires integrationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an integration and releases its name and account quota. Its retained CreateIntegration token receipt can still replay the original result without recreating the integration. Shared provider registrations are retained:…",
            ),
        ),
        (
            "DisassociateCollector",
            deferred(
                "Model requires environmentId, collectorArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes one Collector association from this Environment, preserving its other associations. The stored Collector data-access role is cleared when no Collector associations remain. The Environment owner must have permission to…",
            ),
        ),
        (
            "GetCodeFinding",
            deferred(
                "Model requires environmentId, findingId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the current state and details of a finding.",
            ),
        ),
        (
            "GetCodeIssue",
            deferred(
                "Model requires environmentId, issueId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the current state and details of an issue.",
            ),
        ),
        (
            "GetCodeIssueArtifact",
            deferred(
                "Model requires environmentId, issueId, artifactId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves one immutable artifact attached to an issue, including its content.",
            ),
        ),
        (
            "GetCodeScan",
            deferred(
                "Model requires environmentId, scanRunId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves one customer-visible environment scan.",
            ),
        ),
        (
            "GetCollector",
            deferred(
                "Model requires collectorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the current state of the Collector for the caller's account.",
            ),
        ),
        (
            "GetCollectorDataAccessAuthorization",
            deferred(
                "Model requires accountId, collectorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Authorizes an Environment's access to a Collector's data. Continuum calls this operation with the Environment's Collector data-access role before it reads the Collector's inventory, so grant…",
            ),
        ),
        (
            "GetEnvironment",
            deferred(
                "Model requires environmentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the current state and configuration of an environment.",
            ),
        ),
        (
            "GetIntegration",
            deferred(
                "Model requires integrationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about an integration.",
            ),
        ),
        (
            "GetResourcePolicy",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the resource policy attached to a Continuum resource.",
            ),
        ),
        (
            "ListAssociatedCollectors",
            deferred(
                "Model requires environmentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the Collectors currently associated with one Environment in ascending collectorId order. Each entry carries the Collector ARN, so a listed association can be passed straight to DisassociateCollector, including when the…",
            ),
        ),
        (
            "ListAssociatedEnvironments",
            deferred(
                "Model requires collectorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the Environments currently associated with one Collector in lexicographically ascending environmentArn order. The ARN identifies each Environment across accounts within the Collector's Region.",
            ),
        ),
        (
            "ListCodeFindings",
            deferred(
                "Model requires environmentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the findings in an environment in descending findingId order.",
            ),
        ),
        (
            "ListCodeIssueArtifacts",
            deferred(
                "Model requires environmentId, issueId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the artifacts attached to an issue in descending artifactId order. Content is returned by GetCodeIssueArtifact.",
            ),
        ),
        (
            "ListCodeIssues",
            deferred(
                "Model requires environmentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the issues in an environment in descending issueId order.",
            ),
        ),
        (
            "ListCodeScans",
            deferred(
                "Model requires environmentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists customer-visible scans for one environment in descending scanRunId order.",
            ),
        ),
        (
            "ListCollectors",
            OperationOverlay {
                queryable: vec![path("collectorArn")?],
                ..read(
                    "items",
                    Identity::Arn(path("collectorArn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "ListEnvironments",
            OperationOverlay {
                queryable: vec![path("environmentArn")?],
                ..read(
                    "items",
                    Identity::Arn(path("environmentArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListIntegrations",
            OperationOverlay {
                queryable: vec![path("integrationArn")?],
                ..read(
                    "items",
                    Identity::Arn(path("integrationArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "PutResourcePolicy",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates or replaces the resource policy attached to a Continuum resource.",
            ),
        ),
        (
            "StartIntegrationAuthorization",
            deferred(
                "Model requires providerType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts an OAuth authorization for a new integration and returns the values the caller sends to the provider's authorization endpoint. The authorization belongs to the calling AWS account, can be redeemed once by CreateIntegration…",
            ),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "UpdateCollector",
            deferred(
                "Model requires collectorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the customer-controlled mutable properties of a Collector. Only the name and Kubernetes cluster IAM role can be changed.",
            ),
        ),
        (
            "UpdateEnvironment",
            deferred(
                "Model requires environmentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the customer-controlled mutable properties of an environment. Only the name, the environment's S3 integration source and repository selections, its scan frequency, its network policy, and its patch creation policy can be…",
            ),
        ),
        (
            "UpdateIntegration",
            deferred(
                "Model requires integrationId, providerConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Rotates the credentials used by an Amazon S3 integration. Only S3 integrations can be updated; GitHub integrations cannot be modified. The integration's name, provider type, and bucket remain fixed. The caller must have…",
            ),
        ),
    ])
}
